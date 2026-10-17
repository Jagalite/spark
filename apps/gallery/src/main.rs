#![forbid(unsafe_code)]
//! A native inspection app. All clocks, events, and pacing belong to this host.
mod app;
mod design;
mod pacing;
mod view;

use app::App;
use pacing::Pacing;
use ratatui::{
    Terminal,
    backend::TestBackend,
    crossterm::event::{self, Event},
};
use std::{
    io::{self, IsTerminal},
    time::{Duration, Instant},
};

fn main() -> io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--smoke") => return smoke(),
        Some("--snapshot") => return snapshot(),
        Some("--help") | Some("-h") => {
            println!(
                "spark-gallery\n\n1-6/tab design | q quit | space pause | r restart | [ ] scrub\narrows size | c palette | a ASCII | m color profile | p reduced | s static\n, . speed | n seed | - + input | u unknown progress | v isolated/embedded\n\n--smoke     deterministic offscreen checks, no terminal required\n--snapshot  print the 100x32 gallery at clip time 750ms"
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
            "interactive gallery needs a terminal; use --smoke or --snapshot",
        ));
    }
    let mut terminal = ratatui::try_init()?;
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let origin = Instant::now();
    let mut app = App::default();
    let mut pacing = Pacing::default();
    let mut size = terminal.size()?;
    loop {
        let now = origin.elapsed();
        if pacing.should_draw(now) {
            // Sample once at the latest host time, even after a long stall.
            let clip = app.clip_time(now);
            terminal.draw(|frame| view::render(frame, &app, clip))?;
            app.displayed = clip;
            size = terminal.size()?;
            pacing.presented(origin.elapsed(), app.interval(size));
        }
        // No animation deadline means a genuinely blocking event read. Paused
        // and static views still paint on resize, theme change, or other input.
        let event = match pacing.wait(origin.elapsed()) {
            Some(wait) if event::poll(wait)? => Some(event::read()?),
            Some(_) => None,
            None => Some(event::read()?),
        };
        if let Some(event) = event {
            let now = origin.elapsed();
            if let Event::Resize(width, height) = event {
                size = (width, height).into();
                pacing.invalidate(now, app.interval(size));
            } else if let Event::Key(key) = event {
                let outcome = app.handle_key(key, now);
                if outcome.quit {
                    return Ok(());
                }
                if outcome.changed {
                    pacing.invalidate(now, app.interval(size));
                }
            }
        }
    }
}

fn smoke() -> io::Result<()> {
    let mut frames = 0;
    for (width, height) in [
        (0, 0),
        (1, 1),
        (7, 3),
        (24, 8),
        (48, 18),
        (80, 24),
        (100, 32),
        (140, 42),
    ] {
        let mut terminal =
            Terminal::new(TestBackend::new(width, height)).expect("TestBackend is infallible");
        let mut app = App::default();
        for selection in design::Selection::ALL {
            app.select(selection, Duration::ZERO);
            for embedded in [false, true] {
                app.embedded = embedded;
                for millis in [0, 750, 1199, 1200, 1599, 1600, 987_654_321] {
                    let clip = app.clip_time(Duration::from_millis(millis));
                    terminal
                        .draw(|frame| view::render(frame, &app, clip))
                        .expect("TestBackend is infallible");
                    frames += 1;
                }
                app.profile = spark_runtime::Profile::quiet();
                terminal
                    .draw(|frame| view::render(frame, &app, spark_runtime::ClipTime::MAX))
                    .expect("TestBackend is infallible");
                frames += 1;
                app.profile = spark_runtime::Profile::default();
            }
        }
    }
    println!(
        "gallery smoke: {frames} offscreen frames passed; six designs, isolated/embedded, empty/tiny/normal/large geometry, finite/period boundaries, late time, and quiet fallback"
    );
    Ok(())
}

fn snapshot() -> io::Result<()> {
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).expect("TestBackend is infallible");
    let app = App::default();
    terminal
        .draw(|frame| {
            view::render(
                frame,
                &app,
                spark_runtime::ClipTime::from_nanos(750_000_000),
            )
        })
        .expect("TestBackend is infallible");
    let buffer = terminal.backend().buffer();
    for y in buffer.area.top()..buffer.area.bottom() {
        let line: String = (buffer.area.left()..buffer.area.right())
            .map(|x| buffer[(x, y)].symbol())
            .collect();
        println!("{}", line.trim_end());
    }
    Ok(())
}
