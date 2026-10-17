//! Independent public-API composition checks. No catalogue or host loop is used.
use spark_runtime::{
    ChangeHint, ClipTime, HostTime, Period, Playback, PlaybackRate, RepaintPlanner,
    composition::{
        ClipTrack, CompositionError, Easing, MAX_PARTS, Parallel, Phase, Sequence, Transition,
        WEIGHT_SCALE,
    },
};
use std::time::Duration;

fn period(nanos: u64) -> Period {
    Period::new(Duration::from_nanos(nanos)).unwrap()
}

fn time(nanos: u64) -> ClipTime {
    ClipTime::from_nanos(nanos)
}

#[test]
fn composition_counts_and_checked_duration_ranges_are_enforced() {
    assert_eq!(MAX_PARTS, 256);
    assert!(matches!(Sequence::new(&[]), Err(CompositionError::Empty)));
    assert!(matches!(Parallel::new(&[]), Err(CompositionError::Empty)));
    let durations = [period(1); MAX_PARTS + 1];
    assert!(Sequence::new(&durations[..MAX_PARTS]).is_ok());
    assert!(matches!(
        Sequence::new(&durations),
        Err(CompositionError::TooManyParts)
    ));
    assert!(matches!(
        Sequence::new(&[period(u64::MAX), period(1)]),
        Err(CompositionError::TimeOverflow)
    ));
    assert!(Sequence::new(&[period(u64::MAX - 1), period(1)]).is_ok());
    assert!(matches!(
        ClipTrack::new(ClipTime::MAX, period(1)),
        Err(CompositionError::TimeOverflow)
    ));
    assert!(matches!(
        ClipTrack::new(time(1), period(u64::MAX)),
        Err(CompositionError::TimeOverflow)
    ));
    let track = ClipTrack::new(time(u64::MAX - 1), period(1)).unwrap();
    let tracks = [track; MAX_PARTS + 1];
    assert!(Parallel::new(&tracks[..MAX_PARTS]).is_ok());
    assert!(matches!(
        Parallel::new(&tracks),
        Err(CompositionError::TooManyParts)
    ));
}

#[test]
fn finite_sequence_selects_directly_at_every_seam_and_holds_its_terminal_pose() {
    let durations = [period(2), period(5), period(1), period(11)];
    let sequence = Sequence::new(&durations).unwrap();
    assert_eq!(sequence.duration(), Duration::from_nanos(19));
    let cases = [
        (u64::MAX, 3, 8, 19, 11, true),
        (0, 0, 0, 2, 0, false),
        (19, 3, 8, 19, 11, true),
        (7, 2, 7, 8, 0, false),
        (1, 0, 0, 2, 1, false),
        (8, 3, 8, 19, 0, false),
        (6, 1, 2, 7, 4, false),
        (18, 3, 8, 19, 10, false),
        (2, 1, 2, 7, 0, false),
        (0, 0, 0, 2, 0, false),
    ];
    for (now, index, start, end, local, complete) in cases {
        let sample = sequence.sample(time(now));
        assert_eq!(sample.index(), index);
        assert_eq!(sample.start(), time(start));
        assert_eq!(sample.end(), time(end));
        assert_eq!(sample.local_time(), time(local));
        assert_eq!(sample.complete(), complete);
        assert_eq!(sample.next_boundary(), (!complete).then_some(time(end)));
        if let Some(boundary) = sample.next_boundary() {
            assert!(boundary > time(now));
        }
    }
}

