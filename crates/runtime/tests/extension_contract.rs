//! Independent public-runtime invariants shared by the breadth-stage designs.
use std::time::Duration;

use ratatui_core::{
    buffer::{Buffer, Cell, CellDiffOption},
    layout::Rect,
    style::{Color, Modifier, Style},
};
use spark_runtime::{
    ClipTime, Period, TimeError,
    surface::{CellOp, Glyph, OpaqueSurface},
};

fn host_buffer() -> Buffer {
    let mut cell = Cell::new("h");
    cell.set_style(
        Style::new()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .underline_color(Color::Magenta)
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
    );
    let mut buffer = Buffer::filled(Rect::new(7, 11, 18, 3), cell);
    for y in 11..14 {
        buffer.set_string(8, y, "界e\u{0301}界", Style::new());
        buffer[(15, y)]
            .set_symbol("\x1b[31mreserved\x1b[0m")
            .set_diff_option(CellDiffOption::ForcedWidth(3.try_into().unwrap()));
        buffer[(19, y)].set_diff_option(CellDiffOption::AlwaysUpdate);
        buffer[(21, y)].set_diff_option(CellDiffOption::Skip);
    }
    buffer
}

fn replacement() -> CellOp {
    CellOp::Replace(
        Glyph::new('*').unwrap(),
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )
}

#[test]
fn opaque_scan_order_cannot_change_wide_footprints_or_protection() {
    let protected = [Rect::new(22, 11, 2, 3), Rect::new(10, 12, 1, 1)];
    let before = host_buffer();
    // The request clips off the leading cell of the first wide glyph. Each row
    // also includes a combining grapheme and an escaped forced-width footprint.
    let requested = Rect::new(9, 10, 18, 5);
    let mut coordinates: Vec<_> = (0..requested.height)
        .flat_map(|y| (0..requested.width).map(move |x| (x, y)))
        .collect();
    let mut expected = before.clone();
    for y in 11..14 {
        for x in [10, 13, 14, 18, 20, 24] {
            if x == 10 && y == 12 {
                continue;
            }
            expected[(x, y)].reset();
            expected[(x, y)]
                .set_char('*')
                .set_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        }
    }
    for pass in 0..8 {
        let mut rendered = before.clone();
        let mut surface = OpaqueSurface::new(&mut rendered, requested, &protected);
        for &(x, y) in &coordinates {
            surface.write(x, y, replacement());
            // Repeated writes must not invalidate the row-prefix cache.
            surface.write(x, y, replacement());
        }
        assert_eq!(rendered, expected, "write permutation {pass}");
        coordinates.reverse();
        coordinates.rotate_left(7 + pass);
    }
}

#[test]
fn maximum_coordinates_use_checked_offsets_without_wrapping_into_host_cells() {
    let requested = Rect {
        x: u16::MAX - 2,
        y: u16::MAX - 2,
        width: u16::MAX,
        height: u16::MAX,
    };
    let mut buffer = host_buffer();
    let before = buffer.clone();
    let protected = [Rect {
        x: u16::MAX,
        y: u16::MAX,
        width: u16::MAX,
        height: u16::MAX,
    }];
    let mut surface = OpaqueSurface::new(&mut buffer, requested, &protected);
    for coordinate in [0, 1, 2, 3, 12, 100, u16::MAX - 1, u16::MAX] {
        assert!(!surface.write(coordinate, coordinate, replacement()));
        assert!(!surface.write(coordinate, coordinate, CellOp::Erase(Style::new())));
    }
    assert_eq!(buffer, before);
}

#[test]
fn duration_conversion_accepts_exact_limit_and_rejects_next_nanosecond() {
    let last = Duration::from_nanos(u64::MAX);
    assert_eq!(ClipTime::try_from(last), Ok(ClipTime::MAX));
    assert_eq!(Period::new(last).unwrap().as_nanos(), u64::MAX);
    let overflow = last + Duration::from_nanos(1);
    assert_eq!(ClipTime::try_from(overflow), Err(TimeError::TooLarge));
    assert_eq!(Period::new(overflow), Err(TimeError::TooLarge));
    assert_eq!(ClipTime::saturating_from_duration(overflow), ClipTime::MAX);
}

