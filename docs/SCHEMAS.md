# CSV schemas

Header-only copies are in `schemas/`. A test (`tests/check.rs::schema_files_match_code`) keeps
them identical to the constants in `src/data.rs`. All dates are `YYYY-MM-DD` (draw date in ET).
An empty cell means null, and every null run must be listed in `missing.csv`.

## draws.csv (`data/<game>/draws.csv`)

| Column | Type | Notes |
|---|---|---|
| draw_date | date | unique, ascending |
| era_id | string | must exist in `eras.toml` and match the era for that date |
| w1..w5 | int | white balls in NY order; **must be strictly ascending** (`draws.whites_ascending` FAIL); distinct; 1..white_max of the era |
| bonus | int | Powerball / Mega Ball, 1..bonus_max |
| multiplier | int or empty | Power Play / Megaplier as published by NY; empty = see missing.csv |
| dp_w1..dp_w5, dp_bonus | int or empty | Powerball Double Play; always empty for Mega Millions |
| source | string | `ny:<dataset id>` |

## missing.csv (`data/<game>/missing.csv`)

| Column | Notes |
|---|---|
| game | `powerball` / `megamillions` |
| field | `multiplier`, `double_play`, or `draw` (MISSING_DRAW schedule gaps) |
| first_draw, last_draw | first and last draw date of the contiguous null run |
| rows | number of draws in the run |
| status | `MISSING`, `NOT_APPLICABLE`, `NOT_APPLICABLE_INFERRED`, `MISSING_DRAW` (schedule gap WARN), `UNDOCUMENTED` (fails check) |
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



## ingest_log.csv (`data/ingest_log.csv`)

Appended by `lottotool update` only when a new draw date is added (D-028 / C-08). Not
backfilled. Idempotent on `(game, draw_date)`.

| Column | Notes |
|---|---|
| game | `powerball` / `megamillions` |
| draw_date | newly added draw date |
| ny_rows_updated_at | Socrata views API `rowsUpdatedAt` for that game's dataset (UTC RFC 3339); **dataset-level upper bound** on posting time; empty if the views call failed (WARN only) |
| run_started_at | UTC RFC 3339 of the update run start |
| schedule_slot | cron string for scheduled runs; `manual` for `workflow_dispatch`; `local` for CLI without env |

## meta.json (`data/meta.json`)

Written by `lottotool update`. Fields: `schema_version`, `tool_version`, `updated_at` (UTC RFC 3339), `ingest_log_csv_sha256`, `games.<id>.{rows,first_draw,last_draw,ny_dataset,ny_dataset_url,current_era,draws_csv_sha256,missing_csv_sha256}`. `updated_at` and the file bytes change only when game row/sha fields or `ingest_log_csv_sha256` change (R2-03).

## CHANGES.md (`data/<game>/CHANGES.md`)

Written only when an update changes values on already-committed draw dates (R-05). Markdown table of draw_date / field / old / new.

## Era pooling keys (`eras.toml`)

Optional per-era `white_group` / `bonus_group` (default `"white"` / `"bonus"`). Document independent pools for odds (R-08).
