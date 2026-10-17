//! Independent tests of public breadth-stage APIs on styled host content.
use std::time::Duration;

use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_catalog::{
    AssemblyProgress, BorderRunner, InputError, Level, PrismReveal, Progress, RevealError,
    SegmentedComet, TelemetryEmbers,
};
use spark_runtime::{ClipTime, GlyphProfile, Motion, Palette, Period, Profile, SampleContext};

fn base(area: Rect) -> Buffer {
    let mut cell = Cell::new("h");
    cell.set_style(
        Style::new()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .underline_color(Color::Magenta)
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
    );
    Buffer::filled(area, cell)
}

fn paint(
    time: u64,
    profile: &Profile,
    requested: Rect,
    actual: Rect,
    draw: impl FnOnce(&SampleContext<'_>, Rect, &mut Buffer),
) -> Buffer {
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, profile);
    let mut buffer = base(actual);
    draw(&context, requested, &mut buffer);
    buffer
}

#[test]
fn inline_output_declares_exact_width_and_matches_direct_single_row_cells() {
    let design = SegmentedComet::new().with_seed(u64::MAX);
    let palette = Palette::default();
    for profile in [Profile::default(), Profile::quiet()] {
        for time in [0, 799_999_999, 1_600_000_000, u64::MAX] {
            let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
            let sample = design.sample(&context);
            for width in [0, 1, 7, 8, 16, 24, 255, 256] {
                let line = sample.inline(width).unwrap();
                assert_eq!(line.width(), usize::from(width));
                let area = Rect::new(0, 0, width, 1);
                let mut inline = Buffer::empty(area);
                let mut widget = inline.clone();
                line.render(area, &mut inline);
                sample.render(area, &mut widget);
                assert_eq!(inline, widget, "width={width}, time={time}");
            }
            assert!(sample.inline(257).is_none());
            assert!(sample.inline(u16::MAX).is_none());
            let protected = [Rect::new(0, 0, 256, 1)];
            assert_eq!(
                sample.inline(24),
                sample.with_protected(&protected).inline(24)
            );
        }
    }
}

#[test]
fn border_runner_preserves_all_symbols_backgrounds_underlines_and_exclusions() {
    let design = BorderRunner::new().with_seed(5);
    let profile = Profile::default();
    // Deliberately include forbidden fields in every role. The scoped surface
    // must discard them even when palette resolution provides those fields.
    let override_style = Style::new()
        .fg(Color::Cyan)
        .bg(Color::Red)
        .underline_color(Color::Green)
        .add_modifier(Modifier::BOLD);
    let palette = Palette {
        foreground: override_style,
        muted: override_style,
        accent: override_style,
        highlight: override_style,
        background: Some(Color::Red),
    };
    let requested = Rect::new(2, 2, 24, 6);
    let protected = [Rect::new(9, 2, 8, 1), Rect::new(2, 5, 1, 2)];
    let mut before = base(Rect::new(0, 0, 32, 10));
    before.set_string(3, 2, "界e\u{0301}界", Style::new());
    let mut observed_change = false;
    for time in [0, 400_000_000, 1_399_999_999, 2_799_999_999, u64::MAX] {
        let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
        let mut buffer = before.clone();
        design
            .sample(&context)
            .with_protected(&protected)
            .render(requested, &mut buffer);
        for y in 0..10 {
            for x in 0..32 {
                let after = &buffer[(x, y)];
                let original = &before[(x, y)];
                assert_eq!(after.symbol(), original.symbol());
                assert_eq!(after.bg, original.bg);
                assert_eq!(after.underline_color, original.underline_color);
                let border = (2..26).contains(&x)
                    && (2..8).contains(&y)
                    && (x == 2 || x == 25 || y == 2 || y == 7);
                let excluded = (y == 2 && (9..17).contains(&x))
                    || (x == 2 && (5..7).contains(&y))
                    || (y == 2 && [3, 4, 6, 7].contains(&x));
                if !border || excluded {
                    assert_eq!(after, original, "time={time}, cell=({x},{y})");
                }
                observed_change |= after != original;
            }
        }
    }
    assert!(
        observed_change,
        "a preservation test must also establish an active decoration"
    );
}