#[test]
fn period_step_is_an_exact_direct_lookup_for_extreme_and_shuffled_inputs() {
    let mut seed = 0xc0f5_caba_9134_ef21_u64;
    for period_nanos in [1, 2, 3, 997, 1_600_000_001, u64::MAX / 2, u64::MAX] {
        let period = Period::new(Duration::from_nanos(period_nanos)).unwrap();
        for steps in [0, 1, 2, 257, 65_535, u32::MAX] {
            for _ in 0..64 {
                seed = seed
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let timestamp = ClipTime::from_nanos(seed);
                let expected = (u128::from(seed % period_nanos) * u128::from(steps))
                    / u128::from(period_nanos);
                assert_eq!(u128::from(period.step(timestamp, steps)), expected);
                assert_eq!(period.phase_nanos(timestamp), seed % period_nanos);
                assert_eq!(
                    period.step(timestamp, steps),
                    period.step(ClipTime::from_nanos(seed % period_nanos), steps)
                );
            }
        }
    }
}

#[test]
fn style_permissions_are_an_exact_projection_of_requested_fields() {
    use spark_runtime::surface::{PaintMask, StylePermissions, StyleSurface};

    let original = host_buffer();
    let patch = Style::new()
        .fg(Color::Red)
        .bg(Color::Green)
        .underline_color(Color::White)
        .add_modifier(Modifier::BOLD)
        .remove_modifier(Modifier::ITALIC);
    for bits in 0..16 {
        let permissions = StylePermissions {
            foreground: bits & 1 != 0,
            background: bits & 2 != 0,
            underline_color: bits & 4 != 0,
            modifiers: bits & 8 != 0,
        };
        let mut buffer = original.clone();
        let mut surface = StyleSurface::new(
            &mut buffer,
            original.area,
            &[],
            PaintMask::Region,
            permissions,
        );
        // A one-column combining grapheme must retain its complete symbol.
        assert!(surface.patch(3, 0, patch));
        let mut expected = original[(10, 11)].clone();
        if permissions.foreground {
            expected.fg = Color::Red;
        }
        if permissions.background {
            expected.bg = Color::Green;
        }
        if permissions.underline_color {
            expected.underline_color = Color::White;
        }
        if permissions.modifiers {
            expected.modifier.remove(Modifier::ITALIC);
            expected.modifier.insert(Modifier::BOLD);
        }
        assert_eq!(buffer[(10, 11)], expected, "permission bits {bits}");
        for y in 11..14 {
            for x in 7..25 {
                if (x, y) != (10, 11) {
                    assert_eq!(buffer[(x, y)], original[(x, y)]);
                }
            }
        }
    }
}

#[test]
fn style_surface_preserves_wide_special_protected_and_unrequested_cells() {
    use spark_runtime::surface::{PaintMask, StylePermissions, StyleSurface};

    let before = host_buffer();
    let requested = Rect::new(9, 10, 18, 5);
    let protected = [Rect::new(22, 11, 2, 3), Rect::new(10, 12, 1, 1)];
    let patch = Style::new()
        .fg(Color::Red)
        .bg(Color::Green)
        .underline_color(Color::White)
        .add_modifier(Modifier::BOLD)
        .remove_modifier(Modifier::ITALIC);
    let mut buffer = before.clone();
    let mut surface = StyleSurface::new(
        &mut buffer,
        requested,
        &protected,
        PaintMask::Region,
        StylePermissions::foreground(),
    );
    for y in [4, 1, 2, 3, 0] {
        for x in (0..18).rev() {
            surface.patch(x, y, patch);
        }
    }
    for y in 11..14 {
        for x in 7..25 {
            let mut expected = before[(x, y)].clone();
            if [10, 13, 14, 18, 20, 24].contains(&x) && (x, y) != (10, 12) {
                expected.fg = Color::Red;
            }
            assert_eq!(buffer[(x, y)], expected, "cell ({x},{y})");
        }
    }
}

