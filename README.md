# Crustkit
<img width="800" height="280" alt="crustkit" src="https://github.com/user-attachments/assets/974c04d5-a766-47a7-a0f8-634e9e72a6f7" />

Build Rust TUIs that feel responsive, mouse-aware, and polished without
writing the same terminal plumbing in every app.

Crustkit is a small toolkit on top of Ratatui and Crossterm. It gives you the
boring-but-important pieces: terminal setup and cleanup, responsive layouts,
status lines, key hints, text inputs, dialogs, progress bars, mouse helpers,
resizable sidebars, theme detection, and optional animations.

You keep your app architecture. Crustkit just makes the terminal part feel like
a product instead of a pile of raw event handling.

Crustkit is currently alpha. It is ready to try, but APIs may still move before
`1.0`.

## Why

Crustkit exists because we wanted internal terminal tools to feel as good as the
web apps they support. At [siv.chat](https://siv.chat), we use it for an
internal TUI-based mission control system, similar in shape to the demo: live
status, operator actions, logs, long-running jobs, responsive panes, keyboard
navigation, and mouse-driven controls in one local interface.

That kind of app is where raw Ratatui starts to repeat itself. Every serious
TUI needs terminal lifecycle safety, clear status, resize behavior, mouse math,
dialogs, progress, and consistent key hints. Crustkit packages those pieces so
you can spend more time on your workflow and less time rebuilding terminal UI
infrastructure.
## Demo of a TUI built with Crustkit


https://github.com/user-attachments/assets/08d97908-1641-4c55-8df8-1fd9851a64bb


```bash
cargo run -p crustkit-demo -- --help
cargo run -p crustkit-demo
cargo run -p crustkit-demo -- --no-effects
```

See [`docs/demo.md`](docs/demo.md) and
[`docs/getting-started.md`](docs/getting-started.md) for more examples.

## We're Hiring

If Crustkit-style local tools and operator systems sound interesting, email
[work@siv.chat](mailto:work@siv.chat) for more info.

## Install

```toml
[dependencies]
crustkit = "0.2.0"
color-eyre = "0.6"
crossterm = "0.29"
ratatui = "0.30"
```

## Quick Example

```rust
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use crustkit::{footer, header, run_with_terminal, KeyHint, ManagedTerminal, StatusLine};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Paragraph,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    run_with_terminal(run)
}

fn run(terminal: &mut ManagedTerminal) -> Result<()> {
    let mut refresh_count = 0;
    let mut status = StatusLine::info("ready");

    loop {
        terminal.draw(|frame| {
            let rows = Layout::vertical([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

            frame.render_widget(header("Crustkit demo", Some("small TUI shell")), rows[0]);
            frame.render_widget(
                Paragraph::new(format!("Refresh count: {refresh_count}")),
                rows[1],
            );
            frame.render_widget(
                footer(
                    [KeyHint::new("r", "refresh"), KeyHint::new("q", "quit")],
                    Some(&status),
                ),
                rows[2],
            );
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('r') => {
                    refresh_count += 1;
                    status = StatusLine::success("refreshed");
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

## How It Works

Crustkit keeps the loop simple:

1. `run_with_terminal` enters raw mode, switches to the alternate screen,
   enables mouse/focus events, and restores the terminal on exit.
2. Your app renders from state using normal Ratatui widgets plus Crustkit
   helpers.
3. Keys, mouse events, background task messages, and resize/focus events update
   state.
4. Crustkit gives you reusable UI building blocks so every app does not need to
   reinvent status bars, dialogs, progress, layout, and mouse math.
   
## Web-Inspired TUI Primitives

Crustkit brings a few web-app expectations into terminal UI: responsive panes,
draggable split views, automatic color adaptation, reusable form fields, and
modal dialogs. The terminal still stays fast and app-owned, but the interface
can feel closer to a compact internal web app than a plain command prompt.

### Responsive Layouts

`body_split` works like a small media query for terminal screens. Wide
terminals get side-by-side panes; narrow terminals stack the left pane above the
content pane.

```rust
use crustkit::body_split;

let [sidebar, content] = body_split(
    frame.area(),
    92, // compact below this width
    36, // left pane percentage when wide
    10, // left pane height when compact
);

frame.render_widget(nav_widget, sidebar);
frame.render_widget(content_widget, content);
```

When the terminal narrows, the layout switches from columns to rows without the
app rewriting all its rendering logic.

### Resizable Columns

`ResizableSidebar` gives TUIs a split-pane interaction that feels familiar from
web dashboards and IDEs. It tracks the sidebar, drag handle, content rect, and
mouse resize state for both wide and compact layouts.

```rust
use crustkit::{ResizableSidebar, StatusLine};

let mut sidebar = ResizableSidebar::new(34, 22, 58)
    .with_min_content_width(40)
    .with_compact(90, 10);

let split = sidebar.layout(frame.area());

frame.render_widget(nav_widget, split.sidebar);
frame.render_widget(handle_widget, split.handle);
frame.render_widget(detail_widget, split.content);

if let Some(event) = sidebar.handle_mouse(mouse_event, frame.area()) {
    status = StatusLine::info(format!(
        "sidebar: {} {}",
        event.size(),
        event.axis().unit(),
    ));
}
```

On wide terminals the handle resizes columns. In compact mode, the same state
resizes the top pane by rows.

### Mouse-Aware Lists

Crustkit normalizes mouse coordinates so app code can think in rows, not raw
terminal cells. That makes clickable lists, tabs, and inline controls much less
fragile.

```rust
use crustkit::{left_mouse_click, row_index_at, StatusLine};

if let Some(click) = left_mouse_click(mouse_event) {
    if let Some(index) = row_index_at(list_area, click, 1, items.len()) {
        selected = index;
        status = StatusLine::success("selected row");
    }
}
```

### Auto Theme From Terminal

`AppTheme::detect_or` asks the terminal for its background color and chooses a
light or dark palette. If the terminal does not answer, Crustkit falls back
safely.

```rust
use std::time::Duration;

use crustkit::{AppTheme, StatusLine, ThemeMode};
use ratatui::widgets::{Block, Paragraph};

let theme = AppTheme::detect_or(Duration::from_millis(80), ThemeMode::Dark);
let palette = theme.palette();

let panel = Paragraph::new("Terminal-aware theme")
    .style(theme.panel_style())
    .block(
        Block::bordered()
            .border_style(theme.border_style()),
    );

frame.render_widget(panel, area);
status = StatusLine::info(format!("theme: {:?}", palette.mode));
```

The goal is the same as `prefers-color-scheme` on the web: respect the user's
environment without making every app hand-roll theme detection.

### Input Components

`TextInput` gives inline terminal forms the pieces users expect from web inputs:
labels, placeholder text, focus styling, help text, and a cursor.

```rust
use crustkit::TextInput;

let input = TextInput::new("Search", "Query:")
    .value(search_query.clone())
    .placeholder("type a project, user, or job id")
    .help("enter to run, esc to clear")
    .focused(search_focused);

frame.render_widget(input.widget(), area);
```

The app still owns keyboard handling and state. Crustkit owns the reusable
rendering details.

### Dialog Components

`InputDialog` gives you a centered modal with a label, placeholder, help text,
primary/secondary actions, and pending state.

```rust
use crustkit::{DialogTheme, InputDialog};

let dialog = InputDialog::new("Create mission", "Name:")
    .value(mission_name.clone())
    .placeholder("daily sync")
    .help("names are local until you press save")
    .primary_label("Save")
    .secondary_label("Esc cancel")
    .pending(save_in_flight);

dialog.render(frame, frame.area(), DialogTheme::default());
```

That keeps common modal chrome consistent while your app decides what save,
cancel, validation, and side effects mean.

## Features

```toml
# Smallest dependency graph.
crustkit = { version = "0.2.0", default-features = false }

# CSS-like Flexbox/Grid layout helpers.
crustkit = { version = "0.2.0", features = ["taffy"] }
```

Default features include TachyonFX component effects for alpha builds.

## Local Development

Use the published crate in app manifests:

```toml
[dependencies]
crustkit = "0.2.0"
```

When testing a local checkout from another app, generate a local Cargo patch
instead of committing a path dependency:

```bash
./scripts/use-local-crustkit.sh /path/to/tui-project
```

## Release Checks

```bash
cargo fmt --check
cargo check
cargo check --no-default-features
cargo clippy -- -D warnings
cargo test
cargo test --no-default-features
cargo publish --dry-run --locked -p crustkit
```

## License

Crustkit is dual-licensed under MIT OR Apache-2.0.
