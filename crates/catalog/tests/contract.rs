//! Independent consumer-facing contract tests. Compare complete cells, not only glyphs.
use std::time::Duration;

use ratatui::{
    buffer::{Buffer, Cell, CellDiffOption},
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_catalog::{CometSample, SegmentedComet};
use spark_runtime::{
    ClipTime, ColorProfile, GlyphProfile, Motion, Palette, Period, Profile, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};

fn base(area: Rect) -> Buffer {
    let mut cell = Cell::new("H");
    cell.set_style(
        Style::new()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
    );
    Buffer::filled(area, cell)
}

fn sample(
    design: &SegmentedComet,
    time: u64,
    area: Rect,
    buffer_area: Rect,
    profile: &Profile,
) -> Buffer {
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, profile);
    let mut buffer = base(buffer_area);
    design.sample(&context).render(area, &mut buffer);
    buffer
}

#[test]
fn repeated_and_out_of_order_samples_equal_fresh_construction() {
    let period = Period::new(Duration::from_nanos(1_600_000_001)).unwrap();
    let design = SegmentedComet::new()
        .with_period(period)
        .with_seed(u64::MAX);
    let area = Rect::new(4, 2, 24, 3);
    let buffer_area = Rect::new(0, 0, 35, 8);
    for profile in [Profile::default(), Profile::quiet()] {
        let times = [
            0,
            1,
            800_000_000,
            period.as_nanos() - 1,
            period.as_nanos(),
            period.as_nanos() + 1,
            16_000_000_010,
            u64::MAX,
        ];
        let expected: Vec<_> = times
            .iter()
            .map(|&time| sample(&design, time, area, buffer_area, &profile))
            .collect();
        for index in [7, 0, 3, 3, 1, 6, 2, 5, 4, 0, 7] {
            assert_eq!(
                sample(&design, times[index], area, buffer_area, &profile),
                expected[index],
                "time index {index}"
            );
            let rebuilt = SegmentedComet::new()
                .with_period(period)
                .with_seed(u64::MAX);
            assert_eq!(
                sample(&rebuilt, times[index], area, buffer_area, &profile),
                expected[index]
            );
        }
    }
}

#[test]
fn loops_match_both_sides_of_seam_and_reduce_large_times_exactly() {
    let period = 1_600_000_001;
    let design =
        SegmentedComet::new().with_period(Period::new(Duration::from_nanos(period)).unwrap());
    let area = Rect::new(2, 1, 24, 3);
    let buffer_area = Rect::new(0, 0, 30, 6);
    let profile = Profile::default();
    for time in [0, 1, 33_000_000, period - 1, period, period + 1, u64::MAX] {
        assert_eq!(
            sample(&design, time, area, buffer_area, &profile),
            sample(&design, time % period, area, buffer_area, &profile),
            "time {time}"
        );
    }
    assert_eq!(
        sample(&design, period - 1, area, buffer_area, &profile),
        sample(&design, period * 2 - 1, area, buffer_area, &profile)
    );
    assert_ne!(
        sample(&design, period - 1, area, buffer_area, &profile),
        sample(&design, period, area, buffer_area, &profile),
        "the seam must advance to the next pose rather than freeze"
    );
}

#[test]
fn clipping_preserves_requested_coordinates_and_complete_cell_styles() {
    let design = SegmentedComet::new().with_seed(43);
    let profile = Profile::default();
    for requested in [
        Rect::new(4, 3, 24, 3),
        Rect::new(2, 2, 32, 9),
        Rect::new(3, 1, 7, 3),
        Rect::new(0, 0, 300, 7),
    ] {
        let full_area = Rect::new(0, 0, 320, 14);
        let full = sample(&design, 713_123_456, requested, full_area, &profile);
        for clipped_area in [
            Rect::new(10, 4, 11, 2),
            Rect::new(0, 0, 9, 6),
            Rect::new(20, 2, 8, 9),
            Rect::new(2, 1, 1, 1),
            Rect::new(289, 2, 14, 4),
        ] {
            let clipped = sample(&design, 713_123_456, requested, clipped_area, &profile);
            for y in clipped_area.y..clipped_area.bottom() {
                for x in clipped_area.x..clipped_area.right() {
                    assert_eq!(
                        clipped[(x, y)],
                        full[(x, y)],
                        "requested={requested:?}, clipped={clipped_area:?}, ({x},{y})"
                    );
                }
            }
        }
    }
}

