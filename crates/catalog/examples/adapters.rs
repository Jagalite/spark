#![forbid(unsafe_code)]
//! A deterministic host adapter walkthrough. No source, timer, or task is started.

use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_catalog::{
    AssemblyProgress, TelemetryEmbers,
    adapters::{
        AdapterError, CountedProgress, StreamActivity, StreamEvent, TaskSnapshot, TaskStatus,
    },
};
use spark_runtime::{ClipTime, HostTime, Palette, Period, Profile, SampleContext};
use std::{error::Error, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = match std::env::args().nth(1).as_deref() {
        Some("--smoke") => true,
        Some("--help" | "-h") => {
            println!(
                "adapters: caller-fed counts, explicit task status, and stream activity\n\
                      --smoke  check the same deterministic scenario without printing art"
            );
            return Ok(());
        }
        Some(argument) => return Err(format!("unknown option {argument}; use --help").into()),
        None => false,
    };

    let mut stream =
        StreamActivity::new(4, HostTime::ZERO, Period::new(Duration::from_secs(1))?, 8)?;
    // The host delivers an event. Rendering does not poll or consume the source.
    stream.record(StreamEvent {
        generation: 4,
        sequence: 0,
        at: HostTime::ZERO,
        units: 4,
    })?;
    let counts = CountedProgress::new(7, Some(10))?;
    let task = TaskSnapshot::new(TaskStatus::Running, counts.progress());
    present("Receiving", task, counts, &stream, 250_000_000, smoke)?;

    stream.record(StreamEvent {
        generation: 4,
        sequence: 1,
        at: HostTime::from_nanos(250_000_000),
        units: 4,
    })?;
    let counts = CountedProgress::new(10, Some(10))?;
    // Bytes may have arrived while host-owned verification is still running.
    let task = TaskSnapshot::new(TaskStatus::Running, counts.progress());
    assert!(!task.status().is_terminal());
    present("Verifying", task, counts, &stream, 250_000_000, smoke)?;

    // A separate authoritative host event supplies success. No duration,
    // animation completion, signal decay, or percentage decides this status.
    let task = TaskSnapshot::new(TaskStatus::Succeeded, counts.progress());
    present("Verified", task, counts, &stream, 500_000_000, smoke)?;

    // Replace the source before attaching new events. Queued old events retain
    // their original generation and are rejected instead of lighting new work.
    stream.reset(5, HostTime::from_nanos(500_000_000))?;
    assert_eq!(
        stream.record(StreamEvent {
            generation: 4,
            sequence: 999,
            at: HostTime::from_nanos(600_000_000),
            units: u64::MAX,
        }),
        Err(AdapterError::WrongGeneration)
    );
    assert_eq!(stream.last_sequence(), None);
    let counts = CountedProgress::new(0, None)?;
    let task = TaskSnapshot::new(TaskStatus::Queued, counts.progress());
    present("Replacement", task, counts, &stream, 500_000_000, smoke)?;
    println!("adapter smoke: 4 caller-fed snapshots, explicit success, stale source rejected");
    Ok(())
}

fn present(
    label: &str,
    task: TaskSnapshot,
    counts: CountedProgress,
    stream: &StreamActivity,
    now: u64,
    smoke: bool,
) -> Result<(), AdapterError> {
    let palette = Palette::default();
    let profile = Profile::quiet();
    // The host chooses both time domains. A real host can use Playback for the
    // clip position while preserving monotonic observation time for activity.
    let context = SampleContext::new(ClipTime::from_nanos(now), &palette, &profile);
    let level = stream.level(HostTime::from_nanos(now))?;
    let before = *stream;
    let mut buffer = Buffer::empty(Rect::new(0, 0, 32, 9));
    AssemblyProgress::new()
        .sample(&context, task.progress())
        .render(Rect::new(0, 0, 32, 3), &mut buffer);
    TelemetryEmbers::new()
        .sample(&context, level)
        .render(Rect::new(0, 3, 32, 6), &mut buffer);
    assert_eq!(*stream, before);
    if !smoke {
        let count_label = match counts.total() {
            Some(total) => format!("{} / {total}", counts.completed()),
            None => format!("{} / unknown", counts.completed()),
        };
        println!("{label}: {} | {count_label}", status_label(task.status()),);
        for row in buffer.content.chunks(32) {
            println!(
                "{}",
                row.iter().map(|cell| cell.symbol()).collect::<String>()
            );
        }
        println!();
    }
    Ok(())
}

// Labels and any mapping to status::StatusKind are explicit host policy.
fn status_label(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Queued => "queued",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Cancelled => "cancelled",
    }
}
