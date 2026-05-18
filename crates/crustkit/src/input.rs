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
        let shown_value = if self.value.is_empty() {
            Span::styled(self.placeholder, Style::new().fg(Color::DarkGray))
        } else {
            Span::styled(self.value, input_style)
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled(self.label, Style::new().bold()),
                Span::raw(" "),
                shown_value,
            ]),
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
}
