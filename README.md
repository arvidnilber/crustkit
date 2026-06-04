# Crustkit

Reusable Rust TUI primitives for local command-line tools.

This repo is intentionally a small Rust workspace, not a full application
framework. App-specific parsing, API clients, file naming rules, and output
formats should live in the consuming app crate.

The goal is a "shadcn for Rust TUIs" style toolkit: composable primitives built
on raw Rust, Ratatui, and Crossterm. It should make terminal apps responsive,
memory cheap, clean, powerful, and non-invasive without forcing an app
framework around the caller.

## Alpha Quickstart

Add Crustkit to a Ratatui app:

```toml
[dependencies]
crustkit = "0.1.0"
crossterm = "0.29"
ratatui = "0.30"
```

Then keep the app shell small: parse args, enter Crustkit's terminal lifecycle,
draw pure UI from state, and handle keys/mouse as transitions.

```rust
use color_eyre::Result;
use crustkit::{ManagedTerminal, StatusLine, footer, run_with_terminal};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{layout::Constraint, widgets::Paragraph};

fn main() -> Result<()> {
    color_eyre::install()?;
    run_with_terminal(run)
}

fn run(terminal: &mut ManagedTerminal) -> Result<()> {
    let mut status = StatusLine::info("ready");
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(Paragraph::new("hello from crustkit"), area);
            frame.render_widget(footer([], Some(&status)), area);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('r') => status = StatusLine::success("refreshed"),
                _ => {}
            }
        }
    }
    Ok(())
}
```

Run the public demo for the richer shape:

```bash
cargo run -p crustkit-demo -- --help
cargo run -p crustkit-demo
cargo run -p crustkit-demo -- --no-effects
```

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
- resizable sidebar split state for mouse-driven terminal panes
- optional Taffy layout helpers for mapping CSS-style layout trees to Ratatui
  rectangles
- TachyonFX presets and component effect props enabled by default through the
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

Crustkit enables TachyonFX transitions by default for the alpha because effects
are part of the first-run experience. Consumers that need the smallest
dependency graph can disable default features:

```toml
[dependencies]
crustkit = { version = "0.1.0", default-features = false }
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

## Taffy Layout

Enable the optional `taffy` feature when a terminal screen benefits from
Flexbox or CSS Grid style layout:

```toml
[dependencies]
crustkit = { version = "0.1.0", features = ["taffy"] }
```

Crustkit re-exports the `taffy` crate and provides helpers to compute a
`TaffyTree` against a Ratatui `Rect`, then convert computed node layouts back
into clipped terminal rectangles:

```rust
use crustkit::{TaffyTreeExt, taffy::prelude::*};

tree.compute_terminal_layout(root, frame.area())?;
let sidebar = tree.layout_rect(sidebar_node, frame.area())?;
```

## Resizable Sidebars

Use `ResizableSidebar` when a two-pane TUI should behave like a native
draggable sidebar: render from the returned `sidebar`, `handle`, and `content`
rects, then pass mouse events back into the state object.

```rust
let split = sidebar.layout(area);
sidebar.handle_mouse(mouse, area);
```

## Local Consumption

Apps should depend on the published crate in `Cargo.toml`:

```toml
[dependencies]
crustkit = "0.2.0"
```

For local development, generate a Cargo patch in the consuming project:

```bash
./scripts/use-local-crustkit.sh /path/to/tui-project
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
cargo check --no-default-features
cargo clippy -- -D warnings
cargo test
cargo test --no-default-features
```
