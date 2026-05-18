# Crustkit

Reusable Rust TUI primitives for local command-line tools.

This repo is intentionally a small Rust workspace, not a full application
framework. App-specific parsing, API clients, file naming rules, and output
formats should live in the consuming app crate.

## Workspace

```text
crustkit/
  Cargo.toml
  crates/
    crustkit/
      src/
        terminal.rs
        status.rs
        keys.rs
        shell.rs
        layout.rs
```

## Crates

`crustkit` provides:

- terminal lifecycle helpers for Crossterm raw mode and alternate screen cleanup
- a typed status line model
- key hint vocabulary and footer rendering
- shared header/footer shell helpers
- small adaptive layout helpers

Keep new primitives narrow. If a concept only knows about one app's domain,
put it in that app instead.

## Local Consumption

Standalone local apps can depend on the crate by path while keeping a publishable
version requirement:

```toml
[dependencies]
crustkit = { version = "0.1.0", path = "../crustkit/crates/crustkit" }
```

Published apps can depend on the crates.io release:

```toml
[dependencies]
crustkit = "0.1.0"
```

## Validation

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```
