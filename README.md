# lottery-master-archive

A public, validated archive of **Powerball** and **Mega Millions** winning numbers plus
`lottotool`, a small Rust CLI that updates the archive, checks it, documents every gap, and
computes exact odds per prize tier.

> Not affiliated with, endorsed by, or sponsored by MUSL, the Mega Millions consortium, the
> New York State Gaming Commission, the Florida Lottery or any lottery.
> Always confirm numbers with the official lottery.

## What is in the repo

| Path | Contents |
|---|---|
| `data/powerball/draws.csv` | Powerball draws 2010-02-03 onward (NY Open Data `d6yy-54nr`) |
| `data/megamillions/draws.csv` | Mega Millions draws 2002-05-17 onward (NY Open Data `5xaw-6ayf`) |
| `data/<game>/missing.csv` | Every blank field as a documented range with status and reason |
| `eras.toml` | Matrix eras keyed on first draw date, schedules, null rules; each value `sourced`, `inferred` or `missing` |
| `schemas/*.csv` | Header-only CSV schemas (`payouts.csv` is schema-only in the public repo) |
| `docs/` | `SCHEMAS.md`, `CURRENT_STATE.md`, `DECISIONS.md`, `OPEN_ISSUES.md`, `SESSION_LOG.md`, `REVIEWS/` |
| `agent/AGENT_PROMPT.md`, `AGENTS.md` | Rules for AI agents working on this repo |

## lottotool (v0.1)

```
cargo build --release
./target/release/lottotool update powerball        # NY Open Data -> data/powerball/*.csv
./target/release/lottotool update megamillions
./target/release/lottotool check --fail-on-error   # integrity + staleness; exit 1 on FAIL
./target/release/lottotool eras                     # eras with status labels and jackpot odds
./target/release/lottotool eras --game powerball --simulate 1000000 --seed 42
./target/release/lottotool summary megamillions --era mm-2025
./target/release/lottotool export powerball --format json --from 2025-01-01
```

* `--json` works on every command (global flag).
* `check` prints PASS / WARN / FAIL / SKIP per check; every SKIP states its reason.
  `--no-stale` runs integrity checks only; `--now <RFC3339>` evaluates staleness at a fixed time.
* Simulation uses a seeded ChaCha20 RNG (`rand_chacha`), so the same seed gives the same output.
* `update <game> --payouts --date YYYY-MM-DD [--state fl]` fetches per-tier winners and prizes
  (powerball.com draw-result pages, megamillions.com draw API) into the **gitignored `cache/`**.
  Those tables are never committed (see Limitations).

### Ticket ledger and grade (v0.2 Step 1, D-031)

A private, append-only ledger for tickets you actually bought, checked against the archive.

```
lottotool ticket add --game pb --draw-date 2026-11-11 --whites 3,14,22,41,60 --special 9 --power-play [--cost 3] [--note "..."]
lottotool ticket add --game mm --draw-date 2026-11-13 --whites 7,8,9,10,11 --special 12
lottotool ticket void T0001 --reason "typo in white ball"
lottotool ticket list [--game pb] [--draw-date YYYY-MM-DD]
lottotool grade                     # pending tickets + tickets for each game's latest archived draw
lottotool grade --all [--game mm] [--draw-date YYYY-MM-DD]
```

* `pb` / `mm` are aliases for `powerball` / `megamillions`.
* The ledger is `ledger/tickets/tickets.csv` under the **gitignored `ledger/`**. Rows are only
  appended (never rewritten); `created_at` is UTC. `ticket add` refuses to write if the ledger
  path is not gitignored, and fails closed on an unexpected header.
* `ticket add` validates against `eras.toml` for the draw date: scheduled draw weekday, ball
  ranges, five distinct whites, Power Play only for Powerball, and the price (`price_usd`, plus
  `power_play_price_usd`). `--cost`, if given, must equal that price (strict by decision, D-031). Only eras whose prize
  chart is encoded (`pb-2015`, `mm-2025`) are accepted.
