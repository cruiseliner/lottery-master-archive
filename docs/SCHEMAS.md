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



## tickets.csv (PRIVATE: `ledger/tickets/tickets.csv`, gitignored; D-031)

Never committed (R6-04 CI guard refuses tracked `ledger/` and `tickets*.csv`). Append-only.

`ticket_id,created_at,game,draw_date,era_id,w1,w2,w3,w4,w5,special,power_play,cost_usd,note`

| Column | Meaning |
|---|---|
| ticket_id | `T0001`, `T0002`, ... in append order |
| created_at | UTC, RFC 3339 with `Z`, when the row was added |
| game | `powerball` or `megamillions` |
| draw_date | Draw the ticket is for (must be a scheduled draw weekday) |
| era_id | Era in force on draw_date (`pb-2015` or `mm-2025`) |
| w1..w5 | White balls, sorted ascending, distinct, within the era range |
| special | Powerball / Mega Ball |
| power_play | `true`/`false` (Powerball only) |
| cost_usd | Whole USD, equal to the eras.toml price (+ `power_play_price_usd`) |
| note | Free text, single line |

Void records (PRIVATE: `ledger/tickets/tickets_voids.csv`, append-only, D-031):
`ticket_id,voided_at,reason`. One row per voided ticket (`voided_at` UTC `Z`). Reading fails
closed on a wrong header, an unknown `ticket_id`, or a ticket voided twice. Ticket rows are
never changed; a corrected ticket is a new `tickets.csv` row. Flags are computed, not stored:
`LATE_ENTRY` = `created_at` at/after 22:00 ET on `draw_date` (D-030); `LATE_VOID` = `voided_at`
at/after that cutoff. Both files must end in a newline; a partial last line fails closed.

## ingest_log.csv (`data/ingest_log.csv`)

Appended by `lottotool update` only when a new draw date is added (D-028 / C-08). Not
backfilled. Idempotent on `(game, draw_date)`. Corrupt existing files fail the update
closed (D-029 / R4-01) — never rewritten as empty.

`lottotool check` rules: `ingest_log.header`, `.parse`, `.unique`, `.refs` (every row
matches a draw in that game's `draws.csv`), `.coverage` (every draw with
`draw_date > 2026-10-24` for Powerball or `> 2026-10-23` for Mega Millions has a log
row). Header-only file is fine until those post-cutoff draws appear.

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
