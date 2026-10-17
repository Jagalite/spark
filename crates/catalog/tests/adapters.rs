use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use spark_catalog::{
    AssemblyProgress, Level, Progress, TelemetryEmbers,
    adapters::{
        AdapterError, CountedProgress, StreamActivity, StreamEvent, TaskSnapshot, TaskStatus,
    },
};
use spark_runtime::{ClipTime, HostTime, Palette, Period, Profile, SampleContext};
use std::time::Duration;

fn time(nanos: u64) -> HostTime {
    HostTime::from_nanos(nanos)
}

fn activity(generation: u64, at: u64, decay: u64, capacity: u64) -> StreamActivity {
    StreamActivity::new(
        generation,
        time(at),
        Period::new(Duration::from_nanos(decay)).unwrap(),
        capacity,
    )
    .unwrap()
}

fn event(generation: u64, sequence: u64, at: u64, units: u64) -> StreamEvent {
    StreamEvent {
        generation,
        sequence,
        at: time(at),
        units,
    }
}

#[test]
fn counts_validate_preserve_integers_and_keep_unknown_totals_explicit() {
    for completed in [0, 1, u64::MAX] {
        let counts = CountedProgress::new(completed, None).unwrap();
        assert_eq!(counts.completed(), completed);
        assert_eq!(counts.total(), None);
        assert_eq!(counts.progress(), Progress::unknown());
    }
    let empty = CountedProgress::new(0, Some(0)).unwrap();
    assert_eq!(empty.total(), Some(0));
    assert_eq!(empty.progress(), Progress::unknown());
    for (completed, total) in [(1, 0), (2, 1), (u64::MAX, u64::MAX - 1)] {
        assert_eq!(
            CountedProgress::new(completed, Some(total)),
            Err(AdapterError::CountExceedsTotal)
        );
    }
}

// Compare represented binary rationals using exact integer arithmetic, including
// the very next f64, independently of the implementation's long division.
fn represented_ratio_cmp(value: f64, numerator: u64, denominator: u64) -> std::cmp::Ordering {
    let bits = value.to_bits();
    let exponent = (bits >> 52) & 0x7ff;
    let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    let shift = 1075 - exponent;
    let represented = u128::from(significand) * u128::from(denominator);
    let exact = u128::from(numerator) << shift;
    represented.cmp(&exact)
}

fn assert_maximal_ratio(completed: u64, total: u64) {
    let value = CountedProgress::new(completed, Some(total))
        .unwrap()
        .progress()
        .value()
        .unwrap();
    assert!(value.is_finite() && (0.0..=1.0).contains(&value));
    if completed == 0 {
        assert_eq!(value.to_bits(), 0.0_f64.to_bits());
    } else if completed == total {
        assert_eq!(value, 1.0);
    } else {
        assert!(value < 1.0, "{completed}/{total} rounded to completion");
        assert!(represented_ratio_cmp(value, completed, total).is_le());
        assert!(
            represented_ratio_cmp(f64::from_bits(value.to_bits() + 1), completed, total).is_gt()
        );
    }
}

#[test]
fn count_ratios_are_maximal_conservative_binary_values_even_at_extremes() {
    for total in 1..=256 {
        for completed in 0..=total {
            assert_maximal_ratio(completed, total);
        }
    }
    for total in [1 << 52, (1 << 53) + 1, 1 << 63, u64::MAX - 1, u64::MAX] {
        for completed in [0, 1, 2, total / 3, total / 2, total - 1, total] {
            assert_maximal_ratio(completed, total);
        }
    }
    let mut state = 0x1234_5678_9abc_def0_u64;
    for _ in 0..4096 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let total = state.max(1);
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        assert_maximal_ratio(state % total, total);
    }
    assert_eq!(
        CountedProgress::new(u64::MAX - 1, Some(u64::MAX))
            .unwrap()
            .progress()
            .value(),
        Some(f64::from_bits(1.0_f64.to_bits() - 1))
    );
}

