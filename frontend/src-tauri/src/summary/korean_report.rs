//! Deterministic rendering for the standard Korean template. The model extracts
//! fields; it does not control headings, table columns, or unknown-value labels.
use super::templates::Template;
use serde::Deserialize;

pub(super) fn supports(template: &Template) -> bool {
    // A custom template with the same headings must still retain its custom
    // instructions. Only the unchanged bundled standard template uses this schema.
    let standard: serde_json::Value =
        serde_json::from_str(include_str!("../../templates/standard_meeting.json"))
            .expect("bundled standard template must be valid JSON");
    serde_json::to_value(template).is_ok_and(|value| value == standard)
}

pub(super) const SCHEMA: &str = r#"
출력은 Markdown 대신 다음 JSON 객체 하나로 작성하세요. 키 이름과 자료형을 그대로 지키세요. 코드 블록 없이 JSON만 출력하세요.
{
  "title": "실제 회의 주제",
  "summary": "전체 결론과 현재 상태를 두 문장 이내로 요약",
  "topics": ["중요한 논의와 수치"],
  "decisions": [{"text": "명시적으로 합의한 결정", "evidence": ["S1"]}],
  "actions": [{"task": "실제로 수행하기로 한 구체적인 작업", "owner": "담당자 이름 또는 null", "due": "그 작업의 기한 또는 null", "evidence": ["S2", "S3"]}],
  "unresolved": ["미정·보류·미승인·추가 확인이 필요한 내용"]
}
원칙:
1. 각 배열에 해당 내용이 없으면 []입니다. 빈 항목이나 '없음' 행을 만들지 마세요.
2. 담당자와 기한이 원문에 명시되어 있으면 owner와 due에 각각 추출하세요. 없으면 문자열이 아닌 JSON null입니다.
3. evidence는 근거 문장 앞에 표시된 S번호의 배열입니다. 원문을 베껴 쓰지 마세요. 실제 자료에 있는 번호만 선택하세요. 담당자와 기한을 뒷받침하는 문장이 여러 개면 해당 번호를 모두 포함하세요. 예시 번호를 무조건 복사하지 마세요.
4. 운영 원칙·현재 상태·단순 후보·조건부 재논의는 별도 실행 작업으로 만들지 마세요.
5. decisions에는 회의 종료 시점의 최종 상태만 남기세요. 뒤에서 취소·변경된 옛 계획을 유효한 결정처럼 함께 나열하지 마세요. 예전 계획과 취소가 모두 있으면 취소 결정만 기록합니다. 취소된 작업은 actions에서도 제외합니다. 새로 정해진 담당자와 작업은 보존합니다.
6. 이 예시의 설명 문구를 값으로 복사하지 마세요. summary에는 작업 기한을 새로 풀어 해석하지 마세요.
7. 실행 작업을 하기로 합의한 경우 decisions와 actions 양쪽에 기록하세요. 두 배열은 서로 배타적이지 않습니다. 담당자와 기한이 있는 구체적인 작업을 decisions에만 쓰고 actions에서 누락하지 마세요.
8. unresolved에는 원문에서 실제로 미정·보류·미승인이라고 한 항목만 넣으세요. 취소된 작업을 보고 '새 담당자 미정' 같은 새로운 미결 사항을 만들어내지 마세요.
"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fact {
    text: String,
    evidence: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    task: String,
    owner: Option<String>,
    due: Option<String>,
    evidence: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    title: String,
    summary: String,
    topics: Vec<String>,
    decisions: Vec<Fact>,
    actions: Vec<Action>,
    unresolved: Vec<String>,
}

fn normalized(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn known(value: Option<&str>) -> Option<&str> {
    value
        .map(str::trim)
        .filter(|s| !matches!(*s, "" | "-" | "미정" | "미지" | "없음" | "null"))
}

fn cell(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn source_sentences(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        let end = index + ch.len_utf8();
        let boundary = ch == '\n'
            || (matches!(ch, '.' | '?' | '!' | '。')
                && text[end..].chars().next().is_none_or(char::is_whitespace));
        if boundary {
            let sentence = text[start..end].trim();
            if !sentence.is_empty() {
                result.push(sentence);
            }
            start = end;
        }
    }
    if !text[start..].trim().is_empty() {
        result.push(text[start..].trim());
    }
    // Keep an explicitly labelled speaker turn intact so "제가 ... 할게요"
    // retains the speaker name needed to substantiate its owner field.
    static SPEAKER: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^(?:[가-힣]{2,4}|[A-Za-z]{2,16}):\s").unwrap()
    });
    let mut grouped = Vec::new();
    let mut cursor = 0;
    let mut turn_start = 0;
    let mut labelled = false;
    for sentence in result {
        let start = cursor + text[cursor..].find(sentence).expect("ordered source slice");
        let end = start + sentence.len();
        let new_speaker = SPEAKER.is_match(sentence);
        let continuation = labelled
            && !new_speaker
            && !text[cursor..start].contains('\n')
            && end - turn_start <= 2048;
        if continuation {
            *grouped.last_mut().expect("labelled turn exists") = &text[turn_start..end];
        } else {
            turn_start = start;
            grouped.push(sentence);
        }
        labelled = new_speaker || continuation;
        cursor = end;
    }
    grouped
}

