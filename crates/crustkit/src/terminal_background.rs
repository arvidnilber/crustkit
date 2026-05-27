use std::{
    env,
    io::{self, IsTerminal, Write},
    time::{Duration, Instant},
};

use color_eyre::{
    Result,
    eyre::{Report, eyre},
};
use crossterm::{
    event::{Event, KeyCode, KeyModifiers, poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
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
/// query with raw mode, timeout, tmux/screen passthrough, and response parsing.
pub fn detect_terminal_background_color(
    timeout: Duration,
) -> Result<Option<TerminalBackgroundColor>> {
    if no_color_enabled() {
        return Ok(None);
    }

    match query_terminal_background_color(timeout) {
        Ok(color) => Ok(Some(color)),
        Err(err) => terminal_background_color_from_colorfgbg_env()
            .map(Some)
            .ok_or_else(|| {
                terminal_background_report("could not detect terminal background color", err)
            }),
    }
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

    match query_terminal_background_color(timeout) {
        Ok(color) => Ok(Some(color.theme_mode())),
        Err(err) => terminal_theme_mode_from_colorfgbg_env()
            .map(Some)
            .ok_or_else(|| {
                terminal_background_report("could not detect terminal background theme", err)
            }),
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

fn terminal_background_color_from_colorfgbg_env() -> Option<TerminalBackgroundColor> {
    let value = env::var("COLORFGBG").ok()?;
    let background = value.rsplit(';').find(|part| !part.is_empty())?;
    let code = background.parse::<u8>().ok()?;
    let (r, g, b) = ansi_color_rgb(code)?;
    Some(TerminalBackgroundColor::new(
        u8_channel_to_u16(r),
        u8_channel_to_u16(g),
        u8_channel_to_u16(b),
    ))
}

fn no_color_enabled() -> bool {
    env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
}

fn terminal_background_report(context: &str, err: impl std::fmt::Display) -> Report {
    eyre!("{context}: {err}")
}

fn u16_channel_to_u8(value: u16) -> u8 {
    (u32::from(value) * u32::from(u8::MAX) / u32::from(u16::MAX)) as u8
}

fn u8_channel_to_u16(value: u8) -> u16 {
    u16::from(value) * 257
}

fn query_terminal_background_color(timeout: Duration) -> Result<TerminalBackgroundColor> {
    if env::var_os("INSIDE_EMACS").is_some()
        || !io::stdin().is_terminal()
        || !io::stdout().is_terminal()
        || !io::stderr().is_terminal()
    {
        return Err(eyre!("terminal background query is unsupported"));
    }

    let raw_before = is_raw_mode_enabled()?;
    if !raw_before {
        enable_raw_mode()?;
    }

    let result = write_background_query()
        .and_then(|()| read_background_response(timeout))
        .and_then(|response| parse_terminal_background_response(&response));

    restore_raw_mode(raw_before)?;
    result
}

fn write_background_query() -> Result<()> {
    let query = if env::var_os("TMUX").is_some()
        || env::var("TERM").is_ok_and(|term| term.starts_with("tmux-"))
    {
        "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\"
    } else if env::var("TERM").is_ok_and(|term| term.starts_with("screen")) {
        "\x1bP\x1b]11;?\x07\x1b\\"
    } else {
        "\x1b]11;?\x1b\\"
    };

    let mut stderr = io::stderr();
    stderr.write_all(query.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

fn read_background_response(timeout: Duration) -> Result<String> {
    let start = Instant::now();
    let mut response = String::new();

    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return if response.contains("rgb:") {
                Ok(response)
            } else {
                Err(eyre!("terminal background query timed out"))
            };
        }

        if poll(
            timeout
                .saturating_sub(elapsed)
                .min(Duration::from_millis(25)),
        )? && let Event::Key(key) = read()?
        {
            match (key.code, key.modifiers) {
                (KeyCode::Char('\\'), KeyModifiers::ALT | KeyModifiers::NONE)
                | (KeyCode::Char('g'), KeyModifiers::CONTROL)
                | (KeyCode::Char('\u{0007}'), KeyModifiers::NONE) => return Ok(response),
                (KeyCode::Char(ch), KeyModifiers::NONE) => response.push(ch),
                _ => {}
            }
        }
    }
}

fn restore_raw_mode(raw_before: bool) -> Result<()> {
    let raw_now = is_raw_mode_enabled()?;
    if raw_now == raw_before {
        return Ok(());
    }
    if raw_before {
        enable_raw_mode()?;
    } else {
        disable_raw_mode()?;
    }
    Ok(())
}

fn parse_terminal_background_response(response: &str) -> Result<TerminalBackgroundColor> {
    let rgb = response
        .split_once("rgb:")
        .map(|(_, rgb)| rgb)
        .ok_or_else(|| eyre!("missing rgb marker in terminal background response"))?;
    let mut channels = rgb.split('/');
    let r = parse_x11_channel(
        channels
            .next()
            .ok_or_else(|| eyre!("missing red channel in terminal background response"))?,
    )?;
    let g = parse_x11_channel(
        channels
            .next()
            .ok_or_else(|| eyre!("missing green channel in terminal background response"))?,
    )?;
    let b = parse_x11_channel(
        channels
            .next()
            .ok_or_else(|| eyre!("missing blue channel in terminal background response"))?,
    )?;

    Ok(TerminalBackgroundColor::new(r, g, b))
}

fn parse_x11_channel(raw: &str) -> Result<u16> {
    let hex: String = raw
        .chars()
        .take_while(|ch| ch.is_ascii_hexdigit())
        .take(4)
        .collect();
    if hex.is_empty() {
        return Err(eyre!("empty x11 color channel"));
    }

    let mut value = u16::from_str_radix(&hex, 16)?;
    value <<= (4 - hex.len()) * 4;
    Ok(value)
}

fn ansi_color_rgb(code: u8) -> Option<(u8, u8, u8)> {
    match code {
        0 => Some((0, 0, 0)),
        1 => Some((205, 0, 0)),
        2 => Some((0, 205, 0)),
        3 => Some((205, 205, 0)),
        4 => Some((0, 0, 238)),
        5 => Some((205, 0, 205)),
        6 => Some((0, 205, 205)),
        7 => Some((229, 229, 229)),
        8 => Some((127, 127, 127)),
        9 => Some((255, 0, 0)),
        10 => Some((0, 255, 0)),
        11 => Some((255, 255, 0)),
        12 => Some((92, 92, 255)),
        13 => Some((255, 0, 255)),
        14 => Some((0, 255, 255)),
        15 => Some((255, 255, 255)),
        _ => None,
    }
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

    #[test]
    fn parses_xterm_background_response() {
        let color = parse_terminal_background_response("\u{1b}]11;rgb:0101/8080/ffff")
            .expect("background color");

        assert_eq!(color, TerminalBackgroundColor::new(0x0101, 0x8080, 0xffff));
    }

    #[test]
    fn parses_short_x11_channels() {
        let color = parse_terminal_background_response("rgb:f/80/abc").expect("background color");

        assert_eq!(color, TerminalBackgroundColor::new(0xf000, 0x8000, 0xabc0));
    }
}
