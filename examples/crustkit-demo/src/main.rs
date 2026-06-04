#![forbid(unsafe_code)]

use std::time::{Duration, Instant};

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use crustkit::{
    AppTheme, DialogTheme, InputDialog, KeyHint, ManagedTerminal, NavigationCommand,
    NavigationFocus, ProgressBarTheme, ResizableSidebar, StatusLine, TextInput, ThemeMode,
    TransferProgress, body_split, centered_dialog_rect, drain_events, exit_key_hint, footer,
    inline_item_index_at, is_exit_key, key_hints_line, left_mouse_click, mouse_position,
    navigation_command, rect_contains, row_index_at, run_with_terminal, transfer_progress_gauge,
};
#[cfg(feature = "tachyonfx")]
use crustkit::{
    ComponentEffect, ComponentEffectFilter, ComponentEffectManager, ComponentEffectPattern,
    EffectRepeat,
    effects::tachyonfx::{Interpolation, Motion, pattern::DiagonalDirection},
};
#[cfg(feature = "tachyonfx")]
type DemoEffect = ComponentEffect;
#[cfg(not(feature = "tachyonfx"))]
type DemoEffect = ();
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap},
};

#[derive(Debug, Parser)]
#[command(
    name = "crustkit-demo",
    about = "Showcase Crustkit keyboard, mouse, themes, progress, dialogs, and effects."
)]
struct Args {
    /// Disable TachyonFX animations at runtime.
    #[arg(long)]
    no_effects: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let mut app = App::new(!args.no_effects);
    run_with_terminal(|terminal| app.run(terminal))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview,
    Components,
    Actions,
    MouseLab,
    Effects,
    Themes,
    Credits,
}

