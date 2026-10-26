# Collaboration board

COLLAB PROTOCOL (set by owner and Claude, 2026-10-24)

1. Claude cannot read or write this file by himself. He reads it only when the owner opens a chat with him and shares the link. Nothing here is a review until the owner pastes his reply.
2. Sections: "FOR CLAUDE" (open questions, with commit SHAs, run URLs, test counts) and "FROM CLAUDE" (replies pasted by the owner, verbatim, with the paste date). Never write text attributed to Claude.

---

Public overnight meeting point for `cruiseliner/lottery-master-archive`.
Focus: **Rust `lottotool` + this repo only** (no Lottery Master bot work here).

## FOR CLAUDE

### 2026-10-24 overnight evidence (Grok / A-01..A-07 allowed subset)

**Phase 3.1 commit SHAs (A-01):**
- `655e6cc` — Phase 3.1: Claude R2+R3 (meta no-op, pools, soft stale, relay/RNG hardening)
- `c8945e6` — docs: record Phase 3.1 CI and empty-force_now run ids
- `7bcfe20` — docs: COLLAB board + inbox (pre-protocol; superseded by this file’s protocol header)
- `2e7d563` — COLLAB protocol + A-01/A-02/A-07 docs + DST tests + OI/DECISIONS notes
- `4237273` — fill overnight SHA into COLLAB/ack (docs-only)

**Empty `force_now` green run (A-01):**
https://github.com/cruiseliner/lottery-master-archive/actions/runs/40055704478

**`cargo test --locked` (A-01, re-run after A-07 DST tests):** **39 passed** (0 failed). Breakdown: lib 2, check 13, odds 8, payouts 5, pool 4, rng 7.

**A-02 crosscheck (2026-10-23 ~23:00 America/New_York):**
- Command: `lottotool check --no-stale --crosscheck 10`
- Result: **PASS** — powerball 10/10, megamillions 10/10; summary 26 pass / 0 warn / 0 fail / 4 skip
- Log: [`docs/CROSSCHECK_LOG.md`](CROSSCHECK_LOG.md)
- R2-06 trigger (recorded): **before v0.2 grade ships**

**A-03 (verify only tonight — workflow bumps FROZEN until after Sat 10:17 ET):**
All current pins match their tags via `git ls-remote` / GitHub API:
| Action | Comment tag | Pinned SHA | Verify |
|---|---|---|---|
| actions/checkout | v4.4.0 | `11d5960a326750d5838078e36cf38b85af677262` | OK |
| actions/cache | v4.3.0 | `0057852bfaa89a56745cba8c7296529d2fc39830` | OK |
| actions/upload-artifact | v4.6.2 | `ea165f8d65b6e75b540449e92b4886f43607fa02` | OK |
| actions/download-artifact | v4.2.1 | `95815c38cf2ff2164869cbab79da8d1f422bc89e` | OK |
Node-24 / `ubuntu-24.04` / exact `rust-toolchain` bumps **DEFERRED** (workflows/eras/data frozen until Sat 10:17 ET completes). Offline note: first Node-24 majors are checkout@v5, cache@v5, upload-artifact@v6, download-artifact@v7. Installed rustc: `1.99.1 (58cf8f9c3 2026-10-13)`.

**A-04 R3-03 conditions (accepted):** dry run → one tiny live call → spend limit; allowlisted files only; relay output untrusted / never auto-applied. Recorded in OPEN_ISSUES OI-22 + DECISIONS.

**A-05 OI-19:** before v0.3 must capture per-ticket derived seeds (master seed, model, draw date, index) + algorithm version. Status remains DEFERRED to v0.3 with that gate spelled out.

**A-06 R3-06 red proof:**
- Throwaway branch `proof/r3-06-tracked-cache` @ `5b27b2c` with tracked `cache/PROOF_SHOULD_FAIL.txt`
- Draft PR #1 (closed, not merged)
- Red CI: https://github.com/cruiseliner/lottery-master-archive/actions/runs/40056800648
  - Failed step: **Refuse tracked cache/ or non-header payouts CSVs (R3-06)** (all prior steps green)
- Remote branch deleted; local branch deleted; main has no junk

**A-07 DST tests added** (`tests/check.rs`):
- `dst_fallback_2026_11_01_due_at_1100_utc` — `--now` 2026-11-01T10:59:00Z not due / 11:01Z due (PB Sat 2026-10-31)
- `dst_spring_forward_2027_03_14_due_at_1000_utc` — 2027-03-14T09:59:00Z not due / 10:01Z due (PB Sat 2027-03-13)

**R2-04 pool refusal test names:** `eras_toml_groups_are_derived_from_max`, `mm_2017_and_2025_share_white_pool_not_bonus`, `pb_2012_and_2015_do_not_share_pools`, `refuses_pool_with_differing_maxima`

**R3-08 test names:** `below_uniform_and_zone_multiple_of_n`, `white_ball_chi_square_reasonable`, `exact_hypergeometric_white_match_ways` (plus R3-07 `golden_vector_seed_1_first_five_picks`)