#[test]
fn border_masks_stay_relative_to_requested_geometry_after_clipping() {
    use spark_runtime::surface::{MaskedSurface, PaintMask, StylePermissions, StyleSurface};

    let actual = Rect::new(10, 20, 5, 4);
    let requested = Rect::new(8, 18, 6, 5);
    let original = Buffer::filled(actual, Cell::new("h"));
    let protected = [Rect::new(13, 21, 1, 1)];
    let mut glyphs = original.clone();
    let mut styles = original.clone();
    let mut glyph_surface =
        MaskedSurface::new(&mut glyphs, requested, &protected, PaintMask::Border);
    let mut style_surface = StyleSurface::new(
        &mut styles,
        requested,
        &protected,
        PaintMask::Border,
        StylePermissions::foreground(),
    );
    for y in 0..requested.height {
        for x in 0..requested.width {
            glyph_surface.write(x, y, replacement());
            style_surface.patch(x, y, Style::new().fg(Color::Cyan));
        }
    }
    for y in 20..24 {
        for x in 10..15 {
            let allowed = x <= 13 && y <= 22 && (x == 13 || y == 22) && (x, y) != (13, 21);
            if allowed {
                assert_eq!(glyphs[(x, y)].symbol(), "*");
                assert_eq!(styles[(x, y)].fg, Color::Cyan);
                assert_eq!(styles[(x, y)].symbol(), "h");
            } else {
                assert_eq!(glyphs[(x, y)], original[(x, y)]);
                assert_eq!(styles[(x, y)], original[(x, y)]);
            }
        }
    }
}

#[test]
fn finite_clips_clamp_and_finish_exactly_once_in_time_without_state() {
    use spark_runtime::FiniteClip;

    for duration in [1, 7, 1_200_000_000, u64::MAX] {
        let clip = FiniteClip::new(Period::new(Duration::from_nanos(duration)).unwrap());
        for time in [u64::MAX, 0, duration - 1, duration, 1, 0] {
            let sampled = clip.sample(ClipTime::from_nanos(time));
            assert_eq!(sampled.time.as_nanos(), time.min(duration));
            assert_eq!(sampled.complete, time >= duration);
            for steps in [0, 1, 17, u32::MAX] {
                let expected =
                    u128::from(time.min(duration)) * u128::from(steps) / u128::from(duration);
                assert_eq!(
                    u128::from(clip.step(ClipTime::from_nanos(time), steps)),
                    expected
                );
            }
        }
    }
}

#[test]
fn frame_sequences_validate_before_sampling() {
    use spark_runtime::{FrameSequence, SpriteError};

    assert_eq!(FrameSequence::new(&[]), Err(SpriteError::FrameCount));
    for invalid in [&[0][..], &[1, 1], &[1, 0], &[8, 7], &[u64::MAX, 1]] {
        assert_eq!(
            FrameSequence::new(invalid),
            Err(SpriteError::InvalidDuration)
        );
    }
    let too_many: Vec<_> = (1..=257).collect();
    assert_eq!(FrameSequence::new(&too_many), Err(SpriteError::FrameCount));
    assert!(FrameSequence::new(&too_many[..256]).is_ok());
    assert!(FrameSequence::new(&[u64::MAX]).is_ok());
}

#[test]
fn unequal_holds_and_future_boundaries_remain_exact_across_cycles_and_max_time() {
    use spark_runtime::FrameSequence;

    let ends = [2, 5, 17, 18, 91];
    let sequence = FrameSequence::new(&ends).unwrap();
    assert_eq!(sequence.period().as_nanos(), 91);
    let mut times = vec![0, 1, 2, 4, 5, 16, 17, 18, 90, 91, 92, u64::MAX];
    for cycle in [0, 1, 2, 103, 1_000_000] {
        for end in ends {
            times.extend([cycle * 91 + end - 1, cycle * 91 + end, cycle * 91 + end + 1]);
        }
    }
    times.reverse();
    for time in times {
        let phase = time % 91;
        let index = ends.iter().position(|&end| phase < end).unwrap();
        let sample_time = ClipTime::from_nanos(time);
        assert_eq!(sequence.frame_index(sample_time), index, "time={time}");
        let boundary = time.checked_add(ends[index] - phase);
        assert_eq!(
            sequence.next_boundary(sample_time).map(ClipTime::as_nanos),
            boundary
        );
        if let Some(boundary) = boundary {
            assert!(boundary > time);
            assert_eq!(
                sequence.frame_index(ClipTime::from_nanos(boundary)),
                (index + 1) % ends.len()
            );
        }
    }
    let one_frame = FrameSequence::new(&[u64::MAX]).unwrap();
    assert_eq!(one_frame.next_boundary(ClipTime::ZERO), Some(ClipTime::MAX));
    assert_eq!(one_frame.next_boundary(ClipTime::MAX), None);
}

