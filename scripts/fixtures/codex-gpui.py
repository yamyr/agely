#!/usr/bin/env python3
"""Silent, offline Codex CLI fixture for native manual QA.

Set AGELY_CODEX_BIN to this executable and AGELY_QA_DIR to a disposable folder.
Write `error`, `delay`, or `done` to that folder's `mode` file to test recovery.
This fixture never calls a service or reads Codex credentials.
"""
import json
import os
from pathlib import Path
import sys
import time

args = sys.argv[1:]
if args == ["login", "status"]:
    print("Logged in using ChatGPT (offline QA fixture)")
    sys.exit(0)

output = Path(args[args.index("-o") + 1])
if output.name == "probe.txt":
    output.write_text("OK", encoding="utf-8")
    sys.exit(0)

qa = Path(os.environ["AGELY_QA_DIR"])
mode_file = qa / "mode"
mode = mode_file.read_text().strip() if mode_file.exists() else "ready"
if mode == "delay":
    time.sleep(3)
if mode == "error":
    print("Offline QA fixture: requested failure", file=sys.stderr)
    sys.exit(1)

output.write_text(json.dumps({
    "message": "Offline QA fixture: which experience would you like to share?",
    "options": [] if mode == "done" else ["Supplier negotiation", "Contract manufacturing", "Other"],
    "done": mode == "done",
}), encoding="utf-8")
