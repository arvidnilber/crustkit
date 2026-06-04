use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::{mouse_position, rect_contains};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarResizeAxis {
    Columns,
    Rows,
}

impl SidebarResizeAxis {
    #[must_use]
    pub const fn unit(self) -> &'static str {
        match self {
            Self::Columns => "cols",
            Self::Rows => "rows",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizableSidebar {
    width: u16,
    min_width: u16,
    max_width: u16,
    min_content_width: u16,
    min_content_height: u16,
    min_compact_height: u16,
    compact_below: u16,
    compact_height: u16,
    handle_width: u16,
    hit_slop: u16,
    drag_axis: Option<SidebarResizeAxis>,
}

impl ResizableSidebar {
    #[must_use]
    pub const fn new(width: u16, min_width: u16, max_width: u16) -> Self {
        Self {
            width,
            min_width,
            max_width,
            min_content_width: 20,
            min_content_height: 6,
            min_compact_height: 3,
            compact_below: 80,
            compact_height: 10,
            handle_width: 1,
            hit_slop: 1,
            drag_axis: None,
        }
    }

    #[must_use]
    pub const fn with_min_content_width(mut self, width: u16) -> Self {
        self.min_content_width = width;
        self
    }

    #[must_use]
    pub const fn with_min_content_height(mut self, height: u16) -> Self {
        self.min_content_height = height;
        self
    }

    #[must_use]
    pub const fn with_min_compact_height(mut self, height: u16) -> Self {
        self.min_compact_height = height;
        self
    }

    #[must_use]
    pub const fn with_compact(mut self, compact_below: u16, compact_height: u16) -> Self {
        self.compact_below = compact_below;
        self.compact_height = compact_height;
        self
    }

    #[must_use]
    pub const fn with_handle_width(mut self, width: u16) -> Self {
        self.handle_width = width;
        self
    }

    #[must_use]
    pub const fn with_hit_slop(mut self, columns: u16) -> Self {
        self.hit_slop = columns;
        self
    }

    #[must_use]
    pub const fn width(self) -> u16 {
        self.width
    }

    pub fn set_width(&mut self, width: u16) {
        self.width = width.clamp(self.min_width, self.max_width);
    }

    #[must_use]
    pub const fn compact_height(self) -> u16 {
        self.compact_height
    }

    pub fn set_compact_height(&mut self, height: u16) {
        self.compact_height = height.max(self.min_compact_height);
    }

    #[must_use]
    pub const fn is_dragging(self) -> bool {
        self.drag_axis.is_some()
    }

    #[must_use]
    pub fn layout(self, area: Rect) -> ResizableSidebarLayout {
        if area.width < self.compact_below {
            return self.compact_layout(area);
        }

        let handle_width = self.handle_width.min(area.width);
        let max_sidebar_width = area
            .width
            .saturating_sub(handle_width)
            .saturating_sub(self.min_content_width)
            .min(self.max_width);
        let min_sidebar_width = self.min_width.min(max_sidebar_width);
        let sidebar_width = self.width.clamp(min_sidebar_width, max_sidebar_width);
        let handle_x = area.x.saturating_add(sidebar_width);
        let content_x = handle_x.saturating_add(handle_width);
        let content_width = area
            .width
            .saturating_sub(sidebar_width)
            .saturating_sub(handle_width);

        ResizableSidebarLayout {
            sidebar: Rect::new(area.x, area.y, sidebar_width, area.height),
            handle: Rect::new(handle_x, area.y, handle_width, area.height),
            content: Rect::new(content_x, area.y, content_width, area.height),
            compact: false,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<SidebarResizeEvent> {
        let layout = self.layout(area);
        let position = mouse_position(mouse);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left)
                if rect_contains(
                    self.handle_hit_area(layout.handle, area, layout.axis()),
                    position,
                ) =>
            {
                let axis = layout.axis();
                self.drag_axis = Some(axis);
                let size = self.resize_to_mouse(mouse, area, axis);
                Some(SidebarResizeEvent::Started { axis, size })
            }
            MouseEventKind::Drag(MouseButton::Left) if self.drag_axis.is_some() => {
                let axis = self.drag_axis?;
                let size = self.resize_to_mouse(mouse, area, axis);
                Some(SidebarResizeEvent::Resized { axis, size })
            }
            MouseEventKind::Up(MouseButton::Left) if self.drag_axis.is_some() => {
                let axis = self.drag_axis.take()?;
                let size = self.size_for_axis(axis);
                Some(SidebarResizeEvent::Ended { axis, size })
            }
            _ => None,
        }
    }

    fn compact_layout(self, area: Rect) -> ResizableSidebarLayout {
        let handle_height = u16::from(area.height > 0);
        let max_sidebar_height = area
            .height
            .saturating_sub(handle_height)
            .saturating_sub(self.min_content_height);
        let min_sidebar_height = self.min_compact_height.min(max_sidebar_height);
        let sidebar_height = self
            .compact_height
            .clamp(min_sidebar_height, max_sidebar_height);
        let handle_y = area.y.saturating_add(sidebar_height);
        let content_y = handle_y.saturating_add(handle_height);
        let content_height = area
            .height
            .saturating_sub(sidebar_height)
            .saturating_sub(handle_height);

        ResizableSidebarLayout {
            sidebar: Rect::new(area.x, area.y, area.width, sidebar_height),
            handle: Rect::new(area.x, handle_y, area.width, handle_height),
            content: Rect::new(area.x, content_y, area.width, content_height),
            compact: true,
        }
    }

    fn resize_to_mouse(&mut self, mouse: MouseEvent, area: Rect, axis: SidebarResizeAxis) -> u16 {
        match axis {
            SidebarResizeAxis::Columns => {
                self.resize_to_column(mouse.column, area);
                self.width
            }
            SidebarResizeAxis::Rows => {
                self.resize_to_row(mouse.row, area);
                self.compact_height
            }
        }
    }

    fn resize_to_column(&mut self, column: u16, area: Rect) {
        let layout = self.layout(area);
        let max_width = layout
            .content
            .right()
            .saturating_sub(area.x)
            .saturating_sub(self.min_content_width)
            .min(self.max_width);
        let min_width = self.min_width.min(max_width);
        let requested = column.saturating_sub(area.x);
        self.width = requested.clamp(min_width, max_width);
    }

    fn resize_to_row(&mut self, row: u16, area: Rect) {
        let max_height = area
            .height
            .saturating_sub(1)
            .saturating_sub(self.min_content_height);
        let min_height = self.min_compact_height.min(max_height);
        let requested = row.saturating_sub(area.y);
        self.compact_height = requested.clamp(min_height, max_height);
    }

    fn size_for_axis(self, axis: SidebarResizeAxis) -> u16 {
        match axis {
            SidebarResizeAxis::Columns => self.width,
            SidebarResizeAxis::Rows => self.compact_height,
        }
    }

    fn handle_hit_area(self, handle: Rect, bounds: Rect, axis: SidebarResizeAxis) -> Rect {
        match axis {
            SidebarResizeAxis::Columns => {
                let left = handle.x.saturating_sub(self.hit_slop).max(bounds.x);
                let right = handle
                    .right()
                    .saturating_add(self.hit_slop)
                    .min(bounds.right());

                Rect::new(left, handle.y, right.saturating_sub(left), handle.height)
            }
            SidebarResizeAxis::Rows => {
                let top = handle.y.saturating_sub(self.hit_slop).max(bounds.y);
                let bottom = handle
                    .bottom()
                    .saturating_add(self.hit_slop)
                    .min(bounds.bottom());

                Rect::new(handle.x, top, handle.width, bottom.saturating_sub(top))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizableSidebarLayout {
    pub sidebar: Rect,
    pub handle: Rect,
    pub content: Rect,
    pub compact: bool,
}

impl ResizableSidebarLayout {
    #[must_use]
    pub const fn axis(self) -> SidebarResizeAxis {
        if self.compact {
            SidebarResizeAxis::Rows
        } else {
            SidebarResizeAxis::Columns
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarResizeEvent {
    Started { axis: SidebarResizeAxis, size: u16 },
    Resized { axis: SidebarResizeAxis, size: u16 },
    Ended { axis: SidebarResizeAxis, size: u16 },
}

impl SidebarResizeEvent {
    #[must_use]
    pub const fn axis(self) -> SidebarResizeAxis {
        match self {
            Self::Started { axis, .. } | Self::Resized { axis, .. } | Self::Ended { axis, .. } => {
                axis
            }
        }
    }

    #[must_use]
    pub const fn size(self) -> u16 {
        match self {
            Self::Started { size, .. } | Self::Resized { size, .. } | Self::Ended { size, .. } => {
                size
            }
        }
    }

    #[must_use]
    pub const fn is_finished(self) -> bool {
        matches!(self, Self::Ended { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn body_split_switches_to_vertical_when_narrow() {
        let chunks = body_split(Rect::new(0, 0, 80, 30), 96, 36, 12);

        assert_eq!(chunks[0].height, 12);
        assert_eq!(chunks[1].y, 13);
    }

    #[test]
    fn resizable_sidebar_splits_with_handle_column() {
        let sidebar = ResizableSidebar::new(30, 20, 60).with_min_content_width(40);
        let split = sidebar.layout(Rect::new(2, 3, 100, 20));

        assert_eq!(split.sidebar, Rect::new(2, 3, 30, 20));
        assert_eq!(split.handle, Rect::new(32, 3, 1, 20));
        assert_eq!(split.content, Rect::new(33, 3, 69, 20));
        assert!(!split.compact);
    }

    #[test]
    fn resizable_sidebar_clamps_to_content_room() {
        let sidebar = ResizableSidebar::new(70, 20, 90).with_min_content_width(40);
        let split = sidebar.layout(Rect::new(0, 0, 80, 10));

        assert_eq!(split.sidebar.width, 39);
        assert_eq!(split.content.width, 40);
    }

    #[test]
    fn resizable_sidebar_switches_to_compact_layout() {
        let sidebar = ResizableSidebar::new(30, 20, 60).with_compact(90, 8);
        let split = sidebar.layout(Rect::new(0, 0, 80, 20));

        assert_eq!(split.sidebar, Rect::new(0, 0, 80, 8));
        assert_eq!(split.handle, Rect::new(0, 8, 80, 1));
        assert_eq!(split.content, Rect::new(0, 9, 80, 11));
        assert!(split.compact);
        assert_eq!(split.axis(), SidebarResizeAxis::Rows);
    }

    #[test]
    fn resizable_sidebar_drag_updates_width_until_mouse_up() {
        let mut sidebar = ResizableSidebar::new(30, 20, 70).with_min_content_width(30);
        let area = Rect::new(5, 2, 120, 12);

        assert_eq!(
            sidebar.handle_mouse(mouse_down(35, 4), area),
            Some(SidebarResizeEvent::Started {
                axis: SidebarResizeAxis::Columns,
                size: 30,
            })
        );
        assert!(sidebar.is_dragging());
        assert_eq!(
            sidebar.handle_mouse(mouse_drag(55, 4), area),
            Some(SidebarResizeEvent::Resized {
                axis: SidebarResizeAxis::Columns,
                size: 50,
            })
        );
        assert_eq!(sidebar.width(), 50);
        assert_eq!(
            sidebar.handle_mouse(mouse_up(55, 4), area),
            Some(SidebarResizeEvent::Ended {
                axis: SidebarResizeAxis::Columns,
                size: 50,
            })
        );
        assert!(!sidebar.is_dragging());
    }

    #[test]
    fn compact_resizable_sidebar_drag_updates_height_until_mouse_up() {
        let mut sidebar = ResizableSidebar::new(30, 20, 70)
            .with_compact(90, 8)
            .with_min_compact_height(4)
            .with_min_content_height(6);
        let area = Rect::new(0, 0, 80, 30);

        assert_eq!(
            sidebar.handle_mouse(mouse_down(20, 8), area),
            Some(SidebarResizeEvent::Started {
                axis: SidebarResizeAxis::Rows,
                size: 8,
            })
        );
        assert_eq!(
            sidebar.handle_mouse(mouse_drag(20, 14), area),
            Some(SidebarResizeEvent::Resized {
                axis: SidebarResizeAxis::Rows,
                size: 14,
            })
        );
        assert_eq!(sidebar.compact_height(), 14);
        assert_eq!(
            sidebar.handle_mouse(mouse_up(20, 14), area),
            Some(SidebarResizeEvent::Ended {
                axis: SidebarResizeAxis::Rows,
                size: 14,
            })
        );
        assert!(!sidebar.is_dragging());
    }

    fn mouse_down(column: u16, row: u16) -> MouseEvent {
        mouse(MouseEventKind::Down(MouseButton::Left), column, row)
    }

    fn mouse_drag(column: u16, row: u16) -> MouseEvent {
        mouse(MouseEventKind::Drag(MouseButton::Left), column, row)
    }

    fn mouse_up(column: u16, row: u16) -> MouseEvent {
        mouse(MouseEventKind::Up(MouseButton::Left), column, row)
    }

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }
    }
}
