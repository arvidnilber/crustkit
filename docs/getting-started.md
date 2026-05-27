# Getting Started

Crustkit is a small primitive crate, not an app framework. A normal app owns
its state, commands, parsing, API calls, and persistence. Crustkit owns the
boring terminal pieces.

Start with:

```rust
use color_eyre::Result;
use crustkit::{ManagedTerminal, run_with_terminal};

fn main() -> Result<()> {
    color_eyre::install()?;
    run_with_terminal(run)
}

fn run(terminal: &mut ManagedTerminal) -> Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        // render widgets from current state
    })?;
    Ok(())
}
```

Use these as the baseline:

- `run_with_terminal` for raw mode, alternate screen, focus, mouse capture, and
  cleanup.
- `StatusLine` plus `footer` for current state and key hints.
- `body_split`, `centered_rect`, and `centered_dialog_rect` for stable layouts.
- `TextInput` and `InputDialog` before writing custom form widgets.
- `left_mouse_click`, `row_index_at`, and `inline_item_index_at` for hit tests.
- `AppTheme::detect_or` when terminal background detection should drive light
  or dark mode.

Effects are enabled by default for the alpha:

```toml
crustkit = "0.1.0"
```

Disable them when packaging the smallest dependency graph:

```toml
crustkit = { version = "0.1.0", default-features = false }
```
