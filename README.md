# Crustkit

Reusable Rust TUI primitives for local command-line tools.

This repo is intentionally a small Rust workspace, not a full application
framework. App-specific parsing, API clients, file naming rules, and output
formats should live in the consuming app crate.

The goal is a "shadcn for Rust TUIs" style toolkit: composable primitives built
on raw Rust, Ratatui, and Crossterm. It should make terminal apps responsive,
memory cheap, clean, powerful, and non-invasive without forcing an app
framework around the caller.

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
- shared `ctrl+c` exit-key helpers for raw-mode event loops
- optional TachyonFX presets and component effect props behind the
  `tachyonfx` feature

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
The default enabled binding is `ctrl+r`.

## TachyonFX

Crustkit keeps animations optional. Enable the feature when an app wants
TachyonFX transitions without making every consumer pull the dependency:

```toml
[dependencies]
crustkit = { version = "0.1.0", features = ["tachyonfx"] }
```

`ComponentEffect` covers common fade, dissolve, coalesce, sweep, slide, and
pulse effects plus radial, diagonal, sweep, checkerboard, dissolve, and
coalesce patterns. Components with builders, such as `TextInput` and
`InputDialog`, expose `.effect(...)` and `.component_effect()` props; use
`ComponentEffectManager::render_widget` during drawing and call
`process_frame` after rendering the frame content.

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

## Publishing

Crustkit is crates.io-ready and should be published before apps that depend on
a new Crustkit version.

```bash
cargo publish --dry-run --locked -p crustkit
```

The full release checklist lives in [`docs/release.md`](docs/release.md). The
GitHub release workflow uses a protected `crates-io` environment, prefers
crates.io Trusted Publishing, and supports a one-time scoped `CRATES_IO_TOKEN`
only for the first bootstrap publish.

## Validation

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```
