# Agent prompt: lottery-master-archive

You are working on `cruiseliner/lottery-master-archive`, a public archive of Powerball and Mega
Millions winning numbers with the Rust CLI `lottotool`. The owner is Adam Karba (cruiseliner).

## Non-negotiable rules
1. **No invented data.** Never type, guess or "recall" a draw result, prize amount, date, matrix,
   price or odds figure. Every value comes from a cited source or is computed from one. If you cannot
   source a value, mark it `inferred` (derived from our own data) or `missing`.
2. **Verify, do not recall.** Re-fetch or re-read the source before asserting a fact. Label
   conclusions **Fact** (cited), **Inference** (reasoned from facts) or **High Possibility**.
3. **Secrets.** Never print, request in chat, or commit API keys or tokens. `.env` is gitignored.
   Mask tokens in any command output you show (`sed -E 's/(gh[opsu]_)[A-Za-z0-9_]+/\1***/g'`).
4. **Personal ledger.** The owner's personal ticket ledger stays out of this public repo
   (see `.gitignore`: `ledger/`, `private/`, `*ledger*.csv`, `tickets*.csv`).
5. **Third-party payouts.** Per-tier winners and prizes from powerball.com and megamillions.com go only
   to the gitignored `cache/`, until MUSL and Florida Lottery permission is documented in `docs/DECISIONS.md`.
   Do not use the "Florida Lottery" name in product naming.
6. **Externally visible actions** (creating repos, pushing, releases, issues, messages) need the
   owner's explicit approval for that specific action.

## Working loop
1. Read `docs/CURRENT_STATE.md`, `docs/OPEN_ISSUES.md`, and the latest `docs/SESSION_LOG.md` entry.
2. Make small, reviewable changes, then run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test` and `lottotool check --fail-on-error`.
3. Record decisions in `docs/DECISIONS.md` (ID, date, decision, evidence, alternatives).
4. Append a `docs/SESSION_LOG.md` entry: what changed, commands run, real output excerpts,
   status per item (DRAFTED / COMPILED / TESTED / RUN / NOT RUN).
5. Update `CHANGELOG.md` and `docs/CURRENT_STATE.md`.

## When reviewing (docs/REVIEWS/)
Write findings as: area, severity (blocker / major / minor / nit), evidence (file:line or
command output), and a suggested fix. Separate Facts from Inferences. Do not rewrite the code in a review.
