# Current state

_Last updated: 2026-10-23 ~22:50 America/New_York (Phase 3.1 R2+R3; v0.2 not started)._

**Version:** 0.1.0 (untagged).

**Data:**
- Powerball: 2,015 draws, 2010-02-03..2026-10-21.
- Mega Millions: 2,550 draws, 2002-05-17..2026-10-20.
- `data/meta.json` schema_version **2** (tool_version, per-CSV sha256; stable updated_at).

**Checks:** `lottotool check --fail-on-error --no-stale` → 24 pass / 0 warn / 0 fail / 6 skip.

**Tests:** check 11, odds 8, payouts 5, pool 4, rng 7 (+ lib/crosscheck). Clippy `-D warnings` and rustfmt clean.

**Automation:**
- `update.yml`: 06:17 (stale WARN) + 10:17 (stale FAIL) America/New_York; empty/`force_now` dispatch; no keep-alive commit (D-018); CHANGES artifact on historical edits; `FORCE_NOW` via env.
- `ci.yml`: + R3-06 refuse tracked `cache/` / non-header payouts CSVs.
- `release.yml`: untested (no tag).

**GitHub:** https://github.com/cruiseliner/lottery-master-archive

**Payouts:** cache-only; R3-04/05 HTTP policy; robots noted in SOURCES.

**Relay:** drafted; REVIEWER_PROMPT; allowlist + secret scan; **not run** against API.

See `docs/REVIEWS/2026-10-24_claude_phase3.md`, `docs/OPEN_ISSUES.md`.
