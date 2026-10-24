# Claude Phase 3 review triage (2026-10-24)

Sources: Claude Phase 3 review (areas from Phase 2 follow-up + areas 7–8), owner messages t226u / t227u (via parent). No v0.2 / no release tag / no PAT / relay not run against any API.

Dispositions: **FIXED** / **DISAGREE** / **DEFERRED**.

## R2 (Phase 3.1 priority batch)

| ID | Disposition | Notes |
|---|---|---|
| R2-01 | FIXED | Update runs #3 (`40053361988`) and #4 (`40053755927`) diagnosed in SESSION_LOG. #3 = concurrent rebase CONFLICT on `data/meta.json` (run #2 had pushed updated_at-only 41e1199); staleness skipped. #4 = deliberate red on 2fc0810 (MM STALE FAIL). Empty `force_now` green proof recorded after this commit. |
| R2-02 | FIXED | Removed keep-alive **commit**; retained `::warning::` + step summary. Owner calendar reminder documented (OI-18). Quotes D-013 + Claude concern in D-018. No PAT. No one-time Issue opened (calendar reminder chosen; Issue draft available on request). |
| R2-03 | FIXED | `write_meta_json`: schema_version 2, `tool_version`, per-CSV sha256; `updated_at` only when rows/last_draw/sha256 change; byte-identical skip. Local proof: second `update` left meta byte-identical. |
| R2-04 | FIXED | `eras.toml` groups derived as `{pb\|mm}-{white\|bonus}-{max}`; mm-2017 and mm-2025 share `mm-white-70`. `eras_in_pool` / `refuse_pool_mixed_maxima` / validate(); `summary --pool`; tests/pool.rs. |
| R2-05 | FIXED | 06:17 cron (`github.event.schedule == '17 6 * * *'`) passes `--stale-as-warn`; 10:17 + dispatch FAIL. |
| R2-06 | DEFERRED | Crosscheck cadence + CROSSCHECK_LOG.md — not cheap tonight; before broader live crosscheck automation. |
| R2-07 | DEFERRED (partial) | (a) `FORCE_NOW` via env DONE. (b–d) SHA-tag verify / Node24 / ubuntu-24.04 / pin rust-toolchain version DEFERRED (workflow hygiene). |
| R2-08 | FIXED | When `changed>0`, `::warning::` + upload `data/*/CHANGES.md` artifact. |
| R2-09 | FIXED | Hypergeometric `expected_white_matches` / `variance_white_matches`; `sum(ways)==total()` test; PRIZE_TIERS labelled current-charts-only. R-15 full text was unrecovered; used Claude restatement from t226u (honest note). |

## R3 (areas 7–8, t227u)

| ID | Disposition | Notes |
|---|---|---|
| R3-01 | FIXED | Relay: path allowlist, block `.git/`, scan outbound payload for secret patterns and refuse on hit. Dry-run exercised; **API not called**. |
| R3-02 | FIXED | `agent/REVIEWER_PROMPT.md`; relay uses it (not AGENT_PROMPT). D-019: why Anthropic over Grok; relay output untrusted / never auto-applied. |
| R3-03 | DEFERRED | Before first live relay run (remaining relay hardening). |
| R3-04 | FIXED | `validate_state`: exactly two lowercase letters; wired into `powerball_url`. |
| R3-05 | FIXED | No retry on ordinary 4xx; honor Retry-After on 429; min gap between requests; per-run request cap; robots.txt status in `data/SOURCES.md`. |
| R3-06 | FIXED | CI step fails if `git ls-files cache` non-empty or tracked `*payouts*.csv` has rows beyond header. |
| R3-07 | FIXED | Golden-vector RNG test (seed 1, first five picks); crate versions tightened (`ureq = "=2.12.1"`, `sha2 = "=0.10.9"`; others already exact-ish). |
| R3-08 | FIXED | Stronger RNG tests: white-ball chi-square, exact hypergeometric ways, `below()` uniformity, `zone % n == 0`. |
| R3-09 | DEFERRED | v0.3 design note → OI-19. |

## Labels

Claims below use Fact / Inference / High Possibility as required by project standing rules.
