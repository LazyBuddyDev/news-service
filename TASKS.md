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