#[test]
fn unsupported_border_sizes_cannot_request_animation_only_wakes() {
    let design = BorderRunner::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    let sample = design.sample(&context);
    for size in [
        Size::new(0, 0),
        Size::new(2, 10),
        Size::new(20, 2),
        Size::new(257, 8),
        Size::new(30, 129),
        Size::new(u16::MAX, u16::MAX),
    ] {
        assert_eq!(
            sample.frame_interval(size),
            None,
            "unsupported size {size:?}"
        );
        let mut buffer = base(Rect::new(0, 0, 30, 10));
        let before = buffer.clone();
        sample.render(Rect::new(0, 0, size.width, size.height), &mut buffer);
        assert_eq!(buffer, before);
    }
}

#[test]
fn finite_reveal_finishes_at_the_end_and_still_paints_on_unrelated_redraws() {
    let design = PrismReveal::new("DONE")
        .unwrap()
        .with_duration(Period::new(Duration::from_nanos(100)).unwrap());
    let area = Rect::new(3, 1, 10, 2);
    let actual = Rect::new(0, 0, 18, 4);
    let profile = Profile::default();
    let final_frame = paint(100, &profile, area, actual, |context, rect, buffer| {
        let sample = design.sample(context);
        assert!(sample.complete());
        assert_eq!(sample.frame_interval(rect.as_size()), None);
        sample.render(rect, buffer);
    });
    for time in [99, 0, 100, u64::MAX, 1, 101] {
        let frame = paint(time, &profile, area, actual, |context, rect, buffer| {
            let sample = design.sample(context);
            assert_eq!(sample.complete(), time >= 100);
            sample.render(rect, buffer);
        });
        if time >= 100 {
            assert_eq!(frame, final_frame);
            assert_eq!(
                (3..7).map(|x| frame[(x, 1)].symbol()).collect::<String>(),
                "DONE"
            );
        }
        for y in 0..4 {
            for x in 0..18 {
                if y != 1 || !(3..7).contains(&x) {
                    assert_eq!(frame[(x, y)], base(actual)[(x, y)]);
                }
            }
        }
    }
    let static_frame = paint(
        0,
        &Profile::quiet(),
        area,
        actual,
        |context, rect, buffer| {
            let sample = design.sample(context);
            assert!(sample.complete());
            assert_eq!(sample.frame_interval(rect.as_size()), None);
            sample.render(rect, buffer);
        },
    );
    assert_eq!(
        (3..7)
            .map(|x| static_frame[(x, 1)].symbol())
            .collect::<String>(),
        "DONE"
    );
}

