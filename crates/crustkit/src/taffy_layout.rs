//! Helpers for using Taffy layout trees with Ratatui terminal rectangles.
//!
//! Enable the `taffy` feature to use these helpers:
//!
//! ```toml
//! crustkit = { version = "0.2.0", features = ["taffy"] }
//! ```
//!
//! Crustkit intentionally does not wrap Taffy's style API. Build normal
//! [`taffy::TaffyTree`] nodes, compute the tree against a terminal viewport,
//! then convert each computed layout into a Ratatui [`Rect`].

use ratatui::layout::Rect;
use taffy::{AvailableSpace, Layout, NodeId, Size, TaffyError, TaffyTree};

/// Returns Taffy available space using the terminal area's width and height.
#[must_use]
pub fn available_space_for_rect(area: Rect) -> Size<AvailableSpace> {
    Size {
        width: AvailableSpace::Definite(f32::from(area.width)),
        height: AvailableSpace::Definite(f32::from(area.height)),
    }
}

/// Computes a Taffy tree against the exact size of a Ratatui terminal area.
///
/// The computed positions are still relative to each Taffy node's parent. Use
/// [`layout_rect_for_node`] or [`rect_for_layout`] to convert them to terminal
/// rectangles.
///
/// # Errors
///
/// Returns Taffy's error when `root` is not a valid node for the tree.
pub fn compute_terminal_layout<NodeContext>(
    tree: &mut TaffyTree<NodeContext>,
    root: NodeId,
    area: Rect,
) -> Result<(), TaffyError> {
    tree.compute_layout(root, available_space_for_rect(area))
}

/// Converts a computed Taffy layout into a Ratatui rectangle inside `parent`.
///
/// Taffy layouts are floating-point values. This helper rounds edges to the
/// nearest terminal cell and clips the resulting rectangle to `parent`, which
/// keeps rendering calls inside the caller's assigned frame area.
#[must_use]
pub fn rect_for_layout(parent: Rect, layout: &Layout) -> Rect {
    let parent_left = f32::from(parent.x);
    let parent_top = f32::from(parent.y);
    let parent_right = f32::from(parent.x.saturating_add(parent.width));
    let parent_bottom = f32::from(parent.y.saturating_add(parent.height));

    let left = rounded_edge(parent_left + finite_or_zero(layout.location.x))
        .clamp(parent_left, parent_right);
    let top = rounded_edge(parent_top + finite_or_zero(layout.location.y))
        .clamp(parent_top, parent_bottom);
    let right = rounded_edge(
        parent_left + finite_or_zero(layout.location.x) + non_negative(layout.size.width),
    )
    .clamp(left, parent_right);
    let bottom = rounded_edge(
        parent_top + finite_or_zero(layout.location.y) + non_negative(layout.size.height),
    )
    .clamp(top, parent_bottom);

    Rect::new(
        f32_to_u16(left),
        f32_to_u16(top),
        f32_to_u16(right - left),
        f32_to_u16(bottom - top),
    )
}

/// Reads a computed node layout from `tree` and converts it to a Ratatui rect.
///
/// Pass the Ratatui rectangle assigned to the node's parent as `parent`.
///
/// # Errors
///
/// Returns Taffy's error when `node` is not a valid node for the tree.
pub fn layout_rect_for_node<NodeContext>(
    tree: &TaffyTree<NodeContext>,
    node: NodeId,
    parent: Rect,
) -> Result<Rect, TaffyError> {
    tree.layout(node)
        .map(|layout| rect_for_layout(parent, layout))
}

/// Convenience methods for using Taffy trees in Ratatui render code.
pub trait TaffyTreeExt<NodeContext> {
    /// Computes this tree against a Ratatui terminal area.
    ///
    /// # Errors
    ///
    /// Returns Taffy's error when `root` is not a valid node for the tree.
    fn compute_terminal_layout(&mut self, root: NodeId, area: Rect) -> Result<(), TaffyError>;

    /// Converts a computed node layout to a Ratatui rectangle inside `parent`.
    ///
    /// # Errors
    ///
    /// Returns Taffy's error when `node` is not a valid node for the tree.
    fn layout_rect(&self, node: NodeId, parent: Rect) -> Result<Rect, TaffyError>;
}

impl<NodeContext> TaffyTreeExt<NodeContext> for TaffyTree<NodeContext> {
    fn compute_terminal_layout(&mut self, root: NodeId, area: Rect) -> Result<(), TaffyError> {
        compute_terminal_layout(self, root, area)
    }

    fn layout_rect(&self, node: NodeId, parent: Rect) -> Result<Rect, TaffyError> {
        layout_rect_for_node(self, node, parent)
    }
}

fn rounded_edge(value: f32) -> f32 {
    finite_or_zero(value).round()
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn non_negative(value: f32) -> f32 {
    finite_or_zero(value).max(0.0)
}

fn f32_to_u16(value: f32) -> u16 {
    value.clamp(0.0, f32::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect as TerminalRect;
    use taffy::geometry::Point;
    use taffy::prelude::{Display, FlexDirection, Style, length, percent};

    #[test]
    fn computes_tree_against_terminal_area() -> Result<(), TaffyError> {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let left = tree.new_leaf(Style {
            size: Size {
                width: length(10.0),
                height: length(5.0),
            },
            ..Default::default()
        })?;
        let right = tree.new_leaf(Style {
            size: Size {
                width: length(30.0),
                height: length(5.0),
            },
            ..Default::default()
        })?;
        let root = tree.new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: percent(1.0),
                    height: percent(1.0),
                },
                ..Default::default()
            },
            &[left, right],
        )?;

        let area = TerminalRect::new(2, 3, 40, 10);
        tree.compute_terminal_layout(root, area)?;

        assert_eq!(
            tree.layout_rect(left, area)?,
            TerminalRect::new(2, 3, 10, 5)
        );
        assert_eq!(
            tree.layout_rect(right, area)?,
            TerminalRect::new(12, 3, 30, 5)
        );

        Ok(())
    }

    #[test]
    fn clips_layout_rects_to_parent_area() {
        let layout = Layout {
            location: Point { x: -2.0, y: 3.0 },
            size: Size {
                width: 12.0,
                height: 20.0,
            },
            ..Default::default()
        };

        assert_eq!(
            rect_for_layout(TerminalRect::new(10, 20, 8, 10), &layout),
            TerminalRect::new(10, 23, 8, 7)
        );
    }
}
