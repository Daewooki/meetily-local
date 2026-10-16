//! Opt-in local model evaluation. Fixtures are synthetic; no network or meeting DB is used.
use super::{llm_client::LLMProvider, processor::generate_meeting_summary, templates::Template};
use std::{path::PathBuf, time::Instant};

#[tokio::test]
#[ignore = "requires a locally installed model and MEETILY_EVAL_DATA_DIR/MEETILY_EVAL_OUTPUT"]
async fn evaluate_korean_summary_corpus() {
    let data = PathBuf::from(std::env::var("MEETILY_EVAL_DATA_DIR").unwrap());
    let output = PathBuf::from(std::env::var("MEETILY_EVAL_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/korean-summary-corpus.json"
    ))
    .unwrap();
    let selected = std::env::var("MEETILY_EVAL_CASES").unwrap_or_default();
    let template: Template =
        serde_json::from_str(include_str!("../../templates/standard_meeting.json")).unwrap();
    for case in fixtures.as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        if !selected.is_empty() && !selected.split(',').any(|s| s == id) {
            continue;
        }
        let text = case["transcript"].as_str().unwrap().to_owned()
            + &case["filler"]
                .as_str()
                .unwrap_or("")
                .repeat(case["repeats"].as_u64().unwrap_or(0) as usize)
            + case["ending"].as_str().unwrap_or("");
        let started = Instant::now();
        let result = generate_meeting_summary(
            &reqwest::Client::new(),
            &LLMProvider::BuiltInAI,
            "qwen3.5:4b",
            "",
            &text,
            "",
            "standard_meeting",
            &template,
            32468,
            None,
            None,
            None,
            None,
            None,
            Some(&data),
            None,
            Some("ko"),
            Some("ko"),
            None,
        )
        .await;
        let elapsed = started.elapsed().as_secs_f64();
        let value = match result {
            Ok(summary) => serde_json::json!({"id":id,"seconds":elapsed,
                "chunks":summary.successful_chunk_count,"markdown":summary.final_markdown,
                "english_markdown":summary.english_markdown}),
            Err(error) => serde_json::json!({"id":id,"seconds":elapsed,"error":error}),
        };
        std::fs::write(
            output.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
        println!(
            "{id}: {elapsed:.1}s; success={}",
            value.get("error").is_none()
        );
        assert!(
            value.get("error").is_none(),
            "Local evaluation failed: {id}"
        );
    }
    super::summary_engine::client::force_shutdown_sidecar()
        .await
        .unwrap();
}
