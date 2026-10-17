use std::time::Duration;

use ratatui_core::{
    buffer::{Buffer, Cell, CellDiffOption},
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_runtime::{
    ChangeHint, ClipTime, ColorProfile, GlyphProfile, HostTime, Motion, Palette, Playback,
    RepaintPlanner, SampleContext,
};

use spark_extension_starter::{HangingLantern, LanternSample, NATIVE_SIZE};

const PERIOD_NS: u64 = 1_120_000_000;
const NATIVE: Rect = Rect::new(0, 0, NATIVE_SIZE.width, NATIVE_SIZE.height);

fn base(area: Rect) -> Buffer {
    let mut cell = Cell::new("H");
    cell.set_style(
        Style::new()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .underline_color(Color::Red)
            .add_modifier(Modifier::BOLD | Modifier::ITALIC),
    );
    Buffer::filled(area, cell)
}

fn render(time: u64, profile: &spark_runtime::Profile, requested: Rect, actual: Rect) -> Buffer {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, profile);
    let sample = design.sample(&context);
    let mut buffer = base(actual);
    let borrowed_sample: &LanternSample<'_> = &sample;
    borrowed_sample.render(requested, &mut buffer);
    buffer
}

fn symbols(buffer: &Buffer) -> Vec<String> {
    buffer
        .content
        .chunks(usize::from(buffer.area.width))
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

#[test]
fn independent_open_pose_and_complete_replacement_styles() {
    let buffer = render(
        520_000_000,
        &spark_runtime::Profile::default(),
        NATIVE,
        NATIVE,
    );
    assert_eq!(
        symbols(&buffer),
        [
            "      |      ",
            "    .-+-.    ",
            "   /  '  \\   ",
            "   | -*- |   ",
            "   |  .  |   ",
            "   '-----'   ",
            "     / \\     ",
        ]
    );
    let palette = Palette::default();
    assert_eq!(buffer[(6, 3)].fg, palette.highlight.fg.unwrap());
    assert_eq!(buffer[(5, 3)].fg, palette.accent.fg.unwrap());
    assert_eq!(buffer[(3, 3)].fg, palette.foreground.fg.unwrap());
    for cell in &buffer.content {
        // Opaque replacement resets host styles, even for deliberate spaces.
        // An unknown palette background is not replaced by invented black.
        assert_eq!(cell.bg, Color::Reset);
        assert_eq!(cell.underline_color, Color::Reset);
        assert_eq!(cell.modifier, Modifier::empty());
    }
    assert_eq!(buffer[(0, 0)].symbol(), " ");
    assert_eq!(buffer[(0, 0)].fg, Color::Reset);
}

#[test]
fn owned_and_borrowed_samples_are_equivalent_and_repeatable() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::default();
    let context = SampleContext::new(ClipTime::from_nanos(520_000_000), &palette, &profile);
    let sample = design.sample(&context);
    let mut owned = base(NATIVE);
    sample.render(NATIVE, &mut owned);
    for _ in 0..3 {
        let mut borrowed = base(NATIVE);
        let borrowed_sample: &LanternSample<'_> = &sample;
        borrowed_sample.render(NATIVE, &mut borrowed);
        assert_eq!(owned, borrowed);
    }
}

#[test]
fn direct_seeking_and_unequal_hold_boundaries_need_no_replay() {
    let profile = spark_runtime::Profile::default();
    for time in [
        u64::MAX,
        520_000_000,
        0,
        PERIOD_NS,
        239_999_999,
        840_000_000,
        0,
    ] {
        assert_eq!(
            render(time, &profile, NATIVE, NATIVE),
            render(time % PERIOD_NS, &profile, NATIVE, NATIVE)
        );
    }
    for boundary in [
        240_000_000,
        360_000_000,
        520_000_000,
        840_000_000,
        1_000_000_000,
        PERIOD_NS,
    ] {
        assert_ne!(
            render(boundary - 1, &profile, NATIVE, NATIVE),
            render(boundary, &profile, NATIVE, NATIVE),
            "expected distinct adjacent poses at {boundary}"
        );
    }
}

#[test]
fn reduced_motion_maps_cells_and_advice_to_the_same_time_domain() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let full = spark_runtime::Profile::default();
    let reduced = spark_runtime::Profile {
        motion: Motion::Reduced,
        ..full
    };
    for time in [0, 959_999_999, 960_000_000, 2_080_000_000, u64::MAX] {
        assert_eq!(
            render(time, &reduced, NATIVE, NATIVE),
            render(time / 4, &full, NATIVE, NATIVE)
        );
    }
    for (time, expected) in [
        (0, 960_000_000),
        (960_000_000, 1_440_000_000),
        (2_080_000_000, 3_360_000_000),
    ] {
        let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &reduced);
        assert_eq!(
            design.sample(&context).change_hint(NATIVE_SIZE),
            ChangeHint::At(ClipTime::from_nanos(expected))
        );
    }
}