**Paste-ready reply:** [`docs/REVIEWS/2026-10-24_grok_phase31_ack_response.md`](REVIEWS/2026-10-24_grok_phase31_ack_response.md)

**Saturday watch:** 06:17 / 10:17 ET results will be appended by a separate routine. Workflows / eras.toml / data/ are **frozen** until 10:17 completes.


### 2026-10-24 ~08:58 America/New_York — schedule diagnosis read-only checks (t234u)

**Fact — workflow state:** `gh workflow list` / API: workflow `update` (id 394656475, path `.github/workflows/update.yml`) **`state: active`**. Not disabled. Repo `disabled: false`, Actions present.

**Fact — workflow-file error banner:** Actions page https://github.com/cruiseliner/lottery-master-archive/actions/workflows/update.yml shows **5 runs**, all labeled “Manually run by cruiseliner”. HTML parse: **no** “Invalid workflow file”, “Unable to process”, “This scheduled workflow is disabled”, or “Schedule has been disabled” banner. (Only the generic hidden `ajax-error-message` flash-error template, not a workflow validation banner.)

**Fact — YAML indent on main (`origin/main` tip `159dd5e`):** `timezone:` is indented **inside the same list item** as `cron:` (not a sibling of `schedule:`):

```yaml
on:
  schedule:
    - cron: "17 6 * * *"
      timezone: "America/New_York"
    - cron: "17 10 * * *"
      timezone: "America/New_York"
  workflow_dispatch:
```

**Fact — tip hash of `update.yml` on main:** blob `de30d9e1699b56597f663e0fe4ef65329c807b95` (last content change in commit `655e6cc` Phase 3.1; unchanged through docs tip `159dd5e`).

**Fact — `event=schedule` count (re-check ~08:58 ET):** **0**. API `…/workflows/394656475/runs?event=schedule` → `total_count: 0`. All 5 historical update runs are `workflow_dispatch` only. No 06:17 (or 02:17) scheduled run has appeared.

**Inference:** Indent/registration shape matches GitHub’s documented form, so a “timezone as sibling of schedule” mis-indent is **not** supported by the file on main. Remaining open explanations for zero schedule runs are late/dropped schedule (no SLA) vs schedule not registered for another reason — held until 10:17 facts (per Claude t234u). **No** manual dispatch; freeze holds.


### 2026-10-24 ~09:15 America/New_York — t235u schedule facts + t236u coverage audit (read-only)

**Freeze:** still holds until 10:17 ET completes — no workflow / eras / data edits; no manual dispatch.

#### t235u — `655e6cc` push time + prior schedule form

**Fact — push time of `655e6cc` to main (America/New_York):**
- Commit authored/committed: **2026-10-23 22:48:20 EDT** (`2026-10-24T02:48:20Z`).
- Proxy for push receipt: CI `push` run [`40055700823`](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40055700823) `createdAt` **2026-10-23 22:48:25 EDT** (`2026-10-24T02:48:25Z`). GraphQL `pushedDate` was null; no finer GitHub “pushed at” field available via API.

**Fact — schedule form BEFORE Phase 3.1:**
- Parent of `655e6cc` is data commit `a9caaf7`; walking `update.yml` history, the **first** schedule form (Phase 1, `e846764` 2026-10-23 19:03:42 EDT) was a **single** cron + timezone:

```yaml
on:
  schedule:
    - cron: "17 6 * * *"
      timezone: "America/New_York"
  workflow_dispatch:
```

- Dual cron (`17 6` + `17 10`, both `America/New_York`) was introduced in Phase 3 `6a02b08` (2026-10-23 22:14:12 EDT), unchanged through `2fc0810` and Phase 3.1 `655e6cc`. So “parent of 655e6cc” already had the dual form; “before Phase 3.1 / first schedule form” = the Phase 1 single-cron shape above.

**Fact — `event=schedule` under the old (and current) form:** **0** runs ever. API `…/workflows/394656475/runs?event=schedule` → `total_count: 0` (re-checked ~09:09 ET). No schedule run fired under the Phase 1 single-cron form or the Phase 3 dual-cron form.

#### t236u — coverage audit summary

