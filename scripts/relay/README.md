# scripts/relay (DRAFTED, NOT RUN)

Sends a review package (for example `docs/HANDOFF_PHASE2.md`) to the Anthropic Messages API
with `agent/AGENT_PROMPT.md` as the system prompt, and saves the reply to `docs/REVIEWS/`.

* Configure `scripts/relay/.env` from `.env.example`. `.env` is gitignored; never commit it,
  never paste the key into chat, issues or logs.
* `ANTHROPIC_MODEL` and `ANTHROPIC_VERSION` are required and have no default on purpose.
* `--dry-run` builds the request and logs its size without sending or needing the key.
* Logs go to `scripts/relay/logs/` (gitignored) and are redacted (`sk-ant-***`, `gh?_***`).
* The script refuses to send ledger, `.env`, `private/` or `cache/` paths.
