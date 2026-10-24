# Current state

_Last updated: 2026-10-23 ~22:10 America/New_York (Phase 3 Claude review integration; v0.2 not started)._

**Version:** 0.1.0 (untagged).

**Data:**
- Powerball: 2,015 draws, 2010-02-03..2026-10-21.
- Mega Millions: 2,550 draws, 2002-05-17..2026-10-20.
- `data/meta.json` present (schema_version 1).

**Checks:** `lottotool check --fail-on-error --no-stale` → 24 pass / 0 warn / 0 fail / 6 skip.
`check --no-stale --crosscheck 3` → 26 pass / 0 warn / 0 fail / 4 skip (3/3 match each game).

**Tests:** 27 tests (check 11, odds 6, payouts 5, rng 3, crosscheck unit 2). Clippy `-D warnings` and rustfmt clean.

**Automation:**
- `update.yml`: 06:17 + 10:17 America/New_York, `force_now`, bash pipefail, non-bot keepalive, pinned SHAs.
- `ci.yml`: push/PR, pinned SHAs, `shell: bash`.
- `release.yml`: Linux / Windows / macOS matrix on `v*` tags (untested — no tag).

**GitHub:** https://github.com/cruiseliner/lottery-master-archive

**Payouts:** cache-only; R-16 history scan CLEAN.

**Relay:** drafted, not run.

See `docs/OPEN_ISSUES.md`, `docs/REVIEWS/2026-10-23_claude_phase2.md`.
