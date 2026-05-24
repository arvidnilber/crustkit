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
