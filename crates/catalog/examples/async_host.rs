#![forbid(unsafe_code)]
//! Async event host with one cancellable presentation wait and independent data.
//! Tokio is an example-only dev dependency. There is no task per animation.
//! A bounded input channel applies backpressure; no obsolete-frame queue exists.
mod support;
use ratatui::{
    crossterm::event::{self, Event},
    layout::Size,
};
use std::{
    future::pending,
    io::{self, IsTerminal},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use support::{Host, HostStyle, draw, handle_key, host_time, wait_until};
use tokio::sync::mpsc;

fn main() -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    match std::env::args().nth(1).as_deref() {
        Some("--smoke") => {
            runtime.block_on(cancellation_smoke());
            println!(
                "async host smoke: {:?}; cancellable wait passed",
                support::smoke(HostStyle::Asynchronous)
            );
            return Ok(());
        }
        Some("--help" | "-h") => {
            println!(
                "async_host\n{}\n--smoke  offscreen tests plus cancellable async wait",
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
    let result = runtime.block_on(run(&mut terminal));
    ratatui::restore();
    result
}
async fn cancellation_smoke() {
    let (tx, mut rx) = mpsc::channel(1);
    tx.send("input").await.unwrap();
    tokio::select! {
        event = rx.recv() => assert_eq!(event, Some("input")),
        _ = tokio::time::sleep(Duration::from_secs(60)) => panic!("input must cancel the obsolete wait"),
    }
}
async fn run(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let (tx, mut rx) = mpsc::channel(32);
    let stop = Arc::new(AtomicBool::new(false));
    let input_stop = Arc::clone(&stop);
    // Crossterm's synchronous terminal input is adapted once, outside rendering.
    let input = std::thread::spawn(move || {
        while !input_stop.load(Ordering::Relaxed) {
            match event::poll(Duration::from_millis(25)) {
                Ok(false) => continue,
                Ok(true) => {
                    if tx.blocking_send(event::read()).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.blocking_send(Err(error));
                    break;
                }
            }
        }
    });
    let result = event_loop(terminal, &mut rx).await;
    stop.store(true, Ordering::Relaxed);
    drop(rx); // Release a producer blocked by a full channel before joining.
    input
        .join()
        .map_err(|_| io::Error::other("terminal input thread panicked"))?;
    result
}
async fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    rx: &mut mpsc::Receiver<io::Result<Event>>,
) -> io::Result<()> {
    let origin = Instant::now();
    let mut host = Host::default();
    let mut size = terminal.size()?;
    loop {
        let now = host_time(origin.elapsed());
        host.poll_data_if_due(now);
        if host.next_presentation().is_some_and(|due| now >= due) {
            terminal.draw(|frame| draw(frame, &host, now))?;
            size = terminal.size()?;
            host.presented(now, size);
        }
        let presentation = host.next_presentation();
        let data = host.producer_enabled.then_some(host.next_data);
        // Dropping either sleep on an input/data event cancels it. The next
        // iteration derives one current deadline and samples one current pose.
        tokio::select! {
            event = rx.recv() => match event.transpose()? {
                Some(Event::Resize(width, height)) => { size = Size::new(width, height); host.dirty = true; }
                Some(Event::Key(key)) if handle_key(&mut host, key, host_time(origin.elapsed())) => return Ok(()),
                None => return Ok(()),
                _ => {},
            },
            _ = async {
                match presentation {
                    Some(deadline) => tokio::time::sleep(wait_until(deadline, host_time(origin.elapsed()))).await,
                    None => pending::<()>().await,
                }
            } => {},
            _ = async {
                match data {
                    Some(deadline) => tokio::time::sleep(wait_until(deadline, host_time(origin.elapsed()))).await,
                    None => pending::<()>().await,
                }
            } => { host.poll_data_if_due(host_time(origin.elapsed())); },
        }
        if host.dirty {
            host.refresh_animation(host_time(origin.elapsed()), size);
        }
    }
}
