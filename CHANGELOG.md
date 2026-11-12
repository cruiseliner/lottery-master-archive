# Changelog

All notable changes to this project are documented here. Format: Keep a Changelog; versions: SemVer.

## [Unreleased]

### Added (2026-11-11 — v0.2 Step 2, D-032)
- `lottotool crowd --game pb|mm --whites a,b,c,d,e [--special N]` or `--ticket <id>`, text and `--json`: anti-crowd heuristic score (0–100) with a per-factor breakdown and reasons. Every output states it does not change the odds of winning. Weights are judgment; directions cited to Wang et al. 2016 (JDM). The archive is read only for the exact-past-winner flag. Tests: `tests/crowd.rs` (synthetic combos/draws) plus two CLI tests. OI-34 lists deferred items.
- Adam t51s258 tuning: birthdays and sequence max 20 each; raw total 90 rescaled to 0–100 (bands unchanged); past-winner check also covers Powerball Double Play numbers already in `data/` (from 2026-06-03); playslip patterns won't do; 4-of-5 sequences left in OI-34.
- Claude t52u: exact calibration by full enumeration (C(69,5), C(70,5)) pinned and printed as a reference line; disclaimer says it affects jackpot sharing only; `--ticket` help warns that output contains real picks (D-030); README "don't chase 0".

### Added (2026-11-11 — v0.2 Step 1, D-031)
- `lottotool ticket add|list` and `lottotool grade`: private append-only ticket ledger (`ledger/tickets/tickets.csv`, gitignored) checked against the archive; tiers from the encoded current charts, amounts only from the private payout cache, else `TIER_LOOKUP_NEEDED`; pending draws reported as `PENDING`.
- `eras.toml`: optional `power_play_price_usd` (pb-2015 = 1); `pb` / `mm` CLI aliases.
- tests/tickets.rs (fake data, temp dirs). OI-33 lists deferred items.
- `lottotool ticket void <id> --reason`: append-only void records (`ledger/tickets/tickets_voids.csv`); `list` marks VOID, `grade` excludes. `--cost` strict by decision.
- `.github/workflows/crosscheck.yml`: weekly read-only official-source crosscheck (`--crosscheck 8`), mismatch fails the run (D-025; OI-20 open until first green dispatch on main).
- Claude t49u follow-ups: LATE_ENTRY excluded from the white-match measurement (count printed; CLI test); measurement over all eligible tickets in scope with n, ±chance spread and a <30 note; crosscheck `summarize` FAILs on take = 0; Powerball page date verified; grade flag branches on `late_void`; latest draw by max date; git guard refuses any git failure other than "not a git repository" (tests: missing git, failing git, tracked ledger, outside a work tree warns and proceeds).
- Claude t48u follow-ups: 0-byte ledger files read as empty; duplicate ticket ids fail closed; EDT cutoff test; git guard via `rev-parse --show-toplevel` (subdirectory roots checked, warning outside a repo); accurate mixed-cache message; D-031 clock note; `tests/cli.rs` runs the binary (archived-date refusal, void exclusion/count, history-labeled baseline).
- Claude t47u follow-ups: zero successful comparisons = FAIL; untracked-aware tree check; output in `$RUNNER_TEMP`; `tests/check_format.rs` pins the parsed line format. `ticket add` refuses archived draws; `LATE_ENTRY` / `LATE_VOID` flags; partial-last-line guard on both ledger files; `grade` white-match average vs chance baseline; synthetic README examples.

### Changed (2026-10-24 morning — t235u schedule facts + t236u coverage audit)
- COLLAB: t236u brief pasted verbatim into FROM CLAUDE; t235u body missing from handoff (noted, not invented). FOR CLAUDE: 655e6cc push proxy time, Phase 1 single-cron schedule form, event=schedule=0; coverage audit summary + file links.
- docs: `COVERAGE_AUDIT.md` — archive vs NY feeds (scratch), range/schedule checks; only gap MM 2026-10-23 missing_from_archive.
- No workflow/eras/data edits.