#[test]
fn explicit_task_state_never_follows_progress_or_visual_time() {
    for status in [
        TaskStatus::Queued,
        TaskStatus::Running,
        TaskStatus::Succeeded,
        TaskStatus::Failed,
        TaskStatus::Cancelled,
    ] {
        assert_eq!(
            status.is_terminal(),
            matches!(
                status,
                TaskStatus::Succeeded | TaskStatus::Failed | TaskStatus::Cancelled
            )
        );
        for progress in [
            Progress::unknown(),
            Progress::known(0.0).unwrap(),
            Progress::known(0.4).unwrap(),
            Progress::known(1.0).unwrap(),
        ] {
            let task = TaskSnapshot::new(status, progress);
            let before = task;
            let palette = Palette::default();
            let profile = Profile::quiet();
            let design = AssemblyProgress::new();
            for at in [ClipTime::ZERO, ClipTime::MAX, ClipTime::from_nanos(1)] {
                let context = SampleContext::new(at, &palette, &profile);
                let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 3));
                design
                    .sample(&context, task.progress())
                    .render(buffer.area, &mut buffer);
            }
            assert_eq!(task, before);
            assert_eq!(task.status(), status);
            assert_eq!(task.progress(), progress);
        }
    }
}

#[test]
fn new_activity_is_empty_and_validates_configuration() {
    let decay = Period::new(Duration::from_nanos(100)).unwrap();
    assert_eq!(
        StreamActivity::new(0, HostTime::ZERO, decay, 0),
        Err(AdapterError::ZeroCapacity)
    );
    let stream = StreamActivity::new(3, time(7), decay, 10).unwrap();
    assert_eq!(stream.generation(), 3);
    assert_eq!(stream.last_sequence(), None);
    assert_eq!(stream.decay(), decay);
    assert_eq!(stream.capacity(), 10);
    for at in [7, 10, u64::MAX, 7] {
        assert_eq!(stream.level(time(at)).unwrap(), Level::ZERO);
    }
    assert_eq!(stream.level(time(6)), Err(AdapterError::BackwardsTime));
}

#[test]
fn stream_impulses_and_linear_decay_have_exact_boundaries() {
    let mut stream = activity(0, 10, 100, 10);
    stream.record(event(0, 0, 10, 5)).unwrap();
    assert_eq!(stream.last_sequence(), Some(0));
    assert_eq!(stream.level(time(10)).unwrap().value(), 0.5);
    assert_eq!(stream.level(time(35)).unwrap().value(), 0.25);
    assert!(stream.level(time(59)).unwrap().value() > 0.0);
    assert_eq!(stream.level(time(60)).unwrap(), Level::ZERO);
    assert_eq!(stream.level(HostTime::MAX).unwrap(), Level::ZERO);
    // Queries did not advance the anchor or spend retained credit.
    stream.record(event(0, 1, 35, 5)).unwrap();
    assert_eq!(stream.level(time(35)).unwrap().value(), 0.75);
    assert_eq!(stream.level(time(60)).unwrap().value(), 0.5);
    assert_eq!(stream.level(time(110)).unwrap(), Level::ZERO);
    stream.record(event(0, 2, 35, u64::MAX)).unwrap();
    assert_eq!(stream.level(time(35)).unwrap(), Level::MAX);
    assert_eq!(stream.level(time(135)).unwrap(), Level::ZERO);
}

#[test]
fn rejected_events_and_resets_are_atomic_and_do_not_consume_sequence() {
    let mut stream = activity(8, 10, 100, 10);
    stream.record(event(8, 1, 20, 5)).unwrap();
    let accepted = stream;
    for (bad, error) in [
        (event(7, 2, 30, 10), AdapterError::WrongGeneration),
        (event(9, 2, 30, 10), AdapterError::WrongGeneration),
        (event(8, 1, 30, 10), AdapterError::NonIncreasingSequence),
        (event(8, 0, 30, 10), AdapterError::NonIncreasingSequence),
        (event(8, 2, 19, 10), AdapterError::BackwardsTime),
    ] {
        assert_eq!(stream.record(bad), Err(error));
        assert_eq!(stream, accepted);
    }
    for (generation, at, error) in [
        (8, 30, AdapterError::NonIncreasingGeneration),
        (7, 30, AdapterError::NonIncreasingGeneration),
        (9, 19, AdapterError::BackwardsTime),
    ] {
        assert_eq!(stream.reset(generation, time(at)), Err(error));
        assert_eq!(stream, accepted);
    }
    assert_eq!(stream.level(time(19)), Err(AdapterError::BackwardsTime));
    assert_eq!(stream, accepted);
    stream.record(event(8, 2, 20, 0)).unwrap();
    assert_eq!(stream.last_sequence(), Some(2));
    assert_eq!(stream.level(time(20)).unwrap().value(), 0.5);
}

