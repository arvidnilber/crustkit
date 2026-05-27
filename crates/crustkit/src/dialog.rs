use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap},
};

#[cfg(feature = "tachyonfx")]
use crate::effects::ComponentEffect;
use crate::input::input_value_spans;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogTheme {
    pub panel: Style,
    pub border: Style,
    pub title: Style,
    pub label: Style,
    pub input: Style,
    pub placeholder: Style,
    pub help: Style,
    pub primary_button: Style,
    pub secondary_button: Style,
}

impl Default for DialogTheme {
    fn default() -> Self {
        Self {
            panel: Style::new(),
            border: Style::new().fg(Color::DarkGray),
            title: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            label: Style::new().add_modifier(Modifier::BOLD),
            input: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            placeholder: Style::new().fg(Color::DarkGray),
            help: Style::new().fg(Color::DarkGray),
            primary_button: Style::new()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            secondary_button: Style::new().fg(Color::DarkGray),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(not(feature = "tachyonfx"), derive(Eq))]
pub struct InputDialog {
    title: String,
    label: String,
    value: String,
    placeholder: String,
    help: Option<String>,
    primary_label: String,
    secondary_label: String,
    width: u16,
    height: u16,
    cursor: bool,
    pending: bool,
    #[cfg(feature = "tachyonfx")]
    effect: Option<ComponentEffect>,
}

impl InputDialog {
    pub fn new(title: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            help: None,
            primary_label: "Save".to_string(),
            secondary_label: "Esc cancel".to_string(),
            width: 52,
            height: 9,
            cursor: true,
            pending: false,
            #[cfg(feature = "tachyonfx")]
            effect: None,
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

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn primary_label(mut self, label: impl Into<String>) -> Self {
        self.primary_label = label.into();
        self
    }

    pub fn secondary_label(mut self, label: impl Into<String>) -> Self {
        self.secondary_label = label.into();
        self
    }

    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn cursor(mut self, enabled: bool) -> Self {
        self.cursor = enabled;
        self
    }

    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    #[cfg(feature = "tachyonfx")]
    pub fn effect(mut self, effect: ComponentEffect) -> Self {
        self.effect = Some(effect);
        self
    }

    #[cfg(feature = "tachyonfx")]
    pub const fn component_effect(&self) -> Option<ComponentEffect> {
        self.effect
    }

    pub fn render(self, frame: &mut Frame<'_>, area: Rect, theme: DialogTheme) {
        let rect = centered_dialog_rect(area, self.width, self.height);
        frame.render_widget(Clear, rect);
        frame.render_widget(self.widget(theme), rect);
    }

    pub fn widget(self, theme: DialogTheme) -> Paragraph<'static> {
        let value_spans = input_value_spans(
            self.value,
            self.placeholder,
            theme.input,
            theme.placeholder,
            theme.input.add_modifier(Modifier::REVERSED),
            self.cursor,
        );

        let mut lines = vec![
            Line::from(
                [
                    vec![Span::styled(self.label, theme.label), Span::raw(" ")],
                    value_spans,
                ]
                .concat(),
            ),
            Line::from(""),
        ];

        if let Some(help) = self.help {
            lines.push(Line::from(Span::styled(help, theme.help)));
            lines.push(Line::from(""));
        }

        let primary_label = if self.pending {
            format!(" {}... ", self.primary_label)
        } else {
            format!(" {} ", self.primary_label)
        };

        lines.push(Line::from(vec![
            Span::styled(primary_label, theme.primary_button),
            Span::raw("  "),
            Span::styled(self.secondary_label, theme.secondary_button),
        ]));

        Paragraph::new(lines)
            .style(theme.panel)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title(self.title)
                    .title_style(theme.title)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border)
                    .padding(Padding::new(1, 1, 0, 0))
                    .style(theme.panel),
            )
    }
}

pub fn centered_dialog_rect(area: Rect, max_width: u16, height: u16) -> Rect {
    let width = max_width.min(area.width.saturating_sub(2)).max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);
    let horizontal_margin = area.width.saturating_sub(width) / 2;
    let vertical_margin = area.height.saturating_sub(height) / 2;

    let vertical = Layout::vertical([
        Constraint::Length(vertical_margin),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Length(horizontal_margin),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_dialog_rect_uses_requested_size_when_it_fits() {
        let rect = centered_dialog_rect(Rect::new(0, 0, 100, 40), 50, 10);

        assert_eq!(rect, Rect::new(25, 15, 50, 10));
    }

    #[test]
    fn input_dialog_builder_sets_value() {
        let dialog = InputDialog::new("Count", "Episodes:")
            .value("25")
            .primary_label("Save");

        assert_eq!(dialog.value, "25");
        assert_eq!(dialog.primary_label, "Save");
    }

    #[test]
    fn input_dialog_cursor_can_be_disabled() {
        let dialog = InputDialog::new("Count", "Episodes:").cursor(false);

        assert!(!dialog.cursor);
    }

    #[test]
    fn input_dialog_pending_can_be_enabled() {
        let dialog = InputDialog::new("Save", "Name:").pending(true);

        assert!(dialog.pending);
    }
}
