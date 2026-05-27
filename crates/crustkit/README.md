# Crustkit

Small reusable Rust TUI primitives for command-line applications built with
Ratatui and Crossterm.

Crustkit is intentionally not a full application framework. It provides terminal
lifecycle helpers, status lines, key hints, shell header/footer helpers, transfer
progress widgets, theme primitives, terminal background detection, and small
adaptive layout helpers. App domain logic belongs in the consuming crate.

The crate is meant to stay composable and non-invasive: use the primitives you
need, keep your app state and domain workflow in your own crate, and rely on
Cargo patches for local iteration.

```toml
[dependencies]
crustkit = "0.1.0"
```

```rust
use std::time::Duration;

use crustkit::{AppTheme, ThemeMode};

let theme = AppTheme::detect_or(Duration::from_millis(100), ThemeMode::Dark);
```

CLI reload is opt-in:

```rust
let reload = crustkit::CliReload::enabled();
crustkit::run_reloadable_cli(reload, || app::run(root, reload))?;
```

The default enabled binding is `ctrl+r`.

TachyonFX support is opt-in:

```toml
[dependencies]
crustkit = { version = "0.1.0", features = ["tachyonfx"] }
```

```rust
use crustkit::{
    ComponentEffect, ComponentEffectManager, ComponentEffectPattern, TextInput,
};

let mut effects = ComponentEffectManager::new();

terminal.draw(|frame| {
    let area = frame.area();
    let input = TextInput::new("Search", "Title:")
        .focused(true)
        .effect(
            ComponentEffect::dissolve(450)
                .pattern(ComponentEffectPattern::RadialCenter {
                    transition_width: 3.0,
                }),
        );

    effects.render_widget(
        frame,
        "search-input",
        area,
        input.clone().widget(),
        input.component_effect(),
    );
    effects.process_frame(frame, area);
})?;
```

The `effects` module also re-exports `tachyonfx`, so consumers can drop down to
raw `tachyonfx::fx::*` composition whenever the preset helpers are not enough.
