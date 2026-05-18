use std::time::Duration;

use ratatui::{
    style::Style,
    text::Span,
    widgets::{Block, Gauge},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub elapsed: Duration,
}

impl TransferProgress {
    pub const fn new(downloaded_bytes: u64, total_bytes: Option<u64>, elapsed: Duration) -> Self {
        Self {
            downloaded_bytes,
            total_bytes,
            elapsed,
        }
    }

    pub fn ratio(self) -> Option<f64> {
        let total = self.total_bytes?;
        if total == 0 {
            return Some(1.0);
        }
        Some((self.downloaded_bytes as f64 / total as f64).clamp(0.0, 1.0))
    }

    pub fn bytes_per_second(self) -> Option<f64> {
        let elapsed = self.elapsed.as_secs_f64();
        if elapsed <= f64::EPSILON {
            return None;
        }
        Some(self.downloaded_bytes as f64 / elapsed)
    }

    pub fn summary(self) -> String {
        let size = match self.total_bytes {
            Some(total) => format!(
                "{} / {}",
                format_bytes(self.downloaded_bytes),
                format_bytes(total)
            ),
            None => format_bytes(self.downloaded_bytes),
        };
        let speed = self
            .bytes_per_second()
            .map(format_bytes_per_second)
            .unwrap_or_else(|| "-/s".to_string());
        match self.ratio() {
            Some(ratio) => format!("{:>5.1}%  {size}  {speed}", ratio * 100.0),
            None => format!("  ---%  {size}  {speed}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressBarTheme {
    pub base_style: Style,
    pub bar_style: Style,
    pub label_style: Style,
}

impl ProgressBarTheme {
    pub const fn new(base_style: Style, bar_style: Style, label_style: Style) -> Self {
        Self {
            base_style,
            bar_style,
            label_style,
        }
    }
}

pub fn transfer_progress_gauge<'a>(
    progress: TransferProgress,
    theme: ProgressBarTheme,
) -> Gauge<'a> {
    let gauge = Gauge::default()
        .block(Block::default())
        .style(theme.base_style)
        .gauge_style(theme.bar_style)
        .label(Span::styled(progress.summary(), theme.label_style));

    match progress.ratio() {
        Some(ratio) => gauge.ratio(ratio),
        None => gauge.ratio(0.0),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = UNITS[0];
    for candidate in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = candidate;
    }

    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}

pub fn format_bytes_per_second(bytes_per_second: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_second.max(0.0) as u64))
}

#[cfg(test)]
mod tests {
    use super::{TransferProgress, format_bytes};
    use std::time::Duration;

    #[test]
    fn transfer_progress_reports_ratio() {
        let progress = TransferProgress::new(50, Some(200), Duration::from_secs(2));

        assert_eq!(progress.ratio(), Some(0.25));
        assert_eq!(progress.bytes_per_second(), Some(25.0));
    }

    #[test]
    fn transfer_progress_formats_summary_with_known_size() {
        let progress = TransferProgress::new(1024, Some(2048), Duration::from_secs(1));

        assert_eq!(progress.summary(), " 50.0%  1.0 KB / 2.0 KB  1.0 KB/s");
    }

    #[test]
    fn bytes_use_human_units() {
        assert_eq!(format_bytes(42), "42 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }
}
