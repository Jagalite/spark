use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_catalog::{AssemblyProgress, InputError, Level, Progress, TelemetryEmbers};
use spark_runtime::{
    ClipTime, ColorProfile, GlyphProfile, Motion, Palette, Period, Profile, SampleContext,
};
use std::time::Duration;

fn assembly(seed: u64, time: u64, progress: Progress, area: Rect, profile: Profile) -> Buffer {
    let design = AssemblyProgress::new().with_seed(seed);
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
    let sample = design.sample(&context, progress);
    let mut buffer = Buffer::empty(area);
    let borrowed = &sample;
    borrowed.render(area, &mut buffer);
    buffer
}
fn embers(seed: u64, time: u64, level: Level, area: Rect, profile: Profile) -> Buffer {
    let design = TelemetryEmbers::new().with_seed(seed);
    let palette = Palette::amber();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
    let sample = design.sample(&context, level);
    let mut buffer = Buffer::empty(area);
    let borrowed = &sample;
    borrowed.render(area, &mut buffer);
    buffer
}
fn symbols(buffer: &Buffer) -> Vec<String> {
    (buffer.area.y..buffer.area.bottom())
        .map(|y| {
            (buffer.area.x..buffer.area.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}
fn ascii() -> Profile {
    Profile {
        glyphs: GlyphProfile::Ascii,
        ..Profile::default()
    }
}

#[test]
fn normalized_inputs_validate_without_clamping() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(Progress::known(value), Err(InputError::NonFinite));
        assert_eq!(Level::new(value), Err(InputError::NonFinite));
    }
    for value in [-1.0, -f64::MIN_POSITIVE, 1.000_000_000_000_000_2, f64::MAX] {
        assert_eq!(Progress::try_from(value), Err(InputError::OutOfRange));
        assert_eq!(Level::try_from(value), Err(InputError::OutOfRange));
    }
    for value in [-0.0, 0.0, f64::from_bits(1), 0.5, 1.0] {
        assert_eq!(Progress::try_from(value).unwrap().value(), Some(value));
        assert_eq!(Level::try_from(value).unwrap().value(), value);
    }
    assert_eq!(Progress::unknown().value(), None);
    assert_eq!(Progress::default(), Progress::unknown());
    assert_eq!(Level::default(), Level::ZERO);
    assert_eq!(Level::MAX.value(), 1.0);
    assert!(InputError::NonFinite.to_string().contains("finite"));
    assert!(InputError::OutOfRange.to_string().contains("0..=1"));
}

#[test]
fn full_machine_has_exact_truthful_static_silhouette() {
    let buffer = assembly(
        13,
        0,
        Progress::known(0.5).unwrap(),
        Rect::new(0, 0, 24, 3),
        Profile::quiet(),
    );
    assert_eq!(
        symbols(&buffer),
        [
            "  ----------T---------  ",
            ">|##########..........|<",
            "\\===o===o===o===o===o==/",
        ]
    );
    assert!(
        buffer
            .content
            .iter()
            .all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset)
    );
}

#[test]
fn known_progress_is_never_greater_than_target_at_any_sample_time() {
    let times = [
        0,
        1,
        250_000_000,
        1_799_999_999,
        1_800_000_000,
        7_500_000_000,
        u64::MAX,
    ];
    for width in 12..=80 {
        let slots = width - 4;
        let mut values = vec![0.0, f64::from_bits(1), 0.01, 0.3, 0.5, 0.65, 0.999, 1.0];
        for filled in 1..slots {
            let boundary = f64::from(filled) / f64::from(slots);
            values.extend([f64::from_bits(boundary.to_bits() - 1), boundary]);
        }
        for value in values {
            let progress = Progress::known(value).unwrap();
            for time in times {
                let buffer = assembly(7, time, progress, Rect::new(0, 0, width, 3), ascii());
                let count = (2..width - 2)
                    .filter(|&x| buffer[(x, 1)].symbol() == "#")
                    .count();
                assert!(
                    count as f64 / f64::from(slots) <= value,
                    "width={width}, value={value}, time={time}"
                );
                assert!((value * f64::from(slots) - count as f64) < 1.000_000_000_001);
            }
        }
    }
}