#[test]
fn replacement_clears_activity_and_rejects_late_old_sources() {
    let mut stream = activity(10, 0, 100, 10);
    stream.record(event(10, 99, 5, 10)).unwrap();
    stream.reset(11, time(6)).unwrap();
    assert_eq!(stream.generation(), 11);
    assert_eq!(stream.last_sequence(), None);
    assert_eq!(stream.level(time(6)).unwrap(), Level::ZERO);
    let reset = stream;
    for sequence in [0, 100, u64::MAX] {
        assert_eq!(
            stream.record(event(10, sequence, u64::MAX, u64::MAX)),
            Err(AdapterError::WrongGeneration)
        );
        assert_eq!(stream, reset);
    }
    stream.record(event(11, 0, 6, 2)).unwrap();
    assert!(stream.level(time(6)).unwrap().value() <= 0.2);
    stream.reset(12, time(6)).unwrap();
    assert_eq!(stream.level(time(6)).unwrap(), Level::ZERO);
    // A host may cancel by retiring a generation; the adapter does not declare
    // cancellation or retag old work as belonging to the replacement.
    assert_eq!(
        stream.record(event(11, 1, 7, 10)),
        Err(AdapterError::WrongGeneration)
    );
}

#[test]
fn reads_are_pure_under_dense_sparse_repeated_and_shuffled_sampling() {
    let mut sparse = activity(0, 0, 103, 13);
    let mut dense = sparse;
    for event in [
        event(0, 0, 0, 4),
        event(0, 1, 7, 2),
        event(0, 2, 7, 1),
        event(0, 3, 29, 12),
        event(0, 4, 201, 3),
    ] {
        sparse.record(event).unwrap();
        dense.record(event).unwrap();
        for offset in [1000, 3, 0, 99, 1, 25, 0, 1000] {
            dense.level(time(event.at.as_nanos() + offset)).unwrap();
        }
        assert_eq!(dense, sparse);
    }
    for at in [201, 202, 210, 250, 1000, u64::MAX, 201] {
        assert_eq!(dense.level(time(at)), sparse.level(time(at)));
    }
}

#[test]
fn exact_credit_survives_fractional_decay_and_equivalent_event_partitioning() {
    let mut sparse = activity(0, 0, 97, 11);
    sparse.record(event(0, 0, 0, 11)).unwrap();
    let mut dense = activity(0, 0, 97, 11);
    for sequence in 0..11 {
        dense.record(event(0, sequence, 0, 1)).unwrap();
    }
    for at in 1..=110 {
        dense.record(event(0, at + 10, at, 0)).unwrap();
        assert_eq!(dense.level(time(at)), sparse.level(time(at)));
    }
    assert_eq!(dense.level(time(110)).unwrap(), Level::ZERO);
}

#[test]
fn streamed_levels_match_a_small_exact_integer_reference() {
    // Deliberately nonbinary capacities and decay periods exercise conversion
    // as well as decay, saturation, and zero-input observations. Small reference
    // credit lets the independent f64-bit comparison use simple u128 products.
    for capacity in [1_u64, 3, 7, 11, 64] {
        for decay in [1_u64, 7, 23, 97, 128] {
            let mut stream = activity(0, 0, decay, capacity);
            let maximum = capacity * decay;
            let mut credit = 0_u64;
            let mut previous_time = 0_u64;
            let mut state = 731_u64;
            for sequence in 0..100 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let at = previous_time + state % 13;
                let units = (state >> 32) % (2 * capacity);
                credit = (credit.saturating_sub((at - previous_time) * capacity) + units * decay)
                    .min(maximum);
                stream.record(event(0, sequence, at, units)).unwrap();
                for offset in [0, 1, 3, 10, 1000] {
                    let remaining = credit.saturating_sub(offset * capacity);
                    let value = stream.level(time(at + offset)).unwrap().value();
                    if remaining == 0 {
                        assert_eq!(value, 0.0);
                    } else if remaining == maximum {
                        assert_eq!(value, 1.0);
                    } else {
                        assert!(represented_ratio_cmp(value, remaining, maximum).is_le());
                        let next = f64::from_bits(value.to_bits() + 1);
                        assert!(represented_ratio_cmp(next, remaining, maximum).is_gt());
                    }
                }
                previous_time = at;
            }
        }
    }
}

