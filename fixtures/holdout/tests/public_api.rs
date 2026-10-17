use ratatui_core::{
    buffer::{Buffer, Cell},
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use spark_public_holdout::{CrossWeave, WeaveSample};
use spark_runtime::{
    ChangeHint, ClipTime, HostTime, Palette, Playback, Profile, RepaintPlanner, SampleContext,
};
use std::time::Duration;

fn base(area: Rect) -> Buffer {
    let mut cell = Cell::new("H");
    cell.set_style(
        Style::new()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .add_modifier(Modifier::ITALIC),
    );
    Buffer::filled(area, cell)
}
fn frame(time: u64, profile: &Profile, requested: Rect, actual: Rect) -> Buffer {
    let design = CrossWeave::new().with_seed(u64::MAX);
    let palette = Palette::default();
    let context = SampleContext::new(ClipTime::from_nanos(time), &palette, profile);
    let sample = design.sample(&context);
    let mut buffer = base(actual);
    let borrowed = &sample;
    <&WeaveSample<'_> as Widget>::render(borrowed, requested, &mut buffer);
    buffer
}
#[test]
fn immutable_borrowed_samples_seek_directly_and_repeat() {
    let area = Rect::new(2, 1, 17, 7);
    let actual = Rect::new(0, 0, 22, 10);
    let profile = Profile::default();
    for time in [u64::MAX, 0, 799_999_999, 800_000_000, 80_000_000, 0] {
        assert_eq!(
            frame(time, &profile, area, actual),
            frame(time % 800_000_000, &profile, area, actual)
        );
    }
    assert_ne!(
        frame(0, &profile, area, actual),
        frame(80_000_000, &profile, area, actual)
    );
}
#[test]
fn clipping_never_restarts_the_authored_coordinates() {
    let requested = Rect::new(3, 2, 21, 11);
    let full = frame(
        333_333_333,
        &Profile::default(),
        requested,
        Rect::new(0, 0, 30, 15),
    );
    let actual = Rect::new(9, 5, 8, 5);
    let clipped = frame(333_333_333, &Profile::default(), requested, actual);
    for y in actual.y..actual.bottom() {
        for x in actual.x..actual.right() {
            assert_eq!(clipped[(x, y)], full[(x, y)]);
        }
    }
}
#[test]
fn extension_cannot_overwrite_wide_footprints_or_protected_host_content() {
    let design = CrossWeave::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    let requested = Rect::new(2, 1, 17, 7);
    let protected = [Rect::new(7, 1, 2, 7)];
    let mut buffer = base(Rect::new(0, 0, 23, 10));
    buffer.set_string(1, 3, "界", Style::new());
    buffer.set_string(12, 4, "界", Style::new());
    let before = buffer.clone();
    design
        .sample(&context)
        .with_protected(&protected)
        .render(requested, &mut buffer);
    for y in 0..10 {
        for x in 0..23 {
            if !(2..19).contains(&x)
                || !(1..8).contains(&y)
                || (7..9).contains(&x)
                || (y == 3 && (1..3).contains(&x))
                || (y == 4 && (12..14).contains(&x))
            {
                assert_eq!(buffer[(x, y)], before[(x, y)]);
            }
        }
    }
}
#[test]
fn static_and_tiny_views_paint_without_animation_requests() {
    let design = CrossWeave::new();
    let palette = Palette::default();
    let profile = Profile::quiet();
    let context = SampleContext::new(ClipTime::MAX, &palette, &profile);
    let area = Rect::new(0, 0, 17, 7);
    assert_eq!(
        design.sample(&context).change_hint(area.as_size()),
        ChangeHint::None
    );
    assert_eq!(
        frame(0, &profile, area, area),
        frame(u64::MAX, &profile, area, area)
    );
    assert_ne!(frame(0, &profile, area, area), base(area));
    let profile = Profile::default();
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    for size in [Size::new(0, 0), Size::new(2, 7), Size::new(17, 2)] {
        assert_eq!(design.sample(&context).change_hint(size), ChangeHint::None);
    }
}
#[test]
fn public_deadline_helpers_integrate_without_a_catalog_or_runtime_clock() {
    let design = CrossWeave::new();
    let palette = Palette::default();
    let profile = Profile::default();
    let playback = Playback::new(HostTime::ZERO);
    let context = SampleContext::new(playback.sample(HostTime::ZERO), &palette, &profile);
    let hint = design.sample(&context).change_hint(Size::new(17, 7));
    assert_eq!(hint, ChangeHint::At(ClipTime::from_nanos(80_000_000)));
    let planner = RepaintPlanner::new(Duration::from_millis(16));
    assert_eq!(
        planner.next_deadline(HostTime::ZERO, HostTime::ZERO, &playback, hint),
        Some(HostTime::from_nanos(80_000_000))
    );
}