impl Tab {
    const ALL: [Self; 7] = [
        Self::Overview,
        Self::Components,
        Self::Actions,
        Self::MouseLab,
        Self::Effects,
        Self::Themes,
        Self::Credits,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Components => "Components",
            Self::Actions => "Actions",
            Self::MouseLab => "Mouse Lab",
            Self::Effects => "Effects",
            Self::Themes => "Themes",
            Self::Credits => "Credits",
        }
    }

    const fn glyph(self) -> &'static str {
        match self {
            Self::Overview => "▦",
            Self::Components => "▣",
            Self::Actions => "▶",
            Self::MouseLab => "⌖",
            Self::Effects => "✦",
            Self::Themes => "◈",
            Self::Credits => "◆",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoTheme {
    Auto,
    Dark,
    Light,
    HighContrastDark,
    HighContrastLight,
    Monochrome,
}

impl DemoTheme {
    const ALL: [Self; 6] = [
        Self::Auto,
        Self::Dark,
        Self::Light,
        Self::HighContrastDark,
        Self::HighContrastLight,
        Self::Monochrome,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::HighContrastDark => "HC Dark",
            Self::HighContrastLight => "HC Light",
            Self::Monochrome => "Mono",
        }
    }

    fn app_theme(self) -> AppTheme {
        match self {
            Self::Auto => AppTheme::detect_or(Duration::from_millis(80), ThemeMode::Dark),
            Self::Dark => AppTheme::dark(),
            Self::Light => AppTheme::light(),
            Self::HighContrastDark => AppTheme::new(crustkit::ThemePalette {
                mode: ThemeMode::Dark,
                foreground: Color::White,
                background: Color::Black,
                panel_background: Color::Black,
                panel_alt_background: Color::Black,
                muted_foreground: Color::Rgb(190, 190, 190),
                border: Color::White,
                accent: Color::Yellow,
                accent_foreground: Color::Black,
                selection_foreground: Color::Black,
                selection_background: Color::Yellow,
            }),
            Self::HighContrastLight => AppTheme::new(crustkit::ThemePalette {
                mode: ThemeMode::Light,
                foreground: Color::Black,
                background: Color::White,
                panel_background: Color::White,
                panel_alt_background: Color::White,
                muted_foreground: Color::Rgb(65, 65, 65),
                border: Color::Black,
                accent: Color::Blue,
                accent_foreground: Color::White,
                selection_foreground: Color::White,
                selection_background: Color::Blue,
            }),
            Self::Monochrome => AppTheme::new(crustkit::ThemePalette {
                mode: ThemeMode::Dark,
                foreground: Color::Gray,
                background: Color::Black,
                panel_background: Color::Black,
                panel_alt_background: Color::Black,
                muted_foreground: Color::DarkGray,
                border: Color::Gray,
                accent: Color::White,
                accent_foreground: Color::Black,
                selection_foreground: Color::Black,
                selection_background: Color::White,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Segment {
    Rows,
    Tabs,
    ThemeChips,
}

impl Segment {
    const ALL: [Self; 3] = [Self::Rows, Self::Tabs, Self::ThemeChips];

    const fn label(self) -> &'static str {
        match self {
            Self::Rows => "Rows",
            Self::Tabs => "Tabs",
            Self::ThemeChips => "Theme chips",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectChoice {
    Fade,
    Dissolve,
    Coalesce,
    Sweep,
    Slide,
    Pulse,
}

impl EffectChoice {
    const ALL: [Self; 6] = [
        Self::Fade,
        Self::Dissolve,
        Self::Coalesce,
        Self::Sweep,
        Self::Slide,
        Self::Pulse,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Fade => "fade",
            Self::Dissolve => "dissolve",
            Self::Coalesce => "coalesce",
            Self::Sweep => "sweep",
            Self::Slide => "slide",
            Self::Pulse => "pulse",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Fade => "foreground color fades in",
            Self::Dissolve => "text dissolves through a radial pattern",
            Self::Coalesce => "cells gather into place",
            Self::Sweep => "directional reveal from left to right",
            Self::Slide => "content slides over a shaded backing color",
            Self::Pulse => "foreground pulses with ping-pong timing",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Action {
    label: &'static str,
    detail: &'static str,
    destructive: bool,
}

struct App {
    tab: Tab,
    theme_choice: DemoTheme,
    theme: AppTheme,
    actions: Vec<Action>,
    action_state: ListState,
    focus: NavigationFocus,
    status: StatusLine,
    confirm: Option<Action>,
    segment: Segment,
    effect_choice: EffectChoice,
    progress_started: Instant,
    input_value: String,
    effects_enabled: bool,
    quit: bool,
    hover: Option<crustkit::MouseClick>,
    sidebar: ResizableSidebar,
    body_area: Rect,
    tab_area: Rect,
    action_area: Rect,
    effect_area: Rect,
    theme_area: Rect,
    segment_area: Rect,
    #[cfg(feature = "tachyonfx")]
    effects: ComponentEffectManager<&'static str>,
}

impl App {
    fn new(effects_enabled: bool) -> Self {
        let mut action_state = ListState::default();
        action_state.select(Some(0));
        let theme_choice = DemoTheme::Auto;

        Self {
            tab: Tab::Overview,
            theme_choice,
            theme: theme_choice.app_theme(),
            actions: vec![
                Action {
                    label: "Generate preview",
                    detail: "Simulates a short command and reports through the status line.",
                    destructive: false,
                },
                Action {
                    label: "Open command palette",
                    detail: "Shows the same modal discipline a production app should use.",
                    destructive: false,
                },
                Action {
                    label: "Reset demo state",
                    detail: "Uses a confirmation gate before changing durable state.",
                    destructive: true,
                },
                Action {
                    label: "Export theme sample",
                    detail: "A harmless fake action for testing keyboard and mouse activation.",
                    destructive: false,
                },
            ],
            action_state,
            focus: NavigationFocus::Header,
            status: StatusLine::info("ready"),
            confirm: None,
            segment: Segment::Rows,
            effect_choice: EffectChoice::Fade,
            progress_started: Instant::now(),
            input_value: "crustkit".to_string(),
            effects_enabled,
            quit: false,
            hover: None,
            sidebar: ResizableSidebar::new(34, 22, 58)
                .with_min_content_width(34)
                .with_compact(96, 10)
                .with_hit_slop(2),
            body_area: Rect::default(),
            tab_area: Rect::default(),
            action_area: Rect::default(),
            effect_area: Rect::default(),
            theme_area: Rect::default(),
            segment_area: Rect::default(),
            #[cfg(feature = "tachyonfx")]
            effects: ComponentEffectManager::new(),
        }
    }

    fn run(&mut self, terminal: &mut ManagedTerminal) -> Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.render(frame))?;

            for input in drain_events(Duration::from_millis(80))? {
                match input {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key(key.code, key.modifiers);
                    }
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if is_exit_key(code, modifiers) {
            self.quit = true;
            return;
        }

        if let Some(action) = self.confirm {
            match code {
                KeyCode::Char('y') | KeyCode::Enter => self.confirm_action(action),
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.confirm = None;
                    self.status = StatusLine::info("cancelled");
                }
                _ => {}
            }
            return;
        }

        if self.handle_components_input_key(code, modifiers) {
            return;
        }

        if matches!(code, KeyCode::Char('q')) {
            self.quit = true;
            return;
        }

        match navigation_command(code, self.focus, self.at_first_item()) {
            NavigationCommand::NextTab => self.next_tab(),
            NavigationCommand::PreviousTab => self.previous_tab(),
            NavigationCommand::EnterBody => self.enter_body(),
            NavigationCommand::ExitBody => self.exit_body(),
            NavigationCommand::NextItem => self.next_row(),
            NavigationCommand::PreviousItem => self.previous_row(),
            NavigationCommand::Activate => self.activate_selected(),
            NavigationCommand::None => {}
        }

        match code {
            KeyCode::Char('n') if self.tab == Tab::Effects => self.next_effect(),
            KeyCode::Char('t') => self.next_theme(),
            KeyCode::Char('e') => {
                self.effects_enabled = !self.effects_enabled;
                self.reset_effects();
                self.status = StatusLine::info(if self.effects_enabled {
                    "effects enabled"
                } else {
                    "effects disabled"
                });
            }
            _ => {}
        }
    }

    fn handle_components_input_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if self.tab != Tab::Components || self.focus != NavigationFocus::Body {
            return false;
        }

        match code {
            KeyCode::Char(ch)
                if !modifiers.contains(KeyModifiers::CONTROL)
                    && !modifiers.contains(KeyModifiers::ALT) =>
            {
                self.input_value.push(ch);
                self.status = StatusLine::info("input updated");
                true
            }
            KeyCode::Backspace => {
                self.input_value.pop();
                self.status = StatusLine::info("input updated");
                true
            }
            KeyCode::Delete => {
                self.input_value.clear();
                self.status = StatusLine::info("input cleared");
                true
            }
            KeyCode::Enter => {
                self.status = StatusLine::success(format!("query: {}", self.input_value));
                true
            }
            KeyCode::Esc => {
                self.exit_body();
                self.status = StatusLine::info("input blurred");
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        self.hover = Some(mouse_position(mouse));

        if matches!(self.tab, Tab::Actions | Tab::MouseLab)
            && let Some(event) = self.sidebar.handle_mouse(mouse, self.body_area)
        {
            let message = format!("sidebar {} {}", event.size(), event.axis().unit());
            self.status = if event.is_finished() {
                StatusLine::success(format!("{message} set"))
            } else {
                StatusLine::info(message)
            };
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollDown => self.next_row(),
            MouseEventKind::ScrollUp => self.previous_row(),
            _ => {}
        }

        let Some(click) = left_mouse_click(mouse) else {
            if let Some(position) = self.hover
                && rect_contains(self.action_area, position)
            {
                self.status = StatusLine::info("hovering action list");
            }
            return;
        };

        if let Some(index) = inline_item_index_at(self.tab_area, click, tab_widths()) {
            self.set_tab(Tab::ALL[index]);
            self.focus = NavigationFocus::Header;
            self.status = StatusLine::info("tab selected");
            return;
        }

        if let Some(index) = inline_item_index_at(self.theme_area, click, theme_widths()) {
            self.focus = NavigationFocus::Body;
            self.set_theme(DemoTheme::ALL[index]);
            return;
        }

        if let Some(index) = inline_item_index_at(self.segment_area, click, segment_widths()) {
            self.focus = NavigationFocus::Body;
            self.segment = Segment::ALL[index];
            self.status = StatusLine::info("mouse lab segment selected");
            return;
        }

        if self.tab == Tab::Effects
            && let Some(index) = row_index_at(self.effect_area, click, 0, EffectChoice::ALL.len())
        {
            self.focus = NavigationFocus::Body;
            self.set_effect(EffectChoice::ALL[index]);
            return;
        }

        if let Some(index) = row_index_at(self.action_area, click, 1, self.actions.len()) {
            self.focus = NavigationFocus::Body;
            self.action_state.select(Some(index));
            self.activate_selected();
        }
    }

    fn next_tab(&mut self) {
        let index = Tab::ALL
            .iter()
            .position(|tab| *tab == self.tab)
            .unwrap_or_default();
        self.set_tab(Tab::ALL[(index + 1) % Tab::ALL.len()]);
    }

    fn previous_tab(&mut self) {
        let index = Tab::ALL
            .iter()
            .position(|tab| *tab == self.tab)
            .unwrap_or_default();
        self.set_tab(Tab::ALL[(index + Tab::ALL.len() - 1) % Tab::ALL.len()]);
    }

    fn set_tab(&mut self, tab: Tab) {
        if self.tab != tab {
            self.tab = tab;
            self.focus = NavigationFocus::Header;
            self.reset_effects();
        }
    }

    fn at_first_item(&self) -> bool {
        match self.tab {
            Tab::Actions | Tab::MouseLab => self.action_state.selected().unwrap_or_default() == 0,
            Tab::Effects => self.effect_choice == EffectChoice::ALL[0],
            _ => true,
        }
    }

    fn enter_body(&mut self) {
        self.focus = NavigationFocus::Body;
        match self.tab {
            Tab::Actions | Tab::MouseLab => self.action_state.select(Some(0)),
            Tab::Effects => self.set_effect(EffectChoice::ALL[0]),
            _ => {}
        }
    }

    fn exit_body(&mut self) {
        self.focus = NavigationFocus::Header;
    }

    fn next_row(&mut self) {
        if self.tab == Tab::Effects {
            self.next_effect();
            return;
        }

        let selected = self.action_state.selected().unwrap_or_default();
        self.action_state
            .select(Some((selected + 1) % self.actions.len()));
    }

    fn previous_row(&mut self) {
        if self.tab == Tab::Effects {
            self.previous_effect();
            return;
        }

        let selected = self.action_state.selected().unwrap_or_default();
        self.action_state.select(Some(
            (selected + self.actions.len() - 1) % self.actions.len(),
        ));
    }

    fn activate_selected(&mut self) {
        if self.tab == Tab::Effects {
            self.reset_effects();
            self.status = StatusLine::info(self.effect_choice.label());
            return;
        }

        let Some(action) = self
            .action_state
            .selected()
            .and_then(|index| self.actions.get(index))
            .copied()
        else {
            return;
        };

        if action.destructive {
            self.confirm = Some(action);
            self.status = StatusLine::warning("confirm reset with y/n");
        } else {
            self.confirm_action(action);
        }
    }

    fn confirm_action(&mut self, action: Action) {
        self.confirm = None;
        self.progress_started = Instant::now();
        self.status = StatusLine::success(action.label);
    }

    fn next_theme(&mut self) {
        let index = DemoTheme::ALL
            .iter()
            .position(|theme| *theme == self.theme_choice)
            .unwrap_or_default();
        self.set_theme(DemoTheme::ALL[(index + 1) % DemoTheme::ALL.len()]);
    }

    fn set_theme(&mut self, theme: DemoTheme) {
        self.theme_choice = theme;
        self.theme = theme.app_theme();
        self.reset_effects();
        self.status = StatusLine::info(theme.label());
    }

    fn next_effect(&mut self) {
        let index = EffectChoice::ALL
            .iter()
            .position(|choice| *choice == self.effect_choice)
            .unwrap_or_default();
        self.set_effect(EffectChoice::ALL[(index + 1) % EffectChoice::ALL.len()]);
    }

    fn previous_effect(&mut self) {
        let index = EffectChoice::ALL
            .iter()
            .position(|choice| *choice == self.effect_choice)
            .unwrap_or_default();
        self.set_effect(
            EffectChoice::ALL[(index + EffectChoice::ALL.len() - 1) % EffectChoice::ALL.len()],
        );
    }

    fn set_effect(&mut self, effect: EffectChoice) {
        self.effect_choice = effect;
        self.reset_effects();
        self.status = StatusLine::info(self.effect_choice.label());
    }

    fn reset_effects(&mut self) {
        #[cfg(feature = "tachyonfx")]
        {
            self.effects = ComponentEffectManager::new();
        }
    }

    fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let root = Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(area);

        self.render_demo_header(frame, root[0]);

        self.tab_area = root[1];
        self.body_area = root[2];
        frame.render_widget(self.tabs(), root[1]);

        self.action_area = Rect::default();
        self.effect_area = Rect::default();
        self.theme_area = Rect::default();
        self.segment_area = Rect::default();

        match self.tab {
            Tab::Overview => self.render_overview(frame, root[2]),
            Tab::Components => self.render_components(frame, root[2]),
            Tab::Actions => self.render_actions(frame, root[2]),
            Tab::MouseLab => self.render_mouse_lab(frame, root[2]),
            Tab::Effects => self.render_effects(frame, root[2]),
            Tab::Themes => self.render_themes(frame, root[2]),
            Tab::Credits => self.render_credits(frame, root[2]),
        }

        let hints = [
            KeyHint::new("tab", "view"),
            KeyHint::new("j/k", "move"),
            KeyHint::new("enter", "run"),
            KeyHint::new("n", "effect"),
            KeyHint::new("t", "theme"),
            KeyHint::new("e", "effects"),
            exit_key_hint(),
        ];
        frame.render_widget(footer(hints, Some(&self.status)), root[3]);

        if let Some(action) = self.confirm {
            self.render_confirm(frame, area, action);
        }

        #[cfg(feature = "tachyonfx")]
        if self.effects_enabled {
            self.effects.process_frame(frame, area);
        }
    }

    fn tabs(&self) -> Paragraph<'static> {
        let hover = self.hovered_tab_index();
        Paragraph::new(Line::from(
            Tab::ALL
                .into_iter()
                .enumerate()
                .map(|(index, tab)| {
                    if tab == self.tab {
                        let mut style = self.selection_style();
                        if self.focus == NavigationFocus::Header {
                            style = style.add_modifier(Modifier::UNDERLINED);
                        }
                        Span::styled(format!(" {} {} ", tab.glyph(), tab.label()), style)
                    } else if hover == Some(index) {
                        Span::styled(
                            format!(" {} {} ", tab.glyph(), tab.label()),
                            self.hover_style(),
                        )
                    } else {
                        Span::styled(
                            format!(" {} {} ", tab.glyph(), tab.label()),
                            self.muted_style(),
                        )
                    }
                })
                .collect::<Vec<_>>(),
        ))
    }

    fn render_demo_header(&self, frame: &mut Frame<'_>, area: Rect) {
        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ◆ ", Style::new().fg(self.foreground_color())),
                Span::styled(
                    "C R U S T K I T",
                    Style::new()
                        .fg(self.foreground_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("   ·   ", self.dim_style()),
                Span::styled("primitive lab", self.muted_style()),
                Span::styled("   ·   ", self.dim_style()),
                Span::styled(self.tab.label(), self.accent_style()),
                Span::styled("   ·   ", self.dim_style()),
                Span::styled(
                    if self.effects_enabled {
                        "effects on"
                    } else {
                        "effects off"
                    },
                    self.muted_style(),
                ),
            ]),
            Line::from(""),
        ];
        frame.render_widget(Paragraph::new(lines), area);
    }

    fn render_overview(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let body = body_split(area, 92, 36, 9);
        self.render_panel_block(frame, body[0], "Status");

        let elapsed = self.progress_started.elapsed().as_secs_f64();
        let downloaded = ((elapsed.sin().abs() * 4_000_000.0) as u64).max(512_000);
        let progress =
            TransferProgress::new(downloaded, Some(4_000_000), self.progress_started.elapsed());
        let gauge = transfer_progress_gauge(
            progress,
            ProgressBarTheme::new(
                self.surface_style(),
                self.progress_style(),
                self.progress_label_style(),
            ),
        );

        let left = Layout::vertical([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .margin(1)
        .split(body[0]);

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("alpha-ready primitives", self.accent_style())),
                Line::from("terminal lifecycle, reload, input, dialogs, mouse, progress"),
            ])
            .wrap(Wrap { trim: true }),
            left[0],
        );
        frame.render_widget(gauge, left[1]);

        self.render_panel_block(frame, body[1], "Preview");
        let preview = Paragraph::new(vec![
            Line::from("Default effects are enabled by Cargo feature."),
            Line::from("Use --no-effects to disable runtime animation in this demo."),
            Line::from("Consumers can also use default-features = false."),
        ])
        .style(self.surface_style())
        .wrap(Wrap { trim: true });
        self.render_with_optional_effect(
            frame,
            "overview-preview",
            body[1].inner(Margin::new(1, 1)),
            preview,
        );
    }

    fn render_components(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let body = body_split(area, 104, 42, 14);
        self.render_panel_block(frame, body[0], "Input");
        self.render_panel_block(frame, body[1], "Dialogs + Chrome");

        let left = Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Min(2),
        ])
        .margin(1)
        .split(body[0]);

        let text_input = TextInput::new("TextInput", "Query:")
            .value(self.input_value.clone())
            .placeholder("type a search")
            .help("builder widget, cursor, placeholder, focus style")
            .focused(self.focus == NavigationFocus::Body);
        #[cfg(feature = "tachyonfx")]
        let text_input =
            text_input.effect(ComponentEffect::fade_from_fg(self.tab_accent_color(), 420));
        #[cfg(feature = "tachyonfx")]
        let text_effect = text_input.component_effect();
        #[cfg(not(feature = "tachyonfx"))]
        let text_effect = None;

        self.render_widget_with_effect(
            frame,
            "component-input",
            left[0],
            text_input.widget(),
            text_effect,
        );

        frame.render_widget(
            Paragraph::new(vec![
                key_hints_line([
                    KeyHint::new("tab", "switch"),
                    KeyHint::new("j/k", "move"),
                    KeyHint::new("enter", "activate"),
                ]),
                Line::from(Span::styled(
                    "KeyHint + footer affordances stay compact and app-owned.",
                    self.muted_style(),
                )),
            ])
            .style(self.surface_style()),
            left[1],
        );

        let right = Layout::vertical([
            Constraint::Length(9),
            Constraint::Length(5),
            Constraint::Min(2),
        ])
        .margin(1)
        .split(body[1]);

        let dialog_theme = self.dialog_theme();
        let dialog = InputDialog::new("InputDialog", "Name:")
            .value("Crustkit demo")
            .placeholder("component name")
            .help("same value renderer, centered-dialog sizing, pending button state")
            .primary_label("Save")
            .secondary_label("Esc cancel")
            .pending(self.progress_started.elapsed().as_secs().is_multiple_of(2));
        frame.render_widget(dialog.widget(dialog_theme), right[0]);

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("StatusLine", self.accent_style()),
                    Span::styled("  ", self.dim_style()),
                    self.status.to_span(),
                ]),
                Line::from(Span::styled(
                    "Status is typed in the app state; rendering is just presentation.",
                    self.muted_style(),
                )),
            ])
            .style(self.surface_style())
            .wrap(Wrap { trim: true }),
            right[1],
        );
    }

    fn render_actions(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let body = self.sidebar.layout(area);
        self.render_action_list(frame, body.sidebar);
        self.render_resize_handle(frame, body.handle, body.compact);

        self.render_panel_block(frame, body.content, "Detail");
        let selected = self
            .action_state
            .selected()
            .and_then(|index| self.actions.get(index))
            .unwrap_or(&self.actions[0]);
        let detail = Paragraph::new(vec![
            Line::from(Span::styled(selected.label, self.accent_style())),
            Line::from(""),
            Line::from(selected.detail),
            Line::from(""),
            Line::from(if selected.destructive {
                "This row demonstrates a y/n confirmation gate."
            } else {
                "This row runs immediately and updates status."
            }),
        ])
        .style(self.surface_style())
        .wrap(Wrap { trim: true });
        frame.render_widget(detail, body.content.inner(Margin::new(1, 1)));
    }

    fn render_mouse_lab(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let body = self.sidebar.layout(area);
        self.render_action_list(frame, body.sidebar);
        self.render_resize_handle(frame, body.handle, body.compact);
        self.render_panel_block(frame, body.content, "Mouse Lab");

        let inner = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .margin(1)
        .split(body.content);

        self.segment_area = inner[0];
        frame.render_widget(self.segment_line(), inner[0]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!("Active hit-test: {}", self.segment.label())),
                Line::from("Drag the divider, click rows/chips/tabs, or use the wheel."),
                Line::from("Wheel events move the selected action row."),
            ])
            .style(self.surface_style())
            .wrap(Wrap { trim: true }),
            inner[2],
        );
    }

    fn render_effects(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let body = body_split(area, 104, 40, 12);
        self.render_panel_block(frame, body[0], "Presets");
        self.render_panel_block(frame, body[1], "Patterns + Filters");
        self.effect_area = body[0].inner(Margin::new(1, 1));

        let presets = Paragraph::new(vec![
            self.effect_choice_line(EffectChoice::Fade),
            self.effect_choice_line(EffectChoice::Dissolve),
            self.effect_choice_line(EffectChoice::Coalesce),
            self.effect_choice_line(EffectChoice::Sweep),
            self.effect_choice_line(EffectChoice::Slide),
            self.effect_choice_line(EffectChoice::Pulse),
            Line::from(""),
            Line::from(Span::styled(
                "Press n to change preset, e to toggle processing. default-features = false removes it.",
                self.muted_style(),
            )),
        ])
        .style(self.surface_style())
        .wrap(Wrap { trim: true });

        self.render_widget_with_effect(
            frame,
            "effect-presets",
            self.effect_area,
            presets,
            self.showcase_effect(),
        );

        let right = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("patterns", self.accent_style()),
                Span::raw("  radial, diagonal, sweep, checkerboard, dissolve, coalesce"),
            ]),
            Line::from(vec![
                Span::styled("filters", self.accent_style()),
                Span::raw("   all cells, text cells, inner/outer margins, fg/bg colors"),
            ]),
            Line::from(vec![
                Span::styled("manager", self.accent_style()),
                Span::raw("   keyed effects restart only when props or area change"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("selected", self.accent_style()),
                Span::raw("  "),
                Span::styled(self.effect_choice.description(), self.muted_style()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Effects are cleared on tab/theme/toggle changes so animations do not leak across screens.",
                self.muted_style(),
            )),
        ])
        .style(self.surface_style())
        .wrap(Wrap { trim: true });

        self.render_widget_with_effect(
            frame,
            "effect-patterns",
            body[1].inner(Margin::new(1, 1)),
            right,
            self.pattern_effect(),
        );
    }

    fn render_themes(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.render_panel_block(frame, area, "Themes");
        let inner = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Min(2),
        ])
        .margin(1)
        .split(area);

        self.theme_area = inner[0];
        frame.render_widget(self.theme_line(), inner[0]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!("Selected: {}", self.theme_choice.label())),
                Line::from("Auto asks the terminal for its background and falls back safely."),
                Line::from("Every palette uses high-contrast foreground, muted, border, and selection colors."),
            ])
            .style(self.surface_style())
            .wrap(Wrap { trim: true }),
            inner[1],
        );
    }

    fn render_credits(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let body = body_split(area, 104, 38, 12);
        self.render_panel_block(frame, body[0], "Built On");
        self.render_panel_block(frame, body[1], "Thanks");

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("Ratatui", self.accent_style())),
                Line::from("the rendering model that makes terminal UI feel practical"),
                Line::from(""),
                Line::from(Span::styled("Crossterm", self.accent_style())),
                Line::from("portable terminal input, raw mode, mouse, focus, and screen control"),
                Line::from(""),
                Line::from(Span::styled("TachyonFX", self.accent_style())),
                Line::from("expressive cell effects without making apps own animation plumbing"),
                Line::from(""),
                Line::from(Span::styled("color-eyre + clap", self.accent_style())),
                Line::from("clean error reports and boring, reliable CLI entry points"),
            ])
            .style(self.surface_style())
            .wrap(Wrap { trim: true }),
            body[0].inner(Margin::new(1, 1)),
        );

        frame.render_widget(
            Paragraph::new(vec![
                Line::from("Crustkit exists because these projects made the hard parts approachable."),
                Line::from(""),
                Line::from("Thank you to the maintainers and contributors who keep Rust terminal tooling sharp, portable, and carefully designed."),
                Line::from(""),
                Line::from("And a special thanks to Rust itself: the language keeps pushing us toward explicit ownership, honest errors, fearless refactors, and small fast tools that can last."),
            ])
            .style(self.surface_style())
            .wrap(Wrap { trim: true }),
            body[1].inner(Margin::new(1, 1)),
        );
    }

    fn render_action_list(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.action_area = area;
        self.render_panel_block(frame, area, "Actions");
        let selected = (self.focus == NavigationFocus::Body)
            .then(|| self.action_state.selected())
            .flatten();
        let hover = self.hovered_action_index();
        let items = self
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let marker = if action.destructive { "! " } else { "> " };
                let item = ListItem::new(Line::from(vec![
                    Span::styled(marker, self.accent_style()),
                    Span::raw(action.label),
                ]));
                if hover == Some(index) && selected != Some(index) {
                    item.style(self.hover_style())
                } else {
                    item
                }
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .style(self.surface_style())
            .highlight_style(self.selection_style())
            .highlight_symbol("");

        if self.focus == NavigationFocus::Body {
            frame.render_stateful_widget(
                list,
                area.inner(Margin::new(1, 1)),
                &mut self.action_state,
            );
        } else {
            let mut state = ListState::default();
            frame.render_stateful_widget(list, area.inner(Margin::new(1, 1)), &mut state);
        }
    }

    fn render_confirm(&self, frame: &mut Frame<'_>, area: Rect, action: Action) {
        let rect = centered_dialog_rect(area, 56, 8);
        frame.render_widget(Clear, rect);
        let text = Paragraph::new(vec![
            Line::from(Span::styled("Confirm action", self.accent_style())),
            Line::from(""),
            Line::from(action.label),
            Line::from(""),
            Line::from(vec![
                Span::styled(" y ", self.selection_style()),
                Span::raw(" confirm  "),
                Span::styled(" n ", self.muted_style()),
                Span::raw(" cancel"),
            ]),
        ])
        .block(
            Block::bordered()
                .title("Dialog")
                .border_type(BorderType::Rounded)
                .border_style(self.border_style())
                .padding(Padding::new(1, 1, 0, 0)),
        )
        .style(self.surface_style())
        .wrap(Wrap { trim: true });
        frame.render_widget(text, rect);
    }

    fn render_panel_block(&self, frame: &mut Frame<'_>, area: Rect, title: &'static str) {
        frame.render_widget(
            Block::bordered()
                .title(self.card_title_line(title))
                .border_type(BorderType::Double)
                .border_style(self.border_style())
                .style(self.surface_style()),
            area,
        );
    }

    fn render_resize_handle(&self, frame: &mut Frame<'_>, area: Rect, compact: bool) {
        if area.is_empty() {
            return;
        }

        let hovered = self
            .hover
            .is_some_and(|position| rect_contains(area, position));
        let style = if self.sidebar.is_dragging() {
            self.accent_style()
        } else if hovered {
            self.hover_style()
        } else {
            self.dim_style()
        };

        let lines = if compact {
            vec![Line::from("─".repeat(usize::from(area.width)))]
        } else {
            vec![Line::from("│"); usize::from(area.height)]
        };
        frame.render_widget(Paragraph::new(lines).style(style), area);
    }

    fn surface_style(&self) -> Style {
        Style::new().fg(self.foreground_color())
    }

    #[cfg(feature = "tachyonfx")]
    fn effect_backdrop_color(&self) -> Color {
        bar_bg(self.theme.mode())
    }

    fn muted_style(&self) -> Style {
        Style::new().fg(match self.theme.mode() {
            ThemeMode::Dark => Color::Rgb(150, 155, 175),
            ThemeMode::Light => Color::Rgb(70, 70, 70),
        })
    }

    fn accent_style(&self) -> Style {
        Style::new()
            .fg(self.tab_accent_color())
            .add_modifier(Modifier::BOLD)
    }

    fn border_style(&self) -> Style {
        Style::new().fg(self.border_color())
    }

    fn border_color(&self) -> Color {
        match self.theme.mode() {
            ThemeMode::Dark => Color::Rgb(88, 92, 112),
            ThemeMode::Light => Color::Rgb(132, 132, 142),
        }
    }

    fn foreground_color(&self) -> Color {
        match self.theme.mode() {
            ThemeMode::Dark => Color::Rgb(226, 229, 240),
            ThemeMode::Light => Color::Black,
        }
    }

    fn selection_style(&self) -> Style {
        Style::new()
            .fg(self.tab_accent_color())
            .bg(selection_bg(self.theme.mode()))
            .add_modifier(Modifier::BOLD)
    }

    fn hover_style(&self) -> Style {
        Style::new()
            .fg(self.tab_accent_color())
            .bg(bar_bg(self.theme.mode()))
    }

    fn progress_style(&self) -> Style {
        Style::new()
            .fg(progress_fill(self.tab, self.theme.mode()))
            .bg(bar_bg(self.theme.mode()))
    }

    fn progress_label_style(&self) -> Style {
        Style::new()
            .fg(self.foreground_color())
            .add_modifier(Modifier::BOLD)
    }

    fn effect_choice_line(&self, choice: EffectChoice) -> Line<'static> {
        let active = self.focus == NavigationFocus::Body && choice == self.effect_choice;
        let hover = self
            .hovered_effect_index()
            .and_then(|index| EffectChoice::ALL.get(index));
        let hovered = hover == Some(&choice);
        Line::from(vec![
            Span::styled(if active { "▌ " } else { "  " }, self.accent_style()),
            Span::styled(
                format!("{:<9}", choice.label()),
                if active {
                    self.selection_style()
                } else if hovered {
                    self.hover_style()
                } else {
                    self.accent_style()
                },
            ),
            Span::raw("  "),
            Span::styled(choice.description(), self.muted_style()),
        ])
    }

    fn tab_accent_color(&self) -> Color {
        match (self.tab, self.theme.mode()) {
            (Tab::Overview, ThemeMode::Dark) => Color::Rgb(235, 235, 235),
            (Tab::Overview, ThemeMode::Light) => Color::Rgb(20, 20, 20),
            (Tab::Components, ThemeMode::Dark) => Color::Rgb(110, 195, 220),
            (Tab::Components, ThemeMode::Light) => Color::Rgb(25, 105, 145),
            (Tab::Actions, ThemeMode::Dark) => Color::Rgb(120, 200, 140),
            (Tab::Actions, ThemeMode::Light) => Color::Rgb(40, 130, 70),
            (Tab::MouseLab, ThemeMode::Dark) => Color::Rgb(190, 145, 235),
            (Tab::MouseLab, ThemeMode::Light) => Color::Rgb(110, 65, 165),
            (Tab::Effects, ThemeMode::Dark) => Color::Rgb(235, 190, 110),
            (Tab::Effects, ThemeMode::Light) => Color::Rgb(145, 88, 20),
            (Tab::Themes, ThemeMode::Dark) => Color::Rgb(235, 145, 190),
            (Tab::Themes, ThemeMode::Light) => Color::Rgb(150, 45, 95),
            (Tab::Credits, ThemeMode::Dark) => Color::Rgb(235, 235, 235),
            (Tab::Credits, ThemeMode::Light) => Color::Rgb(20, 20, 20),
        }
    }

    fn card_title_line(&self, label: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled("╣ ", self.border_style()),
            Span::styled(format!("{} ", self.tab.glyph()), self.accent_style()),
            Span::styled(
                label.to_uppercase(),
                self.accent_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ╠", self.border_style()),
        ])
    }

    fn dialog_theme(&self) -> DialogTheme {
        DialogTheme {
            panel: self.surface_style(),
            border: self.border_style(),
            title: self.accent_style(),
            label: self.muted_style().add_modifier(Modifier::BOLD),
            input: self.accent_style(),
            placeholder: self.dim_style(),
            help: self.muted_style(),
            primary_button: self.selection_style(),
            secondary_button: self.muted_style(),
        }
    }

    fn dim_style(&self) -> Style {
        Style::new().fg(match self.theme.mode() {
            ThemeMode::Dark => Color::Rgb(100, 104, 124),
            ThemeMode::Light => Color::Rgb(120, 120, 126),
        })
    }

    fn render_widget_with_effect<W>(
        &mut self,
        frame: &mut Frame<'_>,
        key: &'static str,
        area: Rect,
        widget: W,
        effect: Option<DemoEffect>,
    ) where
        W: ratatui::widgets::Widget,
    {
        #[cfg(not(feature = "tachyonfx"))]
        let _ = (key, effect);

        #[cfg(feature = "tachyonfx")]
        if self.effects_enabled {
            self.effects.render_widget(frame, key, area, widget, effect);
            return;
        }

        frame.render_widget(widget, area);
    }

    #[cfg(feature = "tachyonfx")]
    fn showcase_effect(&self) -> Option<DemoEffect> {
        let effect = match self.effect_choice {
            EffectChoice::Fade => ComponentEffect::fade_from_fg(self.tab_accent_color(), 700),
            EffectChoice::Dissolve => {
                ComponentEffect::dissolve(900).pattern(ComponentEffectPattern::RadialCenter {
                    transition_width: 3.0,
                })
            }
            EffectChoice::Coalesce => ComponentEffect::coalesce(900)
                .pattern(ComponentEffectPattern::Coalesce)
                .filter(ComponentEffectFilter::Text),
            EffectChoice::Sweep => {
                ComponentEffect::sweep_in(Motion::LeftToRight, 8, self.effect_backdrop_color(), 900)
                    .pattern(ComponentEffectPattern::Sweep {
                        direction: Motion::LeftToRight,
                        gradient_length: 8,
                    })
            }
            EffectChoice::Slide => {
                ComponentEffect::slide_in(Motion::LeftToRight, 8, self.effect_backdrop_color(), 900)
            }
            EffectChoice::Pulse => ComponentEffect::pulse_fg(self.tab_accent_color(), 900)
                .repeat(EffectRepeat::PingPong),
        };
        Some(effect.filter(ComponentEffectFilter::Text))
    }

    #[cfg(not(feature = "tachyonfx"))]
    fn showcase_effect(&self) -> Option<DemoEffect> {
        None
    }

    #[cfg(feature = "tachyonfx")]
    fn pattern_effect(&self) -> Option<DemoEffect> {
        let pattern = match self.effect_choice {
            EffectChoice::Fade => ComponentEffectPattern::Diagonal {
                direction: DiagonalDirection::TopLeftToBottomRight,
                transition_width: 2.0,
            },
            EffectChoice::Dissolve => ComponentEffectPattern::Dissolve,
            EffectChoice::Coalesce => ComponentEffectPattern::Coalesce,
            EffectChoice::Sweep => ComponentEffectPattern::Sweep {
                direction: Motion::LeftToRight,
                gradient_length: 8,
            },
            EffectChoice::Slide | EffectChoice::Pulse => ComponentEffectPattern::Checkerboard {
                cell_size: 2,
                transition_width: 2.0,
            },
        };
        Some(
            ComponentEffect::pulse_fg(self.tab_accent_color(), 1400)
                .interpolation(Interpolation::SineInOut)
                .pattern(pattern)
                .filter(ComponentEffectFilter::Text),
        )
    }

    #[cfg(not(feature = "tachyonfx"))]
    fn pattern_effect(&self) -> Option<DemoEffect> {
        None
    }

    fn render_with_optional_effect<W>(
        &mut self,
        frame: &mut Frame<'_>,
        key: &'static str,
        area: Rect,
        widget: W,
    ) where
        W: ratatui::widgets::Widget,
    {
        #[cfg(not(feature = "tachyonfx"))]
        let _ = key;

        #[cfg(feature = "tachyonfx")]
        if self.effects_enabled {
            self.effects.render_widget(
                frame,
                key,
                area,
                widget,
                Some(ComponentEffect::dissolve(500).pattern(
                    ComponentEffectPattern::RadialCenter {
                        transition_width: 3.0,
                    },
                )),
            );
            return;
        }

        frame.render_widget(widget, area);
    }

    fn hovered_tab_index(&self) -> Option<usize> {
        self.hover
            .and_then(|position| inline_item_index_at(self.tab_area, position, tab_widths()))
    }

    fn hovered_action_index(&self) -> Option<usize> {
        self.hover
            .and_then(|position| row_index_at(self.action_area, position, 1, self.actions.len()))
    }

    fn hovered_effect_index(&self) -> Option<usize> {
        (self.tab == Tab::Effects).then_some(()).and_then(|()| {
            self.hover.and_then(|position| {
                row_index_at(self.effect_area, position, 0, EffectChoice::ALL.len())
            })
        })
    }

    fn hovered_theme_index(&self) -> Option<usize> {
        self.hover
            .and_then(|position| inline_item_index_at(self.theme_area, position, theme_widths()))
    }

    fn hovered_segment_index(&self) -> Option<usize> {
        self.hover.and_then(|position| {
            inline_item_index_at(self.segment_area, position, segment_widths())
        })
    }

    fn theme_line(&self) -> Paragraph<'static> {
        let hover = self.hovered_theme_index();
        Paragraph::new(Line::from(
            DemoTheme::ALL
                .into_iter()
                .enumerate()
                .map(|(index, theme)| {
                    if theme == self.theme_choice {
                        Span::styled(format!(" {} ", theme.label()), self.selection_style())
                    } else if hover == Some(index) {
                        Span::styled(format!(" {} ", theme.label()), self.hover_style())
                    } else {
                        Span::styled(format!(" {} ", theme.label()), self.muted_style())
                    }
                })
                .collect::<Vec<_>>(),
        ))
        .alignment(Alignment::Left)
    }

    fn segment_line(&self) -> Paragraph<'static> {
        let hover = self.hovered_segment_index();
        Paragraph::new(Line::from(
            Segment::ALL
                .into_iter()
                .enumerate()
                .map(|(index, segment)| {
                    if segment == self.segment {
                        Span::styled(format!(" {} ", segment.label()), self.selection_style())
                    } else if hover == Some(index) {
                        Span::styled(format!(" {} ", segment.label()), self.hover_style())
                    } else {
                        Span::styled(format!(" {} ", segment.label()), self.muted_style())
                    }
                })
                .collect::<Vec<_>>(),
        ))
    }
}

