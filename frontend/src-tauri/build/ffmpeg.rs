/// Require an explicitly prepared sidecar; build.rs does not download executables.
pub fn ensure_ffmpeg_binary() {
    let target = std::env::var("TARGET").expect("TARGET is required");
    let suffix = if target.contains("windows") {
        ".exe"
    } else {
        ""
    };
    let path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(format!("binaries/ffmpeg-{target}{suffix}"));
    println!("cargo:rerun-if-changed={}", path.display());
    let output = std::process::Command::new(&path)
        .arg("-version")
        .output()
        .expect(
            "Prepare the local sidecar with scripts/build-ffmpeg-local-mac.sh before building.",
        );
    assert!(
        output.status.success(),
        "The prepared FFmpeg executable did not run."
    );
}
