use crossterm::event::KeyCode;
use ratatui::{
    style::Style,
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyHint {
    pub key: &'static str,
    pub action: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationFocus {
    Header,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationCommand {
    None,
    PreviousTab,
    NextTab,
    EnterBody,
    ExitBody,
    PreviousItem,
    NextItem,
    Activate,
}

impl KeyHint {
    pub const fn new(key: &'static str, action: &'static str) -> Self {
        Self { key, action }
    }
}

pub fn key_hints_line<'a, I>(hints: I) -> Line<'a>
where
    I: IntoIterator<Item = KeyHint>,
{
    let mut spans = Vec::new();
    for hint in hints {
        if !spans.is_empty() {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(hint.key, Style::new().bold()));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(hint.action));
    }
    Line::from(spans)
}

/// Map a number key to a tab index, Chrome-style: `1`–`8` select the first
/// eight tabs and `9` always jumps to the last tab, regardless of how many tabs
/// there are. Returns `None` for any other key, when `tab_count` is zero, or
/// when the requested digit exceeds the available tabs.
///
/// This is opt-in and intentionally modifier-agnostic (terminals rarely deliver
/// Cmd/Super). Call it from your key handler **only when a text input is not
/// focused** — otherwise digits typed into a field would switch tabs:
///
/// ```
/// # use crossterm::event::KeyCode;
/// # use crustkit::tab_hotkey;
/// # let editing = false;
/// # let tab_count = 4;
/// # let code = KeyCode::Char('9');
/// if !editing {
///     if let Some(index) = tab_hotkey(code, tab_count) {
///         // switch to `index`
///         # assert_eq!(index, 3);
///     }
/// }
/// ```
pub fn tab_hotkey(code: KeyCode, tab_count: usize) -> Option<usize> {
    if tab_count == 0 {
        return None;
    }
    match code {
        KeyCode::Char('9') => Some(tab_count - 1),
        KeyCode::Char(c @ '1'..='8') => {
            let index = (c as usize) - ('1' as usize);
            (index < tab_count).then_some(index)
        }
        _ => None,
    }
}

/// Convert common terminal navigation keys into a header/body focus command.
///
/// This matches a common header/body interaction pattern:
/// horizontal navigation switches tabs, Down/Enter drops from the header into
/// the body, Up on the first body item returns to the header, and Esc walks
/// focus back to the header before the app decides whether to quit.
pub fn navigation_command(
    code: KeyCode,
    focus: NavigationFocus,
    at_first_item: bool,
) -> NavigationCommand {
    match code {
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => NavigationCommand::NextTab,
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => NavigationCommand::PreviousTab,
        KeyCode::Down | KeyCode::Char('j') => match focus {
            NavigationFocus::Header => NavigationCommand::EnterBody,
            NavigationFocus::Body => NavigationCommand::NextItem,
        },
        KeyCode::Up | KeyCode::Char('k') => {
            if focus == NavigationFocus::Body && at_first_item {
                NavigationCommand::ExitBody
            } else if focus == NavigationFocus::Body {
                NavigationCommand::PreviousItem
            } else {
                NavigationCommand::None
            }
        }
        KeyCode::Enter => match focus {
            NavigationFocus::Header => NavigationCommand::EnterBody,
            NavigationFocus::Body => NavigationCommand::Activate,
        },
        KeyCode::Esc => match focus {
            NavigationFocus::Header => NavigationCommand::None,
            NavigationFocus::Body => NavigationCommand::ExitBody,
        },
        _ => NavigationCommand::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_key_hint_line() {
        let line = key_hints_line([KeyHint::new("q", "quit"), KeyHint::new("Enter", "open")]);

        assert_eq!(line.width(), "q quit  Enter open".len());
    }

    #[test]
    fn tab_hotkey_maps_digits_chrome_style() {
        // 1-based digits select the matching tab.
        assert_eq!(tab_hotkey(KeyCode::Char('1'), 4), Some(0));
        assert_eq!(tab_hotkey(KeyCode::Char('4'), 4), Some(3));
        // 9 always jumps to the last tab, even with fewer than nine tabs.
        assert_eq!(tab_hotkey(KeyCode::Char('9'), 4), Some(3));
        // Digits past the end and non-digits are ignored.
        assert_eq!(tab_hotkey(KeyCode::Char('5'), 4), None);
        assert_eq!(tab_hotkey(KeyCode::Char('a'), 4), None);
        assert_eq!(tab_hotkey(KeyCode::Char('1'), 0), None);
    }

    #[test]
    fn navigation_enters_and_exits_body_from_header() {
        assert_eq!(
            navigation_command(KeyCode::Down, NavigationFocus::Header, true),
            NavigationCommand::EnterBody
        );
        assert_eq!(
            navigation_command(KeyCode::Up, NavigationFocus::Body, true),
            NavigationCommand::ExitBody
        );
        assert_eq!(
            navigation_command(KeyCode::Esc, NavigationFocus::Body, false),
            NavigationCommand::ExitBody
        );
        assert_eq!(
            navigation_command(KeyCode::Esc, NavigationFocus::Header, false),
            NavigationCommand::None
        );
    }

    #[test]
    fn navigation_moves_inside_body_after_first_item() {
        assert_eq!(
            navigation_command(KeyCode::Down, NavigationFocus::Body, false),
            NavigationCommand::NextItem
        );
        assert_eq!(
            navigation_command(KeyCode::Up, NavigationFocus::Body, false),
            NavigationCommand::PreviousItem
        );
    }
}
