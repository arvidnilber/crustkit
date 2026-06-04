use std::{
    env,
    ffi::OsString,
    io::{self, Stdout},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
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
    file_watch: bool,
    watch_debounce_ms: u64,
}

impl CliReload {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            key: 'r',
            key_label: "ctrl+r",
            dev_cargo_loop: false,
            file_watch: false,
            watch_debounce_ms: 400,
        }
    }

    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            key: 'r',
            key_label: "ctrl+r",
            dev_cargo_loop: true,
            file_watch: false,
            watch_debounce_ms: 400,
        }
    }

    pub const fn is_enabled(self) -> bool {
        self.enabled
    }

    pub const fn without_dev_cargo_loop(mut self) -> Self {
        self.dev_cargo_loop = false;
        self
    }

    /// Opt into debounced filesystem-watch reload (a prop the app sets): while
    /// the dev cargo loop runs, saving a watched source file rebuilds and
    /// relaunches automatically, the same effect as pressing the reload key.
    /// Requires the `watch` feature; a no-op without it.
    pub const fn with_file_watch(mut self, on: bool) -> Self {
        self.file_watch = on;
        self
    }

    /// Quiet period the watcher waits for edits to settle before rebuilding, so
    /// a burst of rapid saves coalesces into one reload instead of spamming
    /// rebuilds. Defaults to 400ms.
    pub const fn with_watch_debounce_ms(mut self, ms: u64) -> Self {
        self.watch_debounce_ms = ms;
        self
    }

    pub const fn is_file_watch_enabled(self) -> bool {
        self.file_watch
    }

    pub fn watch_debounce(self) -> Duration {
        Duration::from_millis(self.watch_debounce_ms)
    }

    pub fn matches_key(self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.enabled
            && modifiers.contains(KeyModifiers::CONTROL)
            && matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&self.key))
    }

    pub fn key_hint(self) -> Option<KeyHint> {
        self.enabled.then(|| KeyHint::new(self.key_label, "reload"))
    }
}

pub fn is_exit_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL)
        && matches!(code, KeyCode::Char(ch) if ch.eq_ignore_ascii_case(&'c'))
}

pub fn exit_key_hint() -> KeyHint {
    KeyHint::new("ctrl+c", "quit")
}

pub struct FullRepaintTicker {
    interval: Duration,
    last_clear: Instant,
    pending: bool,
}

impl FullRepaintTicker {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_clear: Instant::now(),
            pending: true,
        }
    }

    pub fn should_clear(&mut self) -> bool {
        let should_clear = self.pending || self.last_clear.elapsed() >= self.interval;
        if should_clear {
            self.last_clear = Instant::now();
            self.pending = false;
        }
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
    let restore = terminal.restore();

    match (result, restore) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(restore_err)) => Err(restore_err),
        (Err(run_err), Ok(())) => Err(run_err),
        (Err(run_err), Err(restore_err)) => {
            Err(run_err.wrap_err(format!("terminal restore also failed: {restore_err}")))
        }
    }
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
        return run_dev_cargo_loop(env::args_os().skip(1).collect(), reload);
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

/// What ended a dev-loop child: a request to rebuild and relaunch, or a real
/// exit whose code the supervisor should propagate.
enum ChildOutcome {
    Rebuild,
    Exited(i32),
}

fn run_dev_cargo_loop(args: Vec<OsString>, reload: CliReload) -> TerminalResult<()> {
    loop {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(&args)
            .env(CLI_RELOAD_CHILD_ENV, "1")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to run cargo reload loop")?;

        match supervise_dev_child(&mut child, reload)? {
            ChildOutcome::Rebuild => continue,
            ChildOutcome::Exited(code) => std::process::exit(code),
        }
    }
}

/// Reload key (`ctrl+r`) exits the child with [`CLI_RELOAD_EXIT_CODE`]; any
/// other code is a genuine exit to forward.
fn classify_dev_exit(status: ExitStatus) -> ChildOutcome {
    if status.code() == Some(CLI_RELOAD_EXIT_CODE) {
        ChildOutcome::Rebuild
    } else {
        ChildOutcome::Exited(status.code().unwrap_or(1))
    }
}