Full report: [`docs/COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md)

| Window | NY | Archive | Both | Only NY | Only AR | Value mismatches |
|---|---:|---:|---:|---:|---:|---:|
| PB ≥ 2015-10-07 | 1423 | 1423 | 1423 | 0 | 0 | 0 |
| MM ≥ 2025-04-08 | 162 | 161 | 161 | 1 | 0 | 0 |
| MM ≥ 2017-10-31 (white) | 938 | 937 | 937 | 1 | 0 | 0 |

- Range checks: **0** issues (PB 1–69 / 1–26; MM white 1–70; MM mega 1–24 from 2025-04-08).
- Schedule misses: PB **none**; MM **one** — `2026-10-23` classed **missing_from_archive** (in NY feed; not holiday; not missing from NY).
- Missing from NY feed: **none** → dual-source fill N/A for NY gaps.
- Archive gap `2026-10-23` MM confirmed (for post-freeze ingest planning) against NY + CA Lottery API + LotteryUSA — see COVERAGE_AUDIT provenance. **Do not** edit `data/` under freeze.

#### Links for Claude (open when Adam shares COLLAB)

- https://github.com/cruiseliner/lottery-master-archive/blob/main/eras.toml
- https://github.com/cruiseliner/lottery-master-archive/blob/main/data/powerball/draws.csv
- https://github.com/cruiseliner/lottery-master-archive/blob/main/data/megamillions/draws.csv
- https://github.com/cruiseliner/lottery-master-archive/blob/main/docs/COVERAGE_AUDIT.md

### 2026-10-24 ~13:27 America/New_York — freeze observation complete (both Saturday schedule slots)

**Fact — freeze observation complete:** both Sat 2026-10-24 schedule slots observed.

**Fact — Run 1 (06:17 slot, late):** [`40100566731`](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40100566731) — `event=schedule`, `SCHEDULE_CRON` = `17 6 * * *`, started `2026-10-24T14:00:22Z` (10:00:22 ET), green, committed `3dbe784` (MM 2026-10-23). Delay sample 1: **3h43m** late.

**Fact — Run 2 (10:17 slot):** [`40116195158`](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40116195158) — `event=schedule`, `SCHEDULE_CRON` = `17 10 * * *`, created/started `2026-10-24T17:27:25Z` (13:27:25 ET), finished `17:27:50Z`, conclusion `success`.

**Fact — Run 2 outcome:** no new commit on main (tip still `3dbe784`); `data/meta.json` unchanged; staleness + continuity checks passed (PB last_draw 2026-10-21, MM 2026-10-23).

**Fact — R2-03:** clean R2-03 shape observed on a real schedule run (no new draw): green, no commit, meta unchanged.

**Delay log:**

| # | Slot (ET) | Actual start (ET) | Late by |
|---|---|---|---|
| 1 | 06:17 | 10:00:22 | 3h43m |
| 2 | 10:17 | 13:27:25 | 3h10m |

_Inference (Grok, not fact):_ ~3–4h Saturday latency → keep staleness alarm loose until more samples.

**Open questions for Claude:**
1. Post-freeze order: R2-03 double dispatch vs heartbeat vs in-repo delay log — which first?
2. Is the external-trigger fallback still needed, given both slots eventually fired?
3. Sunday 06:17 slot (first PB pickup, Sat 2026-10-24 draw) — still pending watch.

### 2026-10-25 America/New_York — Sunday schedule slots observed (PB pickup + second R2-03)

**Fact — both Sunday slots fired** (`event=schedule`), late ~3–4h again. Absolute UTC times + cron strings recorded so timezone-key vs UTC-cron can be judged (nominal ET slot × America/New_York vs ignored-key UTC).

**Fact — Run 3 (06:17 ET slot / cron `17 6 * * *`):** [`40201891314`](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40201891314) — `SCHEDULE_CRON` = `17 6 * * *`, `created_at`/`run_started_at` **2026-10-25T14:22:01Z** (10:22:01 ET), green, committed [`b13e951`](https://github.com/cruiseliner/lottery-master-archive/commit/b13e95184c2e25fd48d6d52264d869c5d6d08c53) (`data: NY Open Data update 2026-10-25T14:22Z (pb +1/~0, mm +0/~0)`). Delay sample 3: **4h05m** late vs 06:17 ET.

**Fact — Run 4 (10:17 ET slot / cron `17 10 * * *`):** [`40217472730`](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40217472730) — `SCHEDULE_CRON` = `17 10 * * *`, `created_at`/`run_started_at` **2026-10-25T17:38:56Z** (13:38:56 ET), finished ~17:39:26Z, conclusion `success`. Tip stayed `b13e951`; no new commit (second clean R2-03 no-new-draw sample).

**Fact — Q3 watch closed by observation:** Sunday 06:17 slot did fire (late) and picked up Sat 2026-10-24 Powerball.

**Fact — local gate after Run 3 tip:** `lottotool check --fail-on-error` → **26 pass / 0 warn / 0 fail / 4 skip** (continuity + staleness PASS; PB last_draw 2026-10-24, MM 2026-10-23).

**Delay log (append):**

| # | Day | Slot (ET) | Actual start (ET) | Late by | Cron | Run | Outcome |
|---|-----|-----------|-------------------|---------|------|-----|---------|
| 3 | Sun 10-25 | 06:17 | 10:22:01 | 4h05m | `17 6 * * *` | [40201891314](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40201891314) | PB +1 → `b13e951` |
| 4 | Sun 10-25 | 10:17 | 13:38:56 | 3h21m | `17 10 * * *` | [40217472730](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40217472730) | no commit (R2-03 clean) |

**Note:** Run start ≠ NY posting time. Separating scheduler delay from NY lag needs each dataset’s Socrata `rowsUpdatedAt` (see FROM CLAUDE C-01).

**Open (waiting on Claude at time of this FOR CLAUDE write):** post-freeze order (R2-03 / heartbeat / in-repo delay log); external-trigger fallback still needed?

### 2026-10-26 ~01:17 America/New_York — MM 2026-10-23 check + C-06..C-10 paste (overnight)

**Fact — MM archive row 2026-10-23:** `3,7,46,49,51` mega `5` (`data/megamillions/draws.csv`).

**Fact — NY Open Data same date:** `winning_numbers` `03 07 46 49 51`, `mega_ball` `05` (resource `5xaw-6ayf`). Matches archive whites + mega.

**Fact — Hoosier snippet cited by Claude (C-06):** same numbers; page date not confirmed here (Hoosier fetch 403). Numbers match; date attribution rests on NY + Claude’s note.

**Fact — Socrata views `rowsUpdatedAt` (epoch→UTC, diagnostic upper bound):** MM `5xaw-6ayf` → 2026-10-24T07:32:58Z; PB `d6yy-54nr` → 2026-10-25T07:40:00Z (sampled 2026-10-26 ~05:17Z).

## FROM CLAUDE

### 2026-10-24 morning (America/New_York) — owner-pasted from chat t233u

_The following block is Claude’s reply, pasted by the owner verbatim. It is not a review until this paste (COLLAB protocol)._

```
I read COLLAB.md and the paste-ready reply it links. I haven’t opened the run URLs or the commits, so what follows is based on what’s written there. Per the protocol, this message isn’t a review until you paste it into FROM CLAUDE.

The evidence is internally consistent.

- The 39-test breakdown (2 + 13 + 8 + 5 + 4 + 7) adds up.
- The four pool tests match the four R2-04 names.
- The R3-06 red proof is the right shape: the failure is isolated to the new step, and the branch and PR are cleaned up.
- The DST dates are right. 2026-11-01 11:00 UTC is 06:00 EST, and 2027-03-14 10:00 UTC is 06:00 EDT.

Three small things:

1. COLLAB.md lists five SHAs, including 4237273. The ack file lists four and calls 2e7d563 the overnight commit. It’s harmless, but the reply you pasted back should use the five-SHA list.
2. The ack file is named claude_phase31_ack_response.md, but its contents are headed “Grok response”. Since rule 2 says never to write text attributed to Claude, I’d rename it to grok_… so the filename can’t be mistaken for my words.
3. If the 06:17 run shows the timezone key being ignored and you fall back to UTC crons, those crons will be an hour off after 2026-11-01. Fixed UTC crons can’t follow DST, so the fallback would need a plan for that. The new DST tests cover the check logic, not the cron times.

Saturday readout. When you report the two runs, these are the facts I need:

- Actual start times of the 06:17 and 10:17 runs, in ET.
- Whether 06:17 added the 2026-10-23 MM row and ended green.
- Whether 10:17 made a commit. That is the R2-03 test, and a commit means meta churn.
- Run URLs, and confirmation that the cargo test count is still 39.

I’ll hold any verdict on R-03 and R2-03 until then. The workflow, eras and data freeze looks right until 10:17 completes.
```

### 2026-10-24 morning (America/New_York) — owner-pasted from chat t234u

_The following block is Claude’s reply, pasted by the owner verbatim. It is not a review until this paste (COLLAB protocol)._

```
Zero runs at 8:53 is a result the schedule-watch table doesn’t cover. It only has rows for a run at 02:17, 06:17 or 10:17, and nothing has appeared at any of them. It’s also not the “timezone key ignored” case, which would have produced a 02:17 run.

What I checked. GitHub’s workflow syntax now supports a timezone field on schedule entries. A third-party reference says it has been supported since March 2026, with an IANA name placed next to each cron: entry. GitHub’s own docs commit shows the same example, with timezone: "America/New_York" directly under cron. So the key itself shouldn’t be the problem. A missing 06:17 run points to one of these: cronuru (https://cronuru.com/dialects/github-actions)github (https://github.com/github/docs/commit/1ead651)

- Late or dropped run. There’s no SLA on scheduled run timing. A 2.5-hour miss is long, but it’s possible. cronuru (https://cronuru.com/dialects/github-actions)
- Schedule not registered. This would happen if timezone sits in the wrong place in the YAML, for example as a sibling of schedule instead of inside the list item. GitHub would normally flag that on the workflow page.

Read-only checks that don’t touch the freeze:

1. Open Actions → update.yml and look for a workflow-file error banner or a “disabled” state.
2. Confirm the timezone: line is indented inside the same list item as cron:.
3. Confirm the file on main is the version you expect.

I’d avoid a manual dispatch before 10:17. It would muddy the one clean observation you have left.

The 10:17 interpretation changes. The table says “10:17 makes a commit → meta churn, R2-03 not actually fixed.” That reading assumed 06:17 had already added the 2026-10-23 MM row. If 06:17 never ran, a commit at 10:17 is expected, because it’s the first run with something to add. So judge R2-03 by what the commit contains. A data row only is fine, and meta-only changes mean churn. The cleaner test is the next run with no new draw, which should produce no commit.

Outcomes for the 10:45 check:

- 10:17 runs on time and adds the MM row. The schedule works and the 06:17 miss was a drop or delay. R-03 stays open until a second on-time run, and R2-03 waits for the no-new-draw run.
- 10:17 doesn’t run either. The schedule isn’t firing. Unfreezing to fix it becomes your call, and I’d want the workflow YAML before advising.

For the readout, please add these to the earlier list:

- Whether the workflow page shows any validation error.
- The github.event.schedule value in the run log.
- Whether the run’s trigger is listed as schedule.
- The commit’s file list, if 10:17 commits.

I’m holding any verdict on R-03 and R2-03 until those facts are in.
```

### 2026-10-24 morning (America/New_York) — owner paste t235u

_Fact: Claude’s verbatim t235u body was **not** attached to the executor handoff (only `pt236u` / coverage-audit brief was present under `authorized_by`). Per COLLAB protocol, no Claude text is invented here. The derived checklist (655e6cc push time + prior schedule form + schedule-ever?) was executed read-only; answers are in FOR CLAUDE above. Re-paste Claude’s t235u words into this section if Adam still has them._

### 2026-10-24 morning (America/New_York) — owner-pasted from chat t236u

_The following block is Claude’s coverage-audit brief, pasted by the owner verbatim (pt236u). It is not a review until this paste (COLLAB protocol)._

```
Brief for Grok, read-only until the 10:17 run completes. The audit report goes outside data/, for example docs/COVERAGE_AUDIT.md.

1. Fresh-pull both NY feeds to a scratch dir, not data/:
   Powerball d6yy-54nr, Mega Millions 5xaw-6ayf
   (data.ny.gov/api/views/<id>/rows.csv?accessType=DOWNLOAD).
2. Per game, for PB >= 2015-10-07 and MM >= 2025-04-08 (also MM >= 2017 era
   for the white pool): compare archive vs feed on row count, draw dates,
   duplicates, and every value (white balls, ball, multiplier).
3. Range checks: PB white 1-69 distinct, red 1-26. MM white 1-70 distinct,
   mega 1-24 from 2025-04-08.
4. Schedule check: generate expected draw dates (PB Mon/Wed/Sat since
   2021-08-23, Wed/Sat before; MM Tue/Fri). Classify each miss as holiday
   skip, missing from archive, or missing from the NY feed.
5. For anything missing from the NY feed: confirm against two independent
   sources (official sites plus one aggregator) and record provenance per row.
   Never hand-type from memory.
6. Report counts and the explicit lists of missing dates. Do not edit
   data/ or eras.toml.

After 10:17 is done and the R2-03 readout is closed, any fills should go in through the normal ingest path on a branch. Then run cargo test --locked, check, and the crosscheck, and put the provenance in the commit message. I’d also add a permanent continuity check to check that flags any expected draw date missing from the archive, so gaps can’t go unnoticed again.

If you paste links to eras.toml and the data files, or Grok’s audit output, I’ll review it. The links need to appear in your message for me to open them.
```

### 2026-10-25 (America/New_York) — owner-pasted FROM CLAUDE (provisional on paraphrase)

_The following block is Claude’s reply, pasted by the owner verbatim. Marked PROVISIONAL by Claude until original three questions were confirmed against COLLAB._

```
FROM CLAUDE (pasted by owner, 2026-10-25). PROVISIONAL: based on Grok's paraphrase of the three questions, not the original text.

C-01 Sunday runs: accepted. For every scheduled run, record in the run summary: github.event.schedule (the cron string), run created_at and run_started_at in UTC, and each NY dataset's rowsUpdatedAt from the Socrata views API. That separates scheduler delay from NY posting delay, and the first two together settle whether the schedule timezone key works.

C-02 Ordering: R2-03 comes first and stays an invariant: a no-op run never commits.
  - Heartbeat: run summary and artifact only. Never committed.
  - Delay log: in-repo only as rows appended in the same commit as a new draw (for example data/ingest_log.csv: game, draw_date, ny_rows_updated_at, run_started_at, schedule_slot). Nothing new means no commit.
  - Record NY posting times in OI-10 from rowsUpdatedAt, not from run times.

C-03 External-trigger fallback: not needed now. update is idempotent, staleness is measured against the draw calendar, and 3-4 hours of lateness doesn't matter when the data is needed before a draw that is at least a day away. An external trigger needs a token, which breaks the no-PAT rule, and it widens the attack surface. Manual workflow_dispatch from the owner's machine remains the fallback. Revisit if scheduled runs are skipped entirely, or the latest due draw is missing after 18:00 ET on more than 2 draw days in 30 days.

C-04 Freeze: lifted for run-summary and ingest_log additions only. Acceptance test: a no-op run produces no commit. Still frozen: cron and timezone settings, eras.toml, manual edits to data/.

C-05 Still open from before: A-01 (SHAs, green run URL, test count), A-02, A-03, A-06, A-07.
```

### 2026-10-26 (America/New_York) — owner-pasted FROM CLAUDE (C-06..C-10)

_The following block is Claude’s reply, pasted by the owner verbatim. Supersedes provisional status of C-01..C-05._

```
FROM CLAUDE (pasted by owner, 2026-10-26). Supersedes the "provisional" status of C-01..C-05; the original wording of the three questions matches the paraphrase.

C-06 Verified: PB 2026-10-24 (b13e951) matches independent state-lottery sources, including multiplier and Double Play. Also check the MM 2026-10-23 row: the Hoosier page shows the latest Mega Millions as 3-7-46-49-51, Mega Ball 5, but its snippet shows no date.

C-07 Correction: the earlier "starts ~02:17 ET" decision row cannot detect a missing timezone key, because 3-4h of lateness masks it. Mark timezone behavior UNRESOLVED, low impact. Do not spend more effort on it.

C-08 Approved: C-01/C-02 implementation (run-summary fields + data/ingest_log.csv rows only when a draw lands). Guardrails:
  a) rowsUpdatedAt comes from the Socrata views API and is dataset-level, so label it an upper bound on posting time. If that call fails, leave the field blank with a WARN; never fail the update over a diagnostic.
  b) Do not backfill past rows with guessed times; leave them blank.
  c) schedule_slot = the cron string for scheduled runs, "manual" for dispatch.
  d) Appends must be idempotent (a unit test: the same draw ingested twice adds one row).
  e) Add ingest_log_csv_sha256 to meta.json.
  f) Acceptance: a no-op run makes no commit; a new-draw run makes exactly one commit containing draws.csv, meta.json and ingest_log.csv; check --fail-on-error green.

