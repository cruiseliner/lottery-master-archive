# Phase 2 handoff: review request for lottery-master-archive

**Phase 1 status:** foundation complete. **v0.2 is NOT started.** This document asks for an independent review before any v0.2 work.

- Repo (public): https://github.com/cruiseliner/lottery-master-archive
- Reviewed commit: `e846764f06b05ab9d3b6cd53b7364bceda1cbf4d` (Phase 1 foundation, 2026-10-23)
- Toolchain: rustc 1.99.1 / cargo 1.99.1 (stable), edition 2024. Approximately 2,500 lines of Rust in src/ and tests/.
- Owner: Adam Karba (cruiseliner). Rules: no invented data; verify, do not recall; no secrets; the personal ledger stays out of the repo; payout tables are not published until MUSL/Florida Lottery permission is granted.

## What exists (status per item)

| Item | Status |
|---|---|
| `lottotool` v0.1: `update`, `check`, `eras`, `summary`, `export`; `--json`, `--fail-on-error`, skip reasons, seeded ChaCha20 simulation | COMPILED, TESTED, RUN |
| Data: Powerball 2,015 draws (2010-02-03..2026-10-21), Mega Millions 2,550 draws (2002-05-17..2026-10-20), from NY Open Data | RUN (idempotent re-run: added 0, changed 0) |
| `eras.toml` (each field labelled sourced / inferred / missing), `missing.csv` registries | DRAFTED, checked by `check` |
| Payout fetchers (powerball.com national and `&oc=fl`, megamillions.com API), output to gitignored `cache/` only | RUN locally; output not committed |
| `.github/workflows/update.yml` (daily 06:17 America/New_York, staleness, keep-alive) | RUN once via workflow_dispatch: success |
| `.github/workflows/ci.yml` | RUN on push: success |
| `.github/workflows/release.yml` (v* tag, tar.gz, SHA256SUMS) | DRAFTED, NOT RUN |
| `scripts/relay/relay.py` (Anthropic Messages API review relay) | DRAFTED; dry-run only, NOT RUN against the API |

## Key files
- README: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/README.md
- Eras: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/eras.toml
- Schemas: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/docs/SCHEMAS.md, plus https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/schemas/
- Sources: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/data/SOURCES.md
- Decisions: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/docs/DECISIONS.md
- Open issues: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/docs/OPEN_ISSUES.md
- Session log: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/docs/SESSION_LOG.md
- Agent prompt: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/agent/AGENT_PROMPT.md and https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/AGENTS.md
- Code: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/src/check.rs, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/src/odds.rs, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/src/ny.rs, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/src/payouts.rs, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/src/rng.rs
- Tests: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/tests/
- Workflows: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/.github/workflows/update.yml, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/.github/workflows/ci.yml, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/.github/workflows/release.yml
- Data: https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/data/powerball/draws.csv, https://github.com/cruiseliner/lottery-master-archive/blob/e846764f06b05ab9d3b6cd53b7364bceda1cbf4d/data/megamillions/draws.csv

## Build and test output (box, 2026-10-23)
```
cargo fmt --all -- --check                          -> OK
cargo clippy --all-targets --locked -- -D warnings  -> Finished (no warnings)
cargo test --locked
  tests/check.rs    test result: ok. 9 passed; 0 failed
  tests/odds.rs     test result: ok. 6 passed; 0 failed
  tests/payouts.rs  test result: ok. 5 passed; 0 failed
  tests/rng.rs      test result: ok. 3 passed; 0 failed
lottotool check --fail-on-error
  WARN  powerball    draws.ranges.inferred_eras   592 rows were checked against eras whose matrix is inferred from data, not sourced (circular)
  WARN  megamillions draws.ranges.inferred_eras   324 rows ... (circular)
  summary: 24 pass, 2 warn, 0 fail, 0 skip        (exit 0)
```
GitHub Actions (2026-10-23, about 19:04-19:06 EDT):
- ci run 40037478995: success. Same 23 tests; integrity gate "22 pass, 2 warn, 0 fail, 2 skip" (the 2 skips are staleness, via `--no-stale`).
- update run 40037509763 (workflow_dispatch): success.
  - Both games: "added 0, changed 0", so the "No data changes." commit step was skipped.
  - Keep-alive: "Last commit age: 0 days" (no commit).
  - Full check: "24 pass, 2 warn, 0 fail, 0 skip"; check-report artifact uploaded.

