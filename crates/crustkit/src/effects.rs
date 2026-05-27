use std::{
    collections::BTreeMap,
    fmt::Debug,
    time::{Duration as StdDuration, Instant},
};

use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::Color,
    widgets::Widget,
};
use tachyonfx::{
    CellFilter, Effect, EffectManager, EffectTimer, Interpolation, Motion, fx,
    pattern::{
        CheckerboardPattern, CoalescePattern, DiagonalDirection, DiagonalPattern, DissolvePattern,
        RadialPattern, SweepPattern,
    },
};

pub use tachyonfx;

pub type UiEffectManager<K = &'static str> = EffectManager<K>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectPreset {
    FadeFromFg(Color),
    FadeToFg(Color),
    FadeFrom {
        fg: Color,
        bg: Color,
    },
    FadeTo {
        fg: Color,
        bg: Color,
    },
    Dissolve,
    Coalesce,
    SweepIn {
        direction: Motion,
        gradient_length: u16,
        randomness: u16,
        faded_color: Color,
    },
    SlideIn {
        direction: Motion,
        gradient_length: u16,
        randomness: u16,
        behind_color: Color,
    },
    PulseFg(Color),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentEffectPattern {
    RadialCenter {
        transition_width: f32,
    },
    Diagonal {
        direction: DiagonalDirection,
        transition_width: f32,
    },
    Sweep {
        direction: Motion,
        gradient_length: u16,
    },
    Checkerboard {
        cell_size: u16,
        transition_width: f32,
    },
    Dissolve,
    Coalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentEffectFilter {
    All,
    Text,
    Inner { horizontal: u16, vertical: u16 },
    Outer { horizontal: u16, vertical: u16 },
    FgColor(Color),
    BgColor(Color),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectRepeat {
    Once,
    Loop,
    PingPong,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentEffect {
    pub preset: EffectPreset,
    pub duration_ms: u32,
    pub interpolation: Interpolation,
    pub filter: Option<ComponentEffectFilter>,
    pub pattern: Option<ComponentEffectPattern>,
    pub repeat: EffectRepeat,
}

impl ComponentEffect {
    pub const fn new(preset: EffectPreset, duration_ms: u32) -> Self {
        Self {
            preset,
            duration_ms,
            interpolation: Interpolation::Linear,
            filter: Some(ComponentEffectFilter::Text),
            pattern: None,
            repeat: EffectRepeat::Once,
        }
    }

    pub const fn fade_from_fg(color: Color, duration_ms: u32) -> Self {
        Self::new(EffectPreset::FadeFromFg(color), duration_ms)
    }

    pub const fn fade_to_fg(color: Color, duration_ms: u32) -> Self {
        Self::new(EffectPreset::FadeToFg(color), duration_ms)
    }

    pub const fn dissolve(duration_ms: u32) -> Self {
        Self::new(EffectPreset::Dissolve, duration_ms)
    }

    pub const fn coalesce(duration_ms: u32) -> Self {
        Self::new(EffectPreset::Coalesce, duration_ms)
    }

    pub const fn sweep_in(
        direction: Motion,
        gradient_length: u16,
        faded_color: Color,
        duration_ms: u32,
    ) -> Self {
        Self::new(
            EffectPreset::SweepIn {
                direction,
                gradient_length,
                randomness: 0,
                faded_color,
            },
            duration_ms,
        )
    }

    pub const fn slide_in(
        direction: Motion,
        gradient_length: u16,
        behind_color: Color,
        duration_ms: u32,
    ) -> Self {
        Self::new(
            EffectPreset::SlideIn {
                direction,
                gradient_length,
                randomness: 0,
                behind_color,
            },
            duration_ms,
        )
    }

    pub const fn pulse_fg(color: Color, duration_ms: u32) -> Self {
        Self {
            preset: EffectPreset::PulseFg(color),
            duration_ms,
            interpolation: Interpolation::SineInOut,
            filter: Some(ComponentEffectFilter::Text),
            pattern: None,
            repeat: EffectRepeat::PingPong,
        }
    }

    pub const fn interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub const fn filter(mut self, filter: ComponentEffectFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub const fn no_filter(mut self) -> Self {
        self.filter = None;
        self
    }

    pub const fn pattern(mut self, pattern: ComponentEffectPattern) -> Self {
        self.pattern = Some(pattern);
        self
    }

    pub const fn repeat(mut self, repeat: EffectRepeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn into_effect(self, area: Rect) -> Effect {
        let timer = EffectTimer::from_ms(self.duration_ms, self.interpolation);
        let effect = match self.preset {
            EffectPreset::FadeFromFg(color) => fx::fade_from_fg(color, timer),
            EffectPreset::FadeToFg(color) => fx::fade_to_fg(color, timer),
            EffectPreset::FadeFrom { fg, bg } => fx::fade_from(fg, bg, timer),
            EffectPreset::FadeTo { fg, bg } => fx::fade_to(fg, bg, timer),
            EffectPreset::Dissolve => fx::dissolve(timer),
            EffectPreset::Coalesce => fx::coalesce(timer),
            EffectPreset::SweepIn {
                direction,
                gradient_length,
                randomness,
                faded_color,
            } => fx::sweep_in(direction, gradient_length, randomness, faded_color, timer),
            EffectPreset::SlideIn {
                direction,
                gradient_length,
                randomness,
                behind_color,
            } => fx::slide_in(direction, gradient_length, randomness, behind_color, timer),
            EffectPreset::PulseFg(color) => fx::fade_to_fg(color, timer),
        };

        let effect = match self.pattern {
            Some(ComponentEffectPattern::RadialCenter { transition_width }) => {
                effect.with_pattern(RadialPattern::center().with_transition_width(transition_width))
            }
            Some(ComponentEffectPattern::Diagonal {
                direction,
                transition_width,
            }) => effect.with_pattern(DiagonalPattern::new(direction, transition_width)),
            Some(ComponentEffectPattern::Sweep {
                direction,
                gradient_length,
            }) => effect.with_pattern(SweepPattern::new(direction, gradient_length)),
            Some(ComponentEffectPattern::Checkerboard {
                cell_size,
                transition_width,
            }) => effect.with_pattern(CheckerboardPattern::new(cell_size, transition_width)),
            Some(ComponentEffectPattern::Dissolve) => effect.with_pattern(DissolvePattern::new()),
            Some(ComponentEffectPattern::Coalesce) => effect.with_pattern(CoalescePattern::new()),
            None => effect,
        };

        let effect = match self.filter {
            Some(filter) => effect.with_filter(filter.into_cell_filter()),
            None => effect,
        };

        match self.repeat {
            EffectRepeat::Once => effect.with_area(area),
            EffectRepeat::Loop => fx::repeating(effect).with_area(area),
            EffectRepeat::PingPong => fx::ping_pong(effect).with_area(area),
        }
    }
}

impl ComponentEffectFilter {
    pub const fn inner(horizontal: u16, vertical: u16) -> Self {
        Self::Inner {
            horizontal,
            vertical,
        }
    }

    pub const fn outer(horizontal: u16, vertical: u16) -> Self {
        Self::Outer {
            horizontal,
            vertical,
        }
    }

    fn into_cell_filter(self) -> CellFilter {
        match self {
            Self::All => CellFilter::All,
            Self::Text => CellFilter::Text,
            Self::Inner {
                horizontal,
                vertical,
            } => CellFilter::Inner(Margin::new(horizontal, vertical)),
            Self::Outer {
                horizontal,
                vertical,
            } => CellFilter::Outer(Margin::new(horizontal, vertical)),
            Self::FgColor(color) => CellFilter::FgColor(color),
            Self::BgColor(color) => CellFilter::BgColor(color),
        }
    }
}

#[derive(Debug)]
pub struct ComponentEffectManager<K: Clone + Debug + Default + Ord + 'static> {
    manager: EffectManager<K>,
    active: BTreeMap<K, (ComponentEffect, Rect)>,
    last_frame: Instant,
}

impl<K: Clone + Debug + Default + Ord + 'static> Default for ComponentEffectManager<K> {
    fn default() -> Self {
        Self {
            manager: EffectManager::default(),
            active: BTreeMap::new(),
            last_frame: Instant::now(),
        }
    }
}

impl<K: Clone + Debug + Default + Ord + 'static> ComponentEffectManager<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self) -> bool {
        self.manager.is_running()
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.manager.add_effect(effect);
    }

    pub fn start(&mut self, key: K, effect: ComponentEffect, area: Rect) {
        self.manager
            .add_unique_effect(key.clone(), effect.into_effect(area));
        self.active.insert(key, (effect, area));
    }

    pub fn start_once(&mut self, key: K, effect: ComponentEffect, area: Rect) {
        if self.active.get(&key) != Some(&(effect, area)) {
            self.start(key, effect, area);
        }
    }

    pub fn clear_key(&mut self, key: &K) {
        self.active.remove(key);
    }

    pub fn render_widget<W>(
        &mut self,
        frame: &mut Frame<'_>,
        key: K,
        area: Rect,
        widget: W,
        effect: Option<ComponentEffect>,
    ) where
        W: Widget,
    {
        frame.render_widget(widget, area);
        if let Some(effect) = effect {
            self.start_once(key, effect, area);
        } else {
            self.active.remove(&key);
        }
    }

    pub fn process_frame(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let elapsed = self.elapsed();
        self.manager
            .process_effects(elapsed.into(), frame.buffer_mut(), area);
    }

    pub fn process_frame_after(&mut self, elapsed: StdDuration, frame: &mut Frame<'_>, area: Rect) {
        self.manager
            .process_effects(elapsed.into(), frame.buffer_mut(), area);
    }

    fn elapsed(&mut self) -> StdDuration {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_frame);
        self.last_frame = now;
        elapsed
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::*;

    #[test]
    fn component_effect_builds_tachyon_effect_with_area() {
        let effect = ComponentEffect::dissolve(250)
            .pattern(ComponentEffectPattern::RadialCenter {
                transition_width: 3.0,
            })
            .into_effect(Rect::new(1, 2, 10, 4));

        assert_eq!(effect.name(), "dissolve");
    }

    #[test]
    fn start_once_does_not_restart_the_same_component_effect() {
        let mut effects = ComponentEffectManager::new();
        let effect = ComponentEffect::fade_from_fg(Color::Black, 120);
        let area = Rect::new(0, 0, 10, 2);

        effects.start_once("input", effect, area);
        effects.start_once("input", effect, area);

        assert_eq!(effects.active.len(), 1);
    }
}
