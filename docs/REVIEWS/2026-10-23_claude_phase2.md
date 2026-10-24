# Claude Phase 2 review — triage (2026-10-23, America/New_York)

Owner brief: t224u / pt224u. Clarifications: t225u. Phase 3 integration; no v0.2.

Claude's full R-01..R-16 paste was not recoverable from agent stores this session; triage follows the owner Priority 1/2 list plus prior-session plan and t225u.

| ID | Topic | Disposition | Notes |
|---|---|---|---|
| R-01 | `shell: bash` / pipefail + `force_now` red/green proof | FIXED | `defaults.run.shell: bash`; Full check step also `shell: bash` + `--now` from `force_now`. |
| R-02 | 60-day schedule inactivity; bot commits | FIXED | Quote docs; WARN days since last **non-bot** commit; keepalive keyed on non-bot age ≥45d. Docs silent on whether bot commits count (Inference: no). |
| R-03 | `on.schedule[].timezone` | FIXED | Kept `America/New_York` (Fact: docs.github.com schedule timezone, retrieved 2026-10-23). Added 10:17 ET cron (R-11). Follow-up: confirm first real scheduled start in Actions UI. |
| R-04 | Historical schedule gaps | FIXED | Gaps → WARN `MISSING_DRAW`; also `field=draw` rows in missing.csv; `[[game.schedule_exception]]` excludes cancelled dates. Due gaps still FAIL via staleness. |
| R-05 | Historical CHANGES | FIXED | On update with changed rows, write `data/<game>/CHANGES.md`; commit message mentions it. |
| R-06 | Official-source crosscheck (OI-15) | FIXED | `check --crosscheck N` local only; PB HTML + MM API. Live: 3/3 match each game (2026-10-23). |
| R-07 | `data/meta.json` | FIXED | Written on every update; committed snapshot included. |
| R-08 | Pooling keys | FIXED | Era `white_group` / `bonus_group` (default white/bonus); exposed in `eras --json`. |
| R-09 | inferred_eras severity | FIXED | WARN → SKIP (still explained). |
| R-10 | Staleness due instant | FIXED | Wall-clock **06:00 ET next calendar day** (was midnight+30h). |
| R-11 | Second cron | FIXED | 10:17 America/New_York (with R-03). |
| R-12 | Pin actions / toolchain | FIXED | checkout/cache/upload/download pinned by SHA; `rust-toolchain.toml` stable+clippy+rustfmt; ci defaults `shell: bash`. |
| R-13 | Ascending whites | FIXED | `draws.whites_ascending` FAIL; all 4565 committed rows pass. |
| R-14 | README odds line + multi-OS release | FIXED | Dropped "Past results do not change future odds" (t225u: ours, not Adam's). Kept non-affiliation + confirm-with-official. `release.yml` matrix: Linux/Windows/macOS. |
| R-15 | (cheap-or-defer; exact Claude wording unrecovered) | DEFERRED | Exact ask not re-verified from Claude paste this session; adjacent pinning/toolchain covered by R-12. |
| R-16 | Payout/winner tables never in git history | FIXED (CLEAN) | `git log --all --stat -- cache schemas/payouts.csv`: only header-only `schemas/payouts.csv` + source/tests. No `cache/` payout tables in history. |

## Doc quotes (R-02 / R-03)

**60-day rule (Fact, docs.github.com schedule event, retrieved 2026-10-23):**
> In a public repository, scheduled workflows are automatically disabled when no repository activity has occurred in 60 days.

Docs do not state whether `github-actions[bot]` commits count as repository activity.

**Timezone (Fact, same page):**
> By default, scheduled workflows run in UTC. You can optionally specify a timezone using an IANA timezone string for timezone-aware scheduling.

Example in docs uses `timezone: "America/New_York"`.

## Still open for Claude areas 7–8

Security / RNG deep-dive from HANDOFF areas 7–8 remain open for Claude's next pass (not blocking this integration).
