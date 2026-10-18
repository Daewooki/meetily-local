//! Explicit, user-triggered publishing. Tokens never leave native code after entry.
//! The vault key lives in the OS credential store; on-disk tokens are AES-256-GCM ciphertext.
#[cfg(target_os = "windows")]
mod windows_vault;
use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::{rngs::OsRng, RngCore};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, time::Duration};
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::Mutex;

pub struct AtlassianState(pub Mutex<()>);

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub site_url: String,
    pub email: String,
    pub token_type: String,
    pub cloud_id: Option<String>,
    pub space_key: String,
    pub space_id: Option<String>,
    pub parent_page_id: String,
    // Keep parentPageId compatible with existing encrypted vaults and receipts.
    #[serde(default)]
    pub parent_type: Option<String>,
    #[serde(default)]
    pub parent_title: Option<String>,
    pub project_key: String,
    #[serde(default)]
    pub board_id: String,
    #[serde(default)]
    pub board_name: Option<String>,
    #[serde(default)]
    pub board_filter: Option<String>,
    #[serde(default)]
    pub board_note: Option<String>,
    pub issue_types: Vec<IssueType>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IssueType {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct JiraComponent {
    pub id: String,
    pub name: String,
}

// Deliberately no Debug implementation: credentials must never enter logs.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub kind: String,
    pub name: String,
    pub site_url: String,
    pub email: String,
    pub token: String,
    pub token_type: String,
    pub space_key: String,
    pub parent_page_id: String,
    pub project_key: String,
    #[serde(default)]
    pub board_id: String,
}

#[derive(Serialize, Deserialize)]
struct StoredConnection {
    connection: Connection,
    encrypted_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub key: String,
    pub meeting_id: String,
    pub connection_id: String,
    pub kind: String,
    pub title: String,
    pub url: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Vault {
    connections: Vec<StoredConnection>,
    #[serde(default)]
    receipts: Vec<Receipt>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishInput {
    pub connection_id: String,
    pub meeting_id: String,
    pub title: String,
    pub body: String,
    pub issue_type_id: Option<String>,
    pub component_id: Option<String>,
}

fn vault_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|_| "앱 저장 폴더를 찾을 수 없습니다.")?
        .join("atlassian-vault.json"))
}

fn read_vault<R: Runtime>(app: &AppHandle<R>) -> Result<Vault, String> {
    match std::fs::read(vault_path(app)?) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| {
            "연결 저장 파일을 읽을 수 없습니다. 원본 파일을 보존한 채 지원을 요청하세요.".into()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vault::default()),
        Err(_) => Err("연결 저장 파일에 접근할 수 없습니다.".into()),
    }
}

fn write_vault<R: Runtime>(app: &AppHandle<R>, vault: &Vault) -> Result<(), String> {
    use std::io::Write;
    let path = vault_path(app)?;
    let dir = path.parent().ok_or("저장 경로가 올바르지 않습니다.")?;
    std::fs::create_dir_all(dir).map_err(|_| "저장 폴더를 만들 수 없습니다.")?;
    let mut temp =
        tempfile::NamedTempFile::new_in(dir).map_err(|_| "임시 저장 파일을 만들 수 없습니다.")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|_| "파일 권한 설정 실패")?;
    }
    let data = serde_json::to_vec_pretty(vault).map_err(|_| "연결 정보를 저장할 수 없습니다.")?;
    temp.write_all(&data)
        .map_err(|_| "연결 정보를 기록할 수 없습니다.")?;
    temp.as_file()
        .sync_all()
        .map_err(|_| "연결 정보를 디스크에 기록하지 못했습니다.")?;
    temp.persist(&path)
        .map_err(|_| "연결 정보를 확정 저장하지 못했습니다.")?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn vault_key(create: bool) -> Result<Vec<u8>, String> {
    use security_framework::passwords::{get_generic_password, set_generic_password};
    const SERVICE: &str = "ai.meetily.local.atlassian-vault";
    match get_generic_password(SERVICE, "aes256-key-v1") {
        Ok(key) if key.len() == 32 => Ok(key),
        Ok(_) => Err("Keychain 암호화 키가 올바르지 않습니다.".into()),
        Err(e) if e.code() == -25300 && create => {
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            set_generic_password(SERVICE, "aes256-key-v1", &key)
                .map_err(|_| "Keychain에 암호화 키를 저장할 수 없습니다.")?;
            Ok(key)
        }
        Err(_) => Err("macOS Keychain을 잠금 해제하고 앱의 키 접근을 허용해 주세요.".into()),
    }
}

#[cfg(target_os = "windows")]
fn vault_key(create: bool) -> Result<Vec<u8>, String> {
    windows_vault::load_key("ai.meetily.local.atlassian-vault/aes256-key-v1", create)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn vault_key(_: bool) -> Result<Vec<u8>, String> {
    Err("현재 연결 저장은 macOS와 Windows에서 제공됩니다.".into())
}

fn aad(connection: &Connection) -> String {
    format!(
        "v1|{}|{}|{}|{}",
        connection.id, connection.kind, connection.site_url, connection.email
    )
}

fn seal(key: &[u8], text: &str, associated: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "암호화 키 오류")?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let encrypted = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: text.as_bytes(),
                aad: associated.as_bytes(),
            },
        )
        .map_err(|_| "토큰 암호화 실패")?;
    let mut result = nonce.to_vec();
    result.extend(encrypted);
    Ok(STANDARD.encode(result))
}

