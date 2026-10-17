//! Application-level fake-clock tests shared by synchronous, async, and paced hosts.
#[path = "../examples/support/mod.rs"]
mod support;
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Size};
use spark_catalog::Progress;
use spark_runtime::{ClipTime, HiddenPolicy, HostTime, Motion, PlaybackRate, Profile};
use std::time::Duration;
use support::{
    ALL_DESIGNS, ALL_LAYOUTS, DataUpdate, Design, Host, HostStyle, LayoutMode, OperationState, draw,
};

fn ms(value: u64) -> HostTime {
    HostTime::from_nanos(value * 1_000_000)
}
fn size() -> Size {
    Size::new(100, 30)
}
fn paint(host: &Host, now: HostTime, size: Size) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(size.width, size.height)).unwrap();
    terminal.draw(|frame| draw(frame, host, now)).unwrap();
    terminal.backend().buffer().clone()
}
fn frame(host: &mut Host, now: HostTime) -> Buffer {
    let buffer = paint(host, now, size());
    host.presented(now, size());
    buffer
}

#[test]
fn all_three_host_smokes_cover_six_designs_four_layouts_and_independent_data() {
    let sync = support::smoke(HostStyle::Synchronous);
    let asynchronous = support::smoke(HostStyle::Asynchronous);
    let paced = support::smoke(HostStyle::Paced);
    for report in [sync, asynchronous, paced] {
        assert_eq!(report.data_polls, 4 * 6 * 4);
        assert!(report.frames > 24);
        assert!(report.final_paints >= 4);
    }
    assert_eq!(
        paced.animation_wakes, 0,
        "preexisting pacing adds no art-only wakes"
    );
}

#[test]
fn all_designs_render_in_every_layout_through_the_same_sample_api() {
    for layout in ALL_LAYOUTS {
        for design in ALL_DESIGNS {
            let mut host = Host::for_design(design, layout);
            let with_art = paint(&host, ms(700), size());
            host.art_enabled = false;
            let base = paint(&host, ms(700), size());
            assert_ne!(with_art, base, "{design:?} in {layout:?} must appear");
        }
    }
}

#[test]
fn event_storm_coalesces_without_bypassing_the_presentation_cap() {
    let mut host = Host::for_design(Design::Comet, LayoutMode::Dashboard);
    host.producer_enabled = false;
    frame(&mut host, ms(0));
    assert_eq!(host.next_presentation(), Some(ms(33)));
    for _event in 0..1000 {
        host.dirty = true;
        host.refresh_animation(ms(1), size());
        assert_eq!(host.next_presentation(), Some(ms(16)));
    }
    assert_eq!(host.frames, 1);
    frame(&mut host, ms(16));
    assert_eq!(host.next_presentation(), Some(ms(49)));
    assert_eq!(host.frames, 2);
}

#[test]
fn long_stalls_sample_current_pose_once_without_frame_or_data_backlog() {
    let mut host = Host::for_design(Design::Comet, LayoutMode::Dashboard);
    frame(&mut host, ms(0));
    assert_eq!(host.next_wake(), Some(ms(33)));
    assert!(host.poll_data_if_due(ms(10_000)));
    assert_eq!(host.data_polls, 1);
    assert_eq!(host.next_data, ms(10_500));
    let current = frame(&mut host, ms(10_000));
    assert_eq!(
        host.playback.sample(ms(10_000)),
        ClipTime::from_nanos(10_000_000_000)
    );
    assert_eq!(host.frames, 2);
    assert_eq!(host.next_presentation(), Some(ms(10_033)));
    assert_eq!(current, paint(&host, ms(10_000), size()));
    assert!(!host.poll_data_if_due(ms(10_000)));
}

#[test]
fn paused_art_is_retained_on_unrelated_redraws_without_animation_wakes() {
    for design in ALL_DESIGNS {
        let mut host = Host::for_design(design, LayoutMode::Dashboard);
        host.producer_enabled = false;
        let before = frame(&mut host, ms(500));
        host.pause_or_resume(ms(501));
        let after = frame(&mut host, ms(10_000));
        assert_eq!(before, after, "{design:?} freezes the displayed pose");
        assert_eq!(host.next_wake(), None);
        for time in [10_100, 20_000, 100_000] {
            assert_eq!(frame(&mut host, ms(time)), before);
            assert_eq!(host.next_wake(), None);
        }
        host.pause_or_resume(ms(100_000));
        assert_eq!(
            host.playback.sample(ms(100_100)),
            ClipTime::from_nanos(600_000_000)
        );
    }
}

