#![forbid(unsafe_code)]

pub mod dialog;
#[cfg(feature = "tachyonfx")]
pub mod effects;
pub mod events;
pub mod input;
pub mod keys;
pub mod layout;
pub mod mouse;
pub mod progress;
pub mod shell;
pub mod status;
#[cfg(feature = "taffy")]
pub mod taffy_layout;
pub mod terminal;
pub mod terminal_background;
pub mod theme;

pub use dialog::{DialogTheme, InputDialog, centered_dialog_rect};
#[cfg(feature = "tachyonfx")]
pub use effects::{
    ComponentEffect, ComponentEffectFilter, ComponentEffectManager, ComponentEffectPattern,
    EffectPreset, EffectRepeat, UiEffectManager,
};
pub use events::drain_events;
pub use input::{TextInput, input_value_spans};
pub use keys::{
    KeyHint, NavigationCommand, NavigationFocus, key_hints_line, navigation_command, tab_hotkey,
};
pub use layout::{
    ResizableSidebar, ResizableSidebarLayout, SidebarResizeAxis, SidebarResizeEvent, body_split,
    centered_rect,
};
pub use mouse::{
    MouseClick, inline_item_index_at, left_mouse_click, mouse_position, rect_contains, row_index_at,
};
pub use progress::{
    ProgressBarTheme, TransferProgress, format_bytes, format_bytes_per_second,
    transfer_progress_gauge,
};
pub use shell::{footer, header};
pub use status::{StatusKind, StatusLine};
#[cfg(feature = "taffy")]
pub use taffy;
#[cfg(feature = "taffy")]
pub use taffy_layout::{
    TaffyTreeExt, available_space_for_rect, compute_terminal_layout, layout_rect_for_node,
    rect_for_layout,
};
pub use terminal::{
    CLI_RELOAD_CHILD_ENV, CLI_RELOAD_EXIT_CODE, CliExit, CliReload, FullRepaintTicker,
    ManagedTerminal, TerminalResult, exit_key_hint, finish_cli_exit, is_exit_key,
    reload_current_process, restore_terminal_state, run_reloadable_cli, run_with_terminal,
};
pub use terminal_background::{
    TerminalBackgroundColor, detect_terminal_background_color, detect_terminal_theme_mode,
    terminal_theme_mode_from_colorfgbg, terminal_theme_mode_or,
};
pub use theme::{AppTheme, SelectionTheme, ThemeMode, ThemePalette};
