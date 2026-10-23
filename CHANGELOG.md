# Changelog

All notable changes to this project are documented here. Format: Keep a Changelog; versions: SemVer.

## [Unreleased]

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