#[test]
fn unknown_and_replaced_operations_do_not_retain_previous_progress() {
    let design = AssemblyProgress::new();
    let palette = Palette::default();
    let profile = ascii();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    let area = Rect::new(0, 0, 24, 3);
    let mut buffer = Buffer::empty(area);
    for progress in [
        Progress::known(1.0).unwrap(),
        Progress::unknown(),
        Progress::known(0.0).unwrap(),
        Progress::known(0.25).unwrap(),
        Progress::unknown(),
    ] {
        design.sample(&context, progress).render(area, &mut buffer);
        assert_eq!(
            buffer,
            assembly(design.seed(), u64::MAX, progress, area, profile)
        );
        let row = &symbols(&buffer)[1];
        assert_eq!(row.contains('?'), progress.value().is_none());
        if progress.value().is_none() {
            assert!(!row.contains('#'));
        }
    }
}

#[test]
fn both_designs_are_deterministic_at_shuffled_and_extreme_times() {
    let area_a = Rect::new(3, 4, 24, 3);
    let area_e = Rect::new(3, 4, 24, 8);
    let level = Level::new(0.72).unwrap();
    let progress = Progress::known(0.65).unwrap();
    let times = [
        0,
        1,
        33_000_000,
        1_799_999_999,
        1_800_000_000,
        1_800_000_001,
        9_000_000_000,
        u64::MAX,
    ];
    for seed in [0, 1, 7, u64::MAX] {
        for profile in [
            Profile::default(),
            ascii(),
            Profile::quiet(),
            Profile {
                motion: Motion::Reduced,
                ..Profile::default()
            },
        ] {
            let expected: Vec<_> = times
                .iter()
                .map(|&t| {
                    (
                        assembly(seed, t, progress, area_a, profile),
                        embers(seed, t, level, area_e, profile),
                    )
                })
                .collect();
            for index in [7, 2, 0, 3, 1, 5, 6, 4, 7, 0] {
                assert_eq!(
                    assembly(seed, times[index], progress, area_a, profile),
                    expected[index].0
                );
                assert_eq!(
                    embers(seed, times[index], level, area_e, profile),
                    expected[index].1
                );
            }
        }
    }
}

#[test]
fn seeds_and_levels_have_visible_effects_without_accumulation() {
    let area = Rect::new(0, 0, 24, 8);
    let half = Level::new(0.5).unwrap();
    assert_ne!(
        embers(7, 700_000_000, half, area, ascii()),
        embers(8, 700_000_000, half, area, ascii())
    );
    assert_ne!(
        embers(7, 700_000_000, half, area, ascii()),
        embers(7, 2_700_000_000, half, area, ascii())
    );
    let design = TelemetryEmbers::new();
    let palette = Palette::amber();
    let profile = ascii();
    let context = SampleContext::new(ClipTime::from_nanos(700_000_000), &palette, &profile);
    let mut reused = Buffer::empty(area);
    for level in [Level::MAX, Level::ZERO, half, Level::MAX, Level::ZERO] {
        design.sample(&context, level).render(area, &mut reused);
        assert_eq!(
            reused,
            embers(design.seed(), 700_000_000, level, area, profile)
        );
    }
    let activity = |level| {
        embers(7, 700_000_000, level, area, ascii())
            .content
            .iter()
            .filter(|cell| cell.symbol() == "*" || cell.symbol() == "+")
            .count()
    };
    assert_eq!(activity(Level::ZERO), 0);
    assert!(activity(Level::MAX) > activity(half));
}

