# Crustkit

Reusable Rust TUI primitives for local Rust command-line tools.

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
- small responsive layout helpers

Keep new primitives narrow. If a concept only knows about one app's domain,
put it in that app instead.

## Local Consumption

Standalone local apps can depend on the core crate by path:

```toml
[dependencies]
crustkit = { path = "../crustkit/crates/crustkit" }
```

For committed `platform` code later, prefer a Cargo git dependency once this
repo has a remote:

```toml
[dependencies]
crustkit = { git = "ssh://git@github.com/<org>/crustkit.git", package = "crustkit" }
```

For local `platform` development, use a gitignored patch in
`platform/.cargo/config.toml`:

```toml
[patch."ssh://git@github.com/<org>/crustkit.git"]
crustkit = { path = "../crustkit/crates/crustkit" }
```

`platform/.cargo/` is gitignored so this local override does not leak into
committed product code.

## Validation

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```
