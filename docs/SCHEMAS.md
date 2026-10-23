# CSV schemas

Header-only copies are in `schemas/`. A test (`tests/check.rs::schema_files_match_code`) keeps
them identical to the constants in `src/data.rs`. All dates are `YYYY-MM-DD` (draw date in ET).
An empty cell means null, and every null run must be listed in `missing.csv`.

## draws.csv (`data/<game>/draws.csv`)

| Column | Type | Notes |
|---|---|---|
| draw_date | date | unique, ascending |
| era_id | string | must exist in `eras.toml` and match the era for that date |
| w1..w5 | int | white balls in NY order (all rows ascending as of 2026-10-23; order not enforced by check), distinct, 1..white_max of the era |
| bonus | int | Powerball / Mega Ball, 1..bonus_max |
| multiplier | int or empty | Power Play / Megaplier as published by NY; empty = see missing.csv |
| dp_w1..dp_w5, dp_bonus | int or empty | Powerball Double Play; always empty for Mega Millions |
| source | string | `ny:<dataset id>` |

## missing.csv (`data/<game>/missing.csv`)

| Column | Notes |
|---|---|
| game | `powerball` / `megamillions` |
| field | `multiplier` or `double_play` |
| first_draw, last_draw | first and last draw date of the contiguous null run |
| rows | number of draws in the run |
| status | `MISSING` (exists but not in source), `NOT_APPLICABLE`, `NOT_APPLICABLE_INFERRED`, `UNDOCUMENTED` (fails check) |
| reason, source | why, and evidence |

## payouts.csv (schema only; data lives in the gitignored `cache/payouts/<game>/<date>_<scope>.csv`)

| Column | Notes |
|---|---|
| game, draw_date | |
| scope | `national` or a state code such as `fl` |
| product | `base`, `power_play`, `double_play`, `megaplier`, `builtin_<N>x` |
| tier | `W+B`, for example `5+1`, `3+0` |
| white_matched, bonus_matched | int, bool |
| winners | int or empty |
| prize_usd | int or empty (empty for jackpot / text-only prizes) |
| prize_text | as displayed, for example `Grand Prize`, `$1,000,000` |
| source_url, fetched_at | provenance (UTC RFC 3339) |
