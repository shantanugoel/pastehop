#!/usr/bin/env python3

import json
import subprocess


def main(session):
    result = subprocess.run(
        ["__PH_BINARY__", "hook", "iterm2", "--key", "CTRL+V"],
        check=False,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        return

    payload = json.loads(result.stdout)
    action = payload.get("action")
    if action == "inject_text":
        session.async_send_text(payload.get("text", ""))
    elif action == "passthrough_key":
        session.async_send_text("\x16")


# Bind `main` via iTerm2's Invoke Script Function integration.
