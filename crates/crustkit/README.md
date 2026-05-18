# Crustkit

Small reusable Rust TUI primitives for command-line applications built with
Ratatui and Crossterm.

Crustkit is intentionally not a full application framework. It provides terminal
lifecycle helpers, status lines, key hints, shell header/footer helpers, transfer
progress widgets, theme primitives, and small adaptive layout helpers. App
domain logic belongs in the consuming crate.

```toml
[dependencies]
crustkit = "0.1.0"
```