#[test]
fn playback_pause_rate_seek_restart_and_hidden_modes_preserve_current_pose() {
    use spark_runtime::{HiddenPolicy, HostTime, Playback, PlaybackRate};
    let host = HostTime::from_nanos;
    let mut playback = Playback::new(host(100));
    assert_eq!(playback.sample(host(150)).as_nanos(), 50);
    playback
        .set_rate(host(150), PlaybackRate::new(3, 2).unwrap())
        .unwrap();
    assert_eq!(playback.sample(host(150)).as_nanos(), 50);
    assert_eq!(playback.sample(host(170)).as_nanos(), 80);
    playback.pause(host(170)).unwrap();
    assert!(!playback.requests_animation());
    assert_eq!(playback.sample(host(500)).as_nanos(), 80);
    playback.resume(host(500)).unwrap();
    assert_eq!(playback.sample(host(510)).as_nanos(), 95);
    playback.set_hidden(host(510), true).unwrap();
    assert!(!playback.requests_animation());
    assert_eq!(playback.sample(host(900)).as_nanos(), 95);
    playback.set_hidden(host(900), false).unwrap();
    assert_eq!(playback.sample(host(910)).as_nanos(), 110);
    playback
        .set_hidden_policy(host(910), HiddenPolicy::CatchUp)
        .unwrap();
    playback.set_hidden(host(910), true).unwrap();
    assert_eq!(playback.sample(host(930)).as_nanos(), 140);
    assert!(!playback.requests_animation());
    playback.set_hidden(host(930), false).unwrap();
    assert_eq!(playback.sample(host(930)).as_nanos(), 140);
    playback.seek(host(940), ClipTime::from_nanos(7)).unwrap();
    assert_eq!(playback.sample(host(940)).as_nanos(), 7);
    playback.restart(host(950)).unwrap();
    assert_eq!(playback.sample(host(950)), ClipTime::ZERO);
    assert!(playback.is_playing());
}

#[test]
fn backwards_playback_commands_reject_without_mutating_any_state() {
    use spark_runtime::{HiddenPolicy, HostTime, Playback, PlaybackError, PlaybackRate};
    let mut playback = Playback::new(HostTime::from_nanos(100));
    playback
        .set_rate(HostTime::from_nanos(110), PlaybackRate::new(5, 3).unwrap())
        .unwrap();
    let before = playback;
    let earlier = HostTime::from_nanos(109);
    let expected = Err(PlaybackError::BackwardsHostTime);
    assert_eq!(playback.pause(earlier), expected);
    assert_eq!(playback, before);
    assert_eq!(playback.resume(earlier), expected);
    assert_eq!(playback, before);
    assert_eq!(playback.seek(earlier, ClipTime::MAX), expected);
    assert_eq!(playback, before);
    assert_eq!(playback.restart(earlier), expected);
    assert_eq!(playback, before);
    assert_eq!(
        playback.set_rate(earlier, PlaybackRate::new(1, 7).unwrap()),
        expected
    );
    assert_eq!(playback, before);
    assert_eq!(playback.set_hidden(earlier, true), expected);
    assert_eq!(playback, before);
    assert_eq!(
        playback.set_hidden_policy(earlier, HiddenPolicy::CatchUp),
        expected
    );
    assert_eq!(playback, before);
    assert_eq!(PlaybackRate::new(0, 1), Err(PlaybackError::ZeroRate));
    assert_eq!(PlaybackRate::new(1, 0), Err(PlaybackError::ZeroRate));
}

#[test]
fn redundant_playback_commands_do_not_discard_fractional_rate_progress() {
    use spark_runtime::{HiddenPolicy, HostTime, Playback, PlaybackRate};
    let rate = PlaybackRate::new(1, 3).unwrap();
    for command in 0..4 {
        let mut playback = Playback::new(HostTime::ZERO);
        playback.set_rate(HostTime::ZERO, rate).unwrap();
        for elapsed in 1..=30 {
            let host = HostTime::from_nanos(elapsed);
            match command {
                0 => playback.resume(host).unwrap(),
                1 => playback.set_hidden(host, false).unwrap(),
                2 => playback.set_rate(host, rate).unwrap(),
                _ => playback
                    .set_hidden_policy(host, HiddenPolicy::Freeze)
                    .unwrap(),
            }
            assert_eq!(
                playback.sample(host).as_nanos(),
                elapsed / 3,
                "command={command}, elapsed={elapsed}"
            );
        }
    }
}

