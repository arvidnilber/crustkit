# Anthropic Export TUI

Rust TUI for exporting raw claude.ai conversation JSON through Anthropic's Compliance API.

This is intentionally built with Ratatui + Crossterm:

- `ratatui` renders the full-screen UI, lists, tables, styles, layout, and responsive terminal panes.
- `crossterm` handles raw terminal mode, alternate screen, and keyboard events.
- `reqwest` talks to Anthropic's API.
- `serde_json` writes raw API responses to disk.

## Install Rust 1.95

This repo pins Rust with `rust-toolchain.toml`, so Cargo will use `1.95.0` inside this folder.

```bash
rustup update stable
rustup toolchain install 1.95.0
rustc --version
cargo --version
```

Expected:

```text
rustc 1.95.0
```

## Install dependencies

Rust libraries are not installed globally for normal app development. Add them to each project's `Cargo.toml`; Cargo downloads and caches them globally under `~/.cargo`, but versions stay locked per project in `Cargo.lock`.

For a new project, the equivalent setup is:

```bash
cargo new my-tui --bin
cd my-tui
cargo add ratatui crossterm color-eyre clap serde serde_json reqwest chrono sanitize-filename url
```

This project already has those dependencies in `Cargo.toml`, so just run:

```bash
cargo build
```

## Run

The app uses the Anthropic Compliance API, which is for claude.ai organization data and requires an Enterprise plan plus a Compliance Access Key with `read:compliance_user_data`.

```bash
export ANTHROPIC_COMPLIANCE_ACCESS_KEY="..."
cargo run -- \
  --user-id user_01... \
  --output-dir "$HOME/Documents/anthropic-conversations"
```

Preselect a project when opening the TUI:

```bash
cargo run -- \
  --user-id user_01... \
  --project-id claude_proj_01... \
  --output-dir "$HOME/Documents/anthropic-conversations"
```

Keys:

```text
↑/↓ or j/k  move project filter
Space       refresh selected project
Enter       export selected project scope
q or Esc    quit
```

Exports are written like this:

```text
anthropic-conversation-export/
  2026-05-12_14-30-15/
    Project Name/
      _manifest.json
      Chat title_claude_chat_....json
```

Each chat JSON preserves the raw chat metadata and raw messages payload.

## Important API boundary

Personal Claude exports are not exposed as a normal public conversation-history API. Anthropic documents that flow as Claude web/desktop Settings -> Privacy -> Export data, then a download link by email.

The programmatic path used by this TUI is Anthropic's Compliance API for claude.ai organization data. It can list projects, filter chats by project, and retrieve messages for each chat.
