# Crustkit: Layout Engine, Component Vision, and Distribution

Date: 2026-05-28
Status: design / vision (most of this is **not built yet**; see "Current state").

This document consolidates three threads into one plan:

1. **Taffy** as crustkit's layout engine.
2. The **full framework architecture** ("Vite + React + shadcn for TUIs").
3. **Distribution & onboarding** — how people (and their agents) discover and
   start a crustkit app, optimizing for *legitimacy* and *simplicity*.

---

## Positioning

> Crustkit is **Vite + React + shadcn for safe-Rust TUIs**.

A composable toolkit on raw Rust + Ratatui + Crossterm that makes terminal apps
responsive, cheap, clean, and non-invasive — without forcing an app framework on
the caller. Distribution is the priority: it should be trivial to go from "I want
a TUI" to a running, good-looking app.

### Non-goals (YAGNI)

Do **not** build a browser in the terminal. Skip: a full CSS engine, 120fps,
real blur / box-shadow / subpixel motion, Framer-Motion feel. The terminal is
cell-based; lean into that.

---

## Current state (what's shipped)

- **Primitives:** terminal lifecycle (`run_with_terminal`, `ManagedTerminal`,
  `restore_terminal_state`), theme (`AppTheme`, `ThemeMode`, palettes), status
  (`StatusLine`), dialogs/input (`InputDialog`, `TextInput`), mouse helpers,
  layout helpers (`body_split`, `centered_rect`), progress, effects.
- **Reload (just shipped):** `CliReload` is now a props-style config. `ctrl+r`
  reload plus an opt-in **debounced file-watch** mode in the dev cargo loop:
  `CliReload::enabled().with_file_watch(true)` (+ `with_watch_debounce_ms`).
  Behind the `watch` feature (`notify` + `notify-debouncer-full`). A burst of
  saves coalesces into one rebuild; relevant files only (`.rs`, `Cargo.toml`).
- **Feature flags:** `default = ["tachyonfx"]`, `watch`, `taffy` (dep added at
  `taffy = "0.10.1"`, **no code yet** — Phase 1 below).
- **Reference consumers:** `sivu` (siv platform operator TUI; opts into watch
  reload via `sivu --hot`) and `plextape`.

---

## The mental model

Not "CSS in the terminal." Instead:

```
Tailwind-ish class → Crustkit Style AST → Taffy layout → Ratatui Rects → Buffer → TachyonFX post effects
```

The layered stack:

| Layer       | Role                                              |
| ----------- | ------------------------------------------------- |
| Taffy       | Layout engine — Flexbox/Grid math, no DOM         |
| Ratatui     | Terminal rendering, widgets, immediate-mode buffer|
| Crossterm   | Keyboard, mouse, resize, raw mode                 |
| TachyonFX   | Cell-based post-processing effects                |
| **Crustkit**| The layer on top: tokens, classes, components     |

Ratatui's immediate-mode + buffer-diff model is the right base: render widgets to
a buffer each frame, diff against the last, write only changes.

---

## Taffy integration (layout engine)

**Taffy owns layout only** — not state, not rendering, not animation. It answers
"this panel is 40 cols wide, this row has 3 children, gap 1, flex-grow…" and we
feed the resulting rectangles to Ratatui.

Pipeline:

```
Component tree
  → resolve classes for current terminal size + interaction state
  → build / update Taffy tree
  → Taffy computes layout
  → convert layout to Ratatui Rect
  → render widgets
  → apply effects / transitions
  → flush terminal diff
```

### Performance model

The core rule: **don't animate the whole app all the time.**

- Render only on: input event, resize, async result, or an active animation tick.
- Run animation ticks only while animations are alive.
- Recompute the Taffy layout only when a layout input changes: terminal size,
  component tree, class/layout props, visibility.
- Target 15–30fps while animating; 60fps only for small local effects.
- hover/focus/active are state transitions, not heavy layout animation.

Event-loop sketch:

```rust
let tick = Duration::from_millis(33); // ~30fps while animating
loop {
    let timeout = if app.animations_active() {
        tick.saturating_sub(last_tick.elapsed())
    } else {
        Duration::from_millis(250)
    };
    if event::poll(timeout)? { /* key / mouse / resize → app.on_* */ }
    if app.animations_active() && now - last_tick >= tick { app.on_tick(now); }
    if app.needs_render() { terminal.draw(|f| app.render(f))?; app.mark_rendered(); }
    if app.should_quit { break; }
}
```

### Interaction & class resolution

```rust
struct InteractionState { focused, hovered, active, disabled, selected: bool }
```

After layout we know each component's `Rect`, so hover detection is point-in-rect.
Class resolution layers variants: base + `hover:` + `focus:` + `active:` +
`disabled:` + responsive breakpoint classes.

### Three animation levels

1. **Style transitions (cheapest, most important):** lerp colors over ~120ms
   between style states (`bg-slate-950 → bg-slate-900`, border, text). A button
   that "breathes" on focus already reads modern.
