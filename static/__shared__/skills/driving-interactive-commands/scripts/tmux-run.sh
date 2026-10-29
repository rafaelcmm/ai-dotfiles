#!/usr/bin/env bash
#
# tmux-run.sh — drive an interactive CLI program (REPL, editor, TUI
# installer) from an agent that can only run non-interactive commands.
#
# The agent's own shell tool has no real terminal, so anything that
# expects one (readline, full-screen redraw, a pseudo-TTY) hangs or
# garbles instead of running. A detached tmux session supplies that
# terminal; this script wraps `tmux new-session`, `send-keys`, and
# `capture-pane` so the agent can start a program, type into it, read
# back what's on screen, and tear it down, all as ordinary one-shot
# commands.
#
# Usage:
#   tmux-run.sh start   <session-name> <command> [args...]
#   tmux-run.sh send    <session-name> <key-or-text> [more keys...]
#   tmux-run.sh capture <session-name>
#   tmux-run.sh stop    <session-name>
#   tmux-run.sh list
#
# Examples:
#   tmux-run.sh start py python3 -i
#   tmux-run.sh send py 'import math' Enter
#   tmux-run.sh send py 'print(math.pi)' Enter
#   tmux-run.sh capture py
#   tmux-run.sh stop py
#
# `send` arguments are passed through to `tmux send-keys` as-is, so
# special key names (Enter, Escape, C-c, Up, BSpace, Space) work
# exactly as tmux documents them. There is no bare "\n" — a command
# typed without a following Enter sits in the input buffer, unsent.
set -euo pipefail

ACTION="${1:-}"
SESSION="${2:-}"

usage() {
  cat <<'USAGE'
Usage: tmux-run.sh <action> <session-name> [args...]

Actions:
  start   <session-name> <command> [args...]   Start a detached session
  send    <session-name> <key-or-text> [...]   Send keys, then show the pane
  capture <session-name>                       Show the current pane
  stop    <session-name>                       Kill the session
  list                                         List all tmux sessions

Examples:
  tmux-run.sh start py python3 -i
  tmux-run.sh send py 'print(1+1)' Enter
  tmux-run.sh capture py
  tmux-run.sh stop py
USAGE
}

case "$ACTION" in
  start)
    COMMAND="${3:?tmux-run.sh start <session-name> <command> [args...]}"
    shift 3
    if [ -n "${SESSION}" ] && tmux has-session -t "$SESSION" 2>/dev/null; then
      echo "Session '$SESSION' already exists — stop it first or pick a new name." >&2
      exit 1
    fi
    tmux new-session -d -s "$SESSION" "$COMMAND" "$@"
    # The program needs a moment to initialize before its first frame is
    # on screen; capturing immediately would show a blank pane.
    sleep 0.3
    echo "Session: $SESSION"
    echo "---"
    tmux capture-pane -t "$SESSION" -p
    ;;

  send)
    if [ $# -lt 3 ]; then
      echo "Error: no keys given to send." >&2
      exit 1
    fi
    shift 2
    tmux send-keys -t "$SESSION" "$@"
    # Give the program a moment to react before capturing its response.
    sleep 0.2
    echo "Session: $SESSION"
    echo "---"
    tmux capture-pane -t "$SESSION" -p
    ;;

  capture)
    echo "Session: $SESSION"
    echo "---"
    tmux capture-pane -t "$SESSION" -p
    ;;

  stop)
    tmux kill-session -t "$SESSION"
    echo "Session '$SESSION' terminated."
    ;;

  list)
    tmux list-sessions
    ;;

  *)
    usage
    exit 1
    ;;
esac