fn unseal(key: &[u8], encoded: &str, associated: &str) -> Result<String, String> {
    let data = STANDARD
        .decode(encoded)
        .map_err(|_| "암호화된 토큰 형식 오류")?;
    if data.len() < 28 {
        return Err("암호화된 토큰이 손상되었습니다.".into());
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "암호화 키 오류")?;
    let plain = cipher
        .decrypt(
            Nonce::from_slice(&data[..12]),
            Payload {
                msg: &data[12..],
                aad: associated.as_bytes(),
            },
        )
        .map_err(|_| "토큰 복호화에 실패했습니다. 연결을 다시 등록해 주세요.")?;
    String::from_utf8(plain).map_err(|_| "토큰 문자 형식 오류".into())
}

fn normalize_site(input: &str) -> Result<String, String> {
    let url = url::Url::parse(input.trim())
        .map_err(|_| "사이트 URL을 https://your-team.atlassian.net 형식으로 입력해 주세요.")?;
    let host = url.host_str().unwrap_or("");
    let prefix = host.strip_suffix(".atlassian.net").unwrap_or("");
    if url.scheme() != "https"
        || prefix.is_empty()
        || prefix.contains('.')
        || !prefix
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/" | "/wiki" | "/wiki/")
    {
        return Err("Atlassian Cloud 사이트의 HTTPS 주소만 사용할 수 있습니다. 예: https://company.atlassian.net".into());
    }
    Ok(format!("https://{host}"))
}

fn validate_input(input: &ConnectionInput) -> Result<Connection, String> {
    let site_url = normalize_site(&input.site_url)?;
    if !matches!(input.kind.as_str(), "confluence" | "jira") {
        return Err("연결 종류가 올바르지 않습니다.".into());
    }
    if !matches!(input.token_type.as_str(), "classic" | "scoped") {
        return Err("토큰 종류를 선택해 주세요.".into());
    }
    if input.name.trim().is_empty()
        || input.name.chars().count() > 100
        || !input.email.contains('@')
        || input.email.contains(['\r', '\n'])
        || input.token.trim().is_empty()
        || input.token.len() > 16000
    {
        return Err("연결 이름, Atlassian 계정 이메일, API 토큰을 확인해 주세요.".into());
    }
    let target = if input.kind == "confluence" {
        input.space_key.trim()
    } else {
        input.project_key.trim()
    };
    if target.is_empty()
        || target.len() > 255
        || !target
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'~')
    {
        return Err("Space Key 또는 Project Key를 확인해 주세요.".into());
    }
    let parent = input.parent_page_id.trim();
    if !parent.is_empty() && numeric_id(&json!(parent)).is_none() {
        return Err("상위 페이지·폴더 ID에는 주소에 표시된 양의 정수를 입력해 주세요.".into());
    }
    let board_id = input.board_id.trim();
    if !board_id.is_empty()
        && (input.kind != "jira"
            || !board_id.bytes().all(|c| c.is_ascii_digit())
            || board_id.parse::<u64>().map_or(true, |id| id == 0))
    {
        return Err("Jira Board ID에는 보드 주소의 마지막 숫자를 입력해 주세요. 예: 1234".into());
    }
    Ok(Connection {
        id: uuid::Uuid::new_v4().to_string(),
        kind: input.kind.clone(),
        name: input.name.trim().into(),
        site_url,
        email: input.email.trim().into(),
        token_type: input.token_type.clone(),
        cloud_id: None,
        space_key: input.space_key.trim().into(),
        space_id: None,
        parent_page_id: parent.into(),
        parent_type: None,
        parent_title: None,
        project_key: input.project_key.trim().into(),
        board_id: board_id.into(),
        board_name: None,
        board_filter: None,
        board_note: None,
        issue_types: vec![],
    })
}

