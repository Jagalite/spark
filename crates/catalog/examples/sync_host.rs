#![forbid(unsafe_code)]
//! Synchronous dirty/deadline host for all six designs and four shared embeddings.
//! The host owns terminal IO, clocks, producer cadence, and operation generations.
//! Paused/static/hidden/complete art stays drawable without animation-only wakes.
mod support;
use ratatui::{
    crossterm::event::{self, Event},
    layout::Size,
};
use std::{
    io::{self, IsTerminal},
    time::Instant,
};
use support::{Host, HostStyle, draw, handle_key, host_time, wait_until};

fn main() -> io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--smoke") => {
            println!(
                "sync host smoke: {:?}",
                support::smoke(HostStyle::Synchronous)
            );
            return Ok(());
        }
        Some("--help" | "-h") => {
            println!(
                "sync_host\n{}\n--smoke  deterministic offscreen lifecycle/data tests",
                support::CONTROLS
            );
            return Ok(());
        }
        Some(arg) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown option {arg}; use --help"),
            ));
        }
        None => {}
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "interactive host needs a terminal; use --smoke",
        ));
    }
    let mut terminal = ratatui::try_init()?;
    let result = run(&mut terminal);
    ratatui::restore();
    result
}
fn run(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let origin = Instant::now();
    let mut host = Host::default();
    let mut size = terminal.size()?;
    loop {
        let now = host_time(origin.elapsed());
        host.poll_data_if_due(now); // Independent of redraw frequency.
        if host.next_presentation().is_some_and(|due| now >= due) {
            terminal.draw(|frame| draw(frame, &host, now))?;
            size = terminal.size()?;
            host.presented(now, size); // One current sample after any long stall.
        }
        let input = match host.next_wake() {
            Some(deadline) => {
                if event::poll(wait_until(deadline, host_time(origin.elapsed())))? {
                    Some(event::read()?)
                } else {
                    None
                }
            }
            None => Some(event::read()?), // No timer when both data and art are idle.
        };
        match input {
            Some(Event::Resize(width, height)) => {
                size = Size::new(width, height);
                host.dirty = true;
            }
            Some(Event::Key(key)) if handle_key(&mut host, key, host_time(origin.elapsed())) => {
                return Ok(());
            }
            _ => {}
        }
        if host.dirty {
            host.refresh_animation(host_time(origin.elapsed()), size);
        }
    }
}
