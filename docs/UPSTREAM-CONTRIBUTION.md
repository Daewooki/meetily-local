# Contributing changes to upstream Meetily

The public fork and an upstream pull request serve different purposes. Keep the fork's release branding, local-only policy and packaging changes out of feature proposals to upstream.

## Upstream workflow

Checked 2026-10-16: [Meetily's contribution guide](https://github.com/Zackriya-Solutions/meetily/blob/devtest/CONTRIBUTING.md) asks contributors to start from `devtest`, discuss a feature in an issue and target PRs at `devtest`. Recheck the guide before submitting. This fork's own PRs target `main`.

Create an upstream fork and a feature branch from its current `devtest`. Port the feature onto that codebase instead of proposing the independent public snapshot as one large replacement. Do not change upstream model-provider settings or platform support as a side effect.

## Proposed first feature: reviewed Atlassian publishing

Offer optional Confluence Cloud / Jira Cloud connections and publication of a reviewed meeting summary or action item. Include classic and scoped token authentication, page/folder parent validation, account and scope errors, duplicate-submission handling, and documentation of exactly what is sent.

The implementation stores an AES key in macOS Keychain or Windows Credential Manager. Before offering the feature across all upstream platforms, agree on the remaining Linux credential-store behavior and validate each platform. Translate connection/publishing labels to upstream's interface conventions. Do not introduce a plaintext credential fallback.

A focused PR should include the native integration module, connection and review screens, command registration, required dependencies, synthetic tests and usage instructions. Preserve the original application's other providers, analytics preferences and update policy.

## Separate proposal: Korean summary quality

Port evidence-backed extraction, corrections/cancellations, UTF-8-safe chunking and the synthetic Korean corpus as a separate change. Make the additional pass specific to the appropriate template/language. Record latency and context-budget tradeoffs; do not claim general accuracy improvements from a few examples.

## Review material

Prepare a short synthetic demo, expected network destinations, account/scope requirements, compatibility notes and actual test results against upstream `devtest`. Do not include personal credentials, organizational links, private meeting data or internal release history. New dependencies need license review and notices.

An upstream PR is a proposal; MIT licensing does not oblige maintainers to accept it. Preserve the MIT copyright and permission notices and confirm rights to submit the changes.
