# Third-party notices

Meetily Local is based on Meetily Community v0.4.1, commit a2cb62e827da7ef59f65064c97233efb2313878e, Copyright (c) 2024 Zackriya Solutions. The original MIT license is preserved in LICENSE.md and in the application bundle.

## FFmpeg

This product uses FFmpeg 8.0.3 as a separate executable, licensed under LGPL-2.1-or-later. It is built from the unmodified official release source, without GPL/nonfree components or external codec libraries. Only local file and pipe protocols are enabled; network protocols are disabled.

- Official source: https://ffmpeg.org/releases/ffmpeg-8.0.3.tar.xz
- SHA-256: 6136812ea6d4e68bdba27e33c2a94382711cdf4f8602ffef056ff792bd6f9818
- Official release signature was verified with fingerprint FCF986EA15E6E293A5644F10B4322F04D67658D8.
- Corresponding source must accompany each application release as `ffmpeg-8.0.3.tar.xz`, together with the build script below.
- This distribution's [v1.0.0 release](https://github.com/Daewooki/meetily-local/releases/tag/v1.0.0) provides that corresponding source and build script alongside the application.
- Build configuration and instructions: `scripts/build-ffmpeg-local-mac.sh`. No source patches are applied.
- License: `frontend/src-tauri/licenses/FFmpeg-LGPL-2.1.txt`.

Users may modify and rebuild this separate executable from the provided source and rebuild/sign the application bundle with their replacement. No additional restriction on modification or reverse engineering of this component is imposed here.

## Other software

Rust and JavaScript dependency license declarations and available license/notice texts are collected in `frontend/src-tauri/licenses/dependency-manifest.json` and `DEPENDENCY-NOTICES.txt`. These include build dependencies and platform alternatives in addition to the runtime dependencies. Individual upstream texts remain authoritative. The release includes these notices in `Meetily Local.app/Contents/Resources/licenses`.

Whisper.cpp and llama.cpp are used through their Rust bindings. Their MIT notices, along with vendored component notices found in the installed source packages, are included in the collected notices.

## Models

The installer does not contain model weights. They are downloaded separately when the user requests local model setup. The models and their source model cards are:

- Whisper weights: https://huggingface.co/ggerganov/whisper.cpp (upstream OpenAI Whisper, MIT)
- Qwen 3.5 4B quantization: https://huggingface.co/unsloth/Qwen3.5-4B-GGUF (Qwen3.5-4B, Apache-2.0)

The model publishers' terms and notices accompany their model cards. Model downloads do not transmit meeting audio or transcripts.

## Local security backport

`@tiptap/core` 2.27.2 is distributed with a key-validation backport in `frontend/patches/@tiptap__core@2.27.2.patch` for GHSA-cp6q-959q-f8rh. Its MIT license and copyright notice are preserved. The patch rejects prototype-manipulating attribute keys and is covered by `frontend/tests/lib/editor-security.test.cjs`.
