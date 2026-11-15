# Current tasks

## LB-20261115-03 — Publish independent connection settings

Owner: Windows Codex. Updated: 2026-11-15. Branch: main, uncommitted. Host release1.0.0 and required module publication are authorized.
- Current: news0.1.2 provides direct/system connection selection and bounded redacted source diagnostics; complete package is staged for modules-v1.0.0. Host owns common UI and signed-catalog deployment.
- `[x]` Commit/push source, verify relevant CI and publish the immutable package with the host module snapshot.
- `[x]` Verify real HTTPS install through the new trusted catalog without business requests.
- Next: coordinate with host TASKS.md; do not mark public delivery complete from local install alone.

## LB-20261115-06 — Xiaohongshu source availability

Owner: Windows Codex. Updated: 2026-11-15. Branch: main. Current package:0.1.2.
- Same-package direct fetch returned source_failed with stage=read_body kind=timeout phase=Global. A 10-second transfer still timed out after65412 bytes; a separate Range probe returned503 and probing stopped. Earlier same-path fetch returned20 rows, so the source is intermittent.
- Evidence: host tmp/verification-news/source-live.json, source-transfer.json and source-table-boundary.json. Diagnostics and failure-preserving-cache tests passed; the source is not fixed.
- `[x]` Once external availability recovers, verify the exact source again. Require complete-list boundary evidence before changing body reading; do not rotate identity/proxy/source or loop retries.
