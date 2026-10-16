# Meetily Local

An independent community fork of [Meetily](https://github.com/Zackriya-Solutions/meetily) for on-device Korean meeting notes and optional Confluence / Jira publishing. This is not an official Meetily release or an Atlassian product.

[한국어 안내](README.ko.md) · [Mac setup and usage](docs/USAGE.md) · [한국어 사용법](LOCAL-MAC.md) · [Data handling](PRIVACY_POLICY.md) · [Contributing](CONTRIBUTING.md)

## What it does

- Record microphone and system audio, or import a recording.
- Transcribe locally with Whisper and summarize locally with Qwen through a bundled inference helper.
- Review and edit results, then explicitly publish a new Confluence page beneath a page or folder, or create a Jira issue.
- Encrypt Atlassian tokens locally with AES-256-GCM and keep the key in macOS Keychain.
- Use a Korean meeting template that checks cited transcript evidence, names and deadlines. Custom templates are also supported; see [quality checks and limitations](docs/SUMMARY-QUALITY.md).

After downloading models, transcription and summarization run offline. Model downloads and Atlassian operations use the network. Original recordings and full transcripts are not automatically attached to publication requests; content you place in the publication body is sent. External AI, usage telemetry and automatic update checks are disabled in this fork.

## Supported distribution

| Requirement | Current target |
| --- | --- |
| Hardware | Apple Silicon Mac, M1 or later |
| OS | macOS 14.2 or later |
| Memory | 16 GB or more recommended |
| Storage | At least 5 GB free, plus recording storage |

Windows, Linux and Intel Mac packages are not provided by this fork. Contributions for those platforms are welcome. The original Meetily project has its own platform support and release schedule.

## Start here

1. Download the DMG or ZIP from [the 1.0.0 release](https://github.com/Daewooki/meetily-local/releases/tag/v1.0.0), or [build it yourself](docs/RELEASING.md). The source checkout does not include release binaries.
2. Copy `Meetily Local.app` to Applications. Current builds use ad-hoc signing and are not Apple-notarized; device management policies may prevent execution.
3. Prepare the local transcription and summary models, approximately 3.3 GB in total.
4. Grant recording permissions when needed, or import an existing recording.
5. Optionally add your own Atlassian Cloud connection. No account, token, meeting or model weight is included in the distribution.
6. Review the summary and destination before publishing.

The interface currently includes Korean setup and connection screens. [English instructions](docs/USAGE.md) explain the controls. A [video guide](docs/VIDEO-GUIDE.md) demonstrates the workflow with sample data.

## Limitations

Meeting databases and recording files are not encrypted by the application. Token encryption is a separate feature. AI output can contain mistakes and omissions; review it before sharing.

This version creates new Confluence pages; it does not update an existing page. Jira assignees, due dates and project-specific required custom fields are not handled by the form. A board is optional, and board filters determine whether an issue appears there.

[Validation record](LOCAL-VALIDATION.md) · [Network behavior](NETWORK-POLICY.md) · [Dependency review](docs/DEPENDENCY-REVIEW.md)

## Build and contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for development, templates, bug reports and platform contributions. Runtime code is in `frontend/src-tauri` and `frontend/src`; `backend/` is an unsupported upstream archive.

## License and origin

Based on Meetily Community v0.4.1, source commit `a2cb62e827da7ef59f65064c97233efb2313878e`. The original [MIT license](LICENSE.md) and Copyright (c) 2024 Zackriya Solutions notice are preserved. The public source history starts with a reviewed source snapshot.

FFmpeg is a separate LGPL-2.1-or-later executable. Its corresponding source and build instructions must accompany application releases. Dependency and model terms are described in [third-party notices](THIRD_PARTY_NOTICES.md).
