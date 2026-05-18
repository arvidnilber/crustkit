use std::{env, time::Duration};

use color_eyre::{
    Result,
    eyre::{Report, eyre},
};
use ratatui::style::Color;

use crate::theme::ThemeMode;

/// Terminal background color reported by OSC 11.
///
/// Terminals commonly report 16-bit channels, though some report 8-bit values
/// that `termbg` expands into this shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalBackgroundColor {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl TerminalBackgroundColor {
    pub const fn new(r: u16, g: u16, b: u16) -> Self {
        Self { r, g, b }
    }

    /// Convert the 16-bit terminal color into a Ratatui 8-bit RGB color.
    pub fn ratatui_color(self) -> Color {
        Color::Rgb(
            u16_channel_to_u8(self.r),
            u16_channel_to_u8(self.g),
            u16_channel_to_u8(self.b),
        )
    }

    /// Return perceived luminance in the `0.0..=1.0` range.
    pub fn perceived_luminance(self) -> f64 {
        let r = f64::from(self.r) / f64::from(u16::MAX);
        let g = f64::from(self.g) / f64::from(u16::MAX);
        let b = f64::from(self.b) / f64::from(u16::MAX);

        (0.2126 * r) + (0.7152 * g) + (0.0722 * b)
    }

    /// Bucket the color into Crustkit's light/dark theme modes.
    pub fn theme_mode(self) -> ThemeMode {
        if self.perceived_luminance() >= 0.5 {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        }
    }
}

/// Query the terminal for its current background color.
///
/// Returns `Ok(None)` when `NO_COLOR` is set. Otherwise this sends an OSC 11
/// query through `termbg`, which handles raw mode, timeouts, TTY checks, tmux,
/// and terminal response parsing.
pub fn detect_terminal_background_color(
    timeout: Duration,
) -> Result<Option<TerminalBackgroundColor>> {
    if no_color_enabled() {
        return Ok(None);
    }

    let rgb = termbg::rgb(timeout)
        .map_err(|err| termbg_report("could not detect terminal background color", err))?;

    Ok(Some(TerminalBackgroundColor::new(rgb.r, rgb.g, rgb.b)))
}

/// Detect whether the terminal background is light or dark.
///
/// Returns `Ok(None)` when `NO_COLOR` is set. If the OSC 11 query fails, this
/// falls back to the `COLORFGBG` environment variable when it contains a
/// standard 16-color background code.
pub fn detect_terminal_theme_mode(timeout: Duration) -> Result<Option<ThemeMode>> {
    if no_color_enabled() {
        return Ok(None);
    }

    match termbg::theme(timeout) {
        Ok(termbg::Theme::Dark) => Ok(Some(ThemeMode::Dark)),
        Ok(termbg::Theme::Light) => Ok(Some(ThemeMode::Light)),
        Err(err) => terminal_theme_mode_from_colorfgbg_env()
            .map(Some)
            .ok_or_else(|| termbg_report("could not detect terminal background theme", err)),
    }
}

/// Detect the terminal theme mode, returning `fallback` on timeout, unsupported
/// terminals, or `NO_COLOR`.
pub fn terminal_theme_mode_or(timeout: Duration, fallback: ThemeMode) -> ThemeMode {
    detect_terminal_theme_mode(timeout)
        .ok()
        .flatten()
        .unwrap_or(fallback)
}

/// Parse a `COLORFGBG` value into a Crustkit theme mode.
///
/// The final semicolon-delimited segment is treated as the background color.
pub fn terminal_theme_mode_from_colorfgbg(value: &str) -> Option<ThemeMode> {
    let background = value.rsplit(';').find(|part| !part.is_empty())?;
    let code = background.parse::<u8>().ok()?;

    match code {
        0..=6 | 8 => Some(ThemeMode::Dark),
        7 | 9..=15 => Some(ThemeMode::Light),
        _ => None,
    }
}

fn terminal_theme_mode_from_colorfgbg_env() -> Option<ThemeMode> {
    let value = env::var("COLORFGBG").ok()?;
    terminal_theme_mode_from_colorfgbg(&value)
}

fn no_color_enabled() -> bool {
    env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
}

fn termbg_report(context: &str, err: impl std::fmt::Display) -> Report {
    eyre!("{context}: {err}")
}

fn u16_channel_to_u8(value: u16) -> u8 {
    (u32::from(value) * u32::from(u8::MAX) / u32::from(u16::MAX)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorfgbg_detects_dark_standard_backgrounds() {
        assert_eq!(
            terminal_theme_mode_from_colorfgbg("15;0"),
            Some(ThemeMode::Dark)
        );
        assert_eq!(
            terminal_theme_mode_from_colorfgbg("7;8"),
            Some(ThemeMode::Dark)
        );
    }

    #[test]
    fn colorfgbg_detects_light_standard_backgrounds() {
        assert_eq!(
            terminal_theme_mode_from_colorfgbg("0;15"),
            Some(ThemeMode::Light)
        );
        assert_eq!(
            terminal_theme_mode_from_colorfgbg("0;9"),
            Some(ThemeMode::Light)
        );
    }

    #[test]
    fn colorfgbg_ignores_unknown_backgrounds() {
        assert_eq!(terminal_theme_mode_from_colorfgbg("0;255"), None);
        assert_eq!(terminal_theme_mode_from_colorfgbg("default"), None);
    }

    #[test]
    fn background_color_converts_to_ratatui_rgb() {
        let color = TerminalBackgroundColor::new(0x0101, 0x8080, u16::MAX);

        assert_eq!(color.ratatui_color(), Color::Rgb(1, 128, 255));
    }

    #[test]
    fn background_color_infers_theme_mode_from_luminance() {
        assert_eq!(
            TerminalBackgroundColor::new(0, 0, 0).theme_mode(),
            ThemeMode::Dark
        );
        assert_eq!(
            TerminalBackgroundColor::new(u16::MAX, u16::MAX, u16::MAX).theme_mode(),
            ThemeMode::Light
        );
    }
}