#[test]
fn extreme_units_periods_times_and_ids_do_not_overflow_or_wrap() {
    let mut stream = activity(u64::MAX - 1, 0, u64::MAX, u64::MAX);
    stream.record(event(u64::MAX - 1, 0, 0, u64::MAX)).unwrap();
    assert_eq!(stream.level(HostTime::ZERO).unwrap(), Level::MAX);
    assert_eq!(
        stream.level(time(1)).unwrap().value(),
        f64::from_bits(1.0_f64.to_bits() - 1)
    );
    // Addition of two near-u128::MAX credits must saturate, never wrap.
    stream.record(event(u64::MAX - 1, 1, 0, u64::MAX)).unwrap();
    assert_eq!(stream.level(HostTime::ZERO).unwrap(), Level::MAX);
    assert_eq!(
        stream.level(time(u64::MAX - 1)).unwrap().value(),
        f64::from_bits((1023 - 64) << 52)
    );
    assert_eq!(stream.level(HostTime::MAX).unwrap(), Level::ZERO);
    stream.reset(u64::MAX, HostTime::MAX).unwrap();
    stream
        .record(event(u64::MAX, u64::MAX, u64::MAX, u64::MAX))
        .unwrap();
    assert_eq!(stream.level(HostTime::MAX).unwrap(), Level::MAX);
    let maxed = stream;
    assert_eq!(
        stream.reset(0, HostTime::MAX),
        Err(AdapterError::NonIncreasingGeneration)
    );
    assert_eq!(
        stream.record(event(u64::MAX, 0, u64::MAX, 1)),
        Err(AdapterError::NonIncreasingSequence)
    );
    assert_eq!(stream, maxed);

    let mut smallest = activity(0, 0, 1, 1);
    smallest.record(event(0, 0, 0, u64::MAX)).unwrap();
    assert_eq!(smallest.level(time(0)).unwrap(), Level::MAX);
    assert_eq!(smallest.level(time(1)).unwrap(), Level::ZERO);
    assert_eq!(smallest.level(HostTime::MAX).unwrap(), Level::ZERO);

    // Near-128-bit denominators and tiny remaining credit stay finite/nonzero.
    let mut tiny = activity(0, 0, u64::MAX, u64::MAX - 1);
    tiny.record(event(0, 0, 0, 1)).unwrap();
    assert_eq!(
        tiny.level(time(1)).unwrap().value(),
        f64::from_bits((1023 - 128) << 52)
    );
    assert_eq!(tiny.level(time(2)).unwrap(), Level::ZERO);
}

#[test]
fn adapter_outputs_feed_existing_designs_without_mutating_source_values() {
    let palette = Palette::default();
    let profile = Profile::quiet();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    let design = AssemblyProgress::new();
    for (completed, total) in [(0, 1), (1, 3), (2, 3), (u64::MAX - 1, u64::MAX), (1, 1)] {
        let counts = CountedProgress::new(completed, Some(total)).unwrap();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 3));
        design
            .sample(&context, counts.progress())
            .render(buffer.area, &mut buffer);
        let filled = (2..22).filter(|&x| buffer[(x, 1)].symbol() == "#").count();
        assert!(filled as u128 <= u128::from(completed) * 20 / u128::from(total));
        assert_eq!(counts.completed(), completed);
    }
    let mut stream = activity(0, 0, 100, 10);
    stream.record(event(0, 0, 0, 5)).unwrap();
    let before = stream;
    let embers = TelemetryEmbers::new();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 8));
    embers
        .sample(&context, stream.level(time(50)).unwrap())
        .render(buffer.area, &mut buffer);
    assert_eq!(stream, before);
}

#[test]
fn all_adapter_errors_are_readable_standard_errors() {
    fn message(error: &dyn std::error::Error) -> String {
        error.to_string()
    }
    for error in [
        AdapterError::CountExceedsTotal,
        AdapterError::ZeroCapacity,
        AdapterError::WrongGeneration,
        AdapterError::NonIncreasingSequence,
        AdapterError::BackwardsTime,
        AdapterError::NonIncreasingGeneration,
    ] {
        assert!(!message(&error).is_empty());
    }
}
