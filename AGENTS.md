# AGENTS.md

Instructions for any AI coding agent in this repository. The full prompt is in
`agent/AGENT_PROMPT.md`; the short version:

* **Build/test:** `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`
* **Data gate:** `cargo run -- check --fail-on-error` (use `--no-stale` for integrity only)
* **Data refresh:** `cargo run -- update powerball` / `update megamillions` (NY Open Data only)
* **Never:**
  * invent or recall data values;
  * commit `.env`, tokens or `cache/`;
  * commit the personal ticket ledger;
  * commit payout tables;
  * take externally visible actions without the owner's approval.
* **Always:**
  * label claims Fact / Inference / High Possibility;
  * log the session in `docs/SESSION_LOG.md`;
  * record decisions in `docs/DECISIONS.md`;
  * keep `eras.toml` statuses honest (`sourced` / `inferred` / `missing`).
* Layout:
  * `src/` (lib plus the `lottotool` bin);
  * `tests/` (integration tests, with synthetic fixtures only);
  * `data/`, `eras.toml`, `schemas/`, `docs/`;
  * `scripts/relay/` (review relay, DRAFTED);
  * `.github/workflows/`.
