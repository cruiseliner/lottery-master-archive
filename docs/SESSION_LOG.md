# Session log

## 2026-10-23 Phase 1 foundation (executor agent, America/New_York times)

**Scope.** Owner approval "proceed" (t219u). The defaults it approved:
- repo `cruiseliner/lottery-master-archive` (public), binary `lottotool`;
- code under MIT; data attributed to NY State Gaming Commission;
- only NY winning numbers are committed, payouts stay in cache;
- stable Rust with clippy and rustfmt.

**Incident.** About 18:49 ET a box snapshot reset wiped the first working copy and the Rust toolchain. Recovery:
- reinstalled the toolchain (rustc/cargo 1.99.1);
- moved the project to `/home/box/lottery-master-archive`;
- rewrote the sources;
- pushed to GitHub as early as possible.

**Work and status:**

| Item | Status |
|---|---|
| eras.toml, schemas, docs, agent prompt | DRAFTED |
| Rust crate (11 modules) | COMPILED; clippy clean with `-D warnings` |
| Tests: 23 integration tests (odds 6, check 9, payouts 5, rng 3) | TESTED; all pass |
| `lottotool update powerball` / `update megamillions` | RUN |
| `lottotool check --fail-on-error` | RUN; 24 pass, 2 warn, 0 fail, 0 skip |
| Payout fetch (cache only) | RUN for PB 2026-10-21 national + fl, PB 2015-10-07, PB 2010-02-03 fl, MM 2026-10-20 |
| workflows | DRAFTED (update.yml dispatched once, see CURRENT_STATE) |
| scripts/relay | DRAFTED, NOT RUN (dry-run only) |

**Row counts (Fact, from `lottotool update`):**

```
powerball: fetched 2015 NY rows from https://data.ny.gov/resource/d6yy-54nr.json
  rows 2015 (2010-02-03 .. 2026-10-21), added 2015, changed 0
  null multiplier 2012-01-18..2014-01-18 rows=210 MISSING
  null double_play 2010-02-03..2021-08-21 rows=1206 NOT_APPLICABLE_INFERRED
  null double_play 2021-08-23..2026-06-01 rows=748 MISSING
megamillions: fetched 2550 NY rows from https://data.ny.gov/resource/5xaw-6ayf.json
  rows 2550 (2002-05-17 .. 2026-10-20), added 2550, changed 0
  null multiplier 2002-05-17..2011-01-14 rows=905 NOT_APPLICABLE
  null multiplier 2025-04-08..2026-10-20 rows=161 MISSING
  null double_play 2002-05-17..2026-10-20 rows=2550 NOT_APPLICABLE
```

**Fixes made during the session:**
- MM 2005 era date corrected to Fri 2005-06-24 (2005-06-22 is a Wednesday).
- chrono `serde` feature added.
- powerball.com returned random encodings, so ureq now uses gzip+brotli and up to 3 attempts.
- PB odds test tightened to the 4-decimal FL rules chart.
- Staleness message formatting fixed.

**Next:** Phase 2 review using `docs/HANDOFF_PHASE2.md`. v0.2 is NOT started.
