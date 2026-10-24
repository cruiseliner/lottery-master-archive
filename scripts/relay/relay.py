#!/usr/bin/env python3
"""Relay a review package to the Anthropic Messages API and save the reply under docs/REVIEWS/.

STATUS: DRAFTED, NOT RUN (Phase 1). Standard library only.

Usage:
    python3 scripts/relay/relay.py docs/HANDOFF_PHASE2.md [more files ...] [--dry-run]

Behaviour:
  * Reads ANTHROPIC_API_KEY, ANTHROPIC_MODEL, ANTHROPIC_VERSION (all required) and optional
    ANTHROPIC_MAX_TOKENS from the environment or scripts/relay/.env (gitignored).
  * System prompt = agent/REVIEWER_PROMPT.md (R3-02; not AGENT_PROMPT.md); user message = the given files concatenated.
  * POST https://api.anthropic.com/v1/messages with headers x-api-key, anthropic-version,
    content-type (verified against docs.anthropic.com/en/api on 2026-10-23).
  * Writes the text reply to docs/REVIEWS/<UTC timestamp>-review.md and a redacted log line
    to scripts/relay/logs/relay.log (gitignored).
  * Refuses to send any file whose path looks like the personal ledger or a secret file,
    and redacts anything key-like from logs and error messages.
"""
from __future__ import annotations

import datetime as _dt
import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
API_URL = "https://api.anthropic.com/v1/messages"

# Anything that looks like a credential is masked before it can reach a log or stderr.
_SECRET_PATTERNS = [
    re.compile(r"sk-ant-[A-Za-z0-9_\-]+"),
    re.compile(r"(gh[opsu]_)[A-Za-z0-9_]+"),
    re.compile(r"(?i)(x-api-key|authorization|api[_-]?key)(\s*[:=]\s*)\S+"),
]
_BLOCKED_PATHS = re.compile(
    r"(?i)(ledger|tickets?[^/]*\.csv$|(^|/)\.env|private/|cache/|(^|/)\.git(/|$)|secrets?)"
)
# Path allowlist for outbound review packages (R3-01). Anything else is refused.
_ALLOW_PREFIXES = (
    "docs/",
    "agent/",
    "eras.toml",
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "CHANGELOG.md",
    "LICENSE",
    "src/",
    "tests/",
    "scripts/relay/README.md",
    ".github/workflows/",
)


def _allowed_rel(rel: str) -> bool:
    if rel.startswith(".git/") or rel == ".git" or "/.git/" in rel:
        return False
    return any(rel == p or rel.startswith(p) for p in _ALLOW_PREFIXES)


def scan_secrets(text: str, label: str) -> None:
    """Refuse to send if the outbound payload looks like it contains a secret (R3-01)."""
    for pat in _SECRET_PATTERNS:
        if pat.search(text):
            sys.exit(f"relay: refusing to send {label}: payload matches a secret pattern")
    # Common PEM / token shapes beyond the named patterns.
    if re.search(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----", text):
        sys.exit(f"relay: refusing to send {label}: PEM private key detected")
    if re.search(r"(?i)(aws_secret_access_key|private[_-]?key)\s*[:=]", text):
        sys.exit(f"relay: refusing to send {label}: credential assignment detected")


def redact(text: str) -> str:
    out = text
    out = _SECRET_PATTERNS[0].sub("sk-ant-***", out)
    out = _SECRET_PATTERNS[1].sub(r"\1***", out)
    out = _SECRET_PATTERNS[2].sub(r"\1\2***", out)
    key = os.environ.get("ANTHROPIC_API_KEY")
    if key:
        out = out.replace(key, "***")
    return out


def log(msg: str) -> None:
    line = f"{_dt.datetime.now(_dt.timezone.utc).isoformat(timespec='seconds')} {redact(msg)}"
    print(line, file=sys.stderr)
    logs = HERE / "logs"
    logs.mkdir(exist_ok=True)
    with open(logs / "relay.log", "a", encoding="utf-8") as fh:
        fh.write(line + "\n")


def load_env() -> None:
    env = HERE / ".env"
    if not env.exists():
        return
    for raw in env.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#") or "=" not in raw:
            continue
        k, v = raw.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip())


def require(name: str) -> str:
    v = os.environ.get(name, "").strip()
    if not v:
        sys.exit(f"relay: {name} is not set (see scripts/relay/.env.example)")
    return v


def main(argv: list[str]) -> int:
    if not argv or "-h" in argv or "--help" in argv:
        print(__doc__)
        return 0
    dry = "--dry-run" in argv
    files = [a for a in argv if a != "--dry-run"]
    unknown = [a for a in files if a.startswith("-")]
    if unknown:
        sys.exit(f"relay: unknown option(s) {unknown}")
    load_env()
    parts = []
    for f in files:
        p = (ROOT / f).resolve()
        if not p.is_relative_to(ROOT):
            sys.exit(f"relay: refusing to send {f} (outside the repo)")
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith(".git/") or "/.git/" in f or f.endswith("/.git"):
            sys.exit(f"relay: refusing to send {rel} (.git blocked; R3-01)")
        if _BLOCKED_PATHS.search(rel):
            sys.exit(f"relay: refusing to send {rel} (ledger/secret/cache/.git path)")
        if not _allowed_rel(rel):
            sys.exit(f"relay: refusing to send {rel} (not on path allowlist; R3-01)")
        if not p.is_file():
            sys.exit(f"relay: no such file {rel}")
        content = p.read_text(encoding="utf-8")
        scan_secrets(content, rel)
        parts.append(f"===== {rel} =====\n{content}")
    system = (ROOT / "agent" / "REVIEWER_PROMPT.md").read_text(encoding="utf-8")
    scan_secrets(system, "agent/REVIEWER_PROMPT.md")
    body_text = "\n\n".join(parts)
    scan_secrets(body_text, "outbound payload")
    model = require("ANTHROPIC_MODEL")
    version = require("ANTHROPIC_VERSION")
    max_tokens = int(os.environ.get("ANTHROPIC_MAX_TOKENS", "8000"))
    payload = {
        "model": model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": [{"role": "user", "content": body_text}],
    }
    log(f"prepared request model={model} files={files} chars={len(body_text)} dry_run={dry}")
    if dry:
        return 0
    key = require("ANTHROPIC_API_KEY")
    req = urllib.request.Request(
        API_URL,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "x-api-key": key,
            "anthropic-version": version,
            "content-type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=600) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        log(f"HTTP {e.code}: {e.read().decode('utf-8', 'replace')[:2000]}")
        return 1
    except urllib.error.URLError as e:
        log(f"network error: {e}")
        return 1
    text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
    stamp = _dt.datetime.now(_dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = ROOT / "docs" / "REVIEWS" / f"{stamp}-review.md"
    out.write_text(
        f"# Review {stamp}\n\nmodel: {data.get('model', model)}\nfiles: {', '.join(files)}\n"
        f"stop_reason: {data.get('stop_reason')}\nusage: {json.dumps(data.get('usage'))}\n\n---\n\n{redact(text)}\n",
        encoding="utf-8",
    )
    log(f"wrote {out.relative_to(ROOT)} ({len(text)} chars)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
