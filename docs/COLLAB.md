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
