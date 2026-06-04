# Crustkit Demo

Run:

```bash
cargo run -p crustkit-demo
```

Useful checks:

```bash
cargo run -p crustkit-demo -- --help
cargo run -p crustkit-demo -- --no-effects
cargo check -p crustkit-demo --no-default-features
```

The demo mirrors production-style interaction patterns without shipping any real
command surfaces:

- `Overview`: terminal lifecycle, status, progress, and default effects.
- `Components`: `TextInput`, `InputDialog`, `KeyHint`, `StatusLine`, and
  themed dialog composition.
- `Actions`: keyboard-first list navigation plus confirm-before-reset flow.
- `Mouse Lab`: tab hit-tests, row hit-tests, wheel scrolling, and inline
  segmented controls.
- `Effects`: preset, pattern, filter, repeat, and keyed manager examples.
- `Themes`: terminal background detection plus six terminal-safe choices
  (auto, dark, light, high-contrast dark, high-contrast light, and
  monochrome).
- `Credits`: brief thanks to the Rust language and the projects Crustkit builds
  on.

Keep the demo fake-command based. It must not run GitHub, deploy, process,
workspace, or platform-specific commands.
