//! Independent consumer tests for the host-selected status family.
use ratatui_core::{
    buffer::{Buffer, Cell, CellDiffOption},
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_catalog::status::{StatusKind, StatusSample, StatusTheme};
use spark_runtime::{
    ChangeHint, ClipTime, ColorProfile, GlyphProfile, Motion, Palette, Period, Profile,
    SampleContext,
};
use std::{num::NonZeroU16, time::Duration};

const STATES: [StatusKind; 4] = [
    StatusKind::Loading,
    StatusKind::Empty,
    StatusKind::Success,
    StatusKind::Error,
];
const NATIVE: Rect = Rect::new(0, 0, 23, 7);
const SIZE: Size = Size::new(23, 7);

fn period(nanos: u64) -> Period {
    Period::new(Duration::from_nanos(nanos)).unwrap()
}
fn ascii(motion: Motion) -> Profile {
    Profile {
        glyphs: GlyphProfile::Ascii,
        motion,
        ..Profile::default()
    }
}
fn render(
    design: &StatusTheme,
    status: StatusKind,
    time: u64,
    profile: Profile,
    area: Rect,
) -> Buffer {
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
    let sample = design.sample(&context, status);
    let mut buffer = Buffer::empty(area);
    let borrowed: &StatusSample<'_> = &sample;
    borrowed.render(area, &mut buffer);
    buffer
}
fn rows(buffer: &Buffer) -> Vec<String> {
    (buffer.area.y..buffer.area.bottom())
        .map(|y| {
            (buffer.area.x..buffer.area.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}
fn host_base(area: Rect) -> Buffer {
    let mut cell = Cell::new("H");
    cell.set_style(
        Style::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .underline_color(Color::Yellow)
            .add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
    );
    Buffer::filled(area, cell)
}

#[test]
fn configuration_and_current_status_are_explicit_and_borrowed() {
    let design = StatusTheme::new()
        .with_loading_period(period(17))
        .with_transition_duration(period(11));
    assert_eq!(design.loading_period(), period(17));
    assert_eq!(design.transition_duration(), period(11));
    assert_eq!(StatusTheme::WIDTH, 23);
    assert_eq!(StatusTheme::HEIGHT, 7);
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for state in STATES {
        let sample = design.sample(&context, state);
        assert_eq!(sample.status(), state);
        let mut owned = Buffer::empty(NATIVE);
        let mut borrowed = owned.clone();
        sample.render(NATIVE, &mut owned);
        let borrowed_sample: &StatusSample<'_> = &sample;
        borrowed_sample.render(NATIVE, &mut borrowed);
        assert_eq!(owned, borrowed);
        assert_eq!(rows(&owned)[6].trim(), state.label());
    }
}

#[test]
fn quiet_states_have_exact_distinct_independent_silhouettes() {
    let expected = [
        [
            "       .-------.       ",
            "      / .-   -. \\      ",
            "  ---<    >o<    >---  ",
            "      \\ '-   -' /      ",
            "       `-------'       ",
            "                       ",
            "        LOADING        ",
        ],
        [
            "       .--- ---.       ",
            "      /         \\      ",
            "  ...<   \\   /   >...  ",
            "      \\   ___   /      ",
            "       `-------'       ",
            "                       ",
            "         EMPTY         ",
        ],
        [
            "       .-------.       ",
            "      /      /  \\      ",
            "  ---<   \\  /    >---  ",
            "      \\   \\/    /      ",
            "       `-------'       ",
            "                       ",
            "        SUCCESS        ",
        ],
        [
            "       .--/ \\--.       ",
            "      /    |    \\      ",
            "  ---<     |     >---  ",
            "      \\    o    /      ",
            "       `--\\ /--'       ",
            "                       ",
            "         ERROR         ",
        ],
    ];
    for (state, expected) in STATES.into_iter().zip(expected) {
        assert!(expected.iter().all(|row| row.len() == 23));
        let buffer = render(&StatusTheme::new(), state, 0, Profile::quiet(), NATIVE);
        assert_eq!(rows(&buffer), expected, "state {state:?}");
        assert!(buffer.content.iter().all(|cell| {
            cell.fg == Color::Reset
                && cell.bg == Color::Reset
                && cell.underline_color == Color::Reset
        }));
        assert_eq!(
            buffer,
            render(
                &StatusTheme::new(),
                state,
                u64::MAX,
                Profile::quiet(),
                NATIVE
            )
        );
    }
}

#[test]
fn labels_are_truthful_at_every_phase_and_never_inferred_from_time() {
    let design = StatusTheme::new();
    for profile in [
        ascii(Motion::Full),
        ascii(Motion::Reduced),
        Profile::quiet(),
    ] {
        for state in STATES {
            for time in [
                0,
                1,
                239_999_999,
                240_000_000,
                480_000_000,
                720_000_000,
                959_999_999,
                960_000_000,
                2_400_000_000,
                u64::MAX,
            ] {
                let actual = render(&design, state, time, profile, NATIVE);
                assert_eq!(rows(&actual)[6].trim(), state.label());
            }
        }
    }
    assert_eq!(
        rows(&render(
            &design,
            StatusKind::Loading,
            u64::MAX,
            ascii(Motion::Full),
            NATIVE
        ))[6]
            .trim(),
        "LOADING"
    );
}

#[test]
fn shuffled_late_and_replaced_samples_keep_no_hidden_state() {
    let design = StatusTheme::new();
    let times = [
        0,
        299_999_999,
        300_000_000,
        720_000_000,
        959_999_999,
        960_000_000,
        2_400_000_000,
        u64::MAX,
    ];
    for profile in [
        ascii(Motion::Full),
        ascii(Motion::Reduced),
        Profile::quiet(),
    ] {
        let palette = Palette::default();
        let mut reused = Buffer::empty(NATIVE);
        for i in [7, 2, 0, 3, 1, 5, 6, 4, 7, 0] {
            for state in [
                StatusKind::Success,
                StatusKind::Loading,
                StatusKind::Error,
                StatusKind::Empty,
            ] {
                let context =
                    SampleContext::new(ClipTime::from_nanos(times[i]), &palette, &profile);
                design.sample(&context, state).render(NATIVE, &mut reused);
                assert_eq!(reused, render(&design, state, times[i], profile, NATIVE));
            }
        }
    }
}

#[test]
fn loading_loops_and_empty_is_quiet_at_all_times() {
    let design = StatusTheme::new();
    let period = design.loading_period().as_nanos();
    for motion in [Motion::Full, Motion::Reduced] {
        let factor = if motion == Motion::Reduced { 4 } else { 1 };
        let profile = ascii(motion);
        for time in [0, 1, period / 3, period - 1] {
            assert_eq!(
                render(&design, StatusKind::Loading, time, profile, NATIVE),
                render(
                    &design,
                    StatusKind::Loading,
                    time + period * factor,
                    profile,
                    NATIVE
                )
            );
        }
        assert_ne!(
            render(&design, StatusKind::Loading, 0, profile, NATIVE),
            render(
                &design,
                StatusKind::Loading,
                period * factor / 2,
                profile,
                NATIVE
            )
        );
        let empty = render(&design, StatusKind::Empty, 0, profile, NATIVE);
        assert_eq!(
            empty,
            render(&design, StatusKind::Empty, u64::MAX, profile, NATIVE)
        );
        let palette = Palette::default();
        let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
        assert_eq!(
            design.sample(&context, StatusKind::Empty).change_hint(SIZE),
            ChangeHint::None
        );
    }
}

#[test]
fn finite_arrivals_settle_exactly_and_never_restart_or_expire() {
    for duration in [1, 2, 3, 7, 11, 960_000_000, u64::MAX] {
        let design = StatusTheme::new().with_transition_duration(period(duration));
        let palette = Palette::default();
        for motion in [Motion::Full, Motion::Reduced] {
            let profile = ascii(motion);
            for state in [StatusKind::Success, StatusKind::Error] {
                let before =
                    SampleContext::new(ClipTime::from_nanos(duration - 1), &palette, &profile);
                let end = SampleContext::new(ClipTime::from_nanos(duration), &palette, &profile);
                assert_eq!(
                    design.sample(&before, state).transition_complete(SIZE),
                    Some(false)
                );
                assert_eq!(
                    design.sample(&before, state).change_hint(SIZE),
                    ChangeHint::At(ClipTime::from_nanos(duration))
                );
                assert_eq!(
                    design.sample(&end, state).transition_complete(SIZE),
                    Some(true)
                );
                assert_eq!(
                    design.sample(&end, state).change_hint(SIZE),
                    ChangeHint::None
                );
                let final_pose = render(&design, state, duration, profile, NATIVE);
                assert_eq!(
                    final_pose,
                    render(&design, state, u64::MAX, profile, NATIVE)
                );
                assert_eq!(
                    final_pose,
                    render(&design, state, 0, ascii(Motion::Static), NATIVE)
                );
                assert_ne!(
                    final_pose,
                    render(&design, state, duration - 1, profile, NATIVE)
                );
            }
        }
    }
}

#[test]
fn reduced_motion_keeps_glyphs_fixed_with_a_finite_style_settle() {
    let design = StatusTheme::new();
    let profile = ascii(Motion::Reduced);
    for state in STATES {
        let first = render(&design, state, 0, profile, NATIVE);
        for time in [
            1,
            479_999_999,
            480_000_000,
            959_999_999,
            960_000_000,
            2_400_000_000,
            4_800_000_000,
            u64::MAX,
        ] {
            assert_eq!(
                rows(&first),
                rows(&render(&design, state, time, profile, NATIVE))
            );
        }
        if matches!(state, StatusKind::Success | StatusKind::Error) {
            assert_ne!(first, render(&design, state, 960_000_000, profile, NATIVE));
        }
    }
}

#[test]
fn every_small_period_boundary_is_exact_and_strictly_future() {
    let palette = Palette::default();
    for p in 1..=37 {
        let design = StatusTheme::new()
            .with_loading_period(period(p))
            .with_transition_duration(period(p));
        for motion in [Motion::Full, Motion::Reduced] {
            let profile = ascii(motion);
            let factor = if motion == Motion::Reduced { 4 } else { 1 };
            let loop_steps = if motion == Motion::Reduced { 4 } else { 8 };
            let finite_steps = if motion == Motion::Reduced { 2 } else { 4 };
            for time in 0..p * factor * 3 {
                let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
                let loop_index = |t| ((t / factor) % p) * loop_steps / p;
                let expected = if p == 1 {
                    None
                } else {
                    (time + 1..=time + p * factor)
                        .find(|&next| loop_index(next) != loop_index(time))
                };
                assert_eq!(
                    design
                        .sample(&context, StatusKind::Loading)
                        .change_hint(SIZE),
                    expected.map_or(ChangeHint::None, |t| ChangeHint::At(ClipTime::from_nanos(
                        t
                    ))),
                    "loop p={p}, t={time}, motion={motion:?}"
                );
                let expected = if time >= p {
                    None
                } else {
                    (time + 1..=p).find(|&next| next * finite_steps / p != time * finite_steps / p)
                };
                for state in [StatusKind::Success, StatusKind::Error] {
                    assert_eq!(
                        design.sample(&context, state).change_hint(SIZE),
                        expected.map_or(ChangeHint::None, |t| ChangeHint::At(
                            ClipTime::from_nanos(t)
                        )),
                        "finite p={p}, t={time}, motion={motion:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn maximum_time_and_period_do_not_overflow_or_fabricate_deadlines() {
    let palette = Palette::default();
    for p in [1, 2, 17, u64::MAX / 4, u64::MAX / 2 + 1, u64::MAX] {
        let design = StatusTheme::new()
            .with_loading_period(period(p))
            .with_transition_duration(period(p));
        for motion in [Motion::Full, Motion::Reduced, Motion::Static] {
            let profile = ascii(motion);
            for time in [0, 1, p - 1, p, u64::MAX - 1, u64::MAX] {
                let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
                for state in STATES {
                    let hint = design.sample(&context, state).change_hint(SIZE);
                    if let ChangeHint::At(next) = hint {
                        assert!(next.as_nanos() > time);
                    }
                    if time == u64::MAX {
                        assert_eq!(hint, ChangeHint::None);
                    }
                    render(&design, state, time, profile, NATIVE);
                }
            }
        }
    }
    let design = StatusTheme::new().with_loading_period(period(u64::MAX));
    let profile = ascii(Motion::Reduced);
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    // The first reduced boundary is ceil(MAX/4)*4, already unrepresentable.
    assert_eq!(
        design
            .sample(&context, StatusKind::Loading)
            .change_hint(SIZE),
        ChangeHint::None
    );
}

#[test]
fn compact_markers_are_static_and_honest_about_limited_label_space() {
    let design = StatusTheme::new();
    let palette = Palette::default();
    let profile = ascii(Motion::Full);
    for (state, marker) in STATES.into_iter().zip(['~', '-', '+', '!']) {
        for width in 0..=26 {
            for height in 0..=8 {
                if width >= 23 && height >= 7 {
                    continue;
                }
                let area = Rect::new(0, 0, width, height);
                let buffer = render(&design, state, 0, profile, area);
                assert_eq!(buffer, render(&design, state, u64::MAX, profile, area));
                let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
                let sample = design.sample(&context, state);
                assert_eq!(sample.change_hint(area.as_size()), ChangeHint::None);
                assert_eq!(
                    sample.transition_complete(area.as_size()),
                    matches!(state, StatusKind::Success | StatusKind::Error).then_some(true)
                );
                if width == 0 || height == 0 {
                    continue;
                }
                let output = rows(&buffer).join("\n");
                if width >= 11 {
                    assert!(output.contains(&format!("[{marker}] {}", state.label())));
                } else if width >= 7 {
                    assert!(output.contains(state.label()));
                } else if width >= 3 {
                    assert!(output.contains(&format!("[{marker}]")));
                    assert!(!output.contains(state.label()));
                } else {
                    assert!(output.contains(marker));
                }
            }
        }
    }
}

#[test]
fn static_and_nonfinite_completion_semantics_are_explicit() {
    let design = StatusTheme::new();
    let palette = Palette::default();
    for motion in [Motion::Full, Motion::Reduced, Motion::Static] {
        let profile = ascii(motion);
        let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
        for state in [StatusKind::Loading, StatusKind::Empty] {
            assert_eq!(
                design.sample(&context, state).transition_complete(SIZE),
                None
            );
        }
        for state in STATES {
            if motion == Motion::Static {
                let sample = design.sample(&context, state);
                assert_eq!(sample.change_hint(SIZE), ChangeHint::None);
                assert_eq!(
                    sample.transition_complete(SIZE),
                    matches!(state, StatusKind::Success | StatusKind::Error).then_some(true)
                );
            }
        }
    }
}

#[test]
fn clipping_retains_requested_coordinates_at_offsets_and_both_edges() {
    let design = StatusTheme::new();
    let palette = Palette::default();
    let profile = Profile::default();
    for requested in [
        Rect::new(3, 2, 23, 7),
        Rect::new(2, 3, 31, 11),
        Rect::new(4, 1, 11, 2),
    ] {
        for state in STATES {
            let context = SampleContext::new(ClipTime::from_nanos(720_000_000), &palette, &profile);
            let mut full = host_base(Rect::new(0, 0, 40, 18));
            design.sample(&context, state).render(requested, &mut full);
            for clip in [
                Rect::new(0, 0, 12, 6),
                Rect::new(8, 5, 16, 7),
                Rect::new(23, 3, 10, 10),
            ] {
                let mut clipped = host_base(clip);
                design
                    .sample(&context, state)
                    .render(requested, &mut clipped);
                for y in clip.y..clip.bottom() {
                    for x in clip.x..clip.right() {
                        assert_eq!(full[(x, y)], clipped[(x, y)], "state={state:?}, at={x},{y}");
                    }
                }
            }
        }
    }
}

#[test]
fn protections_wide_footprints_special_cells_and_neighbors_survive() {
    let design = StatusTheme::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let requested = Rect::new(3, 2, 23, 7);
    let bounds = Rect::new(0, 0, 32, 12);
    let protected = [Rect::new(7, 1, 2, 10), Rect::new(12, 6, 9, 2)];
    let mut base = host_base(bounds);
    base.set_string(2, 4, "界", Style::new());
    base.set_string(14, 4, "界", Style::new());
    base[(20, 4)].diff_option = CellDiffOption::ForcedWidth(NonZeroU16::new(3).unwrap());
    base[(10, 3)].diff_option = CellDiffOption::Skip;
    base[(11, 3)].set_symbol("\u{301}");
    for state in STATES {
        for time in [0, 720_000_000, u64::MAX] {
            let context = SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
            let mut after = base.clone();
            design
                .sample(&context, state)
                .with_protected(&protected)
                .render(requested, &mut after);
            for y in 0..bounds.height {
                for x in 0..bounds.width {
                    let excluded = protected.iter().any(|r| r.contains((x, y).into()));
                    let wide = y == 4 && [2, 3, 14, 15, 20, 21, 22].contains(&x);
                    let special = y == 3 && [10, 11].contains(&x);
                    if !requested.contains((x, y).into()) || excluded || wide || special {
                        assert_eq!(after[(x, y)], base[(x, y)], "state={state:?}, at={x},{y}");
                    }
                }
            }
        }
    }
}

#[test]
fn huge_requests_cap_art_and_checked_coordinate_addition() {
    let design = StatusTheme::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    for state in STATES {
        for area in [
            Rect::new(0, 0, u16::MAX, u16::MAX),
            Rect::new(u16::MAX, u16::MAX, 23, 7),
            Rect::new(u16::MAX - 2, u16::MAX - 2, u16::MAX, u16::MAX),
        ] {
            let mut buffer = host_base(Rect::new(0, 0, 3, 3));
            let before = buffer.clone();
            design.sample(&context, state).render(area, &mut buffer);
            assert_eq!(buffer, before);
        }
        let area = Rect::new(2, 3, 33, 13);
        let mut buffer = host_base(Rect::new(0, 0, 40, 20));
        let before = buffer.clone();
        design.sample(&context, state).render(area, &mut buffer);
        let owned = Rect::new(7, 6, 23, 7);
        for y in 0..20 {
            for x in 0..40 {
                if !owned.contains((x, y).into()) {
                    assert_eq!(buffer[(x, y)], before[(x, y)]);
                }
            }
        }
        assert_eq!(
            buffer
                .content
                .iter()
                .zip(&before.content)
                .filter(|(a, b)| a != b)
                .count(),
            161
        );
    }
}

#[test]
fn all_profiles_resolve_palette_roles_and_preserve_unknown_backgrounds() {
    let design = StatusTheme::new();
    let palette = Palette {
        foreground: Style::new()
            .fg(Color::Rgb(90, 80, 70))
            .underline_color(Color::Rgb(80, 70, 60)),
        muted: Style::new().fg(Color::Indexed(200)),
        accent: Style::new().fg(Color::Rgb(40, 130, 90)),
        highlight: Style::new().fg(Color::Rgb(220, 240, 250)),
        background: None,
    };
    for glyphs in [GlyphProfile::Ascii, GlyphProfile::Unicode] {
        for colors in [
            ColorProfile::TrueColor,
            ColorProfile::Ansi16,
            ColorProfile::Monochrome,
        ] {
            for motion in [Motion::Full, Motion::Reduced, Motion::Static] {
                let profile = Profile {
                    glyphs,
                    colors,
                    motion,
                };
                for state in STATES {
                    for time in [
                        0,
                        300_000_000,
                        720_000_000,
                        960_000_000,
                        2_100_000_000,
                        u64::MAX,
                    ] {
                        let context =
                            SampleContext::new(ClipTime::from_nanos(time), &palette, &profile);
                        let mut buffer = Buffer::empty(NATIVE);
                        design.sample(&context, state).render(NATIVE, &mut buffer);
                        assert!(buffer.content.iter().all(|cell| cell.bg == Color::Reset));
                        if glyphs == GlyphProfile::Ascii {
                            assert!(buffer.content.iter().all(|cell| cell.symbol().is_ascii()));
                        }
                        for cell in &buffer.content {
                            if colors != ColorProfile::TrueColor {
                                assert!(!matches!(cell.fg, Color::Rgb(..) | Color::Indexed(..)));
                                assert!(!matches!(
                                    cell.underline_color,
                                    Color::Rgb(..) | Color::Indexed(..)
                                ));
                            }
                            if colors == ColorProfile::Monochrome {
                                assert_eq!(cell.fg, Color::Reset);
                                assert_eq!(cell.underline_color, Color::Reset);
                            }
                        }
                        let label = &buffer[(8, 6)];
                        if state.label().len() == 7 {
                            let mut expected = Cell::new(" ");
                            expected.set_symbol(label.symbol());
                            expected.set_style(
                                palette.resolve(
                                    palette.foreground.add_modifier(Modifier::BOLD),
                                    colors,
                                ),
                            );
                            assert_eq!(label, &expected);
                        }
                    }
                }
            }
        }
    }
}