#[test]
fn static_hidden_and_empty_presentations_stop_art_only_wakes() {
    for design in ALL_DESIGNS {
        let mut host = Host::for_design(design, LayoutMode::Dashboard);
        host.producer_enabled = false;
        host.profile = Profile::quiet();
        let before = frame(&mut host, ms(0));
        assert_eq!(host.next_wake(), None);
        assert_eq!(frame(&mut host, ms(50_000)), before);
        host.profile.motion = Motion::Full;
        host.toggle_hidden(ms(50_001));
        frame(&mut host, ms(50_001));
        assert_eq!(host.next_wake(), None);
        host.toggle_hidden(ms(80_000));
        host.presented(ms(80_000), Size::new(0, 0));
        assert_eq!(host.next_wake(), None);
    }
}

#[test]
fn completed_reveal_gets_its_final_paint_then_becomes_idle() {
    let mut host = Host::for_design(Design::Reveal, LayoutMode::Request);
    host.producer_enabled = false;
    let mut now = ms(0);
    loop {
        frame(&mut host, now);
        match host.next_presentation() {
            Some(next) => {
                assert!(next > now);
                now = next;
            }
            None => break,
        }
        assert!(
            now <= ms(1300),
            "the final frame must not be lost or keep waking"
        );
    }
    assert!(now >= ms(1200));
    assert_eq!(host.final_paints, 1);
    assert_eq!(host.next_wake(), None);
    let final_pose = frame(&mut host, ms(5000));
    assert_eq!(
        host.final_paints, 1,
        "unrelated redraw isn't a new completion"
    );
    assert_eq!(final_pose, paint(&host, ms(9000), size()));
    assert_eq!(host.next_wake(), None);
}

#[test]
fn replacement_cancellation_and_sorting_keep_generation_and_identity() {
    let mut host = Host::new(LayoutMode::Dashboard);
    let old = *host.selected();
    host.sort_rows();
    assert_eq!(host.selected().id, old.id);
    assert_eq!(host.selected().generation, old.generation);
    host.replace_selected(ms(100));
    let current = *host.selected();
    assert_eq!(current.generation, old.generation + 1);
    assert_eq!(current.progress, Progress::unknown());
    assert!(!host.apply_update(
        DataUpdate {
            id: old.id,
            generation: old.generation,
            progress: Progress::known(1.0).unwrap()
        },
        ms(200)
    ));
    assert_eq!(host.selected().progress, Progress::unknown());
    assert!(!host.active(Design::Reveal));
    assert!(host.apply_update(
        DataUpdate {
            id: current.id,
            generation: current.generation,
            progress: Progress::known(0.25).unwrap()
        },
        ms(201)
    ));
    host.cancel_selected();
    assert!(!host.apply_update(
        DataUpdate {
            id: current.id,
            generation: current.generation,
            progress: Progress::known(1.0).unwrap()
        },
        ms(202)
    ));
    assert_eq!(host.selected().state, OperationState::Cancelled);
    assert!(!host.active(Design::Reveal));
    assert_eq!(host.rejected_updates, 2);
    host.replace_selected(ms(300));
    let replacement = *host.selected();
    assert!(host.apply_update(
        DataUpdate {
            id: replacement.id,
            generation: replacement.generation,
            progress: Progress::known(1.0).unwrap()
        },
        ms(400)
    ));
    assert_eq!(host.reveal_playback.sample(ms(400)), ClipTime::ZERO);
    assert!(host.active(Design::Reveal));
    assert_eq!(host.final_paints, 0);
}

#[test]
fn real_progress_completion_restarts_reveal_without_reusing_old_visual_time() {
    let mut host = Host::new(LayoutMode::Dashboard);
    host.only = Some(Design::Reveal);
    host.producer_enabled = false;
    let selected = *host.selected();
    assert!(host.apply_update(
        DataUpdate {
            id: selected.id,
            generation: selected.generation,
            progress: Progress::known(1.0).unwrap()
        },
        ms(10_000)
    ));
    frame(&mut host, ms(10_000));
    assert_eq!(host.final_paints, 0);
    assert!(host.next_presentation().is_some());
    frame(&mut host, ms(11_200));
    assert_eq!(host.final_paints, 1);
    assert_eq!(host.next_wake(), None);
}

