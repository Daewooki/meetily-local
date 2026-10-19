# Meetily Local: Mac quick start

This independent fork targets Apple Silicon Macs running macOS 14.2 or later. 16 GB RAM and 5 GB free space are recommended; recordings need additional storage. For Windows x64 build and validation status, see [Windows instructions](WINDOWS.md). Linux and Intel Mac packages are not supplied by this fork.

## Install and prepare

Use a release built from this repository or follow [the build guide](RELEASING.md). Copy `Meetily Local.app` into Applications. Builds currently use ad-hoc signing and are not Apple-notarized; contact the distributor or device administrator if execution is blocked.

Open **로컬 모델 · 녹음 권한 준비** (local model and recording setup). Download the Whisper transcription and Qwen summary models, approximately 3.3 GB total. Transcription and summarization then work offline. Grant microphone and system-audio permissions for recording; importing a file does not require recording permissions.

## Record and review

Notify participants and follow applicable recording rules. **회의 녹음 시작** starts recording immediately. Use the red stop button at the bottom to stop, and wait for final transcription. **녹음 파일 가져오기** imports an existing recording.

Generate a summary and compare it with the transcript. Check decisions, people and deadlines, then save edits. AI output is not guaranteed accurate.

## Add Atlassian Cloud connections

Open **Wiki · Jira 연결** in the sidebar. Register Wiki and Jira connections separately with your own Atlassian account. Supported site URLs are HTTPS `*.atlassian.net` tenants. Server/Data Center and custom-hosted domains are not supported.

| Field | Example / meaning |
| --- | --- |
| Connection name | A name you recognize |
| Site URL | `https://your-team.atlassian.net` |
| Email | `user@example.com`, the account that issued the token |
| Token type | Classic or scoped, matching issuance |
| API token | Paste only in the password field |
| Wiki Space Key | `TEAM`; not the numeric Space ID |
| Wiki parent page/folder ID | Optional numeric content ID from the page/folder URL |
| Jira Project Key | Required, for example `PROJ` |
| Jira Board ID | Optional number after `/boards/`, for example `1234` |

Issue tokens at [Atlassian account security](https://id.atlassian.com/manage-profile/security/api-tokens). Do not place real tokens in chats, issues or screenshots. Scoped tokens use the official Atlassian API gateway after Cloud ID discovery.

Select **연결 확인** (check connection), review the returned target, then **연결 추가** (add connection). Checks verify read access; creation permissions are checked by Atlassian during publication.

For scoped Confluence tokens, use `read:space:confluence`, `read:page:confluence`, `write:page:confluence`, and `read:folder:confluence` when choosing a folder. Account permissions must also allow the operation. Pages and folders are detected automatically and must belong to the chosen Space. An empty parent creates a page at the Space root.

A Jira board is optional. Issues are created in the project. Board filters, components, statuses and sprint settings control visibility on a board.

## Publish only reviewed content

- **Wiki 게시:** select a saved connection, verify the parent, review title and body, select the confirmation checkbox, then **페이지 게시**. A new page is created. Repeating the same meeting/connection returns the earlier result; it does not update the page.
- **Jira 등록:** select the connection, issue type and optional component. Write the task title and retain only relevant work and evidence in the body. Confirm, then select **이슈 등록**.

Assignees, due dates and project-specific required custom fields are not supported by the form. A project may reject creation if an additional required field is missing. Duplicate matching Jira submissions are guarded. After an uncertain response, search Atlassian for the result instead of forcing another submission.

The original recording and full transcript are not automatically attached. Anything included in the title or body is transmitted. Opening Jira publication may fetch project components before submitting.

## Storage and troubleshooting

Tokens are encrypted with AES-256-GCM using a macOS Keychain key or a Windows Credential Manager key. Meeting databases and recording files are not encrypted by the app. Data lives under `~/Library/Application Support/ai.meetily.local/` on Mac, or `%APPDATA%\ai.meetily.local` on Windows; recording storage can be selected separately. Network accessibility depends on your Atlassian tenant and organization.

For 401 errors, check email, expiration and token type. For 403 errors, check both token scopes and account permissions. For folder failures, confirm folder-read scope, ID and Space membership. For missing board issues, check the project first and then board filters.

[Windows setup/build](WINDOWS.md) · [Detailed Korean guide](../LOCAL-MAC.md) · [Data handling](../PRIVACY_POLICY.md) · [Network behavior](../NETWORK-POLICY.md) · [Contributing](../CONTRIBUTING.md)