#[test]
fn maximum_sequence_end_is_representable_and_never_wraps_or_reloops() {
    let durations = [period(1), period(u64::MAX - 2), period(1)];
    let sequence = Sequence::new(&durations).unwrap();
    assert_eq!(sequence.duration(), Duration::from_nanos(u64::MAX));
    let before = sequence.sample(time(u64::MAX - 1));
    assert_eq!(before.index(), 2);
    assert_eq!(before.local_time(), ClipTime::ZERO);
    assert_eq!(before.next_boundary(), Some(ClipTime::MAX));
    assert_eq!(before.map_deadline(ClipTime::MAX), Some(ClipTime::MAX));
    let final_sample = sequence.sample(ClipTime::MAX);
    assert_eq!(final_sample.index(), 2);
    assert_eq!(final_sample.local_time(), time(1));
    assert!(final_sample.complete());
    assert_eq!(final_sample.next_boundary(), None);
    assert_eq!(final_sample.map_deadline(ClipTime::MAX), None);
    let single_duration = [period(u64::MAX)];
    let single = Sequence::new(&single_duration).unwrap();
    assert_eq!(
        single.sample(ClipTime::ZERO).map_deadline(ClipTime::MAX),
        Some(ClipTime::MAX)
    );
    assert_eq!(single.sample(ClipTime::MAX).local_time(), ClipTime::MAX);
}

#[test]
fn sequence_child_deadlines_are_local_future_clamped_and_separate_from_cadence() {
    let durations = [period(4), period(7), period(2)];
    let sequence = Sequence::new(&durations).unwrap();
    let sample = sequence.sample(time(6));
    assert_eq!(sample.local_time(), time(2));
    assert_eq!(sample.map_deadline(time(0)), None);
    assert_eq!(sample.map_deadline(time(1)), None);
    assert_eq!(sample.map_deadline(time(2)), None);
    assert_eq!(sample.map_deadline(time(3)), Some(time(7)));
    assert_eq!(sample.map_deadline(time(7)), Some(time(11)));
    assert_eq!(sample.map_deadline(ClipTime::MAX), Some(time(11)));
    assert_eq!(sample.next_boundary(), Some(time(11)));

    // A local discrete boundary is translated before playback mapping. A child
    // Continuous hint remains a host duration and is planned independently.
    let mut playback = Playback::new(HostTime::ZERO);
    playback
        .set_rate(HostTime::ZERO, PlaybackRate::new(2, 1).unwrap())
        .unwrap();
    let planner = RepaintPlanner::new(Duration::from_nanos(1));
    let now = HostTime::from_nanos(3);
    let seam = planner.next_deadline(
        now,
        now,
        &playback,
        ChangeHint::At(sample.next_boundary().unwrap()),
    );
    let cadence = planner.next_deadline(
        now,
        now,
        &playback,
        ChangeHint::Continuous(Duration::from_nanos(10)),
    );
    assert_eq!(seam, Some(HostTime::from_nanos(6)));
    assert_eq!(cadence, Some(HostTime::from_nanos(13)));
    assert_eq!(RepaintPlanner::earliest([seam, cadence]), seam);
}

#[test]
fn delayed_track_phases_do_not_confuse_not_started_and_terminal_local_time() {
    let track = ClipTrack::new(time(7), period(5)).unwrap();
    assert_eq!(track.start(), time(7));
    assert_eq!(track.end(), time(12));
    assert_eq!(track.duration(), period(5));
    for (now, phase, local, boundary) in [
        (12, Phase::Finished, 5, None),
        (0, Phase::Before, 0, Some(7)),
        (7, Phase::Active, 0, Some(12)),
        (u64::MAX, Phase::Finished, 5, None),
        (6, Phase::Before, 0, Some(7)),
        (11, Phase::Active, 4, Some(12)),
        (8, Phase::Active, 1, Some(12)),
    ] {
        let sample = track.sample(time(now));
        assert_eq!(sample.phase(), phase);
        assert_eq!(sample.start(), time(7));
        assert_eq!(sample.end(), time(12));
        assert_eq!(sample.local_time(), time(local));
        assert_eq!(sample.next_boundary(), boundary.map(time));
        if let Some(boundary) = sample.next_boundary() {
            assert!(boundary > time(now));
        }
    }
    let from_zero = ClipTrack::new(ClipTime::ZERO, period(u64::MAX)).unwrap();
    assert_eq!(from_zero.sample(ClipTime::ZERO).phase(), Phase::Active);
    assert_eq!(from_zero.sample(time(u64::MAX - 1)).phase(), Phase::Active);
    assert_eq!(from_zero.sample(ClipTime::MAX).phase(), Phase::Finished);
}

