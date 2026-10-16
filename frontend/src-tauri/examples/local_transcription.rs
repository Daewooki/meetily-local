//! Repeatable offline transcription check using the same engine as the desktop app.
//! cargo run -p meetily --release --example local_transcription -- <audio-file> <models-dir>
use app_lib::{audio::decoder::decode_audio_file, whisper_engine::WhisperEngine};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let audio = PathBuf::from(args.next().expect("audio file path required"));
    let models = PathBuf::from(args.next().expect("local models directory required"));
    let engine = WhisperEngine::new_with_models_dir(Some(models))?;
    engine.discover_models().await?;
    engine.load_model("large-v3-turbo-q5_0").await?;
    let decoded = decode_audio_file(&audio)?;
    let started = std::time::Instant::now();
    let transcript = engine
        .transcribe_audio(decoded.to_whisper_format(), Some("ko".into()))
        .await?;
    println!(
        "{}",
        serde_json::json!({"transcript": transcript, "audioSeconds": decoded.duration_seconds, "transcriptionSeconds": started.elapsed().as_secs_f64()})
    );
    anyhow::ensure!(!transcript.trim().is_empty(), "Transcription was empty");
    Ok(())
}
