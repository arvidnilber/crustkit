# Changelog

## Unreleased

- Enable TachyonFX by default while preserving the slim opt-out path with
  `default-features = false`.
- Add a publish-disabled `crustkit-demo` workspace app with keyboard, mouse,
  theme, progress, dialog, shadow, and runtime `--no-effects` examples.
- Remove the `termbg` dependency and keep terminal background detection inside
  Crustkit to avoid a duplicate Crossterm stack and test/logging dependencies.
- Reduce hot-path allocation churn in layout, input/dialog span construction,
  and progress label formatting.

## 0.1.0 - 2026-05-19

- First public Crustkit release.
- Provides reusable Ratatui/Crossterm primitives for terminal lifecycle,
  status lines, key hints, shell chrome, dialogs, text input, mouse helpers,
  progress gauges, theme palettes, and responsive layout helpers.
- Keeps domain logic out of the framework crate so consuming TUIs can stay
  small and explicit.
