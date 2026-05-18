use std::time::Duration;

use color_eyre::Result;
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub mode: ThemeMode,
    pub foreground: Color,
    pub background: Color,
    pub panel_background: Color,
    pub panel_alt_background: Color,
    pub muted_foreground: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_foreground: Color,
    pub selection_foreground: Color,
    pub selection_background: Color,
}

impl ThemePalette {
    pub const fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            foreground: Color::Rgb(235, 235, 235),
            background: Color::Black,
            panel_background: Color::Black,
            panel_alt_background: Color::Black,
            muted_foreground: Color::DarkGray,
            border: Color::DarkGray,
            accent: Color::Cyan,
            accent_foreground: Color::Black,
            selection_foreground: Color::Black,
            selection_background: Color::Cyan,
        }
    }

    pub const fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            foreground: Color::Black,
            background: Color::White,
            panel_background: Color::White,
            panel_alt_background: Color::White,
            muted_foreground: Color::DarkGray,
            border: Color::Gray,
            accent: Color::Blue,
            accent_foreground: Color::White,
            selection_foreground: Color::White,
            selection_background: Color::Blue,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppTheme {
    palette: ThemePalette,
    selection_bold: bool,
}

impl AppTheme {
    pub const fn new(palette: ThemePalette) -> Self {
        Self {
            palette,
            selection_bold: true,
        }
    }

    pub const fn dark() -> Self {
        Self::new(ThemePalette::dark())
    }

    pub const fn light() -> Self {
        Self::new(ThemePalette::light())
    }

    pub const fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    /// Detect a light or dark app theme from the current terminal background.
    pub fn detect(timeout: Duration) -> Result<Option<Self>> {
        crate::terminal_background::detect_terminal_theme_mode(timeout)
            .map(|mode| mode.map(Self::from_mode))
    }

    /// Detect a light or dark app theme, falling back when detection is unavailable.
    pub fn detect_or(timeout: Duration, fallback: ThemeMode) -> Self {
        Self::from_mode(crate::terminal_background::terminal_theme_mode_or(
            timeout, fallback,
        ))
    }

    pub const fn mode(self) -> ThemeMode {
        self.palette.mode
    }

    pub const fn palette(self) -> ThemePalette {
        self.palette
    }

    pub const fn with_selection_bold(mut self, selection_bold: bool) -> Self {
        self.selection_bold = selection_bold;
        self
    }

    pub const fn selection_theme(self) -> SelectionTheme {
        SelectionTheme::new(
            self.palette.selection_foreground,
            self.palette.selection_background,
        )
        .with_bold(self.selection_bold)
    }

    pub fn base_style(self) -> Style {
        Style::new()
            .fg(self.palette.foreground)
            .bg(self.palette.background)
    }

    pub fn panel_style(self) -> Style {
        Style::new()
            .fg(self.palette.foreground)
            .bg(self.palette.panel_background)
    }

    pub fn panel_alt_style(self) -> Style {
        Style::new()
            .fg(self.palette.foreground)
            .bg(self.palette.panel_alt_background)
    }

    pub fn muted_style(self) -> Style {
        Style::new().fg(self.palette.muted_foreground)
    }

    pub fn accent_style(self) -> Style {
        Style::new()
            .fg(self.palette.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(self) -> Style {
        Style::new().fg(self.palette.border)
    }

    pub fn highlight_style(self) -> Style {
        self.selection_theme().style()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionTheme {
    foreground: Color,
    background: Color,
    bold: bool,
}

impl SelectionTheme {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self {
            foreground,
            background,
            bold: true,
        }
    }

    pub const fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn style(self) -> Style {
        let style = Style::new().fg(self.foreground).bg(self.background);
        if self.bold {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }
}

impl Default for SelectionTheme {
    fn default() -> Self {
        AppTheme::dark().selection_theme()
    }
}

#[cfg(test)]
mod tests {
    use super::{AppTheme, SelectionTheme, ThemeMode};
    use ratatui::style::{Color, Modifier};

    #[test]
    fn selection_theme_default_matches_dark_theme() {
        assert_eq!(
            SelectionTheme::default(),
            AppTheme::dark().selection_theme()
        );
    }

    #[test]
    fn app_theme_from_mode_uses_expected_palette_modes() {
        assert_eq!(AppTheme::from_mode(ThemeMode::Dark).mode(), ThemeMode::Dark);
        assert_eq!(
            AppTheme::from_mode(ThemeMode::Light).mode(),
            ThemeMode::Light
        );
    }

    #[test]
    fn selection_style_bold_is_configurable() {
        let bold_style = AppTheme::dark().selection_theme().style();
        assert!(bold_style.add_modifier.contains(Modifier::BOLD));

        let non_bold_style = AppTheme::light()
            .with_selection_bold(false)
            .selection_theme()
            .style();
        assert!(!non_bold_style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn light_and_dark_selection_colors_differ() {
        let dark_style = AppTheme::dark().selection_theme().style();
        let light_style = AppTheme::light().selection_theme().style();

        assert_eq!(dark_style.fg, Some(Color::Black));
        assert_eq!(dark_style.bg, Some(Color::Cyan));
        assert_eq!(light_style.fg, Some(Color::White));
        assert_eq!(light_style.bg, Some(Color::Blue));
    }
}
