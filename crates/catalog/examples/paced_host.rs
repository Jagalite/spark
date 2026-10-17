#![forbid(unsafe_code)]
//! An already-paced host: one presentation cadence, a slower independent producer.
//! Spark does not add a timer, and a stall never queues obsolete frames.
mod support;
use ratatui::{
    crossterm::event::{self, Event},
    layout::Size,
};
use spark_runtime::HostTime;
use std::{
    io::{self, IsTerminal},
    time::Instant,
};
use support::{FRAME_CAP, Host, HostStyle, add, draw, handle_key, host_time, wait_until};

fn main() -> io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--smoke") => {
            println!("paced host smoke: {:?}", support::smoke(HostStyle::Paced));
            return Ok(());
        }
        Some("--help" | "-h") => {
            println!(
                "paced_host\n{}\n--smoke  fixed-cadence offscreen tests",
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
    let mut next_frame = HostTime::ZERO;
    loop {
        let now = host_time(origin.elapsed());
        host.poll_data_if_due(now);
        if now >= next_frame {
            terminal.draw(|frame| draw(frame, &host, now))?;
            size = terminal.size()?;
            host.presented(now, size);
            next_frame = add(host_time(origin.elapsed()), FRAME_CAP);
        }
        // Existing presentation pacing continues even for static art. No extra
        // animation wake is introduced, and data is never sampled in draw().
        let deadline = if host.producer_enabled {
            next_frame.min(host.next_data)
        } else {
            next_frame
        };
        if event::poll(wait_until(deadline, host_time(origin.elapsed())))? {
            match event::read()? {
                Event::Resize(width, height) => {
                    size = Size::new(width, height);
                    host.dirty = true;
                }
                Event::Key(key) if handle_key(&mut host, key, host_time(origin.elapsed())) => {
                    return Ok(());
                }
                _ => {}
            }
        }
        // Size participates in sampling; it does not change either cadence.
        let _ = size;
    }
}