#[test]
fn static_complete_zero_and_tiny_views_request_no_time_only_frames() {
    let a = AssemblyProgress::new();
    let e = TelemetryEmbers::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    assert_eq!(
        a.sample(&context, Progress::known(1.0).unwrap())
            .frame_interval(Size::new(24, 3)),
        None
    );
    assert_eq!(
        e.sample(&context, Level::ZERO)
            .frame_interval(Size::new(24, 8)),
        None
    );
    for size in [
        Size::new(0, 0),
        Size::new(0, 3),
        Size::new(1, 1),
        Size::new(7, 3),
    ] {
        assert_eq!(
            a.sample(&context, Progress::unknown()).frame_interval(size),
            None
        );
        assert_eq!(e.sample(&context, Level::MAX).frame_interval(size), None);
    }
    assert_eq!(
        a.sample(&context, Progress::unknown())
            .frame_interval(Size::new(12, 3)),
        Some(Duration::from_millis(80))
    );
    assert_eq!(
        e.sample(&context, Level::MAX)
            .frame_interval(Size::new(8, 4)),
        Some(Duration::from_millis(50))
    );
    let quiet = Profile::quiet();
    let context = SampleContext::new(ClipTime::MAX, &palette, &quiet);
    assert_eq!(
        a.sample(&context, Progress::unknown())
            .frame_interval(Size::new(24, 3)),
        None
    );
    assert_eq!(
        e.sample(&context, Level::MAX)
            .frame_interval(Size::new(24, 8)),
        None
    );
    for area in [
        Rect::new(2, 3, 1, 1),
        Rect::new(2, 3, 7, 2),
        Rect::new(2, 3, 24, 8),
    ] {
        for progress in [Progress::unknown(), Progress::known(1.0).unwrap()] {
            assert_eq!(
                assembly(4, 0, progress, area, quiet),
                assembly(4, u64::MAX, progress, area, quiet)
            );
        }
        assert_eq!(
            embers(4, 0, Level::MAX, area, quiet),
            embers(4, u64::MAX, Level::MAX, area, quiet)
        );
    }
    let full = Profile::default();
    assert_eq!(
        assembly(
            4,
            0,
            Progress::known(1.0).unwrap(),
            Rect::new(0, 0, 24, 3),
            full
        ),
        assembly(
            4,
            u64::MAX,
            Progress::known(1.0).unwrap(),
            Rect::new(0, 0, 24, 3),
            full
        )
    );
    assert_eq!(
        embers(4, 0, Level::ZERO, Rect::new(0, 0, 24, 8), full),
        embers(4, u64::MAX, Level::ZERO, Rect::new(0, 0, 24, 8), full)
    );
}

#[test]
fn clipped_buffers_keep_the_requested_coordinate_system() {
    let a = AssemblyProgress::new();
    let e = TelemetryEmbers::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for requested in [
        Rect::new(3, 2, 24, 3),
        Rect::new(3, 2, 24, 8),
        Rect::new(0, 0, 120, 40),
    ] {
        let mut full_a = Buffer::empty(Rect::new(0, 0, 130, 45));
        let mut full_e = full_a.clone();
        a.sample(&context, Progress::known(0.65).unwrap())
            .render(requested, &mut full_a);
        e.sample(&context, Level::MAX)
            .render(requested, &mut full_e);
        let clip = Rect::new(10, 3, 14, 6);
        let mut clipped_a = Buffer::empty(clip);
        let mut clipped_e = Buffer::empty(clip);
        a.sample(&context, Progress::known(0.65).unwrap())
            .render(requested, &mut clipped_a);
        e.sample(&context, Level::MAX)
            .render(requested, &mut clipped_e);
        for y in clip.y..clip.bottom() {
            for x in clip.x..clip.right() {
                assert_eq!(full_a[(x, y)], clipped_a[(x, y)]);
                assert_eq!(full_e[(x, y)], clipped_e[(x, y)]);
            }
        }
    }
}

#[test]
fn explicit_protections_neighbors_and_wide_footprints_survive() {
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    let requested = Rect::new(3, 1, 24, 8);
    let protected = [Rect::new(10, 2, 4, 5)];
    let mut base = Buffer::empty(Rect::new(0, 0, 30, 10));
    for cell in &mut base.content {
        cell.set_char('H').set_style(
            Style::new()
                .fg(Color::LightRed)
                .bg(Color::Blue)
                .add_modifier(Modifier::ITALIC),
        );
    }
    // Wide origin outside the requested area, plus one wholly inside it.
    base[(2, 4)].set_symbol("界");
    base[(3, 4)].set_symbol(" ");
    base[(17, 4)].set_symbol("界");
    base[(18, 4)].set_symbol(" ");
    let mut outputs = [base.clone(), base.clone()];
    AssemblyProgress::new()
        .sample(&context, Progress::known(0.65).unwrap())
        .with_protected(&protected)
        .render(requested, &mut outputs[0]);
    TelemetryEmbers::new()
        .sample(&context, Level::MAX)
        .with_protected(&protected)
        .render(requested, &mut outputs[1]);
    for result in outputs {
        for y in 0..10 {
            for x in 0..30 {
                let outside = !requested.contains((x, y).into());
                let excluded = protected[0].contains((x, y).into());
                let wide = y == 4 && [2, 3, 17, 18].contains(&x);
                if outside || excluded || wide {
                    assert_eq!(result[(x, y)], base[(x, y)], "at {x},{y}");
                }
            }
        }
    }
}

