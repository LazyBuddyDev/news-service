# Functional tasks

## Archived

### LB-20261113-01 — LazyBuddy modular application

Status: completed and archived with LazyBuddy v2.0.0 / modules-v2.0.0. Owner: Leager / Windows Codex. Updated: 2026-11-15. Branch: main. Whole-product evidence and disclosed limits are maintained in the host TASKS.md; no full real-programme stress-test claim.

- `[√]` Own news source, API, manifests, build/package scripts, tests and CI without desktop source imports.
- `[√]` Independently build packages and verify external host/API integration.
- `[√]` Publish under LazyBuddyDev/news-service, MIT copyright 2026 Leager, preserving third-party terms.
- `[√]` Start from one parentless initial commit, per user request; old Git branches/tags are removed and a local ignored recovery bundle is kept.
- `[√]` Final signed host/module snapshots published. User confirmed continuous real playback, series/episode switching and seeking AV behavior; no repeated real account writes. Short synthetic resource baseline reviewed, long real-programme stress testing remains outside the release evidence.

Evidence: news core 7 and HTTP boundary 2 tests; Hongguo Python 80, search 5 (one real-network case ignored), decoder protocol 4 and Go resolver passed. Independent package health/install/remove/HEIC and external-host synthetic playback passed. Test only the affected layer for ordinary changes. CI runs independently in this repository.

Next: no delivery blocker. Keep immutable stable snapshots; investigate newly observed upstream failures with bounded redacted logs. Any future real account writes still need explicit authorization.

2026-11-15 Web test distribution: existing modules-v0.1.0 release now also hosts the separate signed web-catalog.json, with news0.1.0 / hongguo0.2.6 / python-runtime0.1.0. Original news/Python ZIP bytes are reused; the native catalog remains unchanged. Host release aacc5d5 and Hongguo source59a1bcd; both CI passed. An isolated published CLI downloaded/verified/installed the real HTTPS catalog and packages without proxy environment variables, called health, and reclaimed dependencies on removal (11.93s). Actual online application replacement also passed, preserving settings/modules and clearing stage/backup. Evidence belongs to the host tmp/verification-release. Stable publication and final real programme AV/resource acceptance remain open; no real account writes were performed.

2026-11-15 stable closeout: host source b3b4b42 and service59a1bcd; v2.0.0 / modules-v2.0.0 are immutable, signed web catalog now points to stable assets. Actual stable HTTPS install/health/uninstall and application online exit/replace/restart with settings/modules preserved passed. The host-only follow-up fixes seek release snap-back and implicit default mute, with related unit/type and release-WebView checks. Intermittent upstream redirect failures remain a disclosed limitation with logs/manual retry, not a confirmed root-cause fix.
