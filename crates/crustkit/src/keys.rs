use ratatui::{
    style::Style,
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyHint {
    pub key: &'static str,
    pub action: &'static str,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_key_hint_line() {
        let line = key_hints_line([KeyHint::new("q", "quit"), KeyHint::new("Enter", "open")]);

        assert_eq!(line.width(), "q quit  Enter open".len());
    }
}