fn client() -> Result<Client, String> {
    Client::builder()
        .use_native_tls()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(25))
        .connect_timeout(Duration::from_secs(10))
        .user_agent(concat!("Meetily-Local/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| "연결 클라이언트를 만들 수 없습니다.".into())
}

fn api_base(c: &Connection) -> Result<String, String> {
    normalize_site(&c.site_url)?;
    if c.token_type == "scoped" {
        let id = c
            .cloud_id
            .as_deref()
            .ok_or("Cloud ID가 없습니다. 연결을 다시 등록하세요.")?;
        uuid::Uuid::parse_str(id).map_err(|_| "Cloud ID 형식 오류")?;
        Ok(format!("https://api.atlassian.com/ex/{}/{id}", c.kind))
    } else if c.token_type == "classic" {
        Ok(c.site_url.clone())
    } else {
        Err("토큰 종류 오류".into())
    }
}

async fn read_api(
    client: &Client,
    c: &Connection,
    token: &str,
    path: &str,
) -> Result<Value, String> {
    read_api_optional(client, c, token, path)
        .await?
        .ok_or_else(|| http_error(404))
}

// Only a 404 permits trying another content type. Never mask authentication,
// throttling or network failures as a missing page/folder.
async fn read_api_optional(
    client: &Client,
    c: &Connection,
    token: &str,
    path: &str,
) -> Result<Option<Value>, String> {
    let res = client
        .get(format!("{}{path}", api_base(c)?))
        .basic_auth(&c.email, Some(token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| "Atlassian에 연결할 수 없습니다. 네트워크와 사이트 주소를 확인하세요.")?;
    let status = res.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        return Err(http_error(status.as_u16()));
    }
    res.json::<Value>()
        .await
        .map(Some)
        .map_err(|_| "Atlassian 응답을 읽을 수 없습니다.".into())
}

fn http_error(status: u16) -> String {
    match status {
        401 => "인증 실패(401): 계정 이메일·API 토큰·토큰 종류를 확인해 주세요.",
        403 => "권한 부족(403): 토큰 scope와 해당 Space/Project의 접근·작성 권한을 확인해 주세요.",
        404 => "대상을 찾을 수 없습니다(404). 사이트 주소·Space/Project Key·페이지 또는 폴더 ID와 접근 권한을 확인하세요.",
        429 => "요청 횟수 제한(429)에 도달했습니다. 잠시 후 다시 시도하세요.",
        _ => "Atlassian이 요청을 거절했습니다. 대상 설정과 필수 입력 항목을 확인하세요.",
    }.into()
}

async fn resolve_parent_with<F, Fut>(
    c: &Connection,
    mut fetch: F,
) -> Result<Option<(String, String)>, String>
where
    F: FnMut(String) -> Fut,
    Fut: std::future::Future<Output = Result<Option<Value>, String>>,
{
    if c.parent_page_id.is_empty() {
        return Ok(None);
    }
    if numeric_id(&json!(c.parent_page_id)).is_none() {
        return Err("상위 페이지·폴더 ID를 확인해 주세요.".into());
    }
    let space = c.space_id.as_deref().ok_or("Space ID가 없습니다.")?;
    // Existing page connections need no new folder or content-metadata scope.
    // Once detected, use the known type first on publication revalidation.
    let kinds = if c.parent_type.as_deref() == Some("folder") {
        ["folder", "page"]
    } else {
        ["page", "folder"]
    };
    for kind in kinds {
        let Some(content) = fetch(format!("/wiki/api/v2/{kind}s/{}", c.parent_page_id))
            .await
            .map_err(|error| format!("상위 {kind} 조회: {error} 범위 지정 토큰은 read:{kind}:confluence 권한을 확인하세요."))?
        else { continue };
        if numeric_id(&content["id"]) != numeric_id(&json!(c.parent_page_id)) {
            return Err("상위 대상의 ID 응답이 일치하지 않습니다.".into());
        }
        if content["spaceId"].as_str() != Some(space) {
            return Err("상위 페이지·폴더가 선택한 Space에 속하지 않습니다.".into());
        }
        if content["status"].as_str() != Some("current") {
            return Err("게시 가능한 상태의 페이지·폴더를 선택해 주세요. 보관·삭제·초안 상태는 사용할 수 없습니다.".into());
        }
        if content["type"]
            .as_str()
            .is_some_and(|actual| actual != kind)
        {
            return Err("상위 대상은 페이지 또는 폴더여야 합니다.".into());
        }
        let title = content["title"]
            .as_str()
            .ok_or("상위 대상 이름을 읽지 못했습니다.")?;
        return Ok(Some((kind.into(), title.into())));
    }
    Err("상위 페이지·폴더를 찾을 수 없습니다. 같은 Space의 콘텐츠 ID와 읽기 권한을 확인하세요. Space ID나 URL 전체를 입력하는 칸이 아닙니다.".into())
}

async fn resolve_parent(
    client: &Client,
    c: &Connection,
    token: &str,
) -> Result<Option<(String, String)>, String> {
    resolve_parent_with(c, |path| async move {
        read_api_optional(client, c, token, &path).await
    })
    .await
}

async fn verify(input: &ConnectionInput) -> Result<Connection, String> {
    let mut c = validate_input(input)?;
    let client = client()?;
    if c.token_type == "scoped" {
        // The discovery request carries no credentials. Tokens go only to the selected API host.
        let res = client
            .get(format!("{}/_edge/tenant_info", c.site_url))
            .send()
            .await
            .map_err(|_| "Cloud ID를 조회하지 못했습니다.")?;
        if !res.status().is_success() {
            return Err("Cloud ID를 확인할 수 없습니다. 사이트 주소를 확인해 주세요.".into());
        }
        let info: Value = res.json().await.map_err(|_| "Cloud ID 응답 오류")?;
        let id = info["cloudId"]
            .as_str()
            .ok_or("Cloud ID가 응답에 없습니다.")?;
        uuid::Uuid::parse_str(id).map_err(|_| "Cloud ID 형식 오류")?;
        c.cloud_id = Some(id.into());
    }
    if c.kind == "confluence" {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("keys", &c.space_key)
            .finish();
        let data = read_api(
            &client,
            &c,
            input.token.trim(),
            &format!("/wiki/api/v2/spaces?{query}"),
        )
        .await
        .map_err(|error| format!("Confluence 공간 조회: {error}"))?;
        let space = data["results"]
            .as_array()
            .and_then(|xs| {
                xs.iter()
                    .find(|s| s["key"].as_str() == Some(c.space_key.as_str()))
            })
            .ok_or("해당 Space Key를 찾을 수 없거나 읽기 권한이 없습니다.")?;
        c.space_id = Some(space["id"].as_str().ok_or("Space ID가 없습니다.")?.into());
        if let Some((kind, title)) = resolve_parent(&client, &c, input.token.trim()).await? {
            c.parent_type = Some(kind);
            c.parent_title = Some(title);
        }
    } else {
        let data = read_api(
            &client,
            &c,
            input.token.trim(),
            &format!(
                "/rest/api/3/issue/createmeta/{}/issuetypes?maxResults=100",
                c.project_key
            ),
        )
        .await
        .map_err(|error| format!("Jira 프로젝트의 생성 가능한 이슈 유형 조회: {error}"))?;
        let types = data["issueTypes"]
            .as_array()
            .or_else(|| data["values"].as_array())
            .ok_or("이슈 유형을 조회할 수 없습니다. 프로젝트의 이슈 생성 권한을 확인해 주세요.")?;
        c.issue_types = types
            .iter()
            .filter(|x| x["subtask"].as_bool() != Some(true))
            .filter_map(|x| {
                Some(IssueType {
                    id: x["id"].as_str()?.into(),
                    name: x["name"].as_str()?.into(),
                })
            })
            .collect();
        if c.issue_types.is_empty() {
            return Err("생성 가능한 일반 이슈 유형이 없습니다.".into());
        }
        if !c.board_id.is_empty() {
            verify_board(&client, &mut c, input.token.trim()).await?;
        }
    }
    Ok(c)
}

async fn verify_board(client: &Client, c: &mut Connection, token: &str) -> Result<(), String> {
    let path = format!("/rest/agile/1.0/board/{}", c.board_id);
    let board = read_api(client, c, token, &path).await
        .map_err(|error| format!("Jira 보드 조회: {error} 범위 지정 토큰은 read:board-scope:jira-software 권한도 확인하세요."))?;
    c.board_name = Some(
        board["name"]
            .as_str()
            .ok_or("보드 이름을 읽지 못했습니다.")?
            .into(),
    );
    // Board location is not a membership rule: the saved filter may span projects.
    // Fetch the rule for review, but never modify a board or derive issue fields from JQL.
    let filter_result: Result<String, String> = async {
        let config = read_api(client, c, token, &format!("{path}/configuration")).await?;
        let id = numeric_id(&config["filter"]["id"]).ok_or("보드 필터 ID를 읽지 못했습니다.")?;
        let filter = read_api(client, c, token, &format!("/rest/api/3/filter/{id}")).await?;
        Ok(filter["jql"]
            .as_str()
            .ok_or("보드 필터 조건을 읽지 못했습니다.")?
            .into())
    }
    .await;
    match filter_result {
        Ok(jql) => c.board_filter = Some(jql),
        Err(error) => {
            c.board_note = Some(format!(
                "보드는 확인했으나 표시 조건은 조회하지 못했습니다. {error}"
            ))
        }
    }
    Ok(())
}

fn numeric_id(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .filter(|id| *id > 0)
}

#[tauri::command]
pub async fn atlassian_test_connection(input: ConnectionInput) -> Result<Connection, String> {
    verify(&input).await
}

#[tauri::command]
pub async fn atlassian_save_connection<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AtlassianState>,
    input: ConnectionInput,
) -> Result<Connection, String> {
    let c = verify(&input).await?;
    let _lock = state.0.lock().await;
    let mut vault = read_vault(&app)?;
    if vault
        .connections
        .iter()
        .any(|x| x.connection.name == c.name)
    {
        return Err("같은 이름의 연결이 있습니다. 다른 이름을 입력해 주세요.".into());
    }
    let key = vault_key(vault.connections.is_empty())?;
    let encrypted_token = seal(&key, input.token.trim(), &aad(&c))?;
    vault.connections.push(StoredConnection {
        connection: c.clone(),
        encrypted_token,
    });
    write_vault(&app, &vault)?;
    Ok(c)
}