#[test]
fn hidden_freeze_and_catchup_are_host_policies() {
    for policy in [HiddenPolicy::Freeze, HiddenPolicy::CatchUp] {
        let mut host = Host::new(LayoutMode::Dashboard);
        host.playback.set_hidden_policy(ms(0), policy).unwrap();
        host.toggle_hidden(ms(100));
        host.presented(ms(100), size());
        assert_eq!(host.animation_due, None);
        host.toggle_hidden(ms(10_100));
        let expected = if policy == HiddenPolicy::Freeze {
            ms(100)
        } else {
            ms(10_100)
        };
        assert_eq!(
            host.playback.sample(ms(10_100)).as_nanos(),
            expected.as_nanos()
        );
        frame(&mut host, ms(10_100));
        assert!(host.animation_due.is_some());
    }
}

#[test]
fn producer_count_is_independent_of_art_and_presentation_rate() {
    fn simulate(art: bool, paced: bool) -> u64 {
        let mut host = Host::new(LayoutMode::Dashboard);
        host.art_enabled = art;
        let end = ms(3000);
        let mut now = ms(0);
        loop {
            host.poll_data_if_due(now);
            if paced || host.next_presentation().is_some_and(|due| now >= due) {
                host.presented(now, size());
            }
            if now == end {
                return host.data_polls;
            }
            now = if paced {
                support::add(now, Duration::from_millis(16)).min(host.next_data)
            } else {
                host.next_wake().unwrap_or(end)
            }
            .min(end);
        }
    }
    for art in [false, true] {
        for paced in [false, true] {
            assert_eq!(simulate(art, paced), 6);
        }
    }
}

#[test]
fn reader_decorators_preserve_titles_selection_wide_text_caret_and_media() {
    let mut host = Host::new(LayoutMode::Reader);
    let with_art = paint(&host, ms(700), size());
    host.art_enabled = false;
    let base = paint(&host, ms(700), size());
    for region in support::protected_regions(host.layout, with_art.area) {
        for y in region.y..region.bottom() {
            for x in region.x..region.right() {
                assert_eq!(with_art[(x, y)], base[(x, y)], "protected cell {x},{y}");
            }
        }
    }
    assert_ne!(with_art, base);
}

#[test]
fn rapid_resize_reconstruction_and_fractional_rate_commands_stay_safe() {
    let mut host = Host::new(LayoutMode::Dashboard);
    host.set_rate(ms(0), PlaybackRate::new(1, 3).unwrap());
    for (index, (width, height)) in [
        (0, 0),
        (1, 1),
        (7, 3),
        (100, 30),
        (24, 8),
        (160, 50),
        (100, 30),
    ]
    .into_iter()
    .enumerate()
    {
        let now = ms(index as u64 * 100);
        let requested = Size::new(width, height);
        assert_eq!(paint(&host, now, requested), paint(&host, now, requested));
        host.presented(now, requested);
        if let Some(deadline) = host.animation_due {
            assert!(deadline > now);
        }
    }
    assert_eq!(host.playback.sample(ms(600)).as_nanos(), 200_000_000);
}

#[test]
fn table_art_moves_with_its_entity_when_rows_are_reordered() {
    let mut host = Host::new(LayoutMode::Table);
    host.profile = Profile::quiet();
    let before = paint(&host, ms(700), size());
    let old = host.placements(before.area);
    let selected = *host.selected();
    host.sort_rows();
    let after = paint(&host, ms(700), size());
    let new = host.placements(after.area);
    assert_eq!(*host.selected(), selected);
    for design in [Design::Assembly, Design::Comet] {
        let from = old.iter().find(|(kind, _)| *kind == design).unwrap().1;
        let to = new.iter().find(|(kind, _)| *kind == design).unwrap().1;
        assert_ne!(from.y, to.y);
        assert_eq!(from.as_size(), to.as_size());
        for y in 0..from.height {
            for x in 0..from.width {
                assert_eq!(
                    before[(from.x + x, from.y + y)],
                    after[(to.x + x, to.y + y)],
                    "{design:?} must stay attached to the same ID"
                );
            }
        }
    }
}
