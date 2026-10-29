---
name: driving-interactive-commands
description: >-
  Use when a task needs to run an interactive CLI program — a REPL
  (python3 -i, node), a full-screen editor, a TUI installer, a menu-driven
  wizard — that expects a real terminal and will hang or garble when run
  as a plain one-shot shell command. Not for commands that take input via
  stdin redirection or a flag, and not for `git rebase -i`, which this
  harness does not support interactively regardless of terminal.
---

# Driving Interactive Commands

The agent's shell tool runs one-shot, non-interactive commands: it starts
a process, waits for it to exit, and returns the output. A program that
expects a real terminal — readline input, full-screen redraw, raw-mode
key handling — either hangs waiting for a TTY that was never given, or
produces garbled, buffered output. That is the gap this skill closes, not
a limitation of any one command.

A detached `tmux` session supplies the terminal the program expects. The
agent starts the program inside it, sends keystrokes with `send-keys`,
and reads the screen back with `capture-pane` — all as ordinary
non-interactive commands from the agent's own point of view.

## Use it when

- Driving a REPL (`python3 -i`, `node`, `irb`) across multiple turns.
- Operating a full-screen editor (`vim`, `nano`) programmatically.
- Stepping through a TUI installer or menu-driven wizard.
- Any program whose output depends on terminal size, raw mode, or
  readline — curses apps, `htop`-style monitors.

## Don't use it when

- The program accepts input via stdin redirection or a CLI flag — just
  pass it normally.
- It's a one-shot command with no back-and-forth — plain Bash already
  works.
- It's `git rebase -i` — unsupported interactively in this environment
  regardless of terminal; use the non-interactive rebase flags or script
  the sequence of commits instead.

## How

Invoke the bundled script through its interpreter, never as a bare path:
`bash scripts/tmux-run.sh <action> <session-name> [args...]`.

```bash
bash scripts/tmux-run.sh start py python3 -i
bash scripts/tmux-run.sh send py 'import math' Enter
bash scripts/tmux-run.sh send py 'print(math.pi)' Enter
bash scripts/tmux-run.sh capture py
bash scripts/tmux-run.sh stop py
```

`start` launches the detached session and shows its first frame. `send`
forwards its arguments straight to `tmux send-keys` — so tmux key names
(`Enter`, `Escape`, `C-c`, `Up`, `BSpace`, `Space`) work as tmux
documents them — then shows the pane afterward. `capture` re-shows the
pane without sending anything. `stop` kills the session. Run `list` to
see what's still open.

## Rules

- **Always send `Enter` explicitly.** Typed text that isn't followed by
  `Enter` sits in the input buffer; there is no implicit newline.
- **Capture after every send**, not just at the end — a wrong keystroke
  three steps back is cheaper to catch immediately than to debug from the
  final screen.
- **Stop the session when done.** An orphaned tmux session is not
  self-cleaning; `bash scripts/tmux-run.sh stop <name>` every session you
  started, including ones abandoned after a mistake.
- **Pick session names you won't collide with.** `start` refuses to
  reuse a name that's still running — stop it first or choose another.

## Attribution

Adapted from `superpowers-lab`'s experimental
`using-tmux-for-interactive-commands` skill and `tmux-wrapper.sh` (MIT,
© 2025 Jesse Vincent).
