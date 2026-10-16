# Publishing this fork

The public repository for this independent fork is [Daewooki/meetily-local](https://github.com/Daewooki/meetily-local).

1. Choose a public GitHub repository for this independent fork. Use a new root history so unrelated internal commits and metadata are not published.
2. Review the complete source and notices. Run `python3 scripts/check-release-source.py`; inspect documentation, media, commit author email and archives as well. A pattern scan is not a proof that every secret is absent.
3. Downstream maintainers can set `NEXT_PUBLIC_PROJECT_REPOSITORY=https://github.com/OWNER/REPOSITORY` while building. The About screen links to that fork's releases, usage guide, issues and contribution guide. Without an override, it uses `https://github.com/Daewooki/meetily-local`. The original Meetily link is labelled separately.
4. Follow [RELEASING.md](RELEASING.md). Never reuse an older installer that embeds an outdated repository link or notice.
5. Package the new app with the exact FFmpeg corresponding source, build script and third-party notices. Verify the DMG, ZIP, code signature and SHA-256 manifest.
6. Upload only the reviewed source, packages and public demo. Exclude local validation logs, model weights, recordings, connection vaults and private backups.

The application's MIT notice remains mandatory; dependency and model terms are separate. This guide does not establish ownership of additions or override an organization's publication rules.
