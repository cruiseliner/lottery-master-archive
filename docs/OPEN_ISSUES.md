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
| OI-09 | CI | Whether bot commits count as "repository activity" for the 60-day schedule disable | VERIFY (now WARN on non-bot age; keepalive keys on non-bot) |
| OI-10 | CI | Staleness grace of 30h is based on 2 observations of NY `rowsUpdatedAt` (about 03:33-03:38 ET). A slower NY update will turn the run red; that is intended but may be noisy | WATCH |
| OI-11 | Payouts | The powerball.com parser depends on the HTML structure (`winners-table`, `data-label`, `m{N}-pb` classes). Layout changes make the fetch fail loudly (tested) | WATCH |
| OI-12 | Relay | `scripts/relay/relay.py` is DRAFTED and NOT RUN against the API; only `--dry-run`, path blocking and the redaction helper were exercised | OPEN |
| OI-13 | Release | `release.yml` is untested (no tag pushed) | OPEN |
| OI-14 | Spec | Kickoff spec section 6 was not available to the Phase 1 executor; the build followed the parent's explicit list | VERIFY |
| OI-15 | Check | Official-source winning-number crosscheck (`--crosscheck N`) | DONE (R-06); keep off schedule |
| OI-16 | CI | Confirm first real scheduled run starts at 06:17 America/New_York after dual-cron+timezone change | WATCH (first window Sat 2026-10-24) |
| OI-17 | Review | Claude R-15 exact ask unrecovered this session (DEFERRED) | OPEN |

