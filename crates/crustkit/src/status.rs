use ratatui::{
    style::{Color, Style},
    text::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    kind: StatusKind,
    message: String,
}

impl StatusLine {
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(StatusKind::Info, message)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(StatusKind::Success, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(StatusKind::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(StatusKind::Error, message)
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn to_span(&self) -> Span<'_> {
        let style = match self.kind {
            StatusKind::Info => Style::new().dim(),
            StatusKind::Success => Style::new().green(),
            StatusKind::Warning => Style::new().yellow(),
            StatusKind::Error => Style::new().fg(Color::Red),
        };
        Span::styled(self.message.as_str(), style)
    }

    fn new(kind: StatusKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
