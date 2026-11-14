# Functional tasks

## LB-20261113-01 — LazyBuddy modular application

Status: independent source, packaging, API and integration validated. Owner: Leager / Windows Codex. Updated: 2026-11-14. Branch: main. Whole-product acceptance is tracked in LazyBuddyDev/LazyBuddy TASKS.md; this is the same functional requirement, not a phase task.

- `[√]` Own news source, API, manifests, build/package scripts, tests and CI without desktop source imports.
- `[√]` Independently build packages and verify external host/API integration.
- `[√]` Publish under LazyBuddyDev/news-service, MIT copyright 2026 Leager, preserving third-party terms.
- `[√]` Start from one parentless initial commit, per user request; old Git branches/tags are removed and a local ignored recovery bundle is kept.
- `[x]` Final whole-product acceptance and signed binary publication; Hongguo real-account permission/AV checks need the user's QR scan, and real writes need separate authorization.

Evidence: news core 7 and HTTP boundary 2 tests; Hongguo Python 80, search 5 (one real-network case ignored), decoder protocol 4 and Go resolver passed. Independent package health/install/remove/HEIC and external-host synthetic playback passed. Test only the affected layer for ordinary changes. CI runs independently in this repository.

Next: check the initial commit's Actions result, complete required real-account acceptance and prepare separately authorized signed publication. Do not archive the whole requirement prematurely.

2026-11-15 Web test distribution: existing modules-v0.1.0 release now also hosts the separate signed web-catalog.json, with news0.1.0 / hongguo0.2.6 / python-runtime0.1.0. Original news/Python ZIP bytes are reused; the native catalog remains unchanged. Host release aacc5d5 and Hongguo source59a1bcd; both CI passed. An isolated published CLI downloaded/verified/installed the real HTTPS catalog and packages without proxy environment variables, called health, and reclaimed dependencies on removal (11.93s). Actual online application replacement also passed, preserving settings/modules and clearing stage/backup. Evidence belongs to the host tmp/verification-release. Stable publication and final real programme AV/resource acceptance remain open; no real account writes were performed.
