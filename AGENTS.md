# news service development

- This repository owns its business API and build/package tests. Do not import desktop implementation or require a sibling workspace checkout. All external integration uses versioned API, staged packages and explicitly supplied host binaries.
- Keep one TASKS.md entry for the whole LazyBuddy modularization requirement, ID LB-20261113-01; this repository's scope is part of that requirement. Operational commits and script cleanup are evidence, not new tasks. Use literal `[x]` for pending and `[√]` for done.
- Add focused unit/contract tests for non-trivial behavior; use fixtures by default. Do not turn a local change into a full real-account acceptance run. Network/identity/permissions and synthetic/real decoding evidence remain separate.
- Prefer CLI, simple existing code and standard libraries. Research difficult integration against official upstream examples before adding abstractions. Keep UI out of the service, and preserve validation at credentials, network, IPC and local-file boundaries.
- Credentials, device identities, signed URLs and media keys never enter commits, command arguments or public errors. Explicit user authorization is required for real account writes. Do not infer rights from a successful login or one episode.
- Use private dependencies and ignored target/tmp output. Clean verified obsolete artifacts promptly, preserving SDKs, live instances, source and unpushed Git history. Do not run a blanket clean against a shared SDK.
- Confirm own-code and third-party licenses before public distribution. Preserve upstream notices; do not invent a license or release identity. Commit/push/transfer/tag/release are separate actions within current authorization.
