# rink

A tmux session dashboard. View, switch, and manage all your tmux sessions at a glance.

Uses zellij as an outer frame — session list + preview on the left, live tmux terminal on the right.

Inspired by [nacyot/muxdash](https://github.com/nacyot/muxdash). Supports macOS and Linux.

<img width="2296" height="1418" alt="CleanShot 2026-02-23 at 23 02 22@2x" src="https://github.com/user-attachments/assets/e2c32243-83db-4366-8315-96229949d6ae" />

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/seapy/rink/master/scripts/install.sh | bash
```

Downloads a pre-built binary from GitHub Releases.

Supported release targets:

- macOS Apple Silicon: `aarch64-apple-darwin`
- macOS Intel: `x86_64-apple-darwin`
- Linux x86_64: `x86_64-unknown-linux-gnu`
- Linux ARM64: `aarch64-unknown-linux-gnu`

You may need to add to your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Dependencies

`tmux` is required. `zellij` is required for the default split-frame UI; it is not required for `rink --standalone`.

On macOS, missing dependencies are auto-installed via Homebrew on first run.

On Linux, use rink's built-in setup commands:

```bash
rink doctor          # check tmux/zellij availability
rink doctor inspect  # inspect live zellij panes, layout, runtime tty, and tmux sessions
rink doctor reset    # remove generated rink/zellij/tmux runtime state
rink setup           # install missing dependencies
rink setup --dry-run # show what would be installed
```

`rink setup` installs `tmux` with the detected distro package manager (`apt-get`, `dnf`, or `pacman`) and installs `zellij` from the upstream prebuilt GitHub release into `$INSTALL_DIR` or `~/.local/bin`.

If a pre-built rink binary is not available for your platform, build from source:

```bash
cargo install --git https://github.com/seapy/rink
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

## Runtime files

Transient files such as the right-pane tty and Claude Code status markers are stored in `/tmp/rink` by default. Set `RINK_RUNTIME_DIR` to override the parent directory for tests or unusual environments.

If the left dashboard/sidebar is missing, first check whether `rink` was launched from inside an existing zellij pane. Plain `rink` is the split-frame launcher; inside zellij, use `rink --standalone` for dashboard-only mode or run plain `rink` from a shell outside zellij to create the left/right frame.

For live frame diagnostics:

```bash
rink doctor inspect
```

If a previous launch left broken zellij/tmux state behind, reset generated state and start fresh:

```bash
rink doctor reset --dry-run # inspect what would be removed
rink doctor reset           # delete rink's zellij session, tmux frame session, and generated files
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
