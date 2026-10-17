//! Consumer-facing authored-asset tests; expected glyphs are independent of the generator.
use ratatui_core::{
    buffer::{Buffer, Cell, CellDiffOption},
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_catalog::PocketMascot;
use spark_runtime::{
    ChangeHint, ClipTime, ColorProfile, GlyphProfile, Motion, Palette, Profile, SampleContext,
};
use std::time::Duration;

const ENDS: [u64; 8] = [900, 1020, 1180, 1280, 1420, 1620, 1800, 2400];
const POSES: [[&str; 6]; 8] = [
    [
        "....._o.....",
        ".../'  `\\...",
        "../ o  o \\..",
        "..|  <>  |~.",
        "...\\_==_/...",
        "..../  \\....",
    ],
    [
        "....._o.....",
        ".../'  `\\...",
        "../ -  - \\..",
        "..|  <>  |~.",
        "...\\_==_/...",
        "..../  \\....",
    ],
    [
        "............",
        "....__o.....",
        ".../ o o\\...",
        "..|  <>  |~.",
        "...\\_==_/...",
        "..._/  \\_...",
    ],
    [
        "......o_....",
        ".... /  `\\..",
        ".../ o o |..",
        "..~| <>  |..",
        "....\\_==/...",
        "...../ \\....",
    ],
    [
        ".......o....",
        "...../'`\\...",
        "..../ o o\\..",
        "..~.| <> |..",
        ".....\\==/...",
        "..../  \\....",
    ],
    [
        "......o_....",
        ".... /  `\\..",
        ".../ o o |..",
        "..~| <>  |..",
        "....\\_==/...",
        "...../ \\....",
    ],
    [
        "....._o.....",
        ".../'  `\\...",
        "../ o  o \\..",
        "..|  <>  |..",
        "...\\_==_/~..",
        "..../  \\....",
    ],
    [
        "....._o.....",
        ".../'  `\\...",
        "../ o  o \\..",
        "..|  <>  |~.",
        "...\\_==_/...",
        "..../  \\....",
    ],
];
fn base(area: Rect) -> Buffer {
    let mut cell = Cell::new("#");
    cell.set_style(
        Style::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
    );
    Buffer::filled(area, cell)
}
fn render(time: u64, area: Rect, bounds: Rect, profile: Profile) -> Buffer {
    let design = PocketMascot::new();
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
    let mut buffer = base(bounds);
    let view = design.sample(&context);
    // This is also an intentional check of the borrowed Widget implementation.
    let borrowed: &spark_catalog::MascotSample<'_> = &view;
    borrowed.render(area, &mut buffer);
    buffer
}
fn expected_pose(index: usize, bounds: Rect, ox: u16, oy: u16, profile: Profile) -> Buffer {
    let mut buffer = base(bounds);
    let palette = Palette::default();
    for (y, row) in POSES[index].iter().enumerate() {
        for (x, glyph) in row.chars().enumerate() {
            if glyph == '.' {
                continue;
            }
            let role = match glyph {
                'o' => palette.highlight,
                '~' => palette.muted,
                '/' | '\\' | '|' | '_' | '=' => palette.accent,
                _ => palette.foreground,
            };
            if let Some(cell) = buffer.cell_mut((ox + x as u16, oy + y as u16)) {
                cell.reset();
                cell.set_char(glyph)
                    .set_style(palette.resolve(role, profile.colors));
            }
        }
    }
    buffer
}

#[test]
fn every_authored_frame_and_hold_edge_matches_complete_cells() {
    let bounds = Rect::new(0, 0, 12, 6);
    let mut start = 0;
    for (index, end_ms) in ENDS.into_iter().enumerate() {
        let expected = expected_pose(index, bounds, 0, 0, Profile::default());
        for time in [start, start + 1, end_ms * 1_000_000 - 1] {
            assert_eq!(
                render(time, bounds, bounds, Profile::default()),
                expected,
                "frame {index}, time {time}"
            );
        }
        start = end_ms * 1_000_000;
    }
    assert_eq!(
        PocketMascot::new().period().as_duration(),
        Duration::from_millis(2400)
    );
}

#[test]
fn direct_shuffled_seek_and_loop_seams_do_not_advance_state() {
    let area = Rect::new(3, 4, 12, 6);
    let bounds = Rect::new(0, 0, 22, 13);
    let period = 2_400_000_000;
    let times = [
        u64::MAX,
        period + 1,
        899_999_999,
        900_000_000,
        period - 1,
        0,
        1_280_000_000,
        0,
        period,
        u64::MAX,
    ];
    let design = PocketMascot::new();
    let palette = Palette::default();
    let profile = Profile::default();
    for time in times {
        let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
        let view = design.sample(&context);
        let mut actual = base(bounds);
        view.render(area, &mut actual);
        assert_eq!(actual, render(time % period, area, bounds, profile));
        let mut repeated = base(bounds);
        view.render(area, &mut repeated);
        assert_eq!(actual, repeated);
    }
    // The authored final rest pose joins the first attentive pose without a jump.
    assert_eq!(
        render(period - 1, area, bounds, profile),
        render(period, area, bounds, profile)
    );
}

#[test]
fn transparency_preserves_entire_host_cells_while_spaces_erase_explicitly() {
    let bounds = Rect::new(0, 0, 12, 6);
    let before = base(bounds);
    let after = render(0, bounds, bounds, Profile::default());
    assert_eq!(after[(0, 0)], before[(0, 0)]); // Keep, including style.
    assert_eq!(after[(5, 1)].symbol(), " "); // Explicit Erase inside the face.
    assert_ne!(after[(5, 1)], before[(5, 1)]);
    assert_eq!(after[(5, 1)].modifier, Modifier::empty());
    assert_eq!(after[(5, 1)].bg, Color::Reset);
    assert_eq!(after[(5, 1)].fg, Palette::default().foreground.fg.unwrap());
}

#[test]
fn native_art_is_centered_without_stretch_and_clipping_keeps_composition_origin() {
    for (w, h) in [(12, 6), (13, 7), (20, 10), (65, 17)] {
        let area = Rect::new(3, 2, w, h);
        let bounds = Rect::new(0, 0, 75, 24);
        let ox = area.x + (w - 12) / 2;
        let oy = area.y + (h - 6) / 2;
        let actual = render(1_180_000_000, area, bounds, Profile::default());
        assert_eq!(actual, expected_pose(3, bounds, ox, oy, Profile::default()));
        let clipped_bounds = Rect::new(ox + 4, oy + 2, 5, 3);
        let clipped = render(1_180_000_000, area, clipped_bounds, Profile::default());
        for y in clipped_bounds.y..clipped_bounds.bottom() {
            for x in clipped_bounds.x..clipped_bounds.right() {
                assert_eq!(clipped[(x, y)], actual[(x, y)]);
            }
        }
    }
}

#[test]
fn protected_wide_and_special_diff_cells_remain_complete() {
    let bounds = Rect::new(0, 0, 18, 9);
    let area = Rect::new(2, 1, 12, 6);
    let mut before = base(bounds);
    before.set_string(5, 3, "界", Style::new().fg(Color::Green));
    before[(9, 3)].diff_option = CellDiffOption::Skip;
    let protected = [Rect::new(7, 1, 1, 6), Rect::new(0, 6, 18, 1)];
    let palette = Palette::default();
    let profile = Profile::default();
    let design = PocketMascot::new();
    for time in [0, 900_000_000, 1_180_000_000, u64::MAX] {
        let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
        let mut after = before.clone();
        design
            .sample(&context)
            .with_protected(&protected)
            .render(area, &mut after);
        for position in [(5, 3), (6, 3), (9, 3)] {
            assert_eq!(after[position], before[position]);
        }
        for y in 0..bounds.height {
            for x in 0..bounds.width {
                if protected.iter().any(|r| r.contains((x, y).into()))
                    || !area.contains((x, y).into())
                {
                    assert_eq!(after[(x, y)], before[(x, y)]);
                }
            }
        }
    }
}

#[test]
fn quiet_poster_and_reduced_motion_use_explicit_authored_time() {
    let area = Rect::new(0, 0, 12, 6);
    for time in [0, 900_000_000, 1_280_000_000, u64::MAX] {
        let quiet = Profile::quiet();
        assert_eq!(
            render(time, area, area, quiet),
            expected_pose(0, area, 0, 0, quiet)
        );
        let reduced = Profile {
            motion: Motion::Reduced,
            ..Profile::default()
        };
        assert_eq!(
            render(time, area, area, reduced),
            render(time / 4, area, area, Profile::default())
        );
    }
    for colors in [
        ColorProfile::TrueColor,
        ColorProfile::Ansi16,
        ColorProfile::Monochrome,
    ] {
        let ascii = Profile {
            glyphs: GlyphProfile::Ascii,
            colors,
            motion: Motion::Full,
        };
        let unicode = Profile {
            glyphs: GlyphProfile::Unicode,
            ..ascii
        };
        assert_eq!(
            render(1_180_000_000, area, area, ascii),
            render(1_180_000_000, area, area, unicode)
        );
        assert_eq!(
            render(1_180_000_000, area, area, ascii),
            expected_pose(3, area, 0, 0, ascii)
        );
    }
}

#[test]
fn compact_empty_and_extreme_requested_geometries_are_static_and_bounded() {
    let bounds = Rect::new(0, 0, 20, 10);
    for width in 0..=13 {
        for height in 0..=7 {
            let area = Rect::new(2, 1, width, height);
            if width >= 12 && height >= 6 {
                continue;
            }
            let actual = render(0, area, bounds, Profile::default());
            assert_eq!(actual, render(u64::MAX, area, bounds, Profile::default()));
            let mut expected = base(bounds);
            if width > 0 && height > 0 {
                let text = if width >= 3 { "o-o" } else { "o" };
                let ox = area.x + (width - text.len() as u16) / 2;
                let oy = area.y + (height - 1) / 2;
                for (i, glyph) in text.chars().enumerate() {
                    expected[(ox + i as u16, oy)].reset();
                    expected[(ox + i as u16, oy)]
                        .set_char(glyph)
                        .set_style(Palette::default().highlight);
                }
            }
            assert_eq!(actual, expected, "size {width}x{height}");
        }
    }
    for area in [
        Rect::new(u16::MAX - 1, u16::MAX - 1, 12, 6),
        Rect::new(0, 0, u16::MAX, u16::MAX),
    ] {
        assert_eq!(
            render(u64::MAX, area, bounds, Profile::default()),
            base(bounds)
        );
    }
}

#[test]
fn frame_cadence_is_advice_only_and_quiet_presentations_need_none() {
    let design = PocketMascot::new();
    let palette = Palette::default();
    for motion in [Motion::Full, Motion::Reduced, Motion::Static] {
        let profile = Profile {
            motion,
            ..Profile::default()
        };
        let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
        let sample = design.sample(&context);
        for size in [
            Size::new(0, 6),
            Size::new(12, 0),
            Size::new(11, 6),
            Size::new(12, 5),
            Size::new(1, 1),
        ] {
            assert_eq!(sample.frame_interval(size), None);
        }
        assert_eq!(
            sample.frame_interval(Size::new(12, 6)),
            match motion {
                Motion::Full => Some(Duration::from_millis(100)),
                Motion::Reduced => Some(Duration::from_millis(400)),
                Motion::Static => None,
            }
        );
    }
}

#[test]
fn discrete_hints_are_absolute_strictly_future_and_overflow_safe() {
    let design = PocketMascot::new();
    let palette = Palette::default();
    for scale in [1u64, 4] {
        let profile = Profile {
            motion: if scale == 1 {
                Motion::Full
            } else {
                Motion::Reduced
            },
            ..Profile::default()
        };
        let period = 2_400_000_000 * scale;
        for cycle in [0, 1, 2, 5000] {
            let mut start = cycle * period;
            for end in ENDS {
                let boundary = cycle * period + end * 1_000_000 * scale;
                for time in [start, start + 1, boundary - 1] {
                    let context =
                        SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
                    let actual = design.sample(&context).change_hint(Size::new(12, 6));
                    assert_eq!(actual, ChangeHint::At(ClipTime::from_nanos(boundary)));
                    assert!(boundary > time);
                }
                start = boundary;
            }
        }
        let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
        assert_eq!(
            design.sample(&context).change_hint(Size::new(12, 6)),
            ChangeHint::None
        );
        for size in [
            Size::new(0, 0),
            Size::new(0, 6),
            Size::new(11, 6),
            Size::new(12, 5),
        ] {
            let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
            assert_eq!(design.sample(&context).change_hint(size), ChangeHint::None);
        }
    }
    let quiet = Profile::quiet();
    let context = SampleContext::new(ClipTime::ZERO, &palette, &quiet);
    assert_eq!(
        design.sample(&context).change_hint(Size::new(12, 6)),
        ChangeHint::None
    );
}
