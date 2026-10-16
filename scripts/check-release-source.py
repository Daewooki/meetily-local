#!/usr/bin/env python3
"""Fail on obvious credentials or user-data files in the proposed source snapshot.

Reports paths and rule names only, never matched secret values. This is a release
guard, not a substitute for reviewing changes or a comprehensive secret scanner.
"""
from pathlib import Path
import hashlib
import re
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
paths = subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=root).decode().split('\0')
rules = {
    'Atlassian token': re.compile(rb'ATATT[0-9A-Za-z_+=/\-]{40,}'),
    'GitHub token': re.compile(rb'gh[pousr]_[0-9A-Za-z]{30,}|github_pat_[0-9A-Za-z_]{50,}'),
    'private key': re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----'),
}
failures = []
count = 0
# Exact upstream audio-decoder fixture, not a user recording. A changed file
# must be reviewed again rather than exempting the whole fixtures directory.
reviewed_media = {
    'frontend/src-tauri/tests/fixtures/he_aac_48k_5s.m4a':
        'ca0d4d2c4dbaf5933e4e2edd73f3ac0e5c9a5823b7bb34a090415039035102a2',
}
for name in sorted(set(paths)):
    p = root / name
    if not name or not p.is_file(): continue
    count += 1
    data = p.read_bytes()
    reviewed = name in reviewed_media and hashlib.sha256(data).hexdigest() == reviewed_media[name]
    if (p.name == 'atlassian-vault.json' or p.suffix in {'.db','.sqlite','.sqlite3','.p12','.pfx','.key','.gguf','.dmg','.pia','.wav','.mp3','.m4a','.flac','.webm','.mp4','.mov','.m4v','.aiff','.aif','.exe','.msi'}
        or (p.name.startswith('.env') and p.name != '.env.example')
        or 'local-validation' in p.relative_to(root).parts):
        if not reviewed: failures.append((name,'user data or secret file'))
    for label,pattern in rules.items():
        if pattern.search(data): failures.append((name,label))
for name,rule in failures: print(f'BLOCKED {rule}: {name}')
print(f'Checked {count} source files; {len(failures)} findings.')
sys.exit(bool(failures))
