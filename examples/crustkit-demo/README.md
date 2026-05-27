# Crustkit Demo

Publish-disabled showcase app for the alpha release.

```bash
cargo run -p crustkit-demo -- --help
cargo run -p crustkit-demo
cargo run -p crustkit-demo -- --no-effects
```

The demo is intentionally local-only and synthetic. It shows:

- keyboard tabs, row movement, activation, and confirmation gates
- mouse tab, row, theme-chip, segmented-control, hover, and wheel handling
- terminal-safe theme choices: auto, dark, light, high-contrast dark,
  high-contrast light, and monochrome
- progress gauges, status lines, key hints, text input, dialogs, layout, mouse,
  and panel shadows
- TachyonFX presets, patterns, filters, repeat modes, and keyed managers plus
  the runtime `--no-effects` switch
- a credits tab thanking Rust, Ratatui, Crossterm, TachyonFX, color-eyre, and
  clap

Do not add product-specific commands, repository names, workflow names, or
machine-local paths here. The point is to showcase the framework with public,
portable examples.
