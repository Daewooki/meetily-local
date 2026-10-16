//! Only the reviewed, bundled FFmpeg is used. Never download an executable at runtime.
use once_cell::sync::Lazy;
use std::path::PathBuf;

static FFMPEG_PATH: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let executable = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let directory = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bundled = directory.join(executable);
    if bundled.is_file() {
        return Some(bundled);
    }
    #[cfg(debug_assertions)]
    {
        let dev =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/ffmpeg-aarch64-apple-darwin");
        if cfg!(target_os = "macos") && dev.is_file() {
            return Some(dev);
        }
    }
    log::error!("Bundled FFmpeg is missing. Reinstall the complete Meetily Local app.");
    None
});

pub fn find_ffmpeg_path() -> Option<PathBuf> {
    FFMPEG_PATH.clone()
}
