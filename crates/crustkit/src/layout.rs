use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn body_split(
    area: Rect,
    compact_below: u16,
    left_percent: u16,
    left_height: u16,
) -> [Rect; 2] {
    if area.width < compact_below {
        let chunks = Layout::vertical([Constraint::Length(left_height), Constraint::Fill(1)])
            .spacing(1)
            .split(area);
        [chunks[0], chunks[1]]
    } else {
        let chunks = Layout::horizontal([
            Constraint::Percentage(left_percent),
            Constraint::Percentage(100 - left_percent),
        ])
        .spacing(2)
        .split(area);
        [chunks[0], chunks[1]]
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_split_switches_to_vertical_when_narrow() {
        let chunks = body_split(Rect::new(0, 0, 80, 30), 96, 36, 12);

        assert_eq!(chunks[0].height, 12);
        assert_eq!(chunks[1].y, 13);
    }
}
