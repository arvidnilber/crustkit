use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseClick {
    pub column: u16,
    pub row: u16,
}

pub fn left_mouse_click(event: MouseEvent) -> Option<MouseClick> {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => Some(MouseClick {
            column: event.column,
            row: event.row,
        }),
        _ => None,
    }
}

pub fn mouse_position(event: MouseEvent) -> MouseClick {
    MouseClick {
        column: event.column,
        row: event.row,
    }
}

pub fn rect_contains(area: Rect, click: MouseClick) -> bool {
    click.column >= area.x
        && click.column < area.x.saturating_add(area.width)
        && click.row >= area.y
        && click.row < area.y.saturating_add(area.height)
}

pub fn row_index_at(
    area: Rect,
    click: MouseClick,
    first_row_offset: u16,
    row_count: usize,
) -> Option<usize> {
    if !rect_contains(area, click) {
        return None;
    }

    let first_row = area.y.saturating_add(first_row_offset);
    if click.row < first_row {
        return None;
    }

    let index = usize::from(click.row - first_row);
    let visible_rows = usize::from(
        area.height
            .saturating_sub(first_row_offset.saturating_add(1)),
    );
    (index < row_count && index < visible_rows).then_some(index)
}

pub fn inline_item_index_at(
    area: Rect,
    click: MouseClick,
    widths: impl IntoIterator<Item = u16>,
) -> Option<usize> {
    if click.row != area.y || click.column < area.x {
        return None;
    }

    let mut column = area.x;
    let max_column = area.x.saturating_add(area.width);
    for (index, width) in widths.into_iter().enumerate() {
        let next_column = column.saturating_add(width);
        if click.column >= column && click.column < next_column.min(max_column) {
            return Some(index);
        }
        column = next_column;
        if column >= max_column {
            return None;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_index_accounts_for_widget_chrome() {
        let area = Rect::new(4, 10, 40, 8);

        assert_eq!(
            row_index_at(area, MouseClick { column: 6, row: 12 }, 2, 3),
            Some(0)
        );
        assert_eq!(
            row_index_at(area, MouseClick { column: 6, row: 15 }, 2, 3),
            None
        );
    }

    #[test]
    fn inline_item_index_uses_declared_widths() {
        let area = Rect::new(10, 2, 30, 2);
        let widths = [8, 10, 12];

        assert_eq!(
            inline_item_index_at(area, MouseClick { column: 20, row: 2 }, widths),
            Some(1)
        );
        assert_eq!(
            inline_item_index_at(area, MouseClick { column: 20, row: 3 }, widths),
            None
        );
    }
}