### Changed (2026-10-24 morning — Claude t234u schedule diagnosis)
- COLLAB: owner-pasted Claude t234u into FROM CLAUDE (verbatim); FOR CLAUDE records read-only schedule checks (active workflow, YAML indent, tip blob, schedule count 0).
- No workflow/eras/data edits.

### Changed (2026-10-24 morning — Claude t233u doc fixes)
- COLLAB: owner-pasted Claude t233u into FROM CLAUDE (verbatim).
- Renamed ack response to `docs/REVIEWS/2026-10-24_grok_phase31_ack_response.md`; five-SHA list aligned; OI-23/D-026 DST cron-fallback follow-up.

### Changed (Overnight 2026-10-23/24 — Phase 3.1 ack, allowed subset)
- docs: COLLAB protocol (owner+Claude); CROSSCHECK_LOG; OI-19..22 + D-023..025; Phase 3.1 ack response.
- tests: DST due-boundary tests for 2026-11-01 fall-back and 2027-03-14 spring-forward (A-07).
- A-03 workflow/toolchain bumps **not** applied (frozen until after Sat 10:17 ET).

### Changed (Phase 3.1 — Claude Phase 3 / areas 7–8, 2026-10-24)
- meta.json schema 2: tool_version + CSV sha256; updated_at only when data digests change (R2-03).
- update.yml: no keep-alive commit (R2-02); 06:17 stale WARN / 10:17 FAIL (R2-05); CHANGES artifact (R2-08); FORCE_NOW via env.
- eras pool groups derived from max; summary `--pool` refuses mixed maxima (R2-04).
- odds: hypergeometric E/Var helpers; PRIZE_TIERS current-charts-only (R2-09).
- relay: REVIEWER_PROMPT, path allowlist, .git block, secret scan (R3-01/02).
- payouts: state validation; 4xx no-retry / 429 Retry-After; request pacing + cap; SOURCES robots (R3-04/05).
- ci: refuse tracked cache/ or non-header payouts CSVs (R3-06).
- rng: golden vector + stronger distribution tests; tighter ureq/sha2 pins (R3-07/08).

### Changed (Phase 3 — Claude Phase 2 review integration, 2026-10-23)
- Workflows: `shell: bash` / pipefail, dual cron 06:17+10:17 America/New_York, `force_now`, non-bot keepalive, pinned action SHAs, multi-OS `release.yml`.
- Staleness due = 06:00 ET next calendar day; historical schedule gaps WARN (`MISSING_DRAW`) + `schedule_exception`; inferred_eras SKIP; whites must be ascending (FAIL).
- `check --crosscheck N` (local official-source winning numbers); `data/meta.json`; `data/<game>/CHANGES.md` on historical edits; era `white_group`/`bonus_group`.
- README: dropped "Past results do not change future odds"; kept non-affiliation + confirm-with-official.
- `rust-toolchain.toml` (stable + clippy + rustfmt).

## [0.1.0] - 2026-10-23 (Phase 1 foundation; no tag or release yet)
### Added
- `lottotool` CLI with these commands: `update <game>` (NY Open Data), `update --payouts` (local cache only), `check`,
  `eras` (with `--simulate --seed`, ChaCha20), `summary`, `export` (csv/json). `--json` works everywhere.
- `eras.toml`: eras keyed on first draw date, each field labelled sourced/inferred/missing;
  schedules; null rules.
- Data: Powerball 2,015 draws (2010-02-03..2026-10-21) and Mega Millions 2,550 draws
  (2002-05-17..2026-10-20), plus `missing.csv` registries.
- Exact odds for all 9 tiers (PB 5/69+1/26, MM 5/70+1/24 and 5/70+1/25), tested against published charts.
- Workflows: scheduled update with staleness and keep-alive, CI, and a tag release with SHA256SUMS.
- Docs: README (Limitations, Roadmap), SCHEMAS, SOURCES, CURRENT_STATE, DECISIONS,
  OPEN_ISSUES, SESSION_LOG, REVIEWS, agent prompt and the review relay draft.