#[test]
fn track_deadlines_are_absent_outside_active_time_and_do_not_overflow() {
    let track = ClipTrack::new(time(u64::MAX - 9), period(9)).unwrap();
    assert_eq!(
        track.sample(ClipTime::ZERO).map_deadline(ClipTime::MAX),
        None
    );
    let active = track.sample(time(u64::MAX - 5));
    assert_eq!(active.local_time(), time(4));
    assert_eq!(active.map_deadline(time(3)), None);
    assert_eq!(active.map_deadline(time(4)), None);
    assert_eq!(active.map_deadline(time(5)), Some(time(u64::MAX - 4)));
    assert_eq!(active.map_deadline(time(9)), Some(ClipTime::MAX));
    assert_eq!(active.map_deadline(ClipTime::MAX), Some(ClipTime::MAX));
    assert_eq!(
        track.sample(ClipTime::MAX).map_deadline(ClipTime::MAX),
        None
    );
}

#[test]
fn parallel_keeps_independent_caller_order_and_wakes_pending_starts() {
    let tracks = [
        ClipTrack::new(time(9), period(2)).unwrap(),
        ClipTrack::new(time(0), period(3)).unwrap(),
        ClipTrack::new(time(5), period(8)).unwrap(),
        ClipTrack::new(time(5), period(1)).unwrap(),
    ];
    let parallel = Parallel::new(&tracks).unwrap();
    assert_eq!(parallel.duration(), Duration::from_nanos(13));
    for now in [u64::MAX, 13, 0, 3, 5, 6, 8, 9, 10, 11, 12, 2, 0] {
        let mut samples = parallel.sample(time(now));
        assert_eq!(samples.len(), tracks.len());
        for (index, track) in tracks.iter().enumerate() {
            assert_eq!(samples.next(), Some(track.sample(time(now))));
            assert_eq!(samples.len(), tracks.len() - index - 1);
        }
        assert_eq!(samples.next(), None);
        let expected = tracks
            .iter()
            .flat_map(|track| [track.start(), track.end()])
            .filter(|boundary| *boundary > time(now))
            .min();
        assert_eq!(parallel.next_boundary(time(now)), expected);
    }
    // Every existing visual is finished or not started in this gap, but the
    // next delayed start still needs a host wake.
    assert_eq!(parallel.next_boundary(time(4)), Some(time(5)));
}

#[test]
fn one_tick_tracks_and_simultaneous_boundaries_have_exact_phase_edges() {
    let tracks = [
        ClipTrack::new(time(0), period(1)).unwrap(),
        ClipTrack::new(time(1), period(1)).unwrap(),
        ClipTrack::new(time(1), period(1)).unwrap(),
        ClipTrack::new(time(u64::MAX - 1), period(1)).unwrap(),
    ];
    let parallel = Parallel::new(&tracks).unwrap();
    assert_eq!(parallel.duration(), Duration::from_nanos(u64::MAX));
    assert_eq!(parallel.next_boundary(ClipTime::ZERO), Some(time(1)));
    assert_eq!(parallel.next_boundary(time(1)), Some(time(2)));
    assert_eq!(parallel.next_boundary(time(2)), Some(time(u64::MAX - 1)));
    assert_eq!(
        parallel.next_boundary(time(u64::MAX - 1)),
        Some(ClipTime::MAX)
    );
    assert_eq!(parallel.next_boundary(ClipTime::MAX), None);
    assert_eq!(tracks[0].sample(time(1)).phase(), Phase::Finished);
    assert_eq!(tracks[1].sample(time(1)).phase(), Phase::Active);
}