2. **Layout-ish:** animate a `Rect` for toasts, slide-in panels, command palette
   (ease the target y over time).
3. **Post-processing:** TachyonFX cell effects — fade, dissolve, sweep, pulse,
   glow, loading shimmer.

---

## Proposed module breakdown

Start as **feature-gated modules inside the one crate**, split into sub-crates
only if/when compile times or API surface demand it.

```
core        class parser · state resolver · theme tokens · color interpolation · transition manager
layout      Taffy tree builder · Rect conversion · rounding · responsive breakpoints   [feature: taffy]
render      Ratatui widget glue · component renderer · buffer helpers
fx          tiny animation scheduler · TachyonFX integration · color/rect/opacity transitions  [feature: tachyonfx]
components  Button · Card · Input · Select · RadioGroup · CommandMenu · Table · Toast · Sidebar
```

### Open design decisions

- **Single crate w/ features vs multi-crate workspace?** Lean single-crate first.
- **Class syntax:** runtime strings (`"px-3 py-1 hover:bg-slate-900"`, familiar,
  stringly-typed) **vs** a typed builder (safe Rust, less "Tailwind feel"). A
  typed builder with `.class("…")` escape hatch may be the sweet spot.
- **How much Tailwind to support?** A small, fixed utility set (flex/gap/padding/
  size/border/rounded/bg/text/border-color + the variants above). Resist growth.

---

## Distribution & onboarding (the priority)

Optimize for two things at once: **legitimacy** (looks like a real, trustworthy
Rust tool) and **simplicity** ("npx skills"-level friction).

### 1. README sections

- **"Using with Claude Code"** and **"Using with Codex"** — how an agent consumes
  crustkit: point at the scaffolding skill, the rules, and the getting-started
  guide; show the one-line prompt that creates a new app.

### 2. Getting-started guide

`docs/getting-started.md` exists — extend it into a hand-rolled "your first
crustkit app" walkthrough (the non-agent path), so the project reads as legit and
usable without any agent.

### 3. Scaffolding skill ("create-crustkit" for agents)

A crustkit **skill** (Claude Code + Codex) so a short prompt scaffolds a new
crustkit TUI: `cargo` project, small app shell (`main` → `run_with_terminal`),
one example screen (header/footer/status/list), sensible feature flags, and the
local-dev reload wired. This is the "Vite create" moment for TUIs.

### Distribution options (brainstorm — with trade-offs)

- **A. Agent skill only** (`crustkit-init`): short prompt, designed for new apps,
  shipped via a skills marketplace. *Pros:* matches the "npx skills" simplicity;
  meets devs where agents already are. *Cons:* agent-only; no deterministic
  non-agent path → weaker legitimacy on its own.
- **B. `cargo generate` template** (GitHub template repo). *Pros:* Rust-native,
  deterministic, no agent required, the legit source of truth. *Cons:* requires
  `cargo install cargo-generate`.
- **C. `create-crustkit` binary** (`cargo install create-crustkit` →
  `crustkit new my-app`). *Pros:* closest to create-vite legitimacy. *Cons:*
  another crate to maintain.

**Recommendation: B + A.** Make the `cargo generate` template the single source
of truth (legitimacy, deterministic, works without agents), and ship a thin agent
skill that wraps it (simplicity, "npx skills" UX). The skill scaffolds from the
same template, so there is exactly one place the app shape is defined. Add C later
only if a branded `crustkit new` is worth the maintenance.

---

## Phased roadmap

- **Phase 0 — done:** primitives + `ctrl+r`/debounced-watch reload.
- **Phase 1 — Taffy layout primitive** (started): flex/grid spec → Ratatui
  `Rect`s behind the `taffy` feature, with tests. The contained, genuinely-useful
  core the essay says to reach for "when nested layout gets painful."
- **Phase 2 — theme tokens + style AST + color transition manager.**
- **Phase 3 — interaction state + Tailwind-ish class/variant resolver** (small,
  fixed utility set).
- **Phase 4 — component set:** Button, Card, Input, Select, Table, Toast,
  CommandMenu.
- **Phase 5 — fx helpers** on TachyonFX: fade/dissolve/shimmer/pulse.
- **Phase 6 — distribution:** `cargo generate` template + agent scaffolding skill
  + README "Using with Claude Code / Codex" + getting-started rewrite.

Suggested first step after this doc: implement Phase 1 (the Taffy → `Rect`
adapter), since the `taffy = "0.10.1"` dependency and `taffy` feature flag are
already in place.

---

## Notes for the implementing session

- crustkit is a published crate (`0.1.0`); keep new heavy deps **feature-gated**
  (`taffy`, `watch` already are) so default consumers stay lean.
- Local consumers use the `[patch.crates-io]` path (see `scripts/use-local-crustkit.sh`);
  features must be enabled in the consumer's dependency line (sivu now sets
  `features = ["tachyonfx", "watch"]`).
- This document was produced after a session that shipped the reload/watcher,
  fixed the sivu panel drop-shadow, and added the `taffy` dependency.
