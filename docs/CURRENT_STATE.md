# Current state

_Last updated: 2026-10-23 (Phase 1 foundation complete; Phase 2 review not started; v0.2 not started)._

**Version:** 0.1.0 (untagged).

**Data:**
- Powerball: 2,015 draws, 2010-02-03..2026-10-21.
- Mega Millions: 2,550 draws, 2002-05-17..2026-10-20.
- Source: NY Open Data. `lottotool update` is idempotent: a second run reported added 0, changed 0.

**Checks:** `lottotool check --fail-on-error` gives 24 pass, 2 warn (range checks on inferred eras), 0 fail.

**Tests:** 23 integration tests, all passing. Clippy and rustfmt are clean.

**Automation:**
- `update.yml`: daily 06:17 America/New_York plus manual dispatch.
- `ci.yml`: push and PR.
- `release.yml`: v* tags; untested.

**Payouts:** the fetch code works; results go to the local `cache/` only (not published; see D-003 and OI-01).

**Relay:** drafted, not run.

See `docs/OPEN_ISSUES.md` for gaps, and `docs/HANDOFF_PHASE2.md` for the review package.