#[tauri::command]
pub async fn atlassian_connections<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AtlassianState>,
) -> Result<Vec<Connection>, String> {
    let _lock = state.0.lock().await;
    Ok(read_vault(&app)?
        .connections
        .into_iter()
        .map(|x| x.connection)
        .collect())
}

async fn jira_components(
    client: &Client,
    c: &Connection,
    token: &str,
) -> Result<Vec<JiraComponent>, String> {
    if c.kind != "jira" {
        return Err("Jira 연결을 선택해 주세요.".into());
    }
    let data = read_api(
        client,
        c,
        token,
        &format!("/rest/api/3/project/{}/components", c.project_key),
    )
    .await?;
    serde_json::from_value(data).map_err(|_| "프로젝트 컴포넌트 목록을 읽지 못했습니다.".into())
}

#[tauri::command]
pub async fn atlassian_jira_components<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AtlassianState>,
    connection_id: String,
) -> Result<Vec<JiraComponent>, String> {
    let (c, token) = {
        let _lock = state.0.lock().await;
        let vault = read_vault(&app)?;
        let stored = vault
            .connections
            .iter()
            .find(|x| x.connection.id == connection_id)
            .ok_or("연결을 찾을 수 없습니다.")?;
        (
            stored.connection.clone(),
            unseal(
                &vault_key(false)?,
                &stored.encrypted_token,
                &aad(&stored.connection),
            )?,
        )
    };
    jira_components(&client()?, &c, &token).await
}

#[tauri::command]
pub async fn atlassian_delete_connection<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AtlassianState>,
    id: String,
) -> Result<(), String> {
    let _lock = state.0.lock().await;
    let mut vault = read_vault(&app)?;
    vault.connections.retain(|x| x.connection.id != id);
    write_vault(&app, &vault)
}