#[test]
fn resize_reconstruction_is_deterministic_and_neighbors_remain_untouched() {
    let design = SegmentedComet::new();
    let profile = Profile::default();
    let buffer_area = Rect::new(0, 0, 45, 12);
    let untouched = base(buffer_area);
    for (width, height) in [
        (0, 0),
        (1, 1),
        (7, 3),
        (8, 1),
        (16, 2),
        (24, 3),
        (32, 7),
        (8, 3),
        (24, 3),
    ] {
        let requested = Rect::new(5, 2, width, height);
        let rendered = sample(&design, 523_001_717, requested, buffer_area, &profile);
        assert_eq!(
            rendered,
            sample(&design, 523_001_717, requested, buffer_area, &profile)
        );
        for y in buffer_area.y..buffer_area.bottom() {
            for x in buffer_area.x..buffer_area.right() {
                if x < requested.x
                    || x >= requested.right()
                    || y < requested.y
                    || y >= requested.bottom()
                {
                    assert_eq!(rendered[(x, y)], untouched[(x, y)], "neighbor ({x},{y})");
                }
            }
        }
    }
}

#[test]
fn protected_cells_wide_footprints_and_special_diff_cells_are_unchanged() {
    let design = SegmentedComet::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::from_nanos(400_000_000), &palette, &profile);
    let mut buffer = base(Rect::new(0, 0, 40, 5));
    buffer.set_string(2, 2, "界e\u{0301}界", Style::new().fg(Color::Red));
    buffer[(8, 2)].set_diff_option(CellDiffOption::Skip);
    buffer[(9, 2)].set_diff_option(CellDiffOption::AlwaysUpdate);
    buffer[(10, 2)].set_diff_option(CellDiffOption::ForcedWidth(3.try_into().unwrap()));
    #[allow(deprecated)]
    {
        buffer[(14, 2)].skip = true;
    }
    let before = buffer.clone();
    let protected = [Rect::new(16, 1, 2, 3), Rect::new(23, 2, 3, 1)];
    design
        .sample(&context)
        .with_protected(&protected)
        .render(Rect::new(3, 1, 30, 3), &mut buffer);
    for x in [2, 3, 5, 6, 8, 9, 10, 11, 12, 14, 16, 17, 23, 24, 25] {
        assert_eq!(buffer[(x, 2)], before[(x, 2)], "protected/footprint x={x}");
    }
    for y in 1..4 {
        for x in 16..18 {
            assert_eq!(buffer[(x, y)], before[(x, y)]);
        }
    }
    assert_ne!(
        buffer[(4, 2)],
        before[(4, 2)],
        "one-column combining grapheme is safely replaceable"
    );
}

#[test]
fn escaped_forced_width_footprints_are_protected_even_when_origin_is_clipped() {
    let mut buffer = base(Rect::new(0, 0, 12, 1));
    buffer[(1, 0)]
        .set_symbol("\x1b]8;;https://example.invalid\x1b\\link\x1b]8;;\x1b\\")
        .set_diff_option(CellDiffOption::ForcedWidth(5.try_into().unwrap()));
    let before = buffer.clone();
    let mut surface = OpaqueSurface::new(&mut buffer, Rect::new(3, 0, 8, 1), &[]);
    for x in [0, 1, 2, 3, 4, 5, 6, 7, 2, 0] {
        surface.write(
            x,
            0,
            CellOp::Replace(Glyph::new('*').unwrap(), Style::new()),
        );
    }
    for x in 1..6 {
        assert_eq!(
            buffer[(x, 0)],
            before[(x, 0)],
            "forced-width footprint x={x}"
        );
    }
    assert_eq!(buffer[(6, 0)].symbol(), "*");
}

