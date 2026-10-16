#!/usr/bin/env python3
"""Download the two local models; verify their SHA-256 against HF LFS metadata."""
import hashlib
import json
from pathlib import Path
import subprocess
import urllib.request

root = Path.home() / 'Library/Application Support/ai.meetily.local/models'
models = [
    ('ggerganov/whisper.cpp', 'ggml-large-v3-turbo-q5_0.bin', root),
    ('unsloth/Qwen3.5-4B-GGUF', 'Qwen3.5-4B-Q4_K_M.gguf', root / 'summary'),
]

def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        while chunk := f.read(8 * 1024 * 1024):
            h.update(chunk)
    return h.hexdigest()

for repo, name, directory in models:
    with urllib.request.urlopen(f'https://huggingface.co/api/models/{repo}/tree/main', timeout=60) as response:
        entries = json.load(response)
    meta = next(item for item in entries if item['path'] == name)
    expected = meta['lfs']['oid']
    target = directory / name
    directory.mkdir(parents=True, exist_ok=True)
    if target.exists() and digest(target) == expected:
        print(f'Verified existing model: {name}', flush=True)
        continue
    if target.exists():
        raise SystemExit(f'Existing model checksum mismatch; preserved for inspection: {target}')
    partial = target.with_suffix(target.suffix + '.partial')
    print(f'Downloading {name} ({meta["size"] / 1024**3:.2f} GiB)', flush=True)
    subprocess.run(['curl', '--fail', '--location', '--silent', '--show-error', '--retry', '3', '--continue-at', '-', '--output', str(partial), f'https://huggingface.co/{repo}/resolve/main/{name}'], check=True)
    actual = digest(partial)
    if actual != expected:
        raise SystemExit(f'Checksum mismatch: {name}. Incomplete file retained; model NOT installed.')
    partial.replace(target)
    print(f'Installed and SHA-256 verified: {name}', flush=True)
