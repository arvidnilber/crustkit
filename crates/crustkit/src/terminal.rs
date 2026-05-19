use std::{
    env,
    ffi::OsString,
    io::{self, Stdout},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use color_eyre::{Result, eyre::Context};
use crossterm::{
    cursor::{MoveTo, Show},
    event::{
        DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, KeyCode,
        KeyModifiers,
    },
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::KeyHint;

pub type ManagedTerminal = Terminal<CrosstermBackend<Stdout>>;
pub type TerminalResult<T> = Result<T>;
pub const CLI_RELOAD_EXIT_CODE: i32 = 75;
pub const CLI_RELOAD_CHILD_ENV: &str = "CRUSTKIT_RELOAD_CHILD";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliExit {
    Quit,
    Reload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CliReload {
    enabled: bool,
    key: char,
    key_label: &'static str,
    dev_cargo_loop: bool,
}

impl CliReload {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            key: 'r',
            key_label: "r",
            dev_cargo_loop: false,
        }
    }

    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            key: 'r',
            key_label: "r",
            dev_cargo_loop: true,
        }
    }

    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    pub const fn without_dev_cargo_loop(mut self) -> Self {
        self.dev_cargo_loop = false;
        self
    }

    pub fn matches_key(self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.enabled
            && modifiers.is_empty()
            && matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&self.key))
    }

    pub fn key_hint(self) -> Option<KeyHint> {
        self.enabled.then(|| KeyHint::new("reload", self.key_label))
    }
}

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

/// Replace the current CLI process with a fresh copy of the same executable.
///
/// This is intended for TUI apps that expose a "reload" key. Return
/// [`CliExit::Reload`] from the app loop, let `run_with_terminal` restore the
/// terminal, then call this from `main`.
pub fn reload_current_process() -> TerminalResult<()> {
    let _ = restore_terminal_state();
    if env::var_os(CLI_RELOAD_CHILD_ENV).is_some() {
        std::process::exit(CLI_RELOAD_EXIT_CODE);
    }

    let executable = env::current_exe().context("failed to resolve current executable")?;
    let args = env::args_os().skip(1);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let error = Command::new(executable).args(args).exec();
        Err(error).context("failed to reload current executable")
    }

    #[cfg(not(unix))]
    {
        Command::new(executable)
            .args(args)
            .spawn()
            .context("failed to reload current executable")?;
        std::process::exit(0);
    }
}

pub fn run_reloadable_cli(
    reload: CliReload,
    run: impl FnOnce() -> TerminalResult<CliExit>,
) -> TerminalResult<()> {
    if should_start_dev_cargo_loop(reload) {
        return run_dev_cargo_loop(env::args_os().skip(1).collect());
    }

    finish_cli_exit(run()?)
}

pub fn finish_cli_exit(exit: CliExit) -> TerminalResult<()> {
    match exit {
        CliExit::Quit => Ok(()),
        CliExit::Reload => reload_current_process(),
    }
}

fn should_start_dev_cargo_loop(reload: CliReload) -> bool {
    reload.enabled
        && reload.dev_cargo_loop
        && cfg!(debug_assertions)
        && env::var_os(CLI_RELOAD_CHILD_ENV).is_none()
        && Path::new("Cargo.toml").is_file()
}

fn run_dev_cargo_loop(args: Vec<OsString>) -> TerminalResult<()> {
    loop {
        let status = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(&args)
            .env(CLI_RELOAD_CHILD_ENV, "1")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to run cargo reload loop")?;

        if status.code() != Some(CLI_RELOAD_EXIT_CODE) {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
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
