# Open issues

| ID | Area | Issue | Status |
|---|---|---|---|
| OI-01 | Licensing | MUSL (powerball.com) and Florida Lottery permission to publish payout and winner tables. Until granted, these stay in the local cache only | PENDING (owner) |
| OI-02 | Licensing | megamillions.com terms of use not found (`/Terms-of-Use.aspx` returned 404). Payouts are treated as restricted | MISSING |
| OI-03 | Eras | Official start dates, matrices and prices for Powerball before 2015-10-07 and Mega Millions before 2010 are not sourced (inferred from data or MISSING) | OPEN |
| OI-04 | Eras | The exact first draw of the PB 59/35 era lies in 2011-12-10..2012-01-18. The era boundary 2012-01-18 is inferred, and the null-multiplier range starts on the same date | OPEN |
| OI-05 | Data | PB Double Play 2021-08-23..2026-06-01 (748 draws) is not in NY data (MISSING). A possible source is the Florida Lottery history PDF, but its terms restrict reuse | OPEN |
| OI-06 | Data | MM multiplier since 2025-04-08 is MISSING in NY data (built-in multiplier era). Whether the megamillions.com API exposes a per-draw multiplier for this era is not verified | OPEN |
| OI-07 | Data | PB multiplier blank 2012-01-18..2014-01-18 (210 draws). NY gives no explanation | OPEN |
| OI-08 | Check | White-ball order is not enforced (all rows are ascending today) | CLOSED (R-13: `draws.whites_ascending` FAIL) |
| OI-09 | CI | Whether bot commits count as "repository activity" for the 60-day schedule disable | VERIFY (WARN on non-bot age; **no** keep-alive commit — D-018 / R2-02) |
| OI-10 | CI | Staleness grace of 30h is based on NY `rowsUpdatedAt` posting ~03:38 ET (~4h39m after the 23:00 ET draw) on two observations (C-19 / D-007). Remaining lag after NY posts is scheduler lateness (Tue 2026-10-27 samples: 5h36m on `17 6` / 5h09m on `17 10`). A slower NY update will turn the run red; that is intended but may be noisy | WATCH |
| OI-11 | Payouts | The powerball.com parser depends on the HTML structure (`winners-table`, `data-label`, `m{N}-pb` classes). Layout changes make the fetch fail loudly (tested) | WATCH |
| OI-12 | Relay | `scripts/relay/relay.py` is DRAFTED and NOT RUN against the API; only `--dry-run`, path blocking and the redaction helper were exercised | OPEN |
| OI-13 | Release | `release.yml` untested on C-21 stack (rustc 1.99.1, upload v6→download v7, windows-2022/macos-14, `cfg(unix)` atomic tests). **Before next real release:** dry run via `workflow_dispatch` if added, or build-only on a branch — **do not** test with a public tag (Claude Step 5 sign-off). Re-stated as still unproven in Claude's C-21 sign-off (2026-10-31, paraphrase) | OPEN |
| OI-14 | Spec | Kickoff spec section 6 was not available to the Phase 1 executor; the build followed the parent's explicit list | VERIFY |
| OI-15 | Check | Official-source winning-number crosscheck (`--crosscheck N`) | DONE (R-06); keep off schedule |
| OI-16 | CI | Confirm first real scheduled run starts at 06:17 America/New_York after dual-cron+timezone change | WATCH — as of 2026-10-24 ~08:53 ET, **zero** `event=schedule` runs in Actions (06:17 not yet observed) |
| OI-17 | Review | Claude R-15 / R2-09 hypergeometric helpers | CLOSED (R2-09; exact R-15 text was unrecovered, used Claude restatement) |
| OI-18 | CI | Owner calendar reminder: touch the public repo before the 60-day schedule disable (no bot keep-alive commit) | OPEN (R2-02) |
| OI-19 | Design | R3-09 v0.3: before v0.3 starts, capture **per-ticket derived seeds** (master seed, model, draw date, index) + **algorithm version** | DEFERRED to v0.3 (gate spelled out 2026-10-24) |
| OI-20 | CI | R2-06 crosscheck cadence + CROSSCHECK_LOG.md; trigger = before v0.2 grade ships | OPEN (log started 2026-10-24; cadence still DEFERRED) |
| OI-21 | CI | R2-07 SHA↔tag; Node-24 action bumps + rust-toolchain **1.99.1** + ubuntu-24.04: **DONE** C-21 Step 5 (`fe4f5f1`; CI [40794466054](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40794466054); dispatch [40794598258](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40794598258) no-op; Node 20 annotations gone) | **CLOSED** 2026-10-28 |
| OI-22 | Relay | R3-03 before first live call: (1) dry-run (2) one tiny live call (3) spend limit; allowlisted files only; relay output untrusted / never auto-applied | DEFERRED (conditions accepted 2026-10-24) |
| OI-23 | CI | If timezone is ignored and schedule falls back to fixed UTC crons, those crons drift by 1h after 2026-11-01 DST; A-07 DST tests cover **check** due-logic only, not cron wall-clock — plan a DST-aware schedule fallback before adopting UTC crons | OPEN (Claude t233u #3; follow-up) |
| OI-24 | Test | Add a test that a failing write or rename after the temp file exists leaves no stray `.#lottotool-tmp.*` (Claude Step 5 carry-over) | OPEN |
| OI-25 | CI | Fix table-row rendering in the update job summary (Claude Step 5 carry-over) | OPEN |
| OI-26 | CI | Assign `added=$(jq -er …)` first so `set -e` catches a missing key (Claude Step 5 carry-over) | OPEN |
| OI-27 | CI | Local pre-push hook running the same cache/ledger/tickets greps as ci.yml (CI only catches after push) | OPEN |
| OI-28 | CI | Quarterly review of pins: `ubuntu-24.04`, `windows-2022`, `macos-14`, rustc `1.99.1`, and the four Node-24 action SHAs | OPEN |
| OI-29 | Docs | Commit `f552fef` title says “6h20m late”, and the delay table had been switched to a UTC-cron 10h20m. **Resolution 2026-10-31:** the original `f552fef` title (6h20m, ET basis) was correct. The `timezone:` key is documented (D-027 resolved), and the delay table is back on the ET basis with the UTC-cron figures marked superseded. No history rewrite | **CLOSED** 2026-10-31 |
| OI-30 | CI | `update.yml` `data-changes` artifact (upload-artifact v6, gated on `changed != '0'`) has never run: no live run has changed an existing row yet. C-21 sign-off lists it as not exercised. Watch for it on the first run with `~N` > 0; do not force it | WATCH |
