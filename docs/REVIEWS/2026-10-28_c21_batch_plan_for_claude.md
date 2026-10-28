# C-21 post-Wednesday batch plan — for Claude review

**Status: NOT EXECUTED. Awaiting Claude confirm before any of the five steps land on main.**

Date drafted: 2026-10-28 (America/New_York).
Repo tip at draft time (local): includes unpushed docs commit(s) after `bd60431` data; plan sketches match code/workflows on `main` as of `bd60431` + docs.

This document is what Adam can paste to Claude so Claude can give a real confirm. It follows Claude’s C-21 order (owner-pasted COLLAB 2026-10-27). Each step = **one commit** (code or workflow), then a **green manual `workflow_dispatch`** of `update`, then a **no-op check** (no unexpected data commit; `meta.json` + `ingest_log.csv` byte-identical when no new draw).

Motivation Claude already noted: GitHub annotations force Node 20→24 and `ubuntu-latest`→Ubuntu 26 migration (since ~2026-10-19), so pinning `ubuntu-24.04` and Node-24 action majors is well-motivated.

C-20 Wed MM acceptance HOLDS (with UTC-cron lateness correction). Do **not** start this batch until Claude confirms the plan/diffs below.

---

## Preconditions (already green; do not re-open)

- Tue PB acceptance: PASS (C-18).
- Wed MM acceptance: HOLDS (C-20) — data [`bd60431`](https://github.com/cruiseliner/lottery-master-archive/commit/bd604310366167d85dc348f436899bcb5cf0839f), pickup [40727739450](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40727739450), no-op [40755401293](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40755401293).
- Wed lateness corrected to UTC-cron basis: `17 6` → **10h20m** late; `17 10` → **9h16m** late (bears on D-027).
- Discipline: C-09 / R4-05 — one workflow change per commit where practical; C-21 groups code first, then discrete workflow commits.

---

## Step 1 — code: R6-01, R6-02, R6-03, R5-02

**Files:** `src/ingest_log.rs`, `src/data.rs`, `src/check.rs`, tests under `src/ingest_log.rs` / `tests/check.rs` as needed.  
**Why:** Claude R6 review (COLLAB 2026-10-26) + C-13 (R5-02 deferred) + C-21 item 1.

### R6-01 — update `have` inside the append loop (or dedupe)

**Current (fact):** `src/ingest_log.rs` builds `have: HashSet<(String, NaiveDate)>` once from `existing` (lines ~48–51) and never inserts into it inside the `for &draw_date in new_dates` loop. A `new_dates` slice with the same date twice would append two identical keys; later `check` `ingest_log.unique` would FAIL.

**Intended change:**
- After each successful push into `existing`, `have.insert(key);`, **or** dedupe `new_dates` up front.
- Add a unit test that passes a duplicate date in `new_dates` and asserts a single row / `added == 1`.

### R6-02 — refuse rewrite when existing header ≠ `INGEST_LOG_HEADER`

**Current (fact):** Append path never calls `data::header_of`. `data::read_csv` uses `csv::Reader` deserialize (tolerates column reorder / extra columns depending on serde field mapping). After a successful parse, `data::write_csv(..., INGEST_LOG_HEADER)` would rewrite with the canonical header and drop extras.

**Intended change:**
- Before rewrite: `if p.exists() { anyhow::ensure!(data::header_of(&p)? == INGEST_LOG_HEADER, ...); }`
- Add an extra-column fixture test that asserts `Err` and unchanged bytes (same pattern as the corrupt-file tests already present for R4-01 / D-029).

### R6-03 — confirm / fix atomic writes

**Current (fact, verified by reading code):**
- `data::write_csv` (`src/data.rs` ~118–131): opens the destination path directly via `csv::WriterBuilder::from_path(path)` — **not** temp-file + rename.
- `write_meta_json` (`src/main.rs` ~681–686): `std::fs::write(&path, body)` — **not** temp-file + rename.

So R6-03’s “confirm it writes to a temp file and renames” answer is: **it does not today**. CI remains safe because the commit follows the integrity gate, but a local kill mid-write can leave a truncated `ingest_log.csv` / `draws.csv` / `meta.json`.

**Intended change (same code commit as R6-01/02/R5-02, or a tiny follow-up if Claude prefers split):**
- Implement write-to-temp-in-same-dir + `rename` for `write_csv` and `write_meta_json` (covers ingest_log, draws, missing, meta).
- Note in commit message that this closes R6-03 by fixing, not only confirming.

### R5-02 — ingest_log field-format checks

**Current (fact):** `check` already has `ingest_log.{present,header,parse,unique,refs,coverage}` (D-029). Claude C-13 still lists **R5-02 field-format checks** as open; COLLAB does not spell a longer checklist beyond C-08 field semantics.

**Intended change (proposed; Claude should confirm or tighten):**
- Add `ingest_log.fields` (name flexible) that, for every row:
  - `game` ∈ `{powerball, megamillions}`
  - `draw_date` parses as `YYYY-MM-DD` (already typed via `NaiveDate` on parse — keep explicit)
  - `ny_rows_updated_at` is either empty **or** RFC3339 UTC (`…Z`)
  - `run_started_at` is RFC3339 UTC
  - `schedule_slot` is `manual` **or** a cron string matching the known slots (`17 6 * * *` / `17 10 * * *`) — or, if Claude prefers looser, “non-empty and not `local`”
- Fixture tests for blank-OK `ny_rows_updated_at`, bad timestamp → FAIL, bad slot → FAIL.

**Step 1 gate:** `cargo test --locked` + `cargo run --locked -- check --fail-on-error --no-stale` green locally; push; **manual dispatch** of `update` green; if no new draw, tip data files byte-identical (no-op).

---

## Step 2 — workflow: pin `runs-on` to `ubuntu-24.04`

**Files:** `.github/workflows/update.yml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml` (all currently `ubuntu-latest` / matrix `ubuntu-latest`).  
**Why:** C-21 item 2; GH annotations show `ubuntu-latest` migrating to Ubuntu 26 from ~Oct 19 — image can change under scheduled runs. D-007/D-012 schedule must not silently change runner OS mid-flight.

**Intended change:**
- Replace every `runs-on: ubuntu-latest` (and release matrix `os: ubuntu-latest`) with `ubuntu-24.04`.
- No action SHA bumps in this commit (those are step 5).
- Commit message cites C-21 / A-03 sequencing (pin OS before Node majors).

**Step 2 gate:** green manual `update` dispatch + no-op check; CI on the docs/code tip also green.

---

## Step 3 — workflow: R4-02 + R4-03

**File:** `.github/workflows/update.yml` (diagnostics / `fetch_views` block ~lines 90–120; update steps that tee `--json`).  
**Why:** Claude R4-02 / R4-03 (COLLAB); C-21 item 3; C-09 one-cause discipline (this commit = summary hygiene only).

### R4-02 — stop splicing network `${epoch}` into Python source

**Current (fact):**
```bash
epoch=$(curl … | python3 -c '… print(d.get("rowsUpdatedAt") or "")')
…
python3 -c "… int('${epoch}') …"
```
Network text is interpolated into a Python program string.

**Intended change:** one Python process reads the views JSON from stdin (or a temp file), requires `rowsUpdatedAt` to be numeric, converts to UTC RFC3339, prints it — **no** shell interpolation of the epoch into code. On missing/non-numeric: WARN + empty string (never fail the update), matching D-028.

### R4-03 — single source for summary vs logged values

**Current (fact):** shell `fetch_views` fills the step summary; the tool separately records `ny_rows_updated_at` for ingest_log (optional `--json` fields already exposed since [`26fdde6`](https://github.com/cruiseliner/lottery-master-archive/commit/26fdde6093a5c078515b2a4aa96c26f92e3d6da0) for later wiring). Two fetches can diverge.

**Intended change (pick one; Claude may prefer A):**
- **A (preferred):** drop shell views fetch for the values that go into ingest_log; print summary lines from the tool’s `--json` (`ny_rows_updated_at` / recorded fields) after each `update` step.
- **B:** keep one shell fetch, pass the exact strings into the tool via env/flags so summary and log are identical by construction.

**Step 3 gate:** green manual dispatch; job summary shows the same `rowsUpdatedAt` strings the tool recorded; no-op when no draw.

---

## Step 4 — workflow: R6-04 CI guard for tracked `ledger/`

**File:** `.github/workflows/ci.yml` (R3-06 step ~lines 37–56).  
**Why:** Claude R6-04; D-030 private ledger under gitignored `ledger/`; C-21 item 4.

**Current (fact):**
- `.gitignore:15` already has `/ledger/` — `git check-ignore -v ledger/x` → ignored (confirmed earlier).
- CI refuses tracked `cache/` and non-header `*payouts*.csv` only; **no** `ledger/` check yet.

**Intended change:** extend the R3-06 step:
```bash
tracked_ledger=$(git ls-files ledger || true)
if [ -n "$tracked_ledger" ]; then
  echo "::error::Tracked files under ledger/ (private picks; must stay gitignored):"
  echo "$tracked_ledger"
  exit 1
fi
```
Update the OK echo to mention ledger/.

**Step 4 gate:** CI green on main; optional throwaway proof branch (same pattern as A-06 R3-06 cache proof) if Claude wants a red proof — not required to start the guard.

---

## Step 5 — workflow: Node-24 action bumps + SHA↔tag verification

**Files:** `.github/workflows/{update,ci,release}.yml`  
**Why:** A-03 / C-09 / C-21 item 5; GH forcing Node 20→24 on older action majors.

**Current pins (fact, A-03 table):**

| Action | Comment tag | Pinned SHA |
|---|---|---|
| actions/checkout | v4.4.0 | `11d5960a326750d5838078e36cf38b85af677262` |
| actions/cache | v4.3.0 | `0057852bfaa89a56745cba8c7296529d2fc39830` |
| actions/upload-artifact | v4.6.2 | `ea165f8d65b6e75b540449e92b4886f43607fa02` |
| actions/download-artifact | v4.2.1 | `95815c38cf2ff2164869cbab79da8d1f422bc89e` |

**Offline note already in COLLAB:** first Node-24 majors are checkout@v5, cache@v5, upload-artifact@v6, download-artifact@v7.

**Intended change:**
1. Resolve each target tag’s commit via `git ls-remote` / GitHub API.
2. Replace `uses:` lines with `uses: actions/<name>@<fullsha> # <tag>`.
3. Record SHA↔tag verification table in the commit message (and COLLAB FOR CLAUDE), same shape as the A-03 verify table.
4. Do **not** change cron / `timezone:` / eras / data in this commit.
5. Optional exact `rust-toolchain` pin only if Claude still wants it in A-03; otherwise leave toolchain install as today.

**Step 5 gate:** green manual `update` dispatch + no-op; CI green.

---

## Explicit non-goals / holds

- **No private pick numbers** in COLLAB, commits, or run summaries (C-12 / D-030).
- **No** cron or `timezone:` edits in this batch (D-027 remains UNRESOLVED; lateness math corrected in docs only).
- **No** starting any step until Claude confirms this plan (or a revised one).
- C-21 batch remains held even though C-20 HOLDS.

---

## Ask for Claude

1. Confirm or amend the five-step order and the intended diffs above.
2. Especially: R5-02 field rules (proposed list OK?), R6-03 fix-in-step-1 vs confirm-only, R4-03 option A vs B.
3. On confirm: Adam/Grok execute step 1 → green dispatch → … → step 5, pasting each commit SHA + run URL back into COLLAB FOR CLAUDE.

**— end of paste —**