#[test]
fn absolute_advice_is_future_or_none_including_overflow() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    for motion in [Motion::Full, Motion::Reduced] {
        let profile = spark_runtime::Profile {
            motion,
            ..Default::default()
        };
        for time in [0, 240_000_000, PERIOD_NS, u64::MAX - 1, u64::MAX] {
            let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
            match design.sample(&context).change_hint(NATIVE_SIZE) {
                ChangeHint::At(next) => assert!(next.as_nanos() > time),
                ChangeHint::None => assert!(time > u64::MAX - 4 * PERIOD_NS),
                ChangeHint::Continuous(_) => panic!("authored holds use exact boundaries"),
            }
        }
    }
}

#[test]
fn quiet_is_a_visible_static_poster_without_time_only_wakeups() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::quiet();
    let poster = render(0, &profile, NATIVE, NATIVE);
    for time in [0, 1, 520_000_000, u64::MAX] {
        let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
        assert_eq!(
            design.sample(&context).change_hint(NATIVE_SIZE),
            ChangeHint::None
        );
        assert_eq!(render(time, &profile, NATIVE, NATIVE), poster);
    }
    assert_eq!(poster[(6, 3)].symbol(), "*");
    assert_eq!(poster[(5, 3)].symbol(), "-");
    assert!(
        poster
            .content
            .iter()
            .all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset)
    );
}

#[test]
fn native_subsizes_use_one_stationary_center_and_no_hint() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for width in 0..=13 {
        for height in 0..=7 {
            if width == 13 && height == 7 {
                continue;
            }
            let area = Rect::new(2, 1, width, height);
            let actual = Rect::new(0, 0, 20, 10);
            let before = base(actual);
            let after = render(u64::MAX, &profile, area, actual);
            assert_eq!(
                design.sample(&context).change_hint(area.as_size()),
                ChangeHint::None
            );
            assert_eq!(after, render(0, &profile, area, actual));
            let mut changes = 0;
            for y in 0..actual.height {
                for x in 0..actual.width {
                    if after[(x, y)] != before[(x, y)] {
                        changes += 1;
                        assert_eq!((x, y), (area.x + width / 2, area.y + height / 2));
                        assert_eq!(after[(x, y)].symbol(), "*");
                    }
                }
            }
            assert_eq!(changes, usize::from(width > 0 && height > 0));
        }
    }
}

#[test]
fn wide_or_tall_compact_requests_preserve_the_requested_center() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for (requested, actual) in [
        (Rect::new(2, 1, 101, 2), Rect::new(45, 0, 15, 4)),
        (Rect::new(2, 1, 2, 101), Rect::new(0, 45, 5, 15)),
        (Rect::new(0, 0, u16::MAX, 1), Rect::new(32_760, 0, 15, 1)),
    ] {
        let before = base(actual);
        let after = render(u64::MAX, &profile, requested, actual);
        assert_eq!(after, render(0, &profile, requested, actual));
        assert_eq!(
            design.sample(&context).change_hint(requested.as_size()),
            ChangeHint::None
        );
        let center = (
            requested.x + requested.width / 2,
            requested.y + requested.height / 2,
        );
        assert_eq!(after[center].symbol(), "*");
        assert_eq!(
            after
                .content
                .iter()
                .zip(&before.content)
                .filter(|(a, b)| a != b)
                .count(),
            1
        );
    }
}

#[test]
fn larger_views_center_native_art_and_leave_the_rest_of_the_host_untouched() {
    let profile = spark_runtime::Profile::default();
    let area = Rect::new(2, 1, 101, 31);
    let actual = Rect::new(0, 0, 106, 35);
    let before = base(actual);
    let after = render(520_000_000, &profile, area, actual);
    let native = render(520_000_000, &profile, NATIVE, NATIVE);
    let ox = area.x + (area.width - NATIVE.width) / 2;
    let oy = area.y + (area.height - NATIVE.height) / 2;
    for y in 0..actual.height {
        for x in 0..actual.width {
            if (ox..ox + NATIVE.width).contains(&x) && (oy..oy + NATIVE.height).contains(&y) {
                assert_eq!(after[(x, y)], native[(x - ox, y - oy)]);
            } else {
                assert_eq!(after[(x, y)], before[(x, y)]);
            }
        }
    }
}

#[test]
fn clipping_keeps_requested_coordinates_instead_of_recentering() {
    let profile = spark_runtime::Profile::default();
    let requested = Rect::new(3, 2, 19, 11);
    let full = render(520_000_000, &profile, requested, Rect::new(0, 0, 30, 16));
    for actual in [
        Rect::new(9, 5, 8, 5),
        Rect::new(0, 0, 5, 4),
        Rect::new(24, 14, 5, 2),
    ] {
        let clipped = render(520_000_000, &profile, requested, actual);
        for y in actual.y..actual.bottom() {
            for x in actual.x..actual.right() {
                assert_eq!(clipped[(x, y)], full[(x, y)]);
            }
        }
    }
    let actual = Rect::new(0, 0, 20, 10);
    for requested in [
        Rect::new(60_000, 60_000, 100, 100),
        Rect::new(0, 0, u16::MAX, u16::MAX),
    ] {
        assert_eq!(render(u64::MAX, &profile, requested, actual), base(actual));
    }
}