#[test]
fn quiet_profile_is_ascii_monochrome_and_static_but_still_paints() {
    let design = SegmentedComet::new();
    let profile = Profile::quiet();
    let area = Rect::new(0, 0, 24, 3);
    let first = sample(&design, 0, area, area, &profile);
    let last = sample(&design, u64::MAX, area, area, &profile);
    assert_eq!(first, last);
    assert!(first.content.iter().any(|cell| cell.symbol() != " "));
    for cell in &first.content {
        assert!(cell.symbol().is_ascii());
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
    }
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    assert_eq!(design.sample(&context).frame_interval(area.as_size()), None);
}

#[test]
fn reduced_motion_is_quieter_ascii_and_monochrome_with_its_own_loop() {
    let design = SegmentedComet::new();
    let profile = Profile {
        glyphs: GlyphProfile::Ascii,
        colors: ColorProfile::Monochrome,
        motion: Motion::Reduced,
    };
    let area = Rect::new(0, 0, 24, 3);
    let time = 733_333_337;
    let reduced = sample(&design, time, area, area, &profile);
    let cycle = design.period().as_nanos() * 4;
    assert_eq!(reduced, sample(&design, time + cycle, area, area, &profile));
    assert_eq!(
        sample(&design, u64::MAX, area, area, &profile),
        sample(&design, u64::MAX % cycle, area, area, &profile)
    );
    for y in [0, 2] {
        for x in 0..24 {
            assert_eq!(reduced[(x, y)].symbol(), " ", "reduced spark at ({x},{y})");
        }
    }
    for cell in &reduced.content {
        assert!(cell.symbol().is_ascii());
        assert_eq!(cell.fg, Color::Reset);
        assert_eq!(cell.bg, Color::Reset);
    }
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
    assert_eq!(
        design.sample(&context).frame_interval(Size::new(24, 3)),
        Some(Duration::from_millis(100))
    );
}

#[test]
fn borrowed_view_implements_application_widget_without_consuming_sample() {
    let design = SegmentedComet::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::from_nanos(321_654_987), &palette, &profile);
    let view = design.sample(&context);
    let area = Rect::new(3, 2, 24, 3);
    let mut first = base(Rect::new(0, 0, 32, 8));
    let mut second = first.clone();
    let borrowed = &view;
    <&CometSample<'_> as Widget>::render(borrowed, area, &mut first);
    <&CometSample<'_> as Widget>::render(borrowed, area, &mut second);
    assert_eq!(first, second);
    assert_eq!(
        view.frame_interval(area.as_size()),
        Some(Duration::from_millis(33))
    );
}

#[test]
fn maximal_requested_geometry_remains_bounded_on_a_tiny_buffer() {
    let design = SegmentedComet::new().with_seed(u64::MAX);
    let profile = Profile::default();
    let actual = Rect::new(0, 0, 3, 2);
    let original = base(actual);
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
        assert_eq!(
            sample(&design, u64::MAX, requested, actual, &profile),
            original
        );
    }
    // The visible part can also sit at the center of a huge requested rectangle.
    // This allocates six actual cells, never the requested 4-billion-cell region.
    let centered_actual = Rect::new(32_640, 32_766, 3, 2);
    let rendered = sample(
        &design,
        u64::MAX,
        Rect {
            x: 0,
            y: 0,
            width: u16::MAX,
            height: u16::MAX,
        },
        centered_actual,
        &profile,
    );
    assert_eq!(rendered.content.len(), 6);
    assert_ne!(rendered, base(centered_actual));
}