#[test]
fn unsupported_text_fallback_preserves_glyphs_and_stays_inside_declared_text_footprint() {
    for (text, text_width) in [("界e\u{0301}", 3), ("e\u{0301}", 1), ("界界", 4)] {
        let design = PrismReveal::new(text).unwrap();
        assert!(design.uses_style_fallback());
        let palette = Palette::default();
        let profile = Profile::default();
        let area = Rect::new(2, 1, 20, 2);
        let protected = [Rect::new(2 + text_width - 1, 1, 1, 1)];
        let mut before = base(Rect::new(0, 0, 25, 4));
        before.set_string(2, 1, text, Style::new());
        for time in [0, 600_000_000, 1_200_000_000, u64::MAX] {
            let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
            let mut buffer = before.clone();
            design
                .sample(&context)
                .with_protected(&protected)
                .render(area, &mut buffer);
            for y in 0..4 {
                for x in 0..25 {
                    assert_eq!(buffer[(x, y)].symbol(), before[(x, y)].symbol());
                    assert_eq!(buffer[(x, y)].bg, before[(x, y)].bg);
                    assert_eq!(
                        buffer[(x, y)].underline_color,
                        before[(x, y)].underline_color
                    );
                    if y != 1 || x < 2 || x >= 2 + text_width - 1 {
                        assert_eq!(
                            buffer[(x, y)],
                            before[(x, y)],
                            "text={text:?}, cell=({x},{y})"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn reveal_validation_rejects_controls_and_unbounded_authored_content() {
    for invalid in ["a\nb", "\x1b[31m", "\r", "\0", "\t"] {
        assert_eq!(PrismReveal::new(invalid), Err(RevealError::Control));
    }
    assert!(PrismReveal::new(&"a".repeat(256)).is_ok());
    assert_eq!(
        PrismReveal::new(&"a".repeat(257)),
        Err(RevealError::TooLong)
    );
    let design = PrismReveal::new("").unwrap();
    let profile = Profile::default();
    let actual = Rect::new(0, 0, 10, 2);
    let frame = paint(0, &profile, actual, actual, |context, rect, buffer| {
        let sample = design.sample(context);
        assert_eq!(sample.frame_interval(rect.as_size()), None);
        sample.render(rect, buffer);
    });
    assert_eq!(frame, base(actual));
}

#[test]
fn typed_inputs_reject_nonfinite_and_out_of_range_without_silent_clamping() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(Progress::known(value), Err(InputError::NonFinite));
        assert_eq!(Level::new(value), Err(InputError::NonFinite));
    }
    for value in [-1.0, -f64::MIN_POSITIVE, 1.0 + f64::EPSILON, f64::MAX] {
        assert_eq!(Progress::known(value), Err(InputError::OutOfRange));
        assert_eq!(Level::new(value), Err(InputError::OutOfRange));
    }
    for value in [-0.0, 0.0, f64::MIN_POSITIVE, 0.5, 1.0] {
        assert_eq!(Progress::try_from(value).unwrap().value(), Some(value));
        assert_eq!(Level::try_from(value).unwrap().value(), value);
    }
    assert_eq!(Progress::default().value(), None);
    assert_eq!(Level::default(), Level::ZERO);
}

#[test]
fn assembly_cells_never_overstate_target_even_next_to_float_boundaries() {
    let design = AssemblyProgress::new();
    let profile = Profile {
        glyphs: GlyphProfile::Ascii,
        ..Profile::default()
    };
    for width in [12, 24, 79, 80] {
        let slots = width - 4;
        let area = Rect::new(0, 0, width, 3);
        for count in 0..=slots {
            let fraction = f64::from(count) / f64::from(slots);
            let below = if fraction == 0.0 {
                fraction
            } else {
                f64::from_bits(fraction.to_bits() - 1)
            };
            for value in [fraction, below] {
                let input = Progress::known(value).unwrap();
                for time in [0, 999_999_999, u64::MAX] {
                    let buffer = paint(time, &profile, area, area, |context, rect, buffer| {
                        design.sample(context, input).render(rect, buffer);
                    });
                    let filled = (2..width - 2)
                        .filter(|&x| buffer[(x, 1)].symbol() == "#")
                        .count();
                    assert!(
                        filled as f64 / f64::from(slots) <= value,
                        "width={width}, target={value:?}, filled={filled}"
                    );
                    let represented = filled as f64 / f64::from(slots);
                    assert!(value - represented <= 1.0 / f64::from(slots) + f64::EPSILON);
                }
            }
        }
        let unknown = paint(u64::MAX, &profile, area, area, |context, rect, buffer| {
            design
                .sample(context, Progress::unknown())
                .render(rect, buffer);
        });
        assert!((2..width - 2).all(|x| unknown[(x, 1)].symbol() != "#"));
        assert!((2..width - 2).any(|x| unknown[(x, 1)].symbol() == "?"));
    }
}

#[test]
fn data_driven_designs_have_no_obsolete_input_or_time_history() {
    let assembly = AssemblyProgress::new().with_seed(u64::MAX);
    let embers = TelemetryEmbers::new().with_seed(u64::MAX);
    let area = Rect::new(2, 1, 24, 8);
    let actual = Rect::new(0, 0, 30, 12);
    for profile in [Profile::default(), Profile::quiet()] {
        for time in [u64::MAX, 0, 1_800_000_000, 900_000_000, 0] {
            for value in [1.0, 0.0, 0.6, 0.25, 1.0, 0.0] {
                let progress = Progress::known(value).unwrap();
                let level = Level::new(value).unwrap();
                let actual_assembly =
                    paint(time, &profile, area, actual, |context, rect, buffer| {
                        assembly.sample(context, progress).render(rect, buffer);
                    });
                let fresh_assembly =
                    paint(time, &profile, area, actual, |context, rect, buffer| {
                        AssemblyProgress::new()
                            .with_seed(u64::MAX)
                            .sample(context, progress)
                            .render(rect, buffer);
                    });
                assert_eq!(actual_assembly, fresh_assembly);
                let actual_embers = paint(time, &profile, area, actual, |context, rect, buffer| {
                    embers.sample(context, level).render(rect, buffer);
                });
                let fresh_embers = paint(time, &profile, area, actual, |context, rect, buffer| {
                    TelemetryEmbers::new()
                        .with_seed(u64::MAX)
                        .sample(context, level)
                        .render(rect, buffer);
                });
                assert_eq!(actual_embers, fresh_embers);
            }
        }
    }
}

#[test]
fn static_complete_and_zero_input_views_keep_painting_without_time_requests() {
    let assembly = AssemblyProgress::new();
    let embers = TelemetryEmbers::new();
    let area = Rect::new(0, 0, 24, 8);
    for profile in [Profile::default(), Profile::quiet()] {
        let assembly_at = |time| {
            paint(time, &profile, area, area, |context, rect, buffer| {
                let sample = assembly.sample(context, Progress::known(1.0).unwrap());
                assert_eq!(sample.frame_interval(rect.as_size()), None);
                sample.render(rect, buffer);
            })
        };
        let embers_at = |time| {
            paint(time, &profile, area, area, |context, rect, buffer| {
                let sample = embers.sample(context, Level::ZERO);
                assert_eq!(sample.frame_interval(rect.as_size()), None);
                sample.render(rect, buffer);
            })
        };
        assert_eq!(assembly_at(0), assembly_at(u64::MAX));
        assert_eq!(embers_at(0), embers_at(u64::MAX));
        assert_ne!(assembly_at(0), base(area));
        assert_ne!(embers_at(0), base(area));
    }
    let profile = Profile {
        motion: Motion::Static,
        ..Profile::default()
    };
    let low = paint(0, &profile, area, area, |context, rect, buffer| {
        embers.sample(context, Level::ZERO).render(rect, buffer);
    });
    let high = paint(0, &profile, area, area, |context, rect, buffer| {
        embers.sample(context, Level::MAX).render(rect, buffer);
    });
    assert_ne!(low, high, "the static pose must remain level-responsive");
}

fn draw_kind(
    kind: u8,
    context: &SampleContext<'_>,
    requested: Rect,
    buffer: &mut Buffer,
    protected: &[Rect],
) {
    match kind {
        0 => SegmentedComet::new()
            .sample(context)
            .with_protected(protected)
            .render(requested, buffer),
        1 => AssemblyProgress::new()
            .sample(context, Progress::known(0.4).unwrap())
            .with_protected(protected)
            .render(requested, buffer),
        2 => BorderRunner::new()
            .sample(context)
            .with_protected(protected)
            .render(requested, buffer),
        3 => TelemetryEmbers::new()
            .sample(context, Level::new(0.7).unwrap())
            .with_protected(protected)
            .render(requested, buffer),
        4 => spark_catalog::PocketMascot::new()
            .sample(context)
            .with_protected(protected)
            .render(requested, buffer),
        5 => PrismReveal::new("A FRESH RESULT")
            .unwrap()
            .sample(context)
            .with_protected(protected)
            .render(requested, buffer),
        _ => unreachable!(),
    }
}

#[test]
fn all_six_designs_preserve_requested_coordinates_under_partial_buffer_clipping() {
    let profile = Profile::default();
    let requested = Rect::new(4, 3, 24, 8);
    let full_area = Rect::new(0, 0, 35, 14);
    for kind in 0..6 {
        for time in [0, 911_111_111, 1_200_000_000, u64::MAX] {
            let full = paint(
                time,
                &profile,
                requested,
                full_area,
                |context, rect, buffer| {
                    draw_kind(kind, context, rect, buffer, &[]);
                },
            );
            for actual in [
                Rect::new(9, 4, 10, 5),
                Rect::new(0, 0, 8, 7),
                Rect::new(23, 9, 8, 5),
            ] {
                let clipped = paint(
                    time,
                    &profile,
                    requested,
                    actual,
                    |context, rect, buffer| {
                        draw_kind(kind, context, rect, buffer, &[]);
                    },
                );
                for y in actual.y..actual.bottom() {
                    for x in actual.x..actual.right() {
                        assert_eq!(
                            clipped[(x, y)],
                            full[(x, y)],
                            "design={kind}, time={time}, cell=({x},{y})"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn all_six_designs_keep_wide_footprints_and_explicit_protected_rectangles() {
    let requested = Rect::new(2, 2, 24, 8);
    let protected = [Rect::new(12, 2, 3, 8), Rect::new(20, 5, 4, 3)];
    let palette = Palette::default();
    let profile = Profile::default();
    let mut before = base(Rect::new(0, 0, 32, 12));
    for y in 2..10 {
        // The first glyph's leading cell lies outside the requested rectangle.
        before.set_string(1, y, "界", Style::new());
        before.set_string(9, y, "界", Style::new());
        before.set_string(24, y, "界", Style::new());
    }
    for kind in 0..6 {
        for time in [0, 600_000_000, 1_200_000_000, u64::MAX] {
            let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
            let mut buffer = before.clone();
            draw_kind(kind, &context, requested, &mut buffer, &protected);
            for y in 0..12 {
                for x in 0..32 {
                    let outside = !(2..26).contains(&x) || !(2..10).contains(&y);
                    let wide = [1, 2, 9, 10, 24, 25].contains(&x);
                    let protected = (12..15).contains(&x) && (2..10).contains(&y)
                        || (20..24).contains(&x) && (5..8).contains(&y);
                    if outside || wide || protected {
                        assert_eq!(
                            buffer[(x, y)],
                            before[(x, y)],
                            "design={kind}, time={time}, cell=({x},{y})"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn all_six_designs_bound_work_on_huge_requested_rectangles_with_tiny_actual_buffers() {
    let actual = Rect::new(0, 0, 3, 2);
    let profile = Profile::quiet();
    for kind in 0..6 {
        for requested in [
            Rect {
                x: 0,
                y: 0,
                width: u16::MAX,
                height: u16::MAX,
            },
            Rect {
                x: u16::MAX - 1,
                y: u16::MAX - 1,
                width: u16::MAX,
                height: u16::MAX,
            },
            Rect::new(0, 0, 0, u16::MAX),
            Rect::new(0, 0, u16::MAX, 0),
        ] {
            let frame = paint(
                u64::MAX,
                &profile,
                requested,
                actual,
                |context, rect, buffer| {
                    draw_kind(kind, context, rect, buffer, &[]);
                },
            );
            assert_eq!(frame.content.len(), 6);
            if kind != 5 || requested.x != 0 || requested.width == 0 || requested.height == 0 {
                assert_eq!(
                    frame,
                    base(actual),
                    "design={kind}, requested={requested:?}"
                );
            }
        }
    }
}

#[test]
fn mascot_unequal_holds_seams_reduced_motion_and_discrete_deadlines_match() {
    use spark_catalog::PocketMascot;
    use spark_runtime::ChangeHint;
    let design = PocketMascot::new();
    let ends = [900, 1020, 1180, 1280, 1420, 1620, 1800, 2400];
    let period = design.period().as_nanos();
    assert_eq!(period, 2_400_000_000);
    let area = Rect::new(0, 0, 12, 6);
    let full = Profile::default();
    let mut start = 0;
    for end in ends {
        let end = end * 1_000_000;
        let frame_at = |time, profile: &Profile| {
            paint(time, profile, area, area, |context, rect, buffer| {
                design.sample(context).render(rect, buffer);
            })
        };
        let first = frame_at(start, &full);
        assert_eq!(first, frame_at(end - 1, &full));
        assert_eq!(first, frame_at(start + period * 17, &full));
        let reduced = Profile {
            motion: Motion::Reduced,
            ..full
        };
        assert_eq!(first, frame_at(start * 4, &reduced));
        assert_eq!(first, frame_at((end - 1) * 4, &reduced));
        for cycle in [0, 1, 37] {
            let palette = Palette::default();
            let time = ClipTime::from_nanos(start + cycle * period);
            let context = SampleContext::new(time, &palette, &full);
            assert_eq!(
                design.sample(&context).change_hint(area.as_size()),
                ChangeHint::At(ClipTime::from_nanos(end + cycle * period))
            );
        }
        start = end;
    }
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &full);
    assert_eq!(
        design.sample(&context).change_hint(area.as_size()),
        ChangeHint::None
    );
}

#[test]
fn mascot_native_centering_and_keep_erase_operations_are_distinct() {
    use spark_catalog::PocketMascot;
    let design = PocketMascot::new();
    let profile = Profile::default();
    let native = Rect::new(0, 0, 12, 6);
    let frame = paint(0, &profile, native, native, |context, rect, buffer| {
        design.sample(context).render(rect, buffer)
    });
    assert!(
        frame
            .content
            .iter()
            .zip(base(native).content)
            .any(|(after, before)| after == &before)
    );
    assert!(frame.content.iter().any(|cell| cell.symbol() == " "));
    assert!(frame.content.iter().any(|cell| cell.symbol() == "o"));
    let larger = Rect::new(3, 2, 20, 12);
    let actual = Rect::new(0, 0, 28, 17);
    let centered = paint(0, &profile, larger, actual, |context, rect, buffer| {
        design.sample(context).render(rect, buffer)
    });
    for y in 0..17 {
        for x in 0..28 {
            if (7..19).contains(&x) && (5..11).contains(&y) {
                assert_eq!(centered[(x, y)], frame[(x - 7, y - 5)]);
            } else {
                assert_eq!(centered[(x, y)], base(actual)[(x, y)]);
            }
        }
    }
    for size in [
        Size::new(0, 3),
        Size::new(12, 0),
        Size::new(2, 6),
        Size::new(11, 5),
    ] {
        let area = Rect::new(0, 0, size.width, size.height);
        let at = |time| {
            paint(time, &profile, area, area, |context, rect, buffer| {
                let sample = design.sample(context);
                assert_eq!(sample.frame_interval(size), None);
                sample.render(rect, buffer);
            })
        };
        assert_eq!(at(0), at(u64::MAX));
    }
}

#[test]
fn finite_last_frame_is_scheduled_before_time_driven_requests_stop() {
    use spark_runtime::{ChangeHint, HostTime, Playback, PlaybackRate, RepaintPlanner};
    let design = PrismReveal::new("OK")
        .unwrap()
        .with_duration(Period::new(Duration::from_nanos(100)).unwrap());
    let palette = Palette::default();
    let profile = Profile::default();
    let mut playback = Playback::new(HostTime::ZERO);
    playback
        .set_rate(HostTime::ZERO, PlaybackRate::new(3, 2).unwrap())
        .unwrap();
    let now = HostTime::from_nanos(66);
    let context = SampleContext::new(playback.sample(now), &palette, &profile);
    let sample = design.sample(&context);
    assert!(!sample.complete());
    let hint = sample.change_hint(Size::new(2, 1));
    assert_eq!(hint, ChangeHint::At(ClipTime::from_nanos(100)));
    let planner = RepaintPlanner::new(Duration::from_nanos(1));
    let deadline = planner.next_deadline(now, now, &playback, hint).unwrap();
    assert_eq!(deadline, HostTime::from_nanos(67));
    let completed_context = SampleContext::new(playback.sample(deadline), &palette, &profile);
    let completed = design.sample(&completed_context);
    let mut frame = base(Rect::new(0, 0, 2, 1));
    completed.render(frame.area, &mut frame);
    assert_eq!(
        frame.content.iter().map(Cell::symbol).collect::<String>(),
        "OK"
    );
    assert!(completed.complete());
    assert_eq!(
        planner.next_deadline(
            deadline,
            deadline,
            &playback,
            completed.change_hint(frame.area.as_size())
        ),
        None
    );
}

#[test]
fn zero_width_unsupported_text_has_no_time_driven_work() {
    use spark_runtime::ChangeHint;
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    let area = Rect::new(0, 0, 10, 2);
    for text in ["\u{0301}", "\u{200d}"] {
        let design = PrismReveal::new(text).unwrap();
        let sample = design.sample(&context);
        assert!(design.uses_style_fallback());
        assert_eq!(sample.frame_interval(area.as_size()), None);
        assert_eq!(sample.change_hint(area.as_size()), ChangeHint::None);
        let mut buffer = base(area);
        sample.render(area, &mut buffer);
        assert_eq!(buffer, base(area));
    }
}
