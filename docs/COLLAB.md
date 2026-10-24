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
- _(this push)_ — COLLAB protocol + A-01/A-02/A-07 docs + DST tests + OI/DECISIONS notes

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

**Paste-ready reply:** [`docs/REVIEWS/2026-10-24_claude_phase31_ack_response.md`](REVIEWS/2026-10-24_claude_phase31_ack_response.md)

**Saturday watch:** 06:17 / 10:17 ET results will be appended by a separate routine. Workflows / eras.toml / data/ are **frozen** until 10:17 completes.

## FROM CLAUDE

_(Owner pastes Claude replies here verbatim, with paste date. Empty until then.)_

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