#[test]
fn validated_period_and_seed_sweep_has_exact_repeat_equivalence() {
    let profiles = [
        Profile::default(),
        Profile {
            glyphs: GlyphProfile::Ascii,
            colors: ColorProfile::Ansi16,
            motion: Motion::Full,
        },
        Profile::quiet(),
    ];
    let periods = [1, 2, 3, 997, 1_600_000_001, u64::MAX];
    let mut state = 0x7a29_bdec_134f_085d_u64;
    for case in 0..192 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let period = periods[case % periods.len()];
        let design = SegmentedComet::new()
            .with_period(Period::new(Duration::from_nanos(period)).unwrap())
            .with_seed(state.rotate_left(23));
        let requested = Rect::new(
            (state % 9) as u16,
            ((state >> 8) % 5) as u16,
            ((state >> 16) % 310) as u16,
            ((state >> 32) % 12) as u16,
        );
        let actual = Rect::new(4, 2, 41, 15);
        let profile = &profiles[case % profiles.len()];
        assert_eq!(
            sample(&design, state, requested, actual, profile),
            sample(&design, state % period, requested, actual, profile),
            "case {case}, period {period}, requested {requested:?}"
        );
    }
}

#[test]
fn smallest_strip_and_tiny_fallback_are_legible_and_do_not_request_motion() {
    let design = SegmentedComet::new();
    let profile = Profile {
        glyphs: GlyphProfile::Ascii,
        ..Profile::default()
    };
    let tiny_area = Rect::new(0, 0, 7, 1);
    let tiny = sample(&design, 0, tiny_area, tiny_area, &profile);
    assert_eq!(
        tiny.content.iter().map(Cell::symbol).collect::<String>(),
        "   *   "
    );
    assert_eq!(
        tiny,
        sample(&design, u64::MAX, tiny_area, tiny_area, &profile)
    );
    let smallest = Rect::new(0, 0, 8, 1);
    let start = sample(&design, 0, smallest, smallest, &profile);
    assert_eq!(
        start.content.iter().map(Cell::symbol).collect::<String>(),
        ".   .==>"
    );
    let half = sample(&design, 800_000_000, smallest, smallest, &profile);
    assert_eq!(
        half.content.iter().map(Cell::symbol).collect::<String>(),
        ".==>.   "
    );
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for size in [
        Size::new(0, 0),
        Size::new(24, 0),
        Size::new(0, 3),
        Size::new(7, 1),
    ] {
        assert_eq!(design.sample(&context).frame_interval(size), None);
    }
    assert!(
        design
            .sample(&context)
            .frame_interval(smallest.as_size())
            .is_some()
    );
}

#[test]
fn color_profiles_resolve_underline_colors_in_user_style_overrides() {
    let override_style = Style::new()
        .fg(Color::Rgb(249, 17, 23))
        .bg(Color::Rgb(21, 34, 89))
        .underline_color(Color::Rgb(7, 11, 247))
        .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
        .remove_modifier(Modifier::DIM);
    let palette = Palette {
        foreground: override_style,
        muted: override_style,
        accent: override_style,
        highlight: override_style,
        background: None,
    };
    assert_eq!(
        palette.resolve(override_style, ColorProfile::TrueColor),
        override_style
    );
    for colors in [ColorProfile::Ansi16, ColorProfile::Monochrome] {
        let style = palette.resolve(override_style, colors);
        assert_eq!(style.add_modifier, override_style.add_modifier);
        assert_eq!(style.sub_modifier, override_style.sub_modifier);
        for color in [style.fg, style.bg, style.underline_color]
            .into_iter()
            .flatten()
        {
            assert!(
                !matches!(color, Color::Rgb(..) | Color::Indexed(_)),
                "{colors:?} leaked {color:?}"
            );
            if colors == ColorProfile::Monochrome {
                assert_eq!(color, Color::Reset);
            }
        }
        let profile = Profile {
            colors,
            motion: Motion::Static,
            ..Profile::default()
        };
        let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 16, 1));
        SegmentedComet::new()
            .sample(&context)
            .render(buffer.area, &mut buffer);
        for cell in buffer.content {
            for color in [cell.fg, cell.bg, cell.underline_color] {
                assert!(
                    !matches!(color, Color::Rgb(..) | Color::Indexed(_)),
                    "{colors:?} painted {color:?}"
                );
                if colors == ColorProfile::Monochrome {
                    assert_eq!(color, Color::Reset);
                }
            }
        }
    }
}
