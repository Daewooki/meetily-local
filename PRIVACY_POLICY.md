# Meetily Local data handling

Updated 2026-10-19. This describes this independent fork, not other Meetily editions. It documents implemented behavior and is not a security certification.

## Local processing and storage

Audio capture, voice activity detection, transcription and summarization run on the user’s PC. The supported summary path invokes a local inference process; external AI providers are disabled. Recordings, transcripts, summaries, model files and settings are stored locally. Recording storage can use a location selected in the app.

The app does not encrypt the entire meeting database or recording files. Device encryption, permissions, backups and synchronized folders are controlled separately. Do not assume deleting a meeting also removes every recording, backup or previously published copy.

## User-triggered network requests

- Model setup downloads files from publishers and their CDNs without uploading meeting content.
- Atlassian connection checks and publishing authenticate to the selected Atlassian Cloud tenant. Scoped tokens also use the official `api.atlassian.com` gateway.
- Opening Jira publishing may retrieve project components. Connection checks read space, project, page, folder or board information as applicable.
- Publishing sends the reviewed title, body and selected issue settings. Original recordings and full transcripts are not automatically attached. Content pasted into the body is transmitted.
- Links open in the operating system browser when selected.

Atlassian Cloud is a cloud service. Network restrictions and access permissions depend on the user's organization and account; this app does not guarantee access from only a particular network.

## Credentials

API tokens are encrypted locally using AES-256-GCM, with the key in macOS Keychain or the current user’s Windows Credential Manager (persisted on the same device, without credential roaming). Native code uses tokens to authenticate with Atlassian; saved tokens are not returned to the interface. Tokens retain the permissions of their account and scopes. Revoke them through Atlassian when no longer needed.

## Telemetry and updates

The usage-analytics transport and PostHog dependency have been removed. Automatic update checks are disabled. Model setup and Atlassian requests remain network operations. The Windows installer includes Microsoft WebView2 and the Visual C++ runtime; these components have their own Microsoft terms and update behavior. The app does not control operating-system or WebView2 service traffic. This app is not an operating-system firewall.

## Reports and changes

Use synthetic examples in bug reports. Do not upload meetings, connection files, tokens or unredacted logs. Review release notes and [network behavior](NETWORK-POLICY.md) when updating. [Validation limits](LOCAL-VALIDATION.md) remain applicable.
