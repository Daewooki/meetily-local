//! Keep the AES key in the current user's Windows Credential Manager.
//! LOCAL_MACHINE persistence survives sign-out on this PC without credential roaming.
use rand::{rngs::OsRng, RngCore};
use windows_sys::Win32::{
    Foundation::{GetLastError, ERROR_NOT_FOUND},
    Security::Credentials::{
        CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    },
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn read_key(target: &[u16]) -> Result<Option<Vec<u8>>, String> {
    let mut credential = std::ptr::null_mut();
    // SAFETY: target is NUL terminated and credential is an initialized out pointer.
    if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) } == 0 {
        return if unsafe { GetLastError() } == ERROR_NOT_FOUND {
            Ok(None)
        } else {
            Err("Windows 자격 증명 관리자에서 암호화 키를 읽을 수 없습니다.".into())
        };
    }
    if credential.is_null() {
        return Err("Windows 자격 증명 관리자가 올바르지 않은 응답을 반환했습니다.".into());
    }
    // SAFETY: CredReadW owns this allocation until CredFree. Validate the blob before copying.
    let result = unsafe {
        let entry = &*credential;
        let result = if entry.CredentialBlobSize == 32 && !entry.CredentialBlob.is_null() {
            Ok(Some(
                std::slice::from_raw_parts(entry.CredentialBlob, 32).to_vec(),
            ))
        } else {
            Err(
                "Windows에 저장된 암호화 키가 올바르지 않습니다. 기존 키를 덮어쓰지 않았습니다."
                    .into(),
            )
        };
        CredFree(credential.cast());
        result
    };
    result
}

fn store_key(target: &mut [u16], key: &mut [u8]) -> Result<(), String> {
    let mut username = wide("aes256-key-v1");
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: target.as_mut_ptr(),
        CredentialBlobSize: key.len() as u32,
        CredentialBlob: key.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: username.as_mut_ptr(),
        // SAFETY: Remaining CREDENTIALW fields accept zero/null for a generic credential.
        ..unsafe { std::mem::zeroed() }
    };
    // SAFETY: all backing buffers are valid for the synchronous call; Windows copies the blob.
    if unsafe { CredWriteW(&credential, 0) } == 0 {
        return Err("Windows 자격 증명 관리자에 암호화 키를 저장할 수 없습니다.".into());
    }
    Ok(())
}

pub(super) fn load_key(name: &str, create: bool) -> Result<Vec<u8>, String> {
    let mut target = wide(name);
    if let Some(key) = read_key(&target)? {
        return Ok(key);
    }
    if !create {
        return Err("Windows 자격 증명 관리자에 기존 연결의 암호화 키가 없습니다. 연결을 다시 등록해 주세요.".into());
    }
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    store_key(&mut target, &mut key)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Security::Credentials::CredDeleteW;

    struct TestCredential(String);
    impl TestCredential {
        fn new() -> Self {
            Self(format!("ai.meetily.local.test/{}", uuid::Uuid::new_v4()))
        }
    }
    impl Drop for TestCredential {
        fn drop(&mut self) {
            unsafe {
                CredDeleteW(wide(&self.0).as_ptr(), CRED_TYPE_GENERIC, 0);
            }
        }
    }

    #[test]
    fn windows_credential_roundtrip_preserves_key_and_decrypts_token() {
        let target = TestCredential::new();
        assert!(load_key(&target.0, false).is_err());
        assert!(read_key(&wide(&target.0)).unwrap().is_none());
        let key = load_key(&target.0, true).unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(load_key(&target.0, true).unwrap(), key);
        let encrypted = super::super::seal(&key, "synthetic-token", "test-binding").unwrap();
        let reloaded = load_key(&target.0, false).unwrap();
        assert_eq!(
            super::super::unseal(&reloaded, &encrypted, "test-binding").unwrap(),
            "synthetic-token"
        );
    }

    #[test]
    fn malformed_credential_is_never_replaced() {
        let target = TestCredential::new();
        store_key(&mut wide(&target.0), &mut [7u8; 8]).unwrap();
        assert!(load_key(&target.0, true).is_err());
        assert!(load_key(&target.0, false).is_err());
    }
}
