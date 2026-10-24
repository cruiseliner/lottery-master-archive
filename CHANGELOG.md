# Changelog

All notable changes to this project are documented here. Format: Keep a Changelog; versions: SemVer.

## [Unreleased]

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