#[tauri::command]
pub async fn atlassian_receipts<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AtlassianState>,
    meeting_id: String,
) -> Result<Vec<Receipt>, String> {
    let _lock = state.0.lock().await;
    Ok(read_vault(&app)?
        .receipts
        .into_iter()
        .filter(|x| x.meeting_id == meeting_id)
        .collect())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn confluence_body(text: &str) -> String {
    // Render the meeting template's headings, lists and tables without accepting
    // raw HTML, link destinations, image embeds or executable model output.
    let lines: Vec<&str> = text.lines().collect();
    let mut html = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with('|') && i + 1 < lines.len() && table_separator(lines[i + 1]) {
            html.push_str("<table><tbody><tr>");
            for cell in table_cells(line) {
                html.push_str(&format!("<th>{}</th>", escape_html(&cell)));
            }
            html.push_str("</tr>");
            i += 2;
            while i < lines.len() && lines[i].trim().starts_with('|') {
                html.push_str("<tr>");
                for cell in table_cells(lines[i]) {
                    html.push_str(&format!("<td>{}</td>", escape_html(&cell)));
                }
                html.push_str("</tr>");
                i += 1;
            }
            html.push_str("</tbody></table>\n");
            continue;
        }
        if line.starts_with("- ") || line.starts_with("* ") {
            html.push_str("<ul>");
            while i < lines.len() {
                let item = lines[i].trim();
                if !(item.starts_with("- ") || item.starts_with("* ")) {
                    break;
                }
                html.push_str(&format!("<li>{}</li>", escape_html(item[2..].trim())));
                i += 1;
            }
            html.push_str("</ul>\n");
            continue;
        }
        let level = line.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&level) && line.as_bytes().get(level) == Some(&b' ') {
            html.push_str(&format!(
                "<h{level}>{}</h{level}>\n",
                escape_html(line[level + 1..].trim())
            ));
        } else if line.len() > 4 && line.starts_with("**") && line.ends_with("**") {
            html.push_str(&format!(
                "<h2>{}</h2>\n",
                escape_html(&line[2..line.len() - 2])
            ));
        } else if !line.is_empty() {
            html.push_str(&format!("<p>{}</p>\n", escape_html(line)));
        }
        i += 1;
    }
    html
}

fn table_cells(line: &str) -> Vec<String> {
    let mut cells = vec![];
    let mut cell = String::new();
    let mut chars = line.trim().trim_matches('|').chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && matches!(chars.peek(), Some('|' | '\\')) {
            cell.push(chars.next().unwrap());
        } else if ch == '|' {
            cells.push(cell.trim().to_owned());
            cell.clear();
        } else {
            cell.push(ch);
        }
    }
    cells.push(cell.trim().to_owned());
    cells
}

fn table_separator(line: &str) -> bool {
    line.trim().starts_with('|')
        && table_cells(line).iter().all(|c| {
            let dashes = c.trim_matches(':');
            dashes.len() >= 3 && dashes.bytes().all(|b| b == b'-')
        })
}

fn jira_document(text: &str) -> Value {
    json!({"version": 1, "type": "doc", "content": text.lines().map(|line| {
        if line.is_empty() { json!({"type":"paragraph", "content":[]}) }
        else { json!({"type":"paragraph", "content":[{"type":"text", "text":line}]}) }
    }).collect::<Vec<_>>()})
}

fn publish_payload(c: &Connection, input: &PublishInput) -> Result<(String, Value), String> {
    if input.meeting_id.trim().is_empty()
        || input.title.trim().is_empty()
        || input.title.chars().count() > 255
        || input.body.trim().is_empty()
        || input.body.len() > 200_000
    {
        return Err(
            "회의·제목·본문을 확인하세요. 제목은 255자, 본문은 200KB 이하여야 합니다.".into(),
        );
    }
    if c.kind == "confluence" {
        let mut payload = json!({"spaceId":c.space_id.as_deref().ok_or("Space ID가 없습니다.")?, "status":"current", "title":input.title.trim(), "body":{"representation":"storage", "value":confluence_body(&input.body)}});
        if !c.parent_page_id.is_empty() {
            payload["parentId"] = json!(c.parent_page_id);
            Ok(("/wiki/api/v2/pages".into(), payload))
        } else {
            // Without this flag Confluence defaults to the space homepage,
            // which differs from the root location shown in the review screen.
            Ok(("/wiki/api/v2/pages?root-level=true".into(), payload))
        }
    } else {
        let issue_type = input
            .issue_type_id
            .as_deref()
            .ok_or("이슈 유형을 선택해 주세요.")?;
        if !c.issue_types.iter().any(|t| t.id == issue_type) {
            return Err("등록된 프로젝트의 이슈 유형을 선택해 주세요.".into());
        }
        let mut payload = json!({"fields":{"project":{"key":c.project_key}, "issuetype":{"id":issue_type}, "summary":input.title.trim(), "description":jira_document(&input.body)}});
        if let Some(id) = &input.component_id {
            if id.is_empty()
                || !id.bytes().all(|c| c.is_ascii_digit())
                || numeric_id(&json!(id)).is_none()
            {
                return Err("프로젝트 목록에서 컴포넌트를 선택해 주세요.".into());
            }
            payload["fields"]["components"] = json!([{"id":id}]);
        }
        Ok(("/rest/api/3/issue".into(), payload))
    }
}

