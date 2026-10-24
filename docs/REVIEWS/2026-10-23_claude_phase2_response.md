# Response to Claude — Phase 2 review integration (2026-10-23)

Hi Claude — we integrated your Phase 2 findings (R-01..R-16) into `cruiseliner/lottery-master-archive` on main. No v0.2 / no release tag. Please re-review Priority-1/2 items and areas 7–8 (Security, RNG) when you can.

## Triage summary

| ID | Disposition | What changed / where to look |
|---|---|---|
| R-01 | FIXED | `.github/workflows/update.yml`: `defaults.run.shell: bash`; Full check uses `shell: bash` (⇒ `-eo pipefail`) and optional `--now` from `inputs.force_now`. Red/green proof via workflow_dispatch with `force_now` (see SESSION_LOG). |
| R-02 | FIXED | Same workflow: quote 60-day public-repo rule; WARN days since last **non-bot** commit; keepalive only when non-bot age ≥45d. Docs silent on bot commits counting as activity. |
| R-03 | FIXED | Kept `timezone: America/New_York` (GitHub docs support IANA timezone on `schedule`). Added second cron `17 10 * * *` (10:17 ET) for R-11. Please confirm first real scheduled start time in the Actions UI after Sat 2026-10-24 06:17 ET. |
| R-04 | FIXED | `src/check.rs` schedule gaps → WARN `MISSING_DRAW`; `src/ny.rs` documents gaps as `field=draw` in missing.csv; `eras.toml` / `GameDef.schedule_exception` + `expected_draw_day`. Due gaps still FAIL via staleness. Tests updated. |
| R-05 | FIXED | `src/main.rs` writes `data/<game>/CHANGES.md` when historical rows change; update.yml commit message appends note. |
| R-06 | FIXED | New `src/crosscheck.rs` + `lottotool check --crosscheck N` (local only; not on schedule). PB: powerball.com HTML whites/powerball; MM: Drawing.N1–N5,MBall. Live check 2026-10-23: **3/3 PASS** each game. |
| R-07 | FIXED | `data/meta.json` (schema_version 1) written on every update. |
| R-08 | FIXED | Era fields `white_group` / `bonus_group` (defaults `"white"` / `"bonus"`); shown in `eras --json`. Odds still assume independent pools. |
| R-09 | FIXED | `draws.ranges.inferred_eras` severity WARN → **SKIP** (reason retained). |
| R-10 | FIXED | Due instant = **06:00 America/New_York on the next calendar day** (`latest_due_draw` in `src/check.rs`). `stale_grace_hours` kept as documentation. |
| R-11 | FIXED | Second cron 10:17 ET (with R-03). |
| R-12 | FIXED | Action SHAs pinned in `ci.yml` / `update.yml` / `release.yml`; `rust-toolchain.toml` channel=stable + clippy,rustfmt. |
| R-13 | FIXED | `draws.whites_ascending` FAIL if any white tuple not strictly ascending. All committed rows pass. Closes OI-08. |
| R-14 | FIXED | README: dropped "Past results do not change future odds" (owner: that line was ours). Kept non-affiliation + "Always confirm numbers with the official lottery". `release.yml`: Linux x86_64, Windows x86_64, macOS aarch64 matrix + SHA256SUMS. |
| R-15 | DEFERRED | Exact Claude wording for R-15 was not recoverable from agent stores this session; cheap adjacent work folded into R-12. Please restate if still needed. |
| R-16 | FIXED (CLEAN) | Ran `git log --all --stat -- cache schemas/payouts.csv` and similar scans: only header-only `schemas/payouts.csv` + `src/payouts.rs` / `tests/payouts.rs`. **No payout/winner table data in git history.** |

## Verification (America/New_York)

- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`: clean (27 tests).
- `lottotool check --fail-on-error --no-stale`: 24 pass / 0 warn / 0 fail / 6 skip.
- `lottotool check --no-stale --crosscheck 3`: 26 pass / 0 warn / 0 fail / 4 skip (crosscheck 3/3 each game).

## Doc quotes we relied on

60-day (docs.github.com schedule): "In a public repository, scheduled workflows are automatically disabled when no repository activity has occurred in 60 days."

Timezone: "By default, scheduled workflows run in UTC. You can optionally specify a timezone using an IANA timezone string…"

## Still open

- Areas **7 (Security)** and **8 (RNG)** from `docs/HANDOFF_PHASE2.md` for your next pass.
- First scheduled run after timezone+dual-cron change (OI follow-up).
- OI-01..OI-07, OI-10..OI-14 unchanged in substance (licensing, inferred eras, Double Play MISSING, etc.).

## Where to look

- Workflows: `.github/workflows/{update,ci,release}.yml`, `rust-toolchain.toml`
- Check/crosscheck: `src/check.rs`, `src/crosscheck.rs`, `src/eras.rs`, `src/ny.rs`, `src/main.rs`
- Data: `data/meta.json`, `data/<game>/draws.csv` (unchanged row counts), README disclaimer
- Review trail: `docs/REVIEWS/2026-10-23_claude_phase2.md` (this triage), `docs/DECISIONS.md` D-012+, `CHANGELOG.md` [Unreleased]

Thanks — Adam / lottery-master Phase 3 executor
