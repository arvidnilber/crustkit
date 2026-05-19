pub mod dialog;
pub mod input;
pub mod keys;
pub mod layout;
pub mod mouse;
pub mod progress;
pub mod shell;
pub mod status;
pub mod terminal;
pub mod terminal_background;
pub mod theme;

pub use dialog::{DialogTheme, InputDialog, centered_dialog_rect};
pub use input::{TextInput, input_value_spans};
pub use keys::{KeyHint, key_hints_line};
pub use layout::{body_split, centered_rect};
pub use mouse::{
    MouseClick, inline_item_index_at, left_mouse_click, mouse_position, rect_contains, row_index_at,
};
pub use progress::{
    ProgressBarTheme, TransferProgress, format_bytes, format_bytes_per_second,
    transfer_progress_gauge,
};
pub use shell::{footer, header};
pub use status::{StatusKind, StatusLine};
pub use terminal::{
    CLI_RELOAD_CHILD_ENV, CLI_RELOAD_EXIT_CODE, CliExit, CliReload, FullRepaintTicker,
    ManagedTerminal, TerminalResult, finish_cli_exit, reload_current_process,
    restore_terminal_state, run_reloadable_cli, run_with_terminal,
};
pub use terminal_background::{
    TerminalBackgroundColor, detect_terminal_background_color, detect_terminal_theme_mode,
    terminal_theme_mode_from_colorfgbg, terminal_theme_mode_or,
};
pub use theme::{AppTheme, SelectionTheme, ThemeMode, ThemePalette};
