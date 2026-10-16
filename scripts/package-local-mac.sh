#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || exit 1
version=$(python3 -c 'import json;print(json.load(open("frontend/src-tauri/tauri.conf.json"))["version"])')
app='target/release/bundle/macos/Meetily Local.app'
[[ -d "$app" ]] || { echo 'Build the application first.' >&2; exit 1; }
codesign --verify --deep --strict "$app"
python3 scripts/check-release-source.py
python3 - "$app" "$version" <<'PY'
from pathlib import Path
import plistlib, sys
app=Path(sys.argv[1])
info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
assert info['CFBundleShortVersionString']==sys.argv[2], 'Build version mismatch'
for p in app.rglob('*'):
    assert p.name != 'atlassian-vault.json' and p.suffix not in {'.sqlite','.db','.gguf','.wav','.mp4','.mp3','.m4a','.flac','.p12','.pfx','.key'}, 'User data/model in app'
assert (app/'Contents/Resources/licenses/Meetily-LICENSE.txt').is_file(), 'Missing notices'
PY
out="release-artifacts/$version"
mkdir -p "$out"
stage=$(mktemp -d "$(pwd)/$out/.package.XXXXXX")
trap 'rm -rf "$stage"' EXIT
ditto "$app" "$stage/Meetily Local.app"
ln -s /Applications "$stage/Applications"
cp LOCAL-MAC.md "$stage/설치와 사용 안내.md"
base="Meetily-Local-${version}-macos-arm64"
hdiutil create -ov -volname "Meetily Local ${version}" -srcfolder "$stage" -format UDZO "$out/${base}.dmg"
ditto -c -k --sequesterRsrc --keepParent "$app" "$out/${base}.zip"
cp target/release-deps/ffmpeg-8.0.3.tar.xz "$out/"
cp scripts/build-ffmpeg-local-mac.sh "$out/"
cp THIRD_PARTY_NOTICES.md "$out/"
python3 - "$out" <<'PY'
import hashlib, sys
from pathlib import Path
root=Path(sys.argv[1])
files=sorted(p for p in root.iterdir() if p.is_file() and p.name!='SHA256SUMS.txt')
with (root/'SHA256SUMS.txt').open('w') as output:
    for p in files:
        output.write(f'{hashlib.file_digest(p.open("rb"),"sha256").hexdigest()}  {p.name}\n')
print('Packaged:', ', '.join(p.name for p in files))
PY