* `ticket void <id> --reason ...` appends a record to `ledger/tickets/tickets_voids.csv`; the
  ticket row is never changed. Unknown ids and second voids are refused. `ticket list` marks
  voided tickets `VOID`; `grade` leaves them out. Re-enter a corrected ticket with `ticket add`
  (it gets a new id).
* `grade` reports white matches, special-ball match, the tier (e.g. `3+1`), whether it is a
  prize tier on the current chart, and the archive multiplier (Powerball Power Play) when the
  ticket has it and the archive row has a value. A special ball only matches the drawn special
  ball, never a drawn white. Draws not yet in `data/` are `PENDING`.
* Prize **amounts** come only from the private payout cache (`update <game> --payouts`). If
  the cache has no single unambiguous amount for the tier (no cache, jackpot / pari-mutuel
  text, several amounts), `grade` prints `TIER_LOOKUP_NEEDED`; it never computes or invents an
  amount. Grades are computed on demand and never written back into the ledger.
* Nothing here predicts numbers. A grade is a record of what happened, not a strategy.

## Automation

* `.github/workflows/update.yml` runs daily at 06:17 America/New_York (after every draw night):
  update both games, integrity gate, commit, then a full check that fails visibly if a due draw
  is missing. There is also a 45-day keep-alive commit, because GitHub disables schedules in
  public repos after 60 days without activity. It can also be run by hand (`workflow_dispatch`).
* `.github/workflows/crosscheck.yml` runs weekly (Sundays 16:43 America/New_York) and compares
  the latest 8 archived draws per game with the official winning-number pages. It is read-only;
  a mismatch fails the run (D-025).
* `.github/workflows/ci.yml` runs fmt, clippy `-D warnings`, the tests, and the data integrity check.
* `.github/workflows/release.yml`: on a `v*` tag, it builds a Linux binary and publishes it with `SHA256SUMS`.

## Data attribution

Winning numbers come from the **New York State Gaming Commission** via data.ny.gov:
"Lottery Powerball Winning Numbers: Beginning 2010" (`d6yy-54nr`) and "Lottery Mega Millions
Winning Numbers: Beginning 2002" (`5xaw-6ayf`). The datasets' license field was **null** when
checked on 2026-10-23. NY's notes say the official Drawing Unit results prevail over any
discrepancy. See `data/SOURCES.md`.

## Limitations

* **Payouts are not published here.** The powerball.com terms (MUSL) restrict reproduction
  without written consent, and the Florida Lottery terms restrict derivative works and use of
  its name. **Permission from MUSL and from the Florida Lottery is pending**, so per-tier payout
  and winner tables stay in the local cache. `schemas/payouts.csv` is a header only.
* **Pre-2015 Powerball and pre-2010 Mega Millions eras are inferred** from the observed data
  (ball maxima, first and last dates) or marked MISSING. No official source has been retrieved for
  them. `check` flags range checks on those rows as WARN because the check is circular there.
* **Gaps in the source:**
  * The Powerball multiplier is blank 2012-01-18..2014-01-18 (MISSING).
  * Double Play numbers exist in NY data only from 2026-06-03; 2021-08-23..2026-06-01 is MISSING.
  * The Mega Millions multiplier is blank before 2011-01-18 (not applicable) and since
    2025-04-08 (built-in multiplier era, MISSING).
  * All of these are listed in `data/<game>/missing.csv`.
* Draw-day schedules before the first sourced change are inferred from the data.
* The scheduler can be delayed or skipped by GitHub under load. Staleness turns the workflow red
  rather than hiding the problem.
* There are no tax, annuity, expected-value or jackpot-size calculations yet (v0.2 Step 4, not started).

## Roadmap

* **v0.1 (this release):** archive, eras, missing-data registry, check, summary, export,
  exact odds, seeded simulation, scheduled updates.
* **v0.2 (not started):** to be scoped in the Phase 2 review (`docs/HANDOFF_PHASE2.md`).
  Candidates: sourced pre-2015/pre-2010 era data, payout publication if permission is granted,
  a private local ticket checker (personal ledger kept out of the repo), and expected-value and
  tax views.

## License

Code: MIT (`LICENSE`). Data: see the attribution above and `data/SOURCES.md`.
