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

On Ubuntu/Debian Linux servers, install `tmux` from apt and `zellij` from the upstream prebuilt binary:

```bash
sudo apt update
sudo apt install -y tmux curl tar

mkdir -p "$HOME/.local/bin"
tmp=$(mktemp -d)
arch=$(uname -m)
case "$arch" in
  x86_64) zellij_target="x86_64-unknown-linux-musl" ;;
  aarch64|arm64) zellij_target="aarch64-unknown-linux-musl" ;;
  *) echo "Unsupported zellij arch: $arch" >&2; exit 1 ;;
esac
zellij_tag=$(curl -fsSL https://api.github.com/repos/zellij-org/zellij/releases/latest | grep '"tag_name"' | sed 's/.*: "//;s/".*//')
curl -fsSL "https://github.com/zellij-org/zellij/releases/download/${zellij_tag}/zellij-${zellij_target}.tar.gz" -o "$tmp/zellij.tar.gz"
tar -xzf "$tmp/zellij.tar.gz" -C "$tmp"
install -m 0755 "$tmp/zellij" "$HOME/.local/bin/zellij"
rm -rf "$tmp"

# Add this to your shell profile if ~/.local/bin is not already on PATH:
export PATH="$HOME/.local/bin:$PATH"
```

Other Linux options:

```bash
# Fedora
sudo dnf install tmux zellij

# Arch
sudo pacman -S tmux zellij

# If you already have Rust/Cargo
cargo install --locked zellij
```

If a pre-built binary is not available for your platform, build from source:

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

Transient files such as the right-pane tty and Claude Code status markers are stored in `/tmp/rink`.

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
