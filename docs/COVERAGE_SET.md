# v0.2 Step 3: fixed-cost "2 whites" coverage set (D-033)

Spec for `lottotool wheel`. It meets the six requirements in Claude t53u, which Adam approved in t53s272.

## What is guaranteed, and what is not

> **At least one ticket matches 2 or more whites for every possible draw.**

Nothing else is guaranteed. Every run prints the following:

- The total cost. Power Play is included by default for Powerball; `--no-power-play` leaves it out.
- **This guarantees no prize.** 2 whites without the special ball pays $0 in Powerball and in Mega Millions (not a tier in `odds::PRIZE_TIERS`).
- **Expected value is negative.**
- **No better odds:** every ticket has the same odds as any other.
- **Draws are independent.**
- **Special balls are chosen at random (seeded) and carry no guarantee.**

## Why it works

The covered pairs (two numbers on a common ticket) form a graph G on 1..N. Some ticket shares 2+ whites with a draw exactly when two drawn numbers form a covered pair. So the guarantee holds iff G has no independent set of 5 numbers.

The construction splits 1..N into at most 4 groups and covers every pair *inside* each group. Any 5 drawn numbers put 2 in one group (pigeonhole), and that pair is on some ticket.

## Sets used (pinned in `src/wheel.rs`)

| Game | N | Groups (sizes) | Tickets | Cost | Lower bound | Claude's greedy |
|------|---|----------------|---------|------|-------------|-----------------|
| Powerball | 69 | 21, 21, 11, 16 | **63** | $126, or $189 with Power Play | 57 | 73 |
| Mega Millions | 70 | 21, 21, 13, 15 | **65** | $325 | 58 | 74 |

### Lower bound

The guarantee needs α(G) ≤ 4, so the complement of G has no K5. By Turán's theorem, the complement has at most t(N,4) edges (the complete 4-partite graph with near-equal parts). G therefore needs at least C(N,2) − t(N,4) edges, which is the sum of C(gᵢ,2) over 4 near-equal parts:

- **Powerball:** parts 18,17,17,17 give 153 + 3·136 = 561 pairs.
- **Mega Millions:** parts 18,18,17,17 give 2·153 + 2·136 = 578 pairs.

One ticket covers at most C(5,2) = 10 pairs, so Powerball needs ≥ ⌈561/10⌉ = **57** tickets and Mega Millions ≥ ⌈578/10⌉ = **58**. This bound holds for any set, grouped or not; whether it can be reached is open.

### How the sets were found

The search was offline, with a scratch tool outside the repo:

1. Simulated annealing on "pairs inside a group that no ticket covers" found per-group covering sizes. For example, 21 numbers need 21 tickets (the Steiner system S(2,5,21)), 17 need 16, 18 need 18, 11 need 7 and 16 need 15.
2. A small search over 4-part splits of N showed that two groups of 21 beat near-equal groups. Near-equal groups gave 66 tickets for Powerball (17,17,17,18) and 68 for Mega Millions.
3. A final annealing pass over the whole instance allowed tickets that span two groups. It found 63 for Powerball (2 spanning tickets) and 65 for Mega Millions (3).
4. Searches at 62 for Powerball and 64 for Mega Millions found nothing. That only means nothing was found; it isn't a proof.
5. The results are pinned as constants. Tests show that removing any one ticket breaks coverage, so no ticket is redundant.

## The six requirements

1. **Exact guarantee text.** `wheel::GUARANTEE` and `wheel::CAVEATS` appear in every text and JSON result, along with the total cost. The text never claims better odds.
2. **Refuse below the minimum.** If `--max-spend` is less than the full set's cost, the command refuses and shows the minimum budget. It checks this before planning, and it offers no partial or "almost" set.
3. **Verify in code.** `wheel::verify` runs on the pinned base and on every relabelled set, again on exactly what is printed or entered, and before any output. It checks:
   - 1 to 4 groups (pigeonhole)
   - the groups partition 1..N exactly
   - every ticket has 5 distinct whites in 1..N
   - every same-group pair is on some ticket

   The tests cover:
   - Powerball N=69 and Mega Millions N=70
   - removing any single ticket fails
   - 5 groups, a missing number, a number in two groups, duplicate whites and out-of-range whites all fail
   - 20,000 seeded random draws per game
   - an `#[ignore]` exhaustive check over all C(69,5) and C(70,5) draws, base and relabelled (about 1 s in release)
4. **Relabel, then pick by crowd score.**
   - A seeded ChaCha20 RNG draws K uniform permutations of 1..N (`--permutations`, default 16).
   - Each relabelled set is re-verified and gets seeded random specials (uniform in 1..bonus_max, no guarantee).
   - Every ticket is scored with the D-032 crowd scorer. The set with the **lowest mean crowd score** wins; ties go to the lower max, then the earlier permutation.
   - The seed is printed. With `--commit`, the seed and the chosen permutation are recorded privately in `ledger/wheels/<game>_<date>_<seed>.json`, under the gitignore guard.
5. **Ledger rules.** Dry run is the default: tickets are printed locally and nothing is written. `--commit` does the following:
   - refuses at or after the 22:00 ET sales cutoff (no late sets) and refuses archived draws and non-draw days
   - validates every ticket (strict cost) and reads the ledger fail-closed before writing anything
   - runs the gitignore guard for both paths
   - appends each ticket through `tickets::append` (the same path as `ticket add`)
   - writes the private record, and refuses a seed that was already entered

   The output contains real picks, so the help text and the output say to keep it out of COLLAB, commits and logs (D-030).
6. **Spend cap.** `--max-spend` is required, and the total is always printed before any entry.

No draw history, frequency, hot/cold or prediction is used. The archive is read only to refuse archived dates and for the crowd scorer's past-winner flag.
