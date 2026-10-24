# Coverage audit — archive vs NY Open Data

**When:** 2026-10-24 ~09:10–09:15 America/New_York  
**Scope:** Read-only. Fresh NY feeds pulled to scratch (`/tmp/…`, **not** `data/`). Archive rows from `origin/main` `data/*/draws.csv`. **No** edits to `data/` or `eras.toml`.  
**Feeds:**  
- Powerball: `https://data.ny.gov/api/views/d6yy-54nr/rows.csv?accessType=DOWNLOAD`  
- Mega Millions: `https://data.ny.gov/api/views/5xaw-6ayf/rows.csv?accessType=DOWNLOAD`  

Windows (per brief / `eras.toml`):  
- Powerball: `>= 2015-10-07` (`pb-2015`)  
- Mega Millions primary: `>= 2025-04-08` (`mm-2025`, mega 1–24)  
- Mega Millions white-pool: `>= 2017-10-31` (`mm-2017` / `mm-2025`, white 1–70)

---

## Summary counts

| Window | NY rows | Archive rows | Both (matched dates) | Only NY | Only archive | Value mismatches | Dup dates (NY / AR) |
|---|---:|---:|---:|---:|---:|---:|---|
| PB ≥ 2015-10-07 | 1423 | 1423 | 1423 | 0 | 0 | 0 | 0 / 0 |
| MM ≥ 2025-04-08 | 162 | 161 | 161 | 1 | 0 | 0 | 0 / 0 |
| MM ≥ 2017-10-31 (white pool) | 938 | 937 | 937 | 1 | 0 | 0 | 0 / 0 |

Full-history file sizes at pull (header + rows): PB NY 2015 data rows / archive 2015; MM NY 2551 / archive 2550.

---

## Value compare (whites, bonus, multiplier)

On every shared draw date in the windows above: **identical** whites, bonus/mega, and multiplier (including empty MM multiplier post-2025-04-08). **0** mismatches.

---

## Range checks

| Check | Result |
|---|---|
| PB white 1–69 distinct (5) | **0** issues (archive + NY in window) |
| PB red/Powerball 1–26 | **0** issues |
| MM white 1–70 distinct (5) from 2017-10-31 | **0** issues |
| MM mega 1–24 from 2025-04-08 | **0** issues |

---

## Schedule check

Expected draw dates:  
- PB: Mon/Wed/Sat since 2021-08-23; Wed/Sat before that (within ≥ 2015-10-07)  
- MM: Tue/Fri  

End date = max(last NY row, last archive row) in each window.

| Window | Expected dates | Present both | Misses |
|---|---:|---:|---|
| PB ≥ 2015-10-07 | 1423 | 1423 | **none** |
| MM ≥ 2025-04-08 | 162 | 161 | **1** (see below) |
| MM ≥ 2017-10-31 | 938 | 937 | **1** (same date) |

### Classification of misses

| Date | Game | Class | In NY? | In archive? | Notes |
|---|---|---|---|---|---|
| 2026-10-23 | Mega Millions | **missing_from_archive** | yes | no | Expected Tue draw. Present in NY feed; archive tip still `2026-10-20` under freeze / no successful scheduled ingest yet. **Not** a holiday skip. **Not** missing from NY. |

**Holiday skips:** none detected in these windows (no expected draw date absent from **both** NY and archive).

**Missing from NY feed:** **none** in these windows. Dual-source fill protocol (step 5) therefore had **no** NY-gap rows to provenance. The single archive gap below was still confirmed externally for ingest planning after freeze.

---

## Explicit missing-date lists

### Missing from archive (present in NY)

- `2026-10-23` — Mega Millions — whites `03 07 46 49 51`, mega `05`, multiplier empty (NY CSV / JSON).

### Missing from NY feed (present in archive)

- _(none)_

### Absent from both (candidate holiday / never drawn)

- _(none)_ in PB ≥ 2015-10-07, MM ≥ 2025-04-08, or MM ≥ 2017-10-31.

---

## Provenance for 2026-10-23 Mega Millions (archive gap; not an NY gap)

| Source | Retrieved (America/New_York) | Result |
|---|---|---|
| NY Open Data `5xaw-6ayf` CSV + JSON `$where=draw_date='2026-10-23T00:00:00.000'` | 2026-10-24 ~09:10 | `winning_numbers=03 07 46 49 51`, `mega_ball=05` |
| California Lottery API `DrawGamePastDrawResults/15` (MEGA Millions) | 2026-10-24 ~09:12 | MostRecentDraw `2026-10-23`; numbers 3,7,46,49,51 + special 5 |
| LotteryUSA aggregator `lotteryusa.com/mega-millions/` | 2026-10-24 ~09:11 | Draw card “Oct 23, 2026” balls 3,7,46,49,51,5 |

**Do not hand-type into `data/`.** After Sat 10:17 ET freeze lifts and R2-03 readout closes, fill via normal `lottotool update` / ingest on a branch; run `cargo test --locked`, `check`, crosscheck; put provenance in the commit message.

---

## Method notes (Fact)

- Scratch pulls only; compared against `git show origin/main:data/.../draws.csv`.  
- Multiplier empty string treated as `None` on both sides.  
- White-pool window start taken from `eras.toml` `mm-2017` `first_draw = "2017-10-31"` (brief said “≥ 2017”).  
- Freeze held: no workflow / eras / data edits; no manual dispatch for this audit.