#[test]
fn easing_has_known_integer_landmarks_and_exact_time_based_completion() {
    assert_eq!(WEIGHT_SCALE, 1_000_000);
    let duration = period(400);
    let linear = Transition::new(duration);
    let smooth = linear.with_easing(Easing::SmoothStep);
    assert_eq!(linear.duration(), duration);
    assert_eq!(smooth.duration(), duration);
    for (now, linear_weight, smooth_weight) in [
        (400, 1_000_000, 1_000_000),
        (0, 0, 0),
        (300, 750_000, 843_750),
        (100, 250_000, 156_250),
        (200, 500_000, 500_000),
        (u64::MAX, 1_000_000, 1_000_000),
    ] {
        assert_eq!(linear.sample(time(now)).weight(), linear_weight);
        assert_eq!(smooth.sample(time(now)).weight(), smooth_weight);
        assert_eq!(linear.sample(time(now)).complete(), now >= 400);
        assert_eq!(smooth.sample(time(now)).complete(), now >= 400);
    }
    for easing in [Easing::Linear, Easing::SmoothStep] {
        let tiny = Transition::new(period(1)).with_easing(easing);
        assert_eq!(tiny.sample(ClipTime::ZERO).weight(), 0);
        assert!(!tiny.sample(ClipTime::ZERO).complete());
        assert_eq!(tiny.sample(time(1)).weight(), WEIGHT_SCALE);
        assert!(tiny.sample(time(1)).complete());
        let long = Transition::new(period(u64::MAX)).with_easing(easing);
        let almost = long.sample(time(u64::MAX - 1));
        assert!(!almost.complete());
        assert!(almost.weight() < WEIGHT_SCALE);
        assert!(long.sample(ClipTime::MAX).complete());
        assert_eq!(long.sample(ClipTime::MAX).weight(), WEIGHT_SCALE);
    }
}

#[test]
fn integer_interpolation_is_bounded_monotone_and_exact_at_both_endpoints() {
    for easing in [Easing::Linear, Easing::SmoothStep] {
        let transition = Transition::new(period(1000)).with_easing(easing);
        for (from, to) in [
            (0, u16::MAX),
            (u16::MAX, 0),
            (1, 2),
            (2, 1),
            (1700, 1700),
            (217, 60_001),
        ] {
            let mut previous = from;
            let mut previous_weight = 0;
            for now in 0..=1001 {
                let sample = transition.sample(time(now));
                let value = sample.interpolate(from, to);
                assert!((from.min(to)..=from.max(to)).contains(&value));
                assert!(sample.weight() >= previous_weight);
                assert!(if from <= to {
                    value >= previous
                } else {
                    value <= previous
                });
                assert_eq!(
                    u32::from(value) + u32::from(sample.interpolate(to, from)),
                    u32::from(from) + u32::from(to)
                );
                assert_eq!(sample.complete(), now >= 1000);
                previous = value;
                previous_weight = sample.weight();
            }
            assert_eq!(
                transition.sample(ClipTime::ZERO).interpolate(from, to),
                from
            );
            assert_eq!(transition.sample(time(1000)).interpolate(from, to), to);
            assert_eq!(transition.sample(ClipTime::MAX).interpolate(from, to), to);
        }
    }
    let midpoint = Transition::new(period(2)).sample(time(1));
    assert_eq!(midpoint.interpolate(0, 1), 0);
    assert_eq!(midpoint.interpolate(1, 0), 1);
}

#[test]
fn generated_direct_seeks_match_a_bounded_reference_without_history() {
    let durations = [period(1), period(17), period(2), period(31), period(1009)];
    let sequence = Sequence::new(&durations).unwrap();
    let ends = [1_u64, 18, 20, 51, 1060];
    let mut random = 0xc86d_9381_4f27_601b_u64;
    for _ in 0..4096 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let now = if random & 15 == 0 {
            random
        } else {
            random % 1200
        };
        let expected_index = ends
            .iter()
            .position(|end| now < *end)
            .unwrap_or(ends.len() - 1);
        let expected_start = if expected_index == 0 {
            0
        } else {
            ends[expected_index - 1]
        };
        let sample = sequence.sample(time(now));
        assert_eq!(sample.index(), expected_index);
        assert_eq!(sample.start(), time(expected_start));
        assert_eq!(sample.end(), time(ends[expected_index]));
        assert_eq!(
            sample.local_time(),
            time(
                now.saturating_sub(expected_start)
                    .min(durations[expected_index].as_nanos())
            )
        );
        assert_eq!(sample.complete(), now >= 1060);
        // Unrelated reads at either extreme must never advance the sequence.
        sequence.sample(ClipTime::MAX);
        sequence.sample(ClipTime::ZERO);
        assert_eq!(sequence.sample(time(now)), sample);
    }
}
