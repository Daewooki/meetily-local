#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
[[ "${MSYSTEM:-}" == MINGW64 ]] || { echo 'Run this in the MSYS2 MINGW64 shell.' >&2; exit 1; }
release_version=8.0.3
archive="target/release-deps/ffmpeg-${release_version}.tar.xz"
expected=6136812ea6d4e68bdba27e33c2a94382711cdf4f8602ffef056ff792bd6f9818
mkdir -p target/release-deps frontend/src-tauri/binaries
if [[ ! -f "$archive" ]]; then
  curl --fail --location --retry 3 "https://ffmpeg.org/releases/ffmpeg-${release_version}.tar.xz" --output "$archive.partial"
  mv "$archive.partial" "$archive"
fi
actual=$(sha256sum "$archive" | cut -d ' ' -f 1)
[[ "$actual" == "$expected" ]] || { echo 'FFmpeg source checksum mismatch.' >&2; exit 1; }
source_dir="$(pwd)/target/release-deps/ffmpeg-${release_version}"
if [[ ! -f "$source_dir/configure" ]]; then tar -xf "$archive" -C target/release-deps; fi
output="$(pwd)/frontend/src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe"
(
  cd "$source_dir"
  ./configure --arch=x86_64 --target-os=mingw32 --cc=gcc --enable-static --disable-shared --extra-ldflags=-static \
    --disable-autodetect --disable-gpl --disable-nonfree --disable-network --disable-doc --disable-debug \
    --disable-ffplay --disable-ffprobe --disable-everything --enable-small --enable-ffmpeg \
    --enable-protocol=file,pipe \
    --enable-demuxer=mov,matroska,asf,wav,mp3,flac,ogg,aac,concat,pcm_f32le \
    --enable-decoder=aac,aac_fixed,ac3,eac3,alac,mp3,mp3float,mp3adu,mp3adufloat,mp2,mp2float,flac,vorbis,opus,wmav1,wmav2,wmapro,wmalossless,pcm_f32le,pcm_f64le,pcm_s16le,pcm_s24le,pcm_s32le,pcm_u8,pcm_s16be,pcm_s24be,pcm_s32be \
    --enable-parser=aac,aac_latm,ac3,flac,mpegaudio,opus,vorbis \
    --enable-encoder=aac,pcm_s16le --enable-muxer=mp4,mov,ipod,wav,adts \
    --enable-filter=aresample,aformat,anull,abuffer,abuffersink,atrim
  make -j "${CARGO_BUILD_JOBS:-4}" ffmpeg.exe
  cp ffmpeg.exe "$output"
)
chmod 755 "$output"
"$output" -version
# Fail if the supposedly standalone executable needs a MinGW runtime DLL.
if objdump -p "$output" | grep -Ei 'DLL Name:.*(libgcc|libstdc|libwinpthread|msys-)'; then
  echo 'FFmpeg has an undeclared runtime DLL dependency.' >&2
  exit 1
fi
