# Current state

_Last updated: 2026-10-23 ~23:05 America/New_York (overnight A-01/A-02/A-07; workflows frozen until Sat 10:17 ET)._

**Version:** 0.1.0 (untagged).

**Data:**
- Powerball: 2,015 draws, 2010-02-03..2026-10-21.
- Mega Millions: 2,550 draws, 2002-05-17..2026-10-20.
- `data/meta.json` schema_version **2** (tool_version, per-CSV sha256; stable updated_at).

**Checks:** `lottotool check --no-stale --crosscheck 10` → 26 pass / 0 warn / 0 fail / 4 skip (crosscheck 10/10 both games).

**Tests:** `cargo test --locked` → **39 passed** (check 13 incl. 2 DST, odds 8, payouts 5, pool 4, rng 7, lib 2). Clippy / rustfmt not re-bumped overnight.

**Automation:** unchanged overnight (freeze D-024). `update.yml` still dual cron America/New_York; empty force_now green run 40055704478.

**Collab:** [`docs/COLLAB.md`](COLLAB.md) — protocol + FOR CLAUDE. Ack paste: `docs/REVIEWS/2026-10-24_claude_phase31_ack_response.md`.

**GitHub:** https://github.com/cruiseliner/lottery-master-archive

**Relay:** drafted; **not run** against API (OI-22 / D-023).

See OPEN_ISSUES, DECISIONS, CROSSCHECK_LOG.
