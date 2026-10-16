#!/usr/bin/env python3
"""Collect installed dependency notices for the locally built application."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
dest = root / 'frontend/src-tauri/licenses'
dest.mkdir(exist_ok=True)
groups = {}
manifest = []

def collect(name, version, declared, directory):
    manifest.append({'name': name, 'version': version, 'license': declared})
    # Preserve upstream text verbatim; native vendored dependencies also carry notices.
    for path in directory.rglob('*'):
        relative = path.relative_to(directory)
        if len(relative.parts) > 4 or 'node_modules' in relative.parts:
            continue
        if not path.is_file() or not path.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE')):
            continue
        if path.stat().st_size > 2_000_000:
            continue
        try:
            text = path.read_text()
        except (UnicodeError, OSError):
            continue
        digest = hashlib.sha256(text.encode()).hexdigest()
        item = groups.setdefault(digest, {'text': text, 'packages': set()})
        item['packages'].add(f'{name}@{version} — {relative}')

meta = json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--locked','--filter-platform','aarch64-apple-darwin'],cwd=root))
for pkg in meta['packages']:
    if pkg['source']:
        collect('cargo:'+pkg['name'],pkg['version'],pkg.get('license'),Path(pkg['manifest_path']).parent)
for package in sorted((root/'frontend/node_modules/.pnpm').glob('*/node_modules/*/package.json')) + sorted((root/'frontend/node_modules/.pnpm').glob('*/node_modules/@*/*/package.json')):
    data = json.loads(package.read_text())
    collect('npm:'+data['name'],data['version'],data.get('license'),package.parent)
header = 'Installed dependency notices for Meetily Local. Includes build dependencies and platform alternatives. Individual license texts remain authoritative.\n\n'
chunks = [header]
for item in sorted(groups.values(),key=lambda x:sorted(x['packages'])[0]):
    chunks.append('\n'+'='*78+'\n'+'\n'.join(sorted(item['packages']))+'\n\n'+item['text']+'\n')
(dest/'DEPENDENCY-NOTICES.txt').write_text(''.join(chunks))
(dest/'dependency-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
shutil.copyfile(root/'LICENSE.md',dest/'Meetily-LICENSE.txt')
shutil.copyfile(root/'target/release-deps/ffmpeg-8.0.3/COPYING.LGPLv2.1',dest/'FFmpeg-LGPL-2.1.txt')
shutil.copyfile(root/'THIRD_PARTY_NOTICES.md',dest/'THIRD_PARTY_NOTICES.md')
print(f'Collected {len(manifest)} dependency records and {len(groups)} unique notice texts.')