#[test]
fn extreme_requested_rectangles_do_not_expand_art_or_allocate_a_screen() {
    let a = AssemblyProgress::new();
    let e = TelemetryEmbers::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for area in [
        Rect::new(0, 0, 0, 0),
        Rect::new(0, 0, u16::MAX, u16::MAX),
        Rect::new(u16::MAX, u16::MAX, 120, 40),
        Rect::new(u16::MAX - 2, u16::MAX - 2, u16::MAX, u16::MAX),
    ] {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 2, 2));
        let before = buffer.clone();
        a.sample(&context, Progress::unknown())
            .render(area, &mut buffer);
        e.sample(&context, Level::MAX).render(area, &mut buffer);
        assert_eq!(buffer, before);
    }
    let requested = Rect::new(0, 0, 130, 44);
    let buffer = embers(2, u64::MAX, Level::MAX, requested, Profile::default());
    for y in 0..44 {
        for x in 0..130 {
            if !(5..125).contains(&x) || !(2..42).contains(&y) {
                assert_eq!(buffer[(x, y)].symbol(), " ");
                assert_eq!(buffer[(x, y)].fg, Color::Reset);
            }
        }
    }
    assert!(
        buffer
            .content
            .iter()
            .filter(|cell| cell.symbol() != " ")
            .count()
            <= TelemetryEmbers::MAX_PARTICLES * 2 + 60
    );
}

#[test]
fn palette_and_quiet_profiles_are_respected_including_unknown_background() {
    let palette = Palette {
        accent: Style::new()
            .fg(Color::Rgb(250, 50, 20))
            .underline_color(Color::Rgb(42, 70, 90)),
        ..Palette::default()
    };
    for colors in [
        ColorProfile::TrueColor,
        ColorProfile::Ansi16,
        ColorProfile::Monochrome,
    ] {
        let profile = Profile {
            colors,
            glyphs: GlyphProfile::Ascii,
            ..Profile::default()
        };
        let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
        let area = Rect::new(0, 0, 24, 8);
        let mut buffers = [Buffer::empty(area), Buffer::empty(area)];
        AssemblyProgress::new()
            .sample(&context, Progress::known(0.5).unwrap())
            .render(area, &mut buffers[0]);
        TelemetryEmbers::new()
            .sample(&context, Level::MAX)
            .render(area, &mut buffers[1]);
        for buffer in buffers {
            assert!(buffer.content.iter().all(|cell| cell.symbol().is_ascii()));
            assert!(buffer.content.iter().all(|cell| cell.bg == Color::Reset));
            if colors != ColorProfile::TrueColor {
                for cell in buffer.content {
                    assert!(!matches!(cell.fg, Color::Rgb(..) | Color::Indexed(..)));
                    assert!(!matches!(
                        cell.underline_color,
                        Color::Rgb(..) | Color::Indexed(..)
                    ));
                }
            }
        }
    }
}

#[test]
fn assembly_period_edges_repeat_and_reduced_motion_is_direct() {
    let palette = Palette::default();
    let profile = Profile::default();
    let design =
        AssemblyProgress::new().with_period(Period::new(Duration::from_nanos(11)).unwrap());
    let area = Rect::new(0, 0, 24, 3);
    let render = |time| {
        let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
        let mut buffer = Buffer::empty(area);
        design
            .sample(&context, Progress::unknown())
            .render(area, &mut buffer);
        buffer
    };
    assert_eq!(render(0), render(11));
    assert_eq!(render(10), render(21));
    assert_eq!(render(1), render(12));
    let reduced = Profile {
        motion: Motion::Reduced,
        ..Profile::default()
    };
    let progress = Progress::known(0.5).unwrap();
    assert_eq!(
        assembly(2, 1_600_000_000, progress, area, reduced),
        assembly(2, 400_000_000, progress, area, Profile::default())
    );
}