pub(super) fn numbered_source(text: &str) -> String {
    source_sentences(text)
        .iter()
        .enumerate()
        .map(|(i, s)| format!("[S{}] {s}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolve_evidence(ids: &[String], sentences: &[&str]) -> Result<String, String> {
    if ids.is_empty() {
        return Err("결정과 할 일마다 근거 문장의 S번호를 선택하세요.".into());
    }
    ids.iter()
        .map(|id| {
            id.trim()
                .strip_prefix('S')
                .and_then(|s| s.parse::<usize>().ok())
                .and_then(|i| i.checked_sub(1))
                .and_then(|i| sentences.get(i).copied())
                .ok_or_else(|| "evidence에는 실제 자료에 표시된 S번호만 넣으세요.".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|items| items.join(" "))
}

pub(super) fn render(raw: &str, original: &str) -> Result<String, String> {
    let raw = raw.trim();
    let raw = raw
        .strip_prefix("```json")
        .and_then(|s| s.trim().strip_suffix("```"))
        .unwrap_or(raw)
        .trim();
    let report: Report =
        serde_json::from_str(raw).map_err(|_| "정해진 JSON 구조로 작성해 주세요.")?;
    if report.title.trim().is_empty() || report.summary.trim().is_empty() {
        return Err("회의 제목과 요약을 작성해 주세요.".into());
    }
    let sentences = source_sentences(original);
    for decision in &report.decisions {
        if decision.text.trim().is_empty() {
            return Err("빈 결정 항목은 제외해 주세요.".into());
        }
        resolve_evidence(&decision.evidence, &sentences)?;
    }
    for action in &report.actions {
        if action.task.trim().is_empty() {
            return Err("빈 할 일 항목은 제외해 주세요.".into());
        }
        let evidence = resolve_evidence(&action.evidence, &sentences)?;
        for value in [known(action.owner.as_deref()), known(action.due.as_deref())]
            .into_iter()
            .flatten()
        {
            if !normalized(&evidence).contains(&normalized(value)) {
                return Err("담당자와 기한이 실제로 나온 문장의 S번호를 evidence에 포함하세요. 값은 그 문장에 나온 표현 그대로 쓰고, 미언급 값은 null입니다.".into());
            }
        }
    }
    let bullets = |items: &[String], empty: &str| {
        if items.is_empty() {
            empty.to_owned()
        } else {
            items
                .iter()
                .map(|s| format!("- {}", cell(s)))
                .collect::<Vec<_>>()
                .join("\n")
        }
    };
    let decisions = report
        .decisions
        .iter()
        .map(|f| f.text.clone())
        .collect::<Vec<_>>();
    // A separate model-written synopsis can turn an agreed future task into a
    // completed action. Build the visible overview from the checked fields so
    // it cannot introduce an extra status, assignee, or deadline.
    let mut overview = Vec::new();
    if !decisions.is_empty() {
        overview.push(format!(
            "합의된 내용: {}",
            decisions
                .iter()
                .take(2)
                .map(|s| cell(s))
                .collect::<Vec<_>>()
                .join(" / ")
        ));
    }
    if !report.actions.is_empty() {
        overview.push(format!(
            "할 일: {}",
            report
                .actions
                .iter()
                .take(3)
                .map(|a| format!(
                    "{} — {} (기한: {})",
                    cell(known(a.owner.as_deref()).unwrap_or("미정")),
                    cell(&a.task),
                    cell(known(a.due.as_deref()).unwrap_or("미정"))
                ))
                .collect::<Vec<_>>()
                .join(" / ")
        ));
    }
    if let Some(pending) = report.unresolved.first() {
        overview.push(format!("미결: {}", cell(pending)));
    }
    if overview.is_empty() {
        overview.push(format!(
            "논의 내용: {}",
            bullets(&report.topics, "확인된 내용 없음")
        ));
    }
    let actions = if report.actions.is_empty() {
        "확인된 할 일 없음".to_owned()
    } else {
        let rows = report
            .actions
            .iter()
            .map(|a| {
                format!(
                    "| {} | {} | {} | {} |",
                    cell(&a.task),
                    cell(known(a.owner.as_deref()).unwrap_or("미정")),
                    cell(known(a.due.as_deref()).unwrap_or("미정")),
                    cell(
                        &resolve_evidence(&a.evidence, &sentences)
                            .expect("references validated above")
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("| 할 일 | 담당자 | 기한 | 근거 |\n| --- | --- | --- | --- |\n{rows}")
    };
    Ok(format!("# {}\n\n## AI 요약\n{}\n\n## 핵심 내용\n{}\n\n## 결정 사항\n{}\n\n## 할 일\n{}\n\n## 미결 사항\n{}",
        cell(&report.title), overview.join("\n\n"), bullets(&report.topics,"확인된 내용 없음"),
        bullets(&decisions,"확인된 결정 없음"), actions, bullets(&report.unresolved,"없음")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> serde_json::Value {
        serde_json::json!({"title":"시험 운영", "summary":"권한을 확인합니다.", "topics":[],
            "decisions":[], "actions":[{"task":"권한 확인","owner":"지영","due":null,
            "evidence":["S1"]}],"unresolved":[]})
    }

    #[test]
    fn custom_section_instructions_keep_the_custom_template_path() {
        let mut template: Template =
            serde_json::from_str(include_str!("../../templates/standard_meeting.json")).unwrap();
        assert!(supports(&template));
        template.sections[0].instruction = "사용자가 지정한 별도 형식으로 정리".into();
        assert!(!supports(&template));
    }

    #[test]
    fn renderer_keeps_columns_and_unknown_labels_stable() {
        let result = render(&sample().to_string(), "지영은 권한을 확인합니다.").unwrap();
        assert!(result.contains("| 권한 확인 | 지영 | 미정 |"));
        assert!(result.contains("## 결정 사항\n확인된 결정 없음"));
    }

    #[test]
    fn evidence_ids_preserve_original_sentences_and_decimal_values() {
        let text = "[00:00] P95는 2.4초입니다. SDK v2.7.1을 확인합니다.\n지영: 확인하겠습니다.";
        let parts = source_sentences(text);
        assert_eq!(parts.len(), 3);
        assert_eq!(
            resolve_evidence(&["S1".into(), "S3".into()], &parts).unwrap(),
            "[00:00] P95는 2.4초입니다. 지영: 확인하겠습니다."
        );
        assert!(resolve_evidence(&["S0".into()], &parts).is_err());
    }

    #[test]
    fn first_person_commitment_keeps_its_explicit_speaker() {
        let source =
            "지영: 네, 일정은 확정입니다. 제가 시나리오를 정리할게요. 민수: 기한은 미정입니다.";
        let parts = source_sentences(source);
        assert_eq!(parts.len(), 2);
        assert!(parts[0].starts_with("지영:"));
        assert!(parts[0].contains("제가 시나리오를 정리할게요."));
        assert!(parts[1].starts_with("민수:"));
    }

    #[test]
    fn invented_evidence_or_unsubstantiated_deadline_is_rejected() {
        let source = "지영은 권한을 확인합니다.";
        let mut sample = sample();
        sample["actions"][0]["due"] = "금요일".into();
        assert!(render(&sample.to_string(), source).is_err());
        sample["actions"][0]["due"] = serde_json::Value::Null;
        sample["actions"][0]["evidence"] = serde_json::json!(["S999"]);
        assert!(render(&sample.to_string(), source).is_err());
    }

    #[test]
    fn empty_actions_do_not_become_a_fabricated_task_row() {
        let mut sample = sample();
        sample["actions"] = serde_json::json!([]);
        let result = render(&sample.to_string(), "작업 없음").unwrap();
        assert!(result.contains("## 할 일\n확인된 할 일 없음"));
        assert!(!result.contains("| 할 일 |"));
    }

    #[test]
    fn overview_cannot_add_a_completed_status_from_model_prose() {
        let mut sample = sample();
        sample["summary"] = "민수가 오늘 작업을 완료했습니다.".into();
        let result = render(&sample.to_string(), "지영은 권한을 확인합니다.").unwrap();
        assert!(!result.contains("민수"));
        assert!(!result.contains("완료"));
        assert!(result.contains("지영 — 권한 확인 (기한: 미정)"));
    }
}