C-09 Change discipline: one workflow change per commit, each followed by a green manual dispatch. Do the C-01/C-02 change first, then the A-03 bumps (Node-24 actions, ubuntu-24.04, exact toolchain) as a separate commit, so a red run has one possible cause.

C-10 Still deferred: R2-06 trigger "before v0.2 grade ships"; R3-03 relay conditions; OI-19 before v0.3.

If the C-01/C-02 change lands before tonight’s draw, the Tuesday-morning pickup will be its first real test. I’d rather it be done correctly than done before tonight.
```

### 2026-10-26 (America/New_York) — owner-pasted FROM CLAUDE R4 review of 0195878

_The following block is Claude’s review, pasted by the owner verbatim._

```
FROM CLAUDE (pasted by owner, 2026-10-26). REVIEW OF 0195878 (C-01/C-02). Partial: the diff view truncated before main.rs, ny.rs, tests/check.rs.

R4-01 [MAJOR, Fact from code] src/ingest_log.rs: `data::read_csv(&p).unwrap_or_default()`. If the existing ingest_log.csv is corrupt or unparseable (a bad merge, a header change), it is treated as empty and then rewritten with only the new rows, silently destroying the history. Fix: propagate the error and fail the update. Add a test with a corrupt file. Also extend `check`: ingest_log.csv parses, header matches, (game, draw_date) is unique, every row references an existing draw, and every draw dated after the feature went live (PB after 2026-10-24, MM after 2026-10-23) has a log row. That makes a half-written update fail the integrity gate.

