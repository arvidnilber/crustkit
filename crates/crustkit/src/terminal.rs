use std::{
    io::{self, Stdout},
    time::Duration,
};

use color_eyre::Result;
use crossterm::{
    cursor::{MoveTo, Show},
    event::{DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};

pub type ManagedTerminal = Terminal<CrosstermBackend<Stdout>>;
pub type TerminalResult<T> = Result<T>;

pub struct FullRepaintTicker {
    pending: bool,
}

impl FullRepaintTicker {
    pub fn new(_interval: Duration) -> Self {
        Self { pending: true }
    }

    pub fn should_clear(&mut self) -> bool {
        let should_clear = self.pending;
        self.pending = false;
        should_clear
    }

    pub fn force_next(&mut self) {
        self.pending = true;
    }
}

pub fn run_with_terminal<T>(
    run: impl FnOnce(&mut ManagedTerminal) -> TerminalResult<T>,
) -> TerminalResult<T> {
    let mut terminal = TerminalSession::enter()?;
    let result = run(&mut terminal.terminal);
    terminal.restore()?;
    result
}

pub fn restore_terminal_state() -> TerminalResult<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        DisableFocusChange,
        DisableMouseCapture,
        Clear(ClearType::All),
        MoveTo(0, 0),
        LeaveAlternateScreen,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Show
    )?;
    Ok(())
}

struct TerminalSession {
    terminal: ManagedTerminal,
    restored: bool,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableFocusChange,
            EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        Ok(Self {
            terminal,
            restored: false,
        })
    }

    fn restore(&mut self) -> Result<()> {
        if self.restored {
            return Ok(());
        }

        let _ = self.terminal.clear();
        restore_terminal_state()?;
        self.restored = true;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