fn publication_key(c: &Connection, input: &PublishInput) -> String {
    let action = if c.kind == "confluence" {
        "page".into()
    } else {
        let mut key = format!(
            "{}|{}|{}",
            input.issue_type_id.as_deref().unwrap_or(""),
            input.title.trim(),
            input.body.trim()
        );
        // Preserve existing receipt keys for publications without a component.
        if let Some(id) = &input.component_id {
            key.push_str(&format!("|component:{id}"));
        }
        key
    };
    format!(
        "{:x}",
        Sha256::digest(format!("{}|{}|{action}", c.id, input.meeting_id).as_bytes())
    )
}

#[tauri::command]
pub async fn atlassian_publish<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AtlassianState>,
    input: PublishInput,
) -> Result<Receipt, String> {
    // Serializes publications and durably reserves a receipt BEFORE network I/O.
    // A lost response must never silently create a second page/issue on retry.
    let _lock = state.0.lock().await;
    let mut vault = read_vault(&app)?;
    let stored = vault
        .connections
        .iter()
        .find(|x| x.connection.id == input.connection_id)
        .ok_or("연결을 찾을 수 없습니다.")?;
    let c = stored.connection.clone();
    let (path, payload) = publish_payload(&c, &input)?;
    let key = publication_key(&c, &input);
    if let Some(previous) = vault.receipts.iter().find(|r| r.key == key) {
        if previous.status == "published" {
            return Ok(previous.clone());
        }
        return Err("이 요청은 이미 전송됐지만 결과가 확정되지 않았습니다. 중복 생성을 막기 위해 재전송을 중단했습니다. Atlassian에서 제목으로 검색해 확인하세요.".into());
    }
    let token = unseal(&vault_key(false)?, &stored.encrypted_token, &aad(&c))?;
    let client = client()?;
    let endpoint = format!("{}{path}", api_base(&c)?);
    if c.kind == "confluence" {
        // Stop before reserving/sending if a parent was moved, archived or hidden.
        resolve_parent(&client, &c, &token).await?;
    }
    if c.kind == "jira" {
        if let Some(id) = &input.component_id {
            if !jira_components(&client, &c, &token)
                .await?
                .iter()
                .any(|component| &component.id == id)
            {
                return Err(
                    "선택한 컴포넌트가 이 프로젝트에 없습니다. 목록을 다시 불러와 주세요.".into(),
                );
            }
        }
    }
    let mut receipt = Receipt {
        key: key.clone(),
        meeting_id: input.meeting_id,
        connection_id: c.id.clone(),
        kind: c.kind.clone(),
        title: input.title,
        url: None,
        status: "needs_review".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    vault.receipts.push(receipt.clone());
    write_vault(&app, &vault)?;
    let response = client.request(Method::POST, endpoint).basic_auth(&c.email, Some(&token)).json(&payload).send().await
        .map_err(|_| "게시 결과를 확인하지 못했습니다. 네트워크 오류가 발생했습니다. 다시 게시하기 전에 Atlassian에서 해당 제목을 검색해 주세요.")?;
    let status = response.status();
    if !status.is_success() {
        // 5xx can occur after a successful create; preserve the reservation then.
        if status.is_client_error() {
            vault.receipts.retain(|r| r.key != key);
            write_vault(&app, &vault)?;
        }
        let body: Value = response.json().await.unwrap_or(Value::Null);
        let fields = body["errors"]
            .as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        return Err(format!(
            "{}{}",
            http_error(status.as_u16()),
            if fields.is_empty() {
                String::new()
            } else {
                format!(" 확인할 필드: {fields}")
            }
        ));
    }
    let result: Value = response.json().await.map_err(|_| {
        "서버가 게시 요청을 수락했지만 결과를 읽지 못했습니다. Atlassian에서 제목으로 확인하세요."
    })?;
    receipt.url = if c.kind == "confluence" {
        let id = result["id"]
            .as_str()
            .filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
            .ok_or("게시 응답에 페이지 ID가 없습니다. Atlassian에서 확인하세요.")?;
        Some(format!(
            "{}/wiki/spaces/{}/pages/{id}",
            c.site_url, c.space_key
        ))
    } else {
        let id = result["key"]
            .as_str()
            .filter(|id| id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
            .ok_or("게시 응답에 이슈 번호가 없습니다. Atlassian에서 확인하세요.")?;
        Some(format!("{}/browse/{id}", c.site_url))
    };
    receipt.status = "published".into();
    if let Some(slot) = vault.receipts.iter_mut().find(|r| r.key == key) {
        *slot = receipt.clone();
    }
    write_vault(&app, &vault)?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parent_connection() -> Connection {
        let mut c = validate_input(&input("confluence")).unwrap();
        c.space_id = Some("100".into());
        c.parent_page_id = "200".into();
        c
    }
    fn parent(kind: &str) -> Value {
        json!({"id":"200", "spaceId":"100", "status":"current", "type":kind, "title":"팀 회의록"})
    }
    #[tokio::test]
    async fn resolves_pages_without_requiring_folder_permissions() {
        let result = resolve_parent_with(&parent_connection(), |path| async move {
            assert_eq!(path, "/wiki/api/v2/pages/200");
            Ok(Some(parent("page")))
        })
        .await
        .unwrap();
        assert_eq!(result, Some(("page".into(), "팀 회의록".into())));
    }
    #[tokio::test]
    async fn resolves_folder_after_missing_page_and_rechecks_known_folder_directly() {
        let mut calls = vec![];
        let mut c = parent_connection();
        let result = resolve_parent_with(&c, |path| {
            calls.push(path.clone());
            async move {
                Ok(if path.contains("/pages/") {
                    None
                } else {
                    Some(parent("folder"))
                })
            }
        })
        .await
        .unwrap();
        assert_eq!(
            calls,
            ["/wiki/api/v2/pages/200", "/wiki/api/v2/folders/200"]
        );
        assert_eq!(result, Some(("folder".into(), "팀 회의록".into())));
        c.parent_type = Some("folder".into());
        resolve_parent_with(&c, |path| async move {
            assert_eq!(path, "/wiki/api/v2/folders/200");
            Ok(Some(parent("folder")))
        })
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn stops_on_access_errors_and_reports_missing_folder_scope() {
        for code in [401, 403, 429, 500] {
            let error = resolve_parent_with(&parent_connection(), |path| async move {
                assert_eq!(path, "/wiki/api/v2/pages/200");
                Err(http_error(code))
            })
            .await
            .unwrap_err();
            assert!(error.contains(&http_error(code)));
        }
        let error = resolve_parent_with(&parent_connection(), |path| async move {
            if path.contains("/pages/") {
                Ok(None)
            } else {
                Err(http_error(403))
            }
        })
        .await
        .unwrap_err();
        assert!(error.contains("read:folder:confluence"));
        assert!(error.contains("403"));
    }
    #[tokio::test]
    async fn rejects_moved_archived_invalid_or_inaccessible_parents() {
        for kind in ["page", "folder"] {
            for (field, invalid) in [
                ("spaceId", "999"),
                ("id", "201"),
                ("status", "archived"),
                ("status", "draft"),
                ("status", "trashed"),
                ("type", "whiteboard"),
            ] {
                let mut c = parent_connection();
                c.parent_type = Some(kind.into());
                let mut value = parent(kind);
                value[field] = json!(invalid);
                assert!(
                    resolve_parent_with(&c, |_| std::future::ready(Ok(Some(value.clone()))))
                        .await
                        .is_err()
                );
            }
        }
        assert!(
            resolve_parent_with(&parent_connection(), |_| std::future::ready(Ok(None)))
                .await
                .is_err()
        );
        let mut c = parent_connection();
        c.parent_page_id.clear();
        assert_eq!(
            resolve_parent_with(&c, |_| async {
                panic!("Space root needs no parent lookup")
            })
            .await
            .unwrap(),
            None
        );
    }
    #[test]
    fn parent_ids_are_numeric_and_old_vaults_and_publication_keys_remain_compatible() {
        let mut form = input("confluence");
        for invalid in [
            "0",
            "-1",
            "18446744073709551616",
            "200/children",
            "https://team.atlassian.net/wiki/folders/200",
        ] {
            form.parent_page_id = invalid.into();
            assert!(validate_input(&form).is_err());
        }
        let c = parent_connection();
        let mut value = serde_json::to_value(&c).unwrap();
        value.as_object_mut().unwrap().remove("parentType");
        value.as_object_mut().unwrap().remove("parentTitle");
        let mut restored: Connection = serde_json::from_value(value).unwrap();
        assert!(restored.parent_type.is_none());
        assert_eq!(aad(&restored), aad(&c));
        let p = PublishInput {
            connection_id: c.id.clone(),
            meeting_id: "m".into(),
            title: "회의록".into(),
            body: "검토한 요약".into(),
            issue_type_id: None,
            component_id: None,
        };
        let original_key = publication_key(&restored, &p);
        restored.parent_type = Some("folder".into());
        restored.parent_title = Some("회의록 폴더".into());
        let (path, body) = publish_payload(&restored, &p).unwrap();
        assert_eq!(path, "/wiki/api/v2/pages");
        assert_eq!(body["parentId"], "200");
        assert_eq!(body["spaceId"], "100");
        assert_eq!(publication_key(&restored, &p), original_key);
        assert_eq!(aad(&restored), aad(&c));
    }
    fn input(kind: &str) -> ConnectionInput {
        ConnectionInput {
            kind: kind.into(),
            name: "회사".into(),
            site_url: "https://team.atlassian.net/wiki/".into(),
            email: "me@example.com".into(),
            token: "SECRET-test-token".into(),
            token_type: "classic".into(),
            space_key: "TEAM".into(),
            parent_page_id: "".into(),
            project_key: "PROJ".into(),
            board_id: "".into(),
        }
    }
    #[test]
    fn refuses_credential_exfiltration_hosts() {
        for bad in [
            "http://team.atlassian.net",
            "https://team.atlassian.net.evil.com",
            "https://evil.com",
            "https://user@team.atlassian.net",
            "https://team.atlassian.net:444",
            "https://team.atlassian.net/?redirect=evil",
            "https://team.atlassian.net/other",
            "https://a.b.atlassian.net",
        ] {
            assert!(normalize_site(bad).is_err(), "{bad}");
        }
        assert_eq!(
            normalize_site("https://team.atlassian.net/wiki/").unwrap(),
            "https://team.atlassian.net"
        );
    }
    #[test]
    fn ciphertext_is_randomized_authenticated_and_bound_to_connection() {
        let key = [7u8; 32];
        let a = seal(&key, "secret", "connection-one").unwrap();
        let b = seal(&key, "secret", "connection-one").unwrap();
        assert_ne!(a, b);
        assert_eq!(unseal(&key, &a, "connection-one").unwrap(), "secret");
        assert!(unseal(&key, &a, "connection-two").is_err());
        assert!(unseal(&[8u8; 32], &a, "connection-one").is_err());
        let mut damaged = STANDARD.decode(a).unwrap();
        damaged[15] ^= 1;
        assert!(unseal(&key, &STANDARD.encode(damaged), "connection-one").is_err());
        assert!(unseal(&key, "AA==", "connection-one").is_err());
    }
    #[test]
    fn public_connection_never_contains_token() {
        let c = validate_input(&input("confluence")).unwrap();
        let public = serde_json::to_string(&c).unwrap();
        assert!(!public.contains("SECRET"));
        assert!(!public.contains("encryptedToken"));
    }
    #[test]
    fn jira_board_ids_cannot_change_request_paths() {
        let mut form = input("jira");
        for invalid in [
            "../other",
            "1234?token=x",
            "0",
            "-1",
            "18446744073709551616",
        ] {
            form.board_id = invalid.into();
            assert!(validate_input(&form).is_err());
        }
        form.board_id = "1234".into();
        assert_eq!(validate_input(&form).unwrap().board_id, "1234");
        assert_eq!(numeric_id(&json!("1001")), Some(1001));
        assert_eq!(numeric_id(&json!(1001)), Some(1001));
        assert_eq!(numeric_id(&json!("1001/other")), None);
    }
    #[test]
    fn existing_connections_without_board_fields_remain_readable() {
        let mut original = serde_json::to_value(validate_input(&input("jira")).unwrap()).unwrap();
        for field in ["boardId", "boardName", "boardFilter", "boardNote"] {
            original.as_object_mut().unwrap().remove(field);
        }
        let restored: Connection = serde_json::from_value(original).unwrap();
        assert!(restored.board_id.is_empty());
        assert_eq!(restored.project_key, "PROJ");
    }
    #[test]
    fn publication_escapes_html_and_cannot_include_extra_raw_fields() {
        let mut c = validate_input(&input("confluence")).unwrap();
        c.space_id = Some("123".into());
        let p = PublishInput {
            connection_id: c.id.clone(),
            meeting_id: "meeting1".into(),
            title: "한국어 회의".into(),
            body: "## 결정\n<script>alert(1)</script>\nA & B".into(),
            issue_type_id: None,
            component_id: None,
        };
        let (path, body) = publish_payload(&c, &p).unwrap();
        assert_eq!(path, "/wiki/api/v2/pages?root-level=true");
        assert!(body.get("parentId").is_none());
        let html = body["body"]["value"].as_str().unwrap();
        assert!(html.contains("<h2>결정</h2>"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&amp;"));
        assert!(body.get("transcript").is_none());
        assert!(body.get("audio").is_none());
    }
    #[test]
    fn scoped_tokens_use_gateway_and_jira_uses_adf() {
        let mut c = validate_input(&input("jira")).unwrap();
        c.token_type = "scoped".into();
        assert!(api_base(&c).is_err());
        c.cloud_id = Some("00000000-0000-0000-0000-000000000001".into());
        assert!(api_base(&c)
            .unwrap()
            .starts_with("https://api.atlassian.com/ex/jira/"));
        c.issue_types.push(IssueType {
            id: "10001".into(),
            name: "Task".into(),
        });
        let p = PublishInput {
            connection_id: c.id.clone(),
            meeting_id: "m".into(),
            title: "확인".into(),
            body: "담당자 미정\n\n원본 링크".into(),
            issue_type_id: Some("10001".into()),
            component_id: None,
        };
        let (_, body) = publish_payload(&c, &p).unwrap();
        assert_eq!(body["fields"]["description"]["version"], 1);
        assert_eq!(body["fields"]["project"]["key"], "PROJ");
        assert!(body["fields"].get("components").is_none());
        let original_key = publication_key(&c, &p);
        let mut p = p;
        p.component_id = Some("10055".into());
        let (_, with_component) = publish_payload(&c, &p).unwrap();
        assert_eq!(
            with_component["fields"]["components"],
            json!([{"id":"10055"}])
        );
        assert_ne!(original_key, publication_key(&c, &p));
        p.component_id = Some("../other".into());
        assert!(publish_payload(&c, &p).is_err());
    }
    #[test]
    fn renders_summary_lists_and_tables_without_remote_embeds() {
        let html = confluence_body("**할 일**\n- 항목 하나\n- 항목 둘\n\n| 담당자 | 할 일 |\n| :--- | --- |\n| 민수 | <img src='https://evil.test'/> |\n## 결정\n확정");
        assert!(html.contains("<h2>할 일</h2>"));
        assert!(html.contains("<ul><li>항목 하나</li><li>항목 둘</li></ul>"));
        assert!(html.contains("<th>담당자</th>"));
        assert!(html.contains("<td>민수</td>"));
        assert!(html.contains("<h2>결정</h2>"));
        assert!(!html.contains("<img"));
        let escaped = confluence_body(
            r"| 대상 | 작업 |
| --- | --- |
| A\|B | <검토> |",
        );
        assert!(escaped.contains("<td>A|B</td><td>&lt;검토&gt;</td>"));
    }
}