#[cfg(not(feature = "watch"))]
fn supervise_dev_child(child: &mut Child, _reload: CliReload) -> TerminalResult<ChildOutcome> {
    let status = child.wait().context("failed to await reload child")?;
    Ok(classify_dev_exit(status))
}

/// With the `watch` feature and file-watch opted in, race the child against a
/// debounced filesystem watcher: a settled batch of source edits terminates the
/// child so the loop rebuilds. Otherwise just await the child as before.
#[cfg(feature = "watch")]
fn supervise_dev_child(child: &mut Child, reload: CliReload) -> TerminalResult<ChildOutcome> {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};

    if !reload.is_file_watch_enabled() {
        let status = child.wait().context("failed to await reload child")?;
        return Ok(classify_dev_exit(status));
    }

    let changed = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&changed);
    let mut debouncer = new_debouncer(
        reload.watch_debounce(),
        None,
        move |result: DebounceEventResult| {
            if let Ok(events) = result
                && events
                    .iter()
                    .any(|event| event.paths.iter().any(|path| is_watch_relevant(path)))
            {
                flag.store(true, Ordering::Release);
            }
        },
    )
    .context("failed to start file watcher")?;

    // The dev cargo loop runs from the crate directory, so the crate's sources
    // and manifest are the right things to watch. Watching `src` (not `.`) keeps
    // `target/` rebuild churn from re-triggering itself.
    let _ = debouncer.watch(Path::new("src"), RecursiveMode::Recursive);
    let _ = debouncer.watch(Path::new("Cargo.toml"), RecursiveMode::NonRecursive);

    loop {
        if changed.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            // Parent and child share this TTY; the killed child left it in raw
            // mode / alt screen, so reset it before cargo prints build output.
            let _ = restore_terminal_state();
            return Ok(ChildOutcome::Rebuild);
        }
        if let Some(status) = child.try_wait().context("failed to poll reload child")? {
            return Ok(classify_dev_exit(status));
        }
        std::thread::sleep(Duration::from_millis(75));
    }
}

/// Only `.rs` files and `Cargo.toml` should trigger a rebuild; editor swap
/// files and unrelated paths are ignored.
#[cfg(feature = "watch")]
fn is_watch_relevant(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "rs")
        || path.file_name().is_some_and(|name| name == "Cargo.toml")
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use std::time::Duration;

    use super::{CliReload, FullRepaintTicker, exit_key_hint, is_exit_key};

    #[test]
    fn cli_reload_matches_ctrl_r_only_when_enabled() {
        let reload = CliReload::enabled();

        assert!(reload.matches_key(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(!reload.matches_key(KeyCode::Char('r'), KeyModifiers::empty()));
        assert!(!CliReload::disabled().matches_key(KeyCode::Char('r'), KeyModifiers::CONTROL));
    }

    #[test]
    fn cli_reload_key_hint_uses_ctrl_r() {
        let hint = CliReload::enabled().key_hint().expect("reload hint");

        assert_eq!(hint.key, "ctrl+r");
        assert_eq!(hint.action, "reload");
        assert!(CliReload::disabled().key_hint().is_none());
    }

    #[test]
    fn exit_key_matches_ctrl_c() {
        assert!(is_exit_key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(is_exit_key(KeyCode::Char('C'), KeyModifiers::CONTROL));
        assert!(!is_exit_key(KeyCode::Char('c'), KeyModifiers::empty()));
        assert!(!is_exit_key(KeyCode::Char('r'), KeyModifiers::CONTROL));
    }

    #[test]
    fn exit_key_hint_uses_ctrl_c() {
        let hint = exit_key_hint();

        assert_eq!(hint.key, "ctrl+c");
        assert_eq!(hint.action, "quit");
    }

    #[test]
    fn full_repaint_ticker_forces_first_clear_then_waits() {
        let mut ticker = FullRepaintTicker::new(Duration::from_secs(60));

        assert!(ticker.should_clear());
        assert!(!ticker.should_clear());

        ticker.force_next();

        assert!(ticker.should_clear());
    }
}