#[test]
fn protected_content_and_complete_wide_or_special_footprints_survive() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::default();
    let context = SampleContext::new(ClipTime::from_nanos(520_000_000), &palette, &profile);
    let requested = Rect::new(2, 1, 13, 7);
    let actual = Rect::new(0, 0, 18, 10);
    let protected = [Rect::new(5, 2, 3, 2)];
    let mut buffer = base(actual);
    buffer.set_string(1, 3, "界", Style::new()); // Origin before requested left edge.
    buffer.set_string(7, 4, "界", Style::new());
    buffer[(10, 5)].set_diff_option(CellDiffOption::ForcedWidth(3.try_into().unwrap()));
    buffer[(5, 6)].set_diff_option(CellDiffOption::Skip);
    let before = buffer.clone();
    design
        .sample(&context)
        .with_protected(&protected)
        .render(requested, &mut buffer);
    for y in 0..actual.height {
        for x in 0..actual.width {
            if !(2..15).contains(&x)
                || !(1..8).contains(&y)
                || ((5..8).contains(&x) && (2..4).contains(&y))
                || (y == 3 && (1..3).contains(&x))
                || (y == 4 && (7..9).contains(&x))
                || (y == 5 && (10..13).contains(&x))
                || (x == 5 && y == 6)
            {
                assert_eq!(
                    buffer[(x, y)],
                    before[(x, y)],
                    "host cell ({x},{y}) changed"
                );
            }
        }
    }
    assert_ne!(buffer[(8, 1)], before[(8, 1)]);
}

#[test]
fn ascii_and_unicode_profiles_share_safe_ascii_and_colors_follow_the_host() {
    let design = HangingLantern::new();
    let palette = Palette {
        background: Some(Color::Rgb(17, 28, 39)),
        ..Palette::amber()
    };
    for colors in [
        ColorProfile::TrueColor,
        ColorProfile::Ansi16,
        ColorProfile::Monochrome,
    ] {
        let mut frames = Vec::new();
        for glyphs in [GlyphProfile::Ascii, GlyphProfile::Unicode] {
            let profile = spark_runtime::Profile {
                glyphs,
                colors,
                motion: Motion::Full,
            };
            let context = SampleContext::new(ClipTime::from_nanos(520_000_000), &palette, &profile);
            let mut buffer = base(NATIVE);
            design.sample(&context).render(NATIVE, &mut buffer);
            assert_eq!(
                buffer[(6, 3)].fg,
                palette.resolve(palette.highlight, colors).fg.unwrap()
            );
            assert_eq!(
                buffer[(0, 0)].bg,
                palette.resolve(Style::new(), colors).bg.unwrap()
            );
            assert!(buffer.content.iter().all(|cell| cell.symbol().is_ascii()));
            if colors != ColorProfile::TrueColor {
                assert!(
                    buffer
                        .content
                        .iter()
                        .all(|cell| !matches!(cell.fg, Color::Rgb(..))
                            && !matches!(cell.bg, Color::Rgb(..)))
                );
            }
            frames.push(buffer);
        }
        assert_eq!(frames[0], frames[1]);
    }
}

#[test]
fn host_owns_time_pause_visibility_and_presentation_deadlines() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::default();
    let mut playback = Playback::new(HostTime::ZERO);
    let planner = RepaintPlanner::new(Duration::from_millis(16));
    let now = HostTime::from_nanos(239_000_000);
    let context = SampleContext::new(playback.sample(now), &palette, &profile);
    let hint = design.sample(&context).change_hint(NATIVE_SIZE);
    assert_eq!(hint, ChangeHint::At(ClipTime::from_nanos(240_000_000)));
    assert_eq!(
        planner.next_deadline(now, now, &playback, hint),
        Some(HostTime::from_nanos(255_000_000))
    );
    playback.pause(now).unwrap();
    assert_eq!(planner.next_deadline(now, now, &playback, hint), None);
    playback.resume(now).unwrap();
    playback.set_hidden(now, true).unwrap();
    assert_eq!(planner.next_deadline(now, now, &playback, hint), None);
    // A paused/hidden or static sample still paints on an unrelated host redraw.
    let context = SampleContext::new(playback.sample(now), &palette, &profile);
    let mut buffer = base(NATIVE);
    design.sample(&context).render(NATIVE, &mut buffer);
    assert_ne!(buffer, base(NATIVE));
}

#[test]
fn empty_and_fully_protected_native_areas_do_not_write() {
    let design = HangingLantern::new();
    let palette = Palette::default();
    let profile = spark_runtime::Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for requested in [Rect::new(0, 0, 0, 0), NATIVE] {
        let mut buffer = base(NATIVE);
        design
            .sample(&context)
            .with_protected(&[NATIVE])
            .render(requested, &mut buffer);
        assert_eq!(buffer, base(NATIVE));
    }
    assert_eq!(
        design.sample(&context).change_hint(Size::ZERO),
        ChangeHint::None
    );
}
