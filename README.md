# rink

A tmux session dashboard. View, switch, and manage all your tmux sessions at a glance.

Uses zellij as an outer frame — session list + preview on the left, live tmux terminal on the right.

> macOS only. Inspired by [nacyot/muxdash](https://github.com/nacyot/muxdash).

<img width="2296" height="1418" alt="CleanShot 2026-02-23 at 23 02 22@2x" src="https://github.com/user-attachments/assets/e2c32243-83db-4366-8315-96229949d6ae" />

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/seapy/rink/master/scripts/install.sh | bash
```

Downloads a pre-built binary from GitHub Releases. Supports both Apple Silicon and Intel Mac.

tmux and zellij are auto-installed via Homebrew on first run if missing.

You may need to add to your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Usage

```bash
rink
```

Opens a split view: 35% session dashboard on the left, 65% tmux terminal on the right. Reattaches to the previous session on restart.

For dashboard-only mode without the zellij frame:

```bash
rink --standalone
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate |
| `Enter` | Switch session / Focus category |
| `/` | Search |
| `Tab`/`←`/`→` | Collapse/expand category |
| `c` | Create session |
| `x` | Kill session/category |
| `R` | Rename session |
| `C` | Rename category (batch rename) |
| `s` | Cycle sort mode |
| `J`/`K` | Reorder sessions (Custom sort) |
| `r` | Refresh |
| `?` | Help |
| `Esc` | Cancel / Unfocus |
| `Ctrl+x` | Quit |

## Features

### Preview

The bottom panel shows a live preview of the selected session's terminal output. See what's happening in each session without switching to it.

### Categories

Sessions are automatically grouped by the prefix before the separator (`-`):

```
work-api        → work
work-frontend   → work
personal-blog   → personal
scratch         → General
```

- `c` auto-fills the category prefix when creating a session
- `C` batch-renames all sessions in a category

### Sort

Press `s` to cycle:

- **Name** — alphabetical
- **Recent** — last used
- **Windows** — most windows first
- **Custom** — manual order with `J`/`K` (persisted)

### Claude Code Status

Shows Claude Code activity per session:

```
● my-session *    ← working (yellow)
○ other-session ? ← waiting for input (cyan)
○ done-session +  ← done (green)
```

To enable:

```bash
rink hook-install
```

Automatically adds hooks to `~/.claude/settings.json`. Restart Claude Code to activate.