fn tab_widths() -> impl Iterator<Item = u16> {
    Tab::ALL
        .into_iter()
        .map(|tab| cell_width(tab.label()) + cell_width(tab.glyph()) + 3)
}

fn theme_widths() -> impl Iterator<Item = u16> {
    DemoTheme::ALL
        .into_iter()
        .map(|theme| cell_width(theme.label()) + 2)
}

fn segment_widths() -> impl Iterator<Item = u16> {
    Segment::ALL
        .into_iter()
        .map(|segment| cell_width(segment.label()) + 2)
}

fn cell_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn bar_bg(mode: ThemeMode) -> Color {
    match mode {
        ThemeMode::Dark => Color::Rgb(38, 36, 47),
        ThemeMode::Light => Color::Rgb(227, 223, 231),
    }
}

fn selection_bg(mode: ThemeMode) -> Color {
    match mode {
        ThemeMode::Dark => Color::Rgb(54, 49, 68),
        ThemeMode::Light => Color::Rgb(232, 225, 237),
    }
}

fn progress_fill(tab: Tab, mode: ThemeMode) -> Color {
    match (tab, mode) {
        (Tab::Overview | Tab::Credits, ThemeMode::Dark) => Color::Rgb(82, 88, 150),
        (Tab::Components | Tab::Themes, ThemeMode::Dark) => Color::Rgb(42, 104, 132),
        (Tab::Actions, ThemeMode::Dark) => Color::Rgb(42, 112, 72),
        (Tab::MouseLab, ThemeMode::Dark) => Color::Rgb(92, 58, 132),
        (Tab::Effects, ThemeMode::Dark) => Color::Rgb(122, 78, 28),
        (_, ThemeMode::Light) => selection_bg(ThemeMode::Light),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn focused_components_app() -> App {
        let mut app = App::new(false);
        app.tab = Tab::Components;
        app.focus = NavigationFocus::Body;
        app.input_value.clear();
        app
    }

    #[test]
    fn focused_components_input_accepts_text_and_backspace() {
        let mut app = focused_components_app();

        app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('x'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Backspace, KeyModifiers::NONE);

        assert_eq!(app.input_value, "q");
        assert!(!app.quit);
    }

    #[test]
    fn focused_components_input_escape_blurs() {
        let mut app = focused_components_app();

        app.handle_key(KeyCode::Esc, KeyModifiers::NONE);

        assert_eq!(app.focus, NavigationFocus::Header);
        assert_eq!(app.status.message(), "input blurred");
        assert!(!app.quit);
    }

    #[test]
    fn focused_components_input_keeps_ctrl_c_as_global_quit() {
        let mut app = focused_components_app();

        app.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert!(app.quit);
        assert!(app.input_value.is_empty());
    }
}
