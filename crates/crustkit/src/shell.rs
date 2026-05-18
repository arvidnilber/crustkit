use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{KeyHint, StatusLine, key_hints_line};

pub fn header<'a>(
    title: impl Into<Span<'a>>,
    subtitle: Option<impl Into<Span<'a>>>,
) -> Paragraph<'a> {
    let mut spans = vec![
        Span::styled(" ◆ ", Style::new().cyan()),
        title.into().style(Style::new().bold()),
    ];
    if let Some(subtitle) = subtitle {
        spans.push(Span::raw("  "));
        spans.push(subtitle.into().style(Style::new().dim()));
    }

    Paragraph::new(Line::from(spans)).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::DarkGray),
    )
}

pub fn footer<'a, I>(hints: I, status: Option<&'a StatusLine>) -> Paragraph<'a>
where
    I: IntoIterator<Item = KeyHint>,
{
    let mut spans = key_hints_line(hints).spans;
    if let Some(status) = status {
        if !spans.is_empty() {
            spans.push(Span::raw("  "));
        }
        spans.push(status.to_span());
    }

    Paragraph::new(Line::from(spans))
}
