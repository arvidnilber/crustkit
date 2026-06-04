use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event};

/// Poll for up to `timeout`, then drain the entire burst of immediately
/// available events. Returns every event read (an empty vec if `timeout`
/// elapses with no input).
///
/// Draining the burst lets the caller redraw **once** per batch instead of once
/// per event. During a drag-resize — or with any-motion mouse reporting enabled
/// — the event queue floods; redrawing per event causes visible flicker. The
/// intended loop is: drain, handle every returned event, then draw a single
/// frame.
///
/// ```no_run
/// # use std::time::Duration;
/// # use crustkit::drain_events;
/// # fn redraw() {}
/// # fn handle(_: &crossterm::event::Event) {}
/// loop {
///     redraw();
///     for event in drain_events(Duration::from_millis(150))? {
///         handle(&event);
///     }
/// # break;
/// }
/// # Ok::<(), color_eyre::Report>(())
/// ```
pub fn drain_events(timeout: Duration) -> Result<Vec<Event>> {
    let mut events = Vec::new();
    if event::poll(timeout)? {
        events.push(event::read()?);
        while event::poll(Duration::ZERO)? {
            events.push(event::read()?);
        }
    }
    Ok(events)
}