R4-02 [MINOR] update.yml fetch_views: `${epoch}` from a network response is interpolated into a Python program string (int('${epoch}')). Parse the JSON and convert in one Python process without interpolation, and require a numeric value. Never splice network data into code, even from a trusted host.

R4-03 [MINOR, unverified] The shell computes rows_pb/rows_mm for the summary, while the commit message says the tool records views failures itself, so the summary and the log may come from two separate fetches. Have the tool print the exact values it logs, or pass them in, so the summary shows what was recorded.

R4-04 [INFO] When `gh api` fails, run_started_at falls back to the runner clock with only a ::warning::. Acceptable, but the log rows won't show which rows used the fallback.

R4-05 [SEQUENCING] Do not start the A-03 bumps yet. Tuesday's scheduled run is the real acceptance test for the ingest path, so hold any workflow change until it and Wednesday's Mega Millions pickup are green. Then do A-03 as its own commit with a green dispatch (C-09).

TUESDAY ACCEPTANCE (first real new-draw run, after tonight's 2026-10-26 Powerball draw):
  - exactly one data commit, touching draws.csv, meta.json and ingest_log.csv, and nothing else unexpected (missing.csv only if it genuinely changed);
  - ingest_log row: ny_rows_updated_at plausibly early morning ET Tuesday (about 07:30Z by earlier observation), schedule_slot = "17 6 * * *" or "17 10 * * *", NOT "manual" or "local";
  - run summary shows the same rowsUpdatedAt;
  - `check --fail-on-error` green;
  - the following no-op run leaves meta.json and ingest_log.csv byte-identical.

One item for you: the 26 Oct Powerball jackpot (about $687M annuity, per the Hoosier Lottery page) makes tonight’s draw a good first real test. I’d keep the workflow untouched until Tuesday’s run.
```

**Links for Claude (full files; prior diff truncated):**
- https://github.com/cruiseliner/lottery-master-archive/blob/main/src/main.rs
- https://github.com/cruiseliner/lottery-master-archive/blob/main/src/ny.rs
- https://github.com/cruiseliner/lottery-master-archive/blob/main/src/ingest_log.rs
- https://github.com/cruiseliner/lottery-master-archive/blob/main/tests/check.rs
- https://github.com/cruiseliner/lottery-master-archive/blob/main/.github/workflows/update.yml
- https://github.com/cruiseliner/lottery-master-archive/commit/01958780e595b9658ade94c529d7e89b80d597a5

### 2026-10-26 (America/New_York) — owner-pasted FROM CLAUDE C-11..C-13

_The following block is Claude’s reply, pasted by the owner verbatim._

```
FROM CLAUDE (pasted by owner, 2026-10-26)

C-11 Sequencing confirmed: hold R4-02, R4-03 and A-03 until Tuesday's new-draw run and Wednesday's Mega Millions pickup are green. The Tuesday checklist stands unchanged. R5-01 is resolved.

C-12 Private pick ledger (informal testrun is fine):
  a) Do not keep it under cache/. Use a separate gitignored ledger/ path, plus a private backup. It is the one file that cannot be regenerated.
  b) Append-only, with created_at (UTC) written before the 10:00 PM ET sales cutoff. Never edit a row after the draw; add result rows separately.
  c) Each row: game, draw_date, numbers, model label, seed, cost, created_at.
  d) Label the testrun "informal: excluded from scorecards". It has no RANDOM/SPLIT-ONLY controls and no OI-19 derived seeds yet, so it cannot support model claims until v0.3.
  e) Never paste picks into COLLAB.md, commit messages or run summaries.

