#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != Darwin || "$(uname -m)" != arm64 ]]; then
  echo "This build script currently targets Apple Silicon Macs." >&2
  exit 1
fi
command -v cargo >/dev/null
command -v cmake >/dev/null
command -v node >/dev/null
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
export NEXT_TELEMETRY_DISABLED=1
export MACOSX_DEPLOYMENT_TARGET=14.2
bash scripts/build-ffmpeg-local-mac.sh
cargo build --locked -p llama-helper --release --features metal
mkdir -p frontend/src-tauri/binaries
cp target/release/llama-helper frontend/src-tauri/binaries/llama-helper-aarch64-apple-darwin
cd frontend
npx --yes pnpm@10.11.0 install --frozen-lockfile
cd ..
python3 scripts/collect-license-notices.py
cd frontend
./node_modules/.bin/tauri build --bundles app