#[test]
fn future_clip_deadlines_use_ceiling_division_from_the_anchor_phase() {
    use spark_runtime::{HostTime, Playback, PlaybackRate};
    for (numerator, denominator) in [(1, 3), (2, 3), (3, 2), (7, 1), (u32::MAX, u32::MAX - 1)] {
        let mut playback = Playback::new(HostTime::from_nanos(11));
        playback
            .set_rate(
                HostTime::from_nanos(11),
                PlaybackRate::new(numerator, denominator).unwrap(),
            )
            .unwrap();
        for now in [11, 12, 13, 14, 20, 30] {
            let current = playback.sample(HostTime::from_nanos(now)).as_nanos();
            for distance in [1, 2, 7, 33] {
                let target = ClipTime::from_nanos(current + distance);
                let deadline = playback
                    .host_for_clip(HostTime::from_nanos(now), target)
                    .unwrap();
                let expected = 11
                    + (u128::from(target.as_nanos()) * u128::from(denominator))
                        .div_ceil(u128::from(numerator));
                assert_eq!(u128::from(deadline.as_nanos()), expected);
                assert!(playback.sample(deadline) >= target);
                assert!(playback.sample(HostTime::from_nanos(deadline.as_nanos() - 1)) < target);
            }
        }
    }
}

#[test]
fn repaint_planning_has_no_inactive_wakes_and_no_early_or_nonfuture_deadline() {
    use spark_runtime::{ChangeHint, HostTime, Playback, PlaybackRate, RepaintPlanner};
    let host = HostTime::from_nanos;
    let planner = RepaintPlanner::new(Duration::from_nanos(10));
    let mut playback = Playback::new(HostTime::ZERO);
    playback
        .set_rate(HostTime::ZERO, PlaybackRate::new(1, 3).unwrap())
        .unwrap();
    let hint = ChangeHint::At(ClipTime::from_nanos(7));
    assert_eq!(
        planner.next_deadline(host(13), host(10), &playback, hint),
        Some(host(21))
    );
    assert_eq!(
        planner.next_deadline(host(30), host(29), &playback, hint),
        Some(host(39))
    );
    assert_eq!(
        planner.next_deadline(
            host(50),
            host(20),
            &playback,
            ChangeHint::Continuous(Duration::from_nanos(10))
        ),
        Some(host(51))
    );
    assert_eq!(
        planner.next_deadline(host(50), host(20), &playback, ChangeHint::None),
        None
    );
    playback.pause(host(50)).unwrap();
    for hint in [
        hint,
        ChangeHint::Continuous(Duration::from_nanos(1)),
        ChangeHint::None,
    ] {
        assert_eq!(
            planner.next_deadline(host(100), host(20), &playback, hint),
            None
        );
    }
    playback.resume(host(100)).unwrap();
    playback.set_hidden(host(100), true).unwrap();
    assert_eq!(
        planner.next_deadline(host(100), host(20), &playback, hint),
        None
    );
    playback.set_hidden(host(100), false).unwrap();
    assert_eq!(
        planner.next_deadline(HostTime::MAX, host(20), &playback, hint),
        None
    );
    assert_eq!(
        RepaintPlanner::earliest([None, Some(host(23)), Some(host(17)), None]),
        Some(host(17))
    );
    assert_eq!(RepaintPlanner::earliest([None, None]), None);
}

#[test]
fn playback_extreme_rates_and_time_saturate_instead_of_overflowing() {
    use spark_runtime::{HostTime, Playback, PlaybackRate};
    let mut playback = Playback::new(HostTime::ZERO);
    playback
        .set_rate(HostTime::ZERO, PlaybackRate::new(u32::MAX, 1).unwrap())
        .unwrap();
    assert_eq!(playback.sample(HostTime::MAX), ClipTime::MAX);
    playback.seek(HostTime::MAX, ClipTime::MAX).unwrap();
    assert_eq!(playback.sample(HostTime::MAX), ClipTime::MAX);
    let mut slow = Playback::new(HostTime::from_nanos(u64::MAX - 1));
    slow.set_rate(
        HostTime::from_nanos(u64::MAX - 1),
        PlaybackRate::new(1, u32::MAX).unwrap(),
    )
    .unwrap();
    assert_eq!(
        slow.host_for_clip(HostTime::from_nanos(u64::MAX - 1), ClipTime::from_nanos(1)),
        None
    );
}