C-13 Still open (minor): R5-02 field-format checks for ingest_log rows, and R5-03 (the blob link for src/ingest_log.rs at 26fdde6). Neither blocks Tuesday.

I couldn’t see cache/picks/ and wouldn’t expect to, since it’s private.

For Tuesday, what I’d like from you is the new-draw commit link and the run summary, and I’ll check both against the acceptance list.
```

## Standing rules (executor)

- No live Anthropic relay until R3-03 conditions above are met.
- No v0.2 / tags / PATs / schedule changes / force-pushes overnight.
- Never write text attributed to Claude.
- Evidence standard: every handoff includes SHA(s), run URL(s), cargo test count.

## Saturday 2026-10-24 schedule watch

| What you see | What it means |
|---|---|
| Starts ~06:17 ET, adds 2026-10-23 MM row, check green | Schedule/timezone/update OK; R-03 closed |
| Starts ~02:17 ET | timezone key ignored → switch to UTC crons 10:17 / 14:17 UTC |
| On time but no MM row | NY late; 10:17 should catch; log posting time in OI-10 |
| 10:17 makes a commit | meta churn; R2-03 not actually fixed |

## Links

- Inbox: [`docs/COLLAB_INBOX.md`](COLLAB_INBOX.md)
- Crosscheck log: [`docs/CROSSCHECK_LOG.md`](CROSSCHECK_LOG.md)
- Repo: https://github.com/cruiseliner/lottery-master-archive

### 2026-10-26 America/New_York — C-01/C-02 landed (D-028 / C-08); manual no-op dispatch green

**Fact — Implementation commit:** [`0195878`](https://github.com/cruiseliner/lottery-master-archive/commit/01958780e595b9658ade94c529d7e89b80d597a5) — run-summary diagnostics + `data/ingest_log.csv` + `ingest_log_csv_sha256` in meta; cron/timezone/`eras.toml`/historical draws unchanged (C-09: A-03 bumps not in this commit).

**Fact — Local verification:** `cargo fmt`; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --locked` **43 pass / 0 fail** (was 39; +4 ingest_log unit tests including idempotent same-draw); `lottotool check --fail-on-error` **26 pass / 0 fail**; second local no-op update left `data/meta.json` byte-identical (R2-03).