Jackpot odds from `lottotool eras`:
- pb-2010: 1 in 195,249,054 (matrix inferred)
- pb-2012: 1 in 175,223,510 (matrix inferred)
- pb-2015: 1 in 292,201,338 (sourced)
- mm-2002: 1 in 135,145,920
- mm-2005: 1 in 175,711,536
- mm-2013: 1 in 258,890,850
- mm-2017: 1 in 302,575,350
- mm-2025: 1 in 290,472,336

## Latest session log entry (summary)
Phase 1 was built on 2026-10-23 after the owner approved the defaults.

**Incident:** at about 18:49 EDT a box reset wiped the first working copy and the toolchain. The work was rebuilt in a new directory and pushed early.

**Fixes made during the session:**
- MM 2005 era date corrected to Fri 2005-06-24.
- powerball.com served random identity, gzip and br encodings, so ureq now decodes gzip+brotli and retries up to 3 times.
- PB odds test tightened to the 4-decimal FL rules chart.
- Relay argument and path handling hardened.

Full entry: docs/SESSION_LOG.md.

## Open issues (from docs/OPEN_ISSUES.md)
- OI-01: MUSL and Florida Lottery permission for payout tables. PENDING (owner).
- OI-02: megamillions.com terms of use not found (404). MISSING.
- OI-03: Pre-2015 PB and pre-2010 MM era dates, matrices and prices are unsourced.
- OI-04: The PB 59/35 first draw is somewhere in 2011-12-10..2012-01-18; the boundary is inferred.
- OI-05: PB Double Play 2021-08-23..2026-06-01 (748 draws) is MISSING from NY data.
- OI-06: MM multiplier since 2025-04-08 (161 draws) is MISSING.
- OI-07: PB multiplier blank 2012-01-18..2014-01-18 (210 draws).
- OI-08: White-ball order is not enforced by `check`.
- OI-09: It is unverified whether a bot commit counts as activity for the 60-day keep-alive (Inference).
- OI-10: The 30h staleness grace rests on 2 observations.
- OI-11: The powerball.com HTML parser is fragile; it fails loudly if the layout changes.
- OI-12: The relay has not been run against the API.
- OI-13: release.yml is untested.
- OI-14: Kickoff spec section 6 was unavailable to the Phase 1 executor; check the build against it.

## Review areas (please report as: area, severity, evidence, fix; label Fact or Inference)
1. **Data integrity:**
   - Is `src/ny.rs` normalisation correct (field parsing, multiplier and double_play handling)?
   - Is `missing.csv` complete and honest?
   - Are there any silent fills?
2. **Era model:**
   - Are the `eras.toml` boundaries and labels defensible?
   - Is it correct to treat inferred-era range checks as WARN rather than FAIL?
3. **Odds:**
   - `src/odds.rs` exact combinatorics, and the tier tables for PB and both MM matrices.
   - The rounding in `round_half_up`.
   - Whether the tests really pin the published charts.
4. **check semantics:**
   - The staleness rule (draw due 30h after 00:00 ET).
   - `--no-stale` and skip reasons.
   - Exit codes.
   - Missed edge cases: DST, schedule changes, holidays or no-draw days.
5. **Workflows:**
   - Cron with `timezone`, the permissions, concurrency, and the commit/keep-alive logic.
   - Should failure on stale data come before or after the commit?
   - release.yml correctness.
6. **Licensing and compliance:**
   - Is the attribution adequate?
   - Is anything restricted committed? (Payouts must be cache-only; there must be no "Florida Lottery" product naming.)
7. **Security:**
   - `.gitignore` coverage (`.env`, `cache/`, ledger patterns).
   - Relay redaction and path blocking.
   - Any secret in history.
8. **RNG and simulation:** ChaCha20 seeding, unbiased sampling, and test tolerances.
9. **Docs:** whether README Limitations and Roadmap are accurate; what is missing for a v0.2 plan.

**Please do not start v0.2.** Return the review for the owner to triage into docs/DECISIONS.md and docs/OPEN_ISSUES.md.
