# ✅ Todo

A fast, keyboard-driven TUI todo list application built with Rust.

## Features

- Interactive terminal UI
- Add, remove, and list todos
- Mark tasks as complete
- Keyboard navigation

## Installation

### One-liner (macOS / Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/Yasar195/rust_cli_todo/release/install.sh | sh
```

> Automatically detects your OS and architecture, downloads the right binary, and installs it to `/usr/local/bin`.

**Options:**

```sh
# Install a specific version
curl -fsSL https://raw.githubusercontent.com/Yasar195/rust_cli_todo/release/install.sh | sh -s -- --version v1.2.3

# Install to a custom directory
curl -fsSL https://raw.githubusercontent.com/Yasar195/rust_cli_todo/release/install.sh | sh -s -- --install-dir ~/.local/bin
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Yasar195/rust_cli_todo/release/install.ps1 | iex
```

> Installs to `%LOCALAPPDATA%\Programs\todo` and adds it to your user PATH automatically.

**Options:**

```powershell
# Install a specific version
.\install.ps1 -Version v1.2.3

# Install to a custom directory
.\install.ps1 -InstallDir "C:\Tools\todo"
```

### Build from source

```sh
git clone https://github.com/Yasar195/rust_cli_todo
cd rust_cli_todo
cargo build --release
./target/release/todo
```

### Pre-built binaries

Download the latest binary for your platform from the [Releases](https://github.com/Yasar195/rust_cli_todo/releases/latest) page.

| Platform | File |
|----------|------|
| Linux (glibc) | `todo-linux-amd64.tar.gz` |
| Linux (musl / Alpine) | `todo-linux-musl-amd64.tar.gz` |
| macOS (Intel) | `todo-macos-amd64.tar.gz` |
| macOS (Apple Silicon) | `todo-macos-arm64.tar.gz` |
| Windows | `todo-windows-amd64.exe.zip` |

## Keybindings

| Key | Action |
|-----|--------|
| `a` | Add todo |
| `d` | Delete todo |
| `Space` | Toggle complete |
| `q` | Quit |
| `↑ / ↓` | Navigate |

## Requirements

- Rust 1.70+ *(build from source only)*

## License

MIT
