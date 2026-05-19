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
- terminal background color/theme detection for automatic light/dark palettes
- opt-in CLI reload helpers for local TUI development

Keep new primitives narrow. If a concept only knows about one app's domain,
put it in that app instead.

## CLI Reload

Apps can opt in to reload with one switch:

```rust
let reload = crustkit::CliReload::enabled();
crustkit::run_reloadable_cli(reload, || app::run(root, reload))?;
```

Pass the same `CliReload` into the app state and include
`reload.key_hint()` in the footer key commands. When disabled with
`CliReload::disabled()`, the key does not match and no footer hint is shown.

In debug builds launched from a Cargo crate directory, Crustkit supervises
`cargo run` and reloads by rebuilding and restarting. Outside that local
development shape, reload falls back to replacing the current executable.

## Local Consumption

Apps should depend on the published crate in `Cargo.toml`:

```toml
[dependencies]
crustkit = "0.1.0"
```

For local development, generate a Cargo patch in the consuming project:

```bash
/Users/arvidnilber/Documents/Projects/rust-tui/crustkit/scripts/use-local-crustkit.sh /path/to/tui-project
```

The helper reads `CRUSTKIT_PATH` from the target project's `.env` or
`.env.local`, then writes `.cargo/config.toml`. If no env value is present, it
uses the `crustkit` checkout that contains the helper script.

## Validation

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```
