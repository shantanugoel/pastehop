#!/bin/sh

response="$("__PH_BINARY__" hook kitty --key CTRL+V --foreground-process "${KITTY_CHILD_CMDLINE:-}")" || exit 0

parsed="$(printf '%s' "$response" | python3 -c 'import json, sys; data = json.load(sys.stdin); print(data.get("action", "")); print(data.get("text", "")); print(data.get("message", ""))')"
action=$(printf '%s\n' "$parsed" | sed -n '1p')
text=$(printf '%s\n' "$parsed" | sed -n '2p')
message=$(printf '%s\n' "$parsed" | sed -n '3p')
target="id:${KITTY_WINDOW_ID:-active}"

case "$action" in
  inject_text)
    kitty @ send-text --match "$target" "$text"
    ;;
  passthrough_key)
    kitty @ send-key --match "$target" ctrl+v
    ;;
  error)
    printf '%s\n' "$message" >&2
    ;;
esac