#[test]
fn bounded_generated_playback_command_traces_preserve_transition_invariants() {
    use spark_runtime::{HiddenPolicy, HostTime, Playback, PlaybackError, PlaybackRate};
    let mut playback = Playback::new(HostTime::ZERO);
    let mut random = 0x81af_9809_248a_10d3_u64;
    let mut elapsed = 0;
    for _ in 0..4096 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        elapsed += 1 + random % 101;
        let now = HostTime::from_nanos(elapsed);
        let before = playback;
        let pose = playback.sample(now);
        let command = (random >> 24) % 8;
        match command {
            0 => playback.pause(now).unwrap(),
            1 => playback.resume(now).unwrap(),
            2 => playback
                .set_rate(
                    now,
                    PlaybackRate::new(1 + random as u32 % 7, 1 + (random >> 32) as u32 % 7)
                        .unwrap(),
                )
                .unwrap(),
            3 => playback.set_hidden(now, random & 1 != 0).unwrap(),
            4 => playback
                .set_hidden_policy(
                    now,
                    if random & 1 == 0 {
                        HiddenPolicy::Freeze
                    } else {
                        HiddenPolicy::CatchUp
                    },
                )
                .unwrap(),
            5 => playback
                .seek(now, ClipTime::from_nanos(random % 1_000_000))
                .unwrap(),
            6 => playback.restart(now).unwrap(),
            _ => {
                // Arbitrary reads, including an extreme future pose, cannot
                // change the anchor or legal command ordering.
                playback.sample(HostTime::ZERO);
                playback.sample(HostTime::MAX);
                assert_eq!(playback, before);
                playback.resume(now).unwrap();
            }
        }
        match command {
            5 => assert_eq!(playback.sample(now).as_nanos(), random % 1_000_000),
            6 => assert_eq!(playback.sample(now), ClipTime::ZERO),
            _ => assert_eq!(playback.sample(now), pose),
        }
        let committed = playback;
        assert_eq!(
            playback.pause(HostTime::from_nanos(elapsed - 1)),
            Err(PlaybackError::BackwardsHostTime)
        );
        assert_eq!(playback, committed);
        let later = playback.sample(HostTime::from_nanos(elapsed + 500));
        if !playback.is_playing()
            || (playback.is_hidden() && playback.hidden_policy() == HiddenPolicy::Freeze)
        {
            assert_eq!(later, playback.sample(now));
        } else {
            assert!(later >= playback.sample(now));
        }
    }
}

#[test]
fn equivalent_rational_rates_preserve_phase_when_alternated() {
    use spark_runtime::{HostTime, Playback, PlaybackRate};
    let third = PlaybackRate::new(1, 3).unwrap();
    let equivalents = [
        third,
        PlaybackRate::new(2, 6).unwrap(),
        PlaybackRate::new(3, 9).unwrap(),
        PlaybackRate::new(u32::MAX / 3, u32::MAX).unwrap(),
    ];
    for rate in equivalents {
        assert_eq!(rate, third);
        assert_eq!(rate.numerator(), 1);
        assert_eq!(rate.denominator(), 3);
    }
    assert_eq!(
        PlaybackRate::new(u32::MAX, u32::MAX).unwrap(),
        PlaybackRate::default()
    );
    let mut playback = Playback::new(HostTime::ZERO);
    playback.set_rate(HostTime::ZERO, third).unwrap();
    for elapsed in 1..=30 {
        let now = HostTime::from_nanos(elapsed);
        playback
            .set_rate(now, equivalents[elapsed as usize % equivalents.len()])
            .unwrap();
        assert_eq!(playback.sample(now).as_nanos(), elapsed / 3);
        assert_eq!(
            playback.host_for_clip(now, ClipTime::from_nanos(11)),
            Some(HostTime::from_nanos(33))
        );
    }
}
