from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from shlex import split as shlex_split

from kitty.boss import Boss
from kittens.tui.handler import result_handler

PH_BINARY = r"__PH_BINARY__"


def main(args: list[str]) -> str:
    return ""


@result_handler(no_ui=True)
def handle_result(args: list[str], answer: str, target_window_id: int, boss: Boss) -> None:
    window = boss.window_id_map.get(target_window_id)
    if window is None:
        return

    try:
        metadata = load_window_metadata(window.id, window, boss)
        write_log(
            "resolved window metadata "
            f"window_id={window.id} cmdline={metadata['cmdline']!r} "
            f"title={metadata['title']!r} cwd={metadata['cwd']!r}"
        )
        response = run_hook(metadata)
    except Exception as exc:
        report_error(f"pastehop: {exc}")
        passthrough_clipboard(window, boss)
        return

    action = response.get("action")
    write_log(f"hook response action={action!r} message={response.get('message')!r}")
    if action == "inject_text":
        text = response.get("text")
        if text:
            boss.call_remote_control(
                window,
                ("send-text", f"--match=id:{window.id}", "--bracketed-paste=auto", text),
            )
    elif action == "passthrough_key":
        passthrough_clipboard(window, boss)
    elif action == "error":
        message = response.get("message")
        if message:
            report_error(f"pastehop: {message}")


def load_window_metadata(window_id: int, window: Any, boss: Boss) -> dict[str, str]:
    child = getattr(window, "child", None)
    title = normalize_title(getattr(window, "title", ""))
    foreground_cmdline = normalize_cmdline(getattr(child, "foreground_cmdline", None))
    foreground_cwd = normalize_cwd(getattr(child, "foreground_cwd", None))
    foreground_processes = getattr(child, "foreground_processes", []) or []
    ssh_kitten_cmdline = normalize_cmdline(window.ssh_kitten_cmdline())

    selected_cmdline = select_cmdline(
        foreground_cmdline,
        foreground_processes,
        ssh_kitten_cmdline,
        title,
    )
    write_log(
        "foreground details "
        f"window_id={window_id} foreground_cmdline={foreground_cmdline!r} "
        f"ssh_kitten_cmdline={ssh_kitten_cmdline!r} "
        f"foreground_processes={json.dumps(foreground_processes, default=str)}"
    )

    if selected_cmdline or foreground_cwd or title:
        return {"cmdline": selected_cmdline, "cwd": foreground_cwd, "title": title}

    raw = boss.call_remote_control(
        window,
        ("ls", f"--match=id:{window_id}", "--output-format=json"),
    )
    payload = json.loads(raw)
    window_data = find_window(payload, window_id)
    if window_data is None:
        raise RuntimeError(f"kitty metadata unavailable for window {window_id}")

    return {
        "cmdline": select_cmdline(
            normalize_cmdline(window_data.get("cmdline")),
            [],
            "",
            normalize_title(window_data.get("title")),
        ),
        "cwd": normalize_cwd(window_data.get("cwd")),
        "title": normalize_title(window_data.get("title")),
    }


def find_window(payload: Any, window_id: int) -> dict[str, Any] | None:
    if isinstance(payload, list):
        for item in payload:
            match = find_window(item, window_id)
            if match is not None:
                return match
        return None

    if not isinstance(payload, dict):
        return None

    if payload.get("id") == window_id and "cmdline" in payload:
        return payload

    for key in ("tabs", "windows"):
        nested = payload.get(key)
        if isinstance(nested, list):
            match = find_window(nested, window_id)
            if match is not None:
                return match

    return None


def normalize_cmdline(cmdline: Any) -> str:
    if isinstance(cmdline, list):
        return " ".join(str(part) for part in cmdline if str(part).strip())
    if cmdline is None:
        return ""
    return str(cmdline)


def normalize_cwd(cwd: Any) -> str:
    if cwd is None:
        return ""
    return str(cwd)


def normalize_title(title: Any) -> str:
    if title is None:
        return ""
    return str(title)


def select_cmdline(
    foreground_cmdline: str,
    foreground_processes: list[Any],
    ssh_kitten_cmdline: str,
    title: str,
) -> str:
    candidates: list[str] = []
    if foreground_cmdline:
        candidates.append(foreground_cmdline)

    for process in foreground_processes:
        if isinstance(process, dict):
            cmdline = normalize_cmdline(process.get("cmdline"))
            if cmdline:
                candidates.append(cmdline)

    if ssh_kitten_cmdline:
        candidates.append(ssh_kitten_cmdline)
    if title:
        candidates.append(title)

    for candidate in candidates:
        if looks_like_ssh_cmdline(candidate):
            return candidate

    for candidate in candidates:
        if candidate and not looks_like_shell_cmdline(candidate):
            return candidate

    return foreground_cmdline or ssh_kitten_cmdline or title


def looks_like_ssh_cmdline(value: str) -> bool:
    tokens = tokenize(value)
    if not tokens:
        return False

    first = basename(tokens[0])
    if first == "ssh":
        return True
    if first == "kitten" and len(tokens) > 1 and tokens[1] == "ssh":
        return True
    if first == "kitty" and len(tokens) > 2 and tokens[1] == "+kitten" and tokens[2] == "ssh":
        return True
    if first == "wezterm" and len(tokens) > 1 and tokens[1] == "ssh":
        return True
    return False


def looks_like_shell_cmdline(value: str) -> bool:
    tokens = tokenize(value)
    if not tokens:
        return False
    return basename(tokens[0]) in {"sh", "bash", "zsh", "fish", "nu", "dash"}


def tokenize(value: str) -> list[str]:
    try:
        return shlex_split(value)
    except ValueError:
        return value.split()


def basename(value: str) -> str:
    return value.rsplit("/", 1)[-1]


def run_hook(metadata: dict[str, str]) -> dict[str, Any]:
    command = [PH_BINARY, "hook", "kitty"]
    if metadata["cmdline"]:
        command.append(f"--foreground-process={metadata['cmdline']}")
    if metadata["cwd"]:
        command.append(f"--cwd={metadata['cwd']}")

    completed = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        if not detail:
            detail = f"hook exited with status {completed.returncode}"
        raise RuntimeError(detail)

    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError("invalid hook response") from exc


def passthrough_clipboard(window: Any, boss: Boss) -> None:
    boss.call_remote_control(
        window,
        ("action", f"--match=id:{window.id}", "paste_from_clipboard"),
    )


def report_error(message: str) -> None:
    write_log(message, force=True)
    notify_error(message)
    print(message, file=sys.stderr)


def notify_error(message: str) -> None:
    try:
        subprocess.run(
            [
                "kitten",
                "notify",
                "--app-name",
                "pastehop",
                "--icon",
                "error",
                "--urgency",
                "critical",
                "pastehop",
                message,
            ],
            check=False,
            capture_output=True,
            text=True,
        )
    except Exception:
        pass


def write_log(message: str, force: bool = False) -> None:
    if not force and os.environ.get("PH_KITTY_DEBUG") != "1":
        return
    try:
        path = debug_log_path()
        path.parent.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now(timezone.utc).isoformat()
        with path.open("a", encoding="utf-8") as handle:
            handle.write(f"{timestamp} {message}\n")
    except Exception:
        pass


def debug_log_path() -> Path:
    if cache_home := os.environ.get("XDG_CACHE_HOME"):
        return Path(cache_home) / "pastehop" / "kitty.log"
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Caches" / "pastehop" / "kitty.log"
    return Path.home() / ".cache" / "pastehop" / "kitty.log"