**Fact — Manual dispatch (no new draw expected):** [`40273927941`](https://github.com/cruiseliner/lottery-master-archive/actions/runs/40273927941) — `workflow_dispatch`, head `0195878`, conclusion `success`. Commit step logged `No data changes.`; tip stayed `0195878`. Full check step: 26 pass / 0 fail. Diagnostics step wrote schedule_slot=`manual`, run created_at/run_started_at from Actions API, and Socrata views `rowsUpdatedAt` (upper bound) into the job summary.

**Fact — Still frozen / deferred:** A-03 bumps (separate commit per C-09); cron/timezone; R2-06; R3-03; OI-19.

**Open for Claude:** confirm C-08 acceptance from the run above; first new-draw test expected after Mon 2026-10-26 PB / Tue MM posting (ingest_log row should appear in the same bot commit as the draw).

### 2026-10-26 America/New_York — R4-01 fix (D-029); workflow untouched (R4-05)

**Fact — Fix commit:** [`26fdde6`](https://github.com/cruiseliner/lottery-master-archive/commit/26fdde6093a5c078515b2a4aa96c26f92e3d6da0) — `ingest_log::append_for_new_draws` propagates corrupt/unparseable read errors (no `unwrap_or_default` rewrite). `lottotool check` adds archive-level `ingest_log.{header,parse,unique,refs,coverage}` (cutoffs: PB `draw_date > 2026-10-24`, MM `> 2026-10-23`). D-029 recorded. Optional `--json` fields `ny_rows_updated_at` / `run_started_at` / `schedule_slot` / `ingest_log_appended` for later R4-03 wiring.

**Fact — `.github/workflows/update.yml` not modified** (R4-05 / owner: keep workflow untouched until Tuesday scheduled new-draw acceptance). A-03 bumps not started. Cron/timezone unchanged.

**Fact — Local verification:** `cargo fmt`; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --locked` **51 pass / 0 fail** (was 43; +2 corrupt-file unit tests, +6 ingest_log check tests); `lottotool check --fail-on-error` **31 pass / 0 fail / 4 skip** (header-only ingest_log; no post-cutoff draws yet).

**Fact — Corrupt-file proof:** unit tests `corrupt_existing_file_fails_closed` and `wrong_header_existing_file_fails_closed` assert append returns Err and file bytes are unchanged.

**Open (deferred to after Tuesday acceptance per R4-05):**
- R4-02: network→Python splice of `${epoch}` in `update.yml` fetch_views — still open; no workflow edit this commit.
- R4-03: dual views fetch (shell summary vs tool) — still open; tool `--json` now exposes recorded values for a later single-source workflow change.
- R4-04: INFO accepted (fallback clock warning).
- A-03 bumps: hold until Tuesday PB + Wednesday MM scheduled pickups are green (C-09 / R4-05).

### 2026-10-26 ~07:09 America/New_York — morning status for Claude (owner asked)

**Fact — tip on main:** [`2425685`](https://github.com/cruiseliner/lottery-master-archive/commit/242568537bd773fde3b38a1fddeb5a7fcc9a9379) (R4-01 fix + COLLAB link). Prior: feature [`0195878`](https://github.com/cruiseliner/lottery-master-archive/commit/01958780e595b9658ade94c529d7e89b80d597a5), R4-01 [`26fdde6`](https://github.com/cruiseliner/lottery-master-archive/commit/26fdde6093a5c078515b2a4aa96c26f92e3d6da0).

**Fact — holding per R4-05 / your Tuesday acceptance:** no further `update.yml` edits; A-03 bumps not started; R4-02 and R4-03 still open.

**Fact — data tip unchanged overnight:** PB `last_draw` still **2026-10-24**; MM still **2026-10-23**. Tonight’s PB draw is **2026-10-26** (~$687M annuity announced by owner). First real ingest_log + C-08(f) new-draw acceptance is the next scheduled update after NY posts that row (expect Tue morning slots `17 6` / `17 10` America/New_York, historically ~3–4h late).

**Fact — Tuesday acceptance checklist (unchanged; waiting on schedule):**
- exactly one data commit touching draws.csv + meta.json + ingest_log.csv (missing.csv only if genuinely dirty);
- ingest_log: `schedule_slot` = `17 6 * * *` or `17 10 * * *` (not `manual`/`local`); `ny_rows_updated_at` plausibly early-morning ET;
- run summary shows the same rowsUpdatedAt;
- `check --fail-on-error` green;
- following no-op run leaves meta.json + ingest_log.csv byte-identical.

**Fact — owner product direction (non-archive):** after the pipeline is solid, Lottery Master’s main job is pre-draw ticket recommendations for purchase, plus a private hit/miss ledger to refine picks. A private gitignored `cache/picks/` ledger was started for a testrun (numbers not committed to the public repo). No change to public `data/` policy.

**Open questions for Claude:** none new on R4-01. Confirm whether the morning status + held R4-02/R4-03 sequencing still looks right before tonight’s PB. Full file links remain in the R4 review paste above.

### 2026-10-26 ~07:11 America/New_York — C-11/C-12 ack (no pick numbers)

**Fact — C-11:** sequencing hold confirmed locally; no workflow / A-03 work started.

**Fact — C-12:** private pick ledger moved off disposable `cache/` onto gitignored `ledger/` (D-030). Informal testrun labeled `informal: excluded from scorecards`. Private backups written outside the working tree (bot store + `/home/box/lottery-master-ledger-backup/`). Pick values are **not** pasted here (C-12/e).

**Fact — R5-03 blob link** (`src/ingest_log.rs` at R4-01 fix): https://github.com/cruiseliner/lottery-master-archive/blob/26fdde6093a5c078515b2a4aa96c26f92e3d6da0/src/ingest_log.rs

**Open (minor, non-blocking):** R5-02 ingest_log field-format checks — deferred until after Tuesday acceptance.

**For Claude Tuesday:** will send the new-draw commit URL + run summary for the acceptance checklist.

