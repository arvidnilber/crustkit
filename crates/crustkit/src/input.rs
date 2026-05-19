use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInput {
    title: String,
    label: String,
    value: String,
    placeholder: String,
    focused: bool,
    help: Option<String>,
    cursor: bool,
}

impl TextInput {
    pub fn new(title: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            focused: false,
            help: None,
            cursor: true,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn cursor(mut self, enabled: bool) -> Self {
        self.cursor = enabled;
        self
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn widget(self) -> Paragraph<'static> {
        let border_style = if self.focused {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        let input_style = if self.focused {
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };
        let value_spans = input_value_spans(
            self.value,
            self.placeholder,
            input_style,
            Style::new().fg(Color::DarkGray),
            input_style.add_modifier(Modifier::REVERSED),
            self.focused && self.cursor,
        );

        let mut lines = vec![
            Line::from(
                [
                    vec![
                        Span::styled(self.label, Style::new().bold()),
                        Span::raw(" "),
                    ],
                    value_spans,
                ]
                .concat(),
            ),
            Line::from(""),
        ];

        if let Some(help) = self.help {
            lines.push(Line::from(Span::styled(
                help,
                Style::new().fg(Color::DarkGray),
            )));
        }

        Paragraph::new(lines).block(
            Block::bordered()
                .title(self.title)
                .border_type(BorderType::Rounded)
                .border_style(border_style),
        )
    }
}

pub fn input_value_spans(
    value: String,
    placeholder: String,
    input_style: Style,
    placeholder_style: Style,
    cursor_style: Style,
    show_cursor: bool,
) -> Vec<Span<'static>> {
    let cursor = if show_cursor && cursor_blink_on() {
        Span::styled(" ", cursor_style)
    } else if show_cursor {
        Span::raw(" ")
    } else {
        Span::raw("")
    };

    if value.is_empty() {
        vec![cursor, Span::styled(placeholder, placeholder_style)]
    } else {
        vec![Span::styled(value, input_style), cursor]
    }
}

fn cursor_blink_on() -> bool {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    (millis / 500).is_multiple_of(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_input_with_value() {
        let input = TextInput::new("Search", "Show:")
            .value("The Office")
            .focused(true);

        assert_eq!(input.value, "The Office");
        assert!(input.focused);
    }

    #[test]
    fn cursor_can_be_disabled() {
        let input = TextInput::new("Search", "Show:").cursor(false);

        assert!(!input.cursor);
    }
}
