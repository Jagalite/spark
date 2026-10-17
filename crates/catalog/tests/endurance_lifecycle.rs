//! Bounded, seeded integration endurance of the actual example Host and draw path.
//!
//! Workload declared before execution: 2 seeds × 3 modeled scheduling styles ×
//! 4 initial layouts × 7 art selections × 192 command checkpoints = 32,256.
//! Geometry stays within 100×30; bursts use a 32-slot channel and 512 deliveries.
//! A separate producer trace compares identical observation/stall times across
//! art modes/cadences. No wall-clock, emulator, OS-thread, or heap-growth claim is
//! made: observable host rows/capacity, bounded channel depth, scalar deadlines,
//! actual TestBackend cells, and explicitly counted presentations are checked.
#![forbid(unsafe_code)]

#[path = "../examples/support/mod.rs"]
mod support;

use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Rect, Size},
};
use spark_catalog::{Level, Progress};
use spark_runtime::{ClipTime, HiddenPolicy, HostTime, Motion, PlaybackRate, Profile};
use support::{
    ALL_DESIGNS, ALL_LAYOUTS, DataUpdate, Design, Host, HostStyle, LayoutMode, OperationState, draw,
};
use tokio::sync::mpsc;

const SEEDS: [u64; 2] = [0x6a09_e667_f3bc_c909, 0xbb67_ae85_84ca_a73b];
const STYLES: [HostStyle; 3] = [
    HostStyle::Synchronous,
    HostStyle::Asynchronous,
    HostStyle::Paced,
];
const CHECKPOINTS: usize = 192;
const CHANNEL_BOUND: usize = 32;
const BURST: usize = 512;
const SIZES: [Size; 8] = [
    Size::new(0, 0),
    Size::new(1, 1),
    Size::new(7, 3),
    Size::new(24, 8),
    Size::new(42, 16),
    Size::new(64, 24),
    Size::new(80, 26),
    Size::new(100, 30),
];

fn ms(value: u64) -> HostTime {
    HostTime::from_nanos(value * 1_000_000)
}
fn later(now: HostTime, millis: u64) -> HostTime {
    HostTime::from_nanos(now.as_nanos() + millis * 1_000_000)
}
struct Seed(u64);
impl Seed {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn index(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}
fn paint(host: &Host, now: HostTime, size: Size) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(size.width, size.height)).unwrap();
    terminal
        .draw(|frame| draw(frame, host, now))
        .unwrap()
        .buffer
        .clone()
}

struct Driver {
    host: Host,
    terminal: Terminal<TestBackend>,
    style: HostStyle,
    size: Size,
    next_frame: HostTime,
    expected_data: HostTime,
    expected_polls: u64,
    row_capacity: usize,
    presentations: u64,
    maximum_channel: usize,
    last_frame: Buffer,
}
impl Driver {
    fn new(style: HostStyle, layout: LayoutMode, only: Option<Design>) -> Self {
        let mut host = only.map_or_else(|| Host::new(layout), |d| Host::for_design(d, layout));
        host.producer_enabled = true;
        let size = Size::new(64, 24);
        let row_capacity = host.rows.capacity();
        Self {
            terminal: Terminal::new(TestBackend::new(size.width, size.height)).unwrap(),
            host,
            style,
            size,
            next_frame: HostTime::ZERO,
            expected_data: ms(500),
            expected_polls: 0,
            row_capacity,
            presentations: 0,
            maximum_channel: 0,
            last_frame: Buffer::empty(Rect::new(0, 0, size.width, size.height)),
        }
    }
    fn resize(&mut self, size: Size) {
        self.size = size;
        self.terminal.backend_mut().resize(size.width, size.height);
        self.terminal
            .resize(Rect::new(0, 0, size.width, size.height))
            .unwrap();
        self.host.dirty = true;
    }
    fn present(&mut self, now: HostTime) {
        if let Some(last) = self.host.last_presented {
            assert!(now >= later(last, 16), "presentation cap: {:?}", self.style);
        }
        self.last_frame = self
            .terminal
            .draw(|frame| draw(frame, &self.host, now))
            .unwrap()
            .buffer
            .clone();
        // Physical diffs omit invisible wide-glyph continuation cells. Exact
        // preservation uses the complete submitted buffer; backend output is
        // compared under the same qualified terminal-diff semantics.
        assert!(
            self.terminal
                .backend()
                .buffer()
                .diff(&self.last_frame)
                .is_empty()
        );
        self.host.presented(now, self.size);
        self.presentations += 1;
        self.next_frame = later(now, 16);
        assert_eq!(self.host.frames, self.presentations);
        assert!(!self.host.dirty);
        assert!(self.host.animation_due.is_none_or(|due| due > now));
    }
    fn service(&mut self, now: HostTime) {
        let before = self.host.data_polls;
        let expected = self.host.producer_enabled && now >= self.expected_data;
        if expected {
            self.expected_polls += 1;
            self.expected_data = later(now, 500);
        }
        assert_eq!(self.host.poll_data_if_due(now), expected);
        assert_eq!(self.host.data_polls, before + u64::from(expected));
        assert_eq!(self.host.data_polls, self.expected_polls);
        assert_eq!(self.host.next_data, self.expected_data);
        assert!(!self.host.poll_data_if_due(now), "no producer backlog");
        let due = if self.style == HostStyle::Paced {
            now >= self.next_frame
        } else {
            self.host.next_presentation().is_some_and(|due| now >= due)
        };
        let frames = self.host.frames;
        if due {
            self.present(now);
        }
        assert_eq!(self.host.frames - frames, u64::from(due));
        assert!(self.host.next_presentation().is_none_or(|due| due > now));
        self.check_state();
    }
    fn check_state(&self) {
        assert_eq!(self.host.rows.len(), 2);
        assert_eq!(self.host.rows.capacity(), self.row_capacity);
        assert_eq!(self.host.rows.iter().filter(|row| row.id == 42).count(), 1);
        assert_eq!(self.host.rows.iter().filter(|row| row.id == 7).count(), 1);
        assert_eq!(self.host.selected_id, 42);
        for row in &self.host.rows {
            if let Some(value) = row.progress.value() {
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            }
        }
        assert_eq!(
            self.host.playback.is_hidden(),
            self.host.reveal_playback.is_hidden()
        );
        assert_eq!(
            self.host.playback.is_playing(),
            self.host.reveal_playback.is_playing()
        );
        assert_eq!(self.host.playback.rate(), self.host.reveal_playback.rate());
    }
    fn assert_fresh_and_protected(&mut self, now: HostTime) {
        let actual = self.last_frame.clone();
        assert_eq!(
            actual,
            paint(&self.host, now, self.size),
            "fresh base, no trails"
        );
        let enabled = self.host.art_enabled;
        self.host.art_enabled = false;
        let base = paint(&self.host, now, self.size);
        self.host.art_enabled = enabled;
        for region in support::protected_regions(self.host.layout, actual.area) {
            for y in region.y..region.bottom() {
                for x in region.x..region.right() {
                    assert_eq!(actual[(x, y)], base[(x, y)], "protected {x},{y}");
                }
            }
        }
    }
    fn quiet_redraw(&mut self, now: HostTime) {
        assert!(
            !self.host.playback.is_playing()
                || self.host.playback.is_hidden()
                || self.host.profile.motion == Motion::Static
        );
        let current = paint(&self.host, now, self.size);
        assert_eq!(current, paint(&self.host, later(now, 91_337), self.size));
        self.host.refresh_animation(now, self.size);
        assert_eq!(self.host.animation_due, None, "quiet art requests no timer");
        // Separate producer/dirty input wakes from animation-only advice.
        let (producer, dirty) = (self.host.producer_enabled, self.host.dirty);
        self.host.producer_enabled = false;
        self.host.dirty = false;
        assert_eq!(self.host.next_wake(), None);
        self.host.producer_enabled = producer;
        self.host.dirty = dirty;
    }
    fn rejected_completion(&mut self, update: DataUpdate, now: HostTime) {
        let rows = self.host.rows.clone();
        let playback = self.host.reveal_playback;
        let dirty = self.host.dirty;
        let final_paints = self.host.final_paints;
        let rejected = self.host.rejected_updates;
        assert!(!self.host.apply_update(update, now));
        assert_eq!(self.host.rows, rows);
        assert_eq!(self.host.reveal_playback, playback);
        assert_eq!(self.host.dirty, dirty);
        assert_eq!(self.host.final_paints, final_paints);
        assert_eq!(self.host.rejected_updates, rejected + 1);
    }
    fn invalidation(&mut self, serial: usize, now: HostTime) {
        if serial.is_multiple_of(5) {
            self.host.sort_rows();
        } else {
            self.host.dirty = true;
        }
        self.host.refresh_animation(now, self.size);
    }
    fn burst(&mut self, now: HostTime) {
        let before = self.host.frames;
        let selected = *self.host.selected();
        if self.style == HostStyle::Asynchronous {
            // Same channel bound as async_host. try_send exposes backpressure;
            // drain and retry the same event, never discard or accumulate it.
            let (tx, mut rx) = mpsc::channel(CHANNEL_BOUND);
            let mut received = 0;
            for serial in 0..BURST {
                if let Err(error) = tx.try_send(serial) {
                    let pending = match error {
                        mpsc::error::TrySendError::Full(value) => value,
                        mpsc::error::TrySendError::Closed(_) => panic!("input closed"),
                    };
                    let event = rx.try_recv().unwrap();
                    assert_eq!(event, received);
                    self.invalidation(event, now);
                    received += 1;
                    tx.try_send(pending).unwrap();
                }
                self.maximum_channel = self.maximum_channel.max(rx.len());
                assert!(rx.len() <= CHANNEL_BOUND);
            }
            while let Ok(event) = rx.try_recv() {
                assert_eq!(event, received);
                self.invalidation(event, now);
                received += 1;
            }
            assert_eq!(received, BURST);
            assert_eq!(rx.len(), 0);
        } else {
            for serial in 0..BURST {
                self.invalidation(serial, now);
            }
        }
        assert_eq!(*self.host.selected(), selected);
        assert_eq!(self.host.frames, before, "input storm must coalesce");
        assert!(self.host.dirty);
    }
    fn assert_current_after_stall(&mut self, start: HostTime) -> HostTime {
        // After the generated history, restore an independently calculable
        // timeline and prove that a long stall does not replay obsolete frames.
        self.host.producer_enabled = false;
        self.host.art_enabled = true;
        self.host.profile = Profile::default();
        self.host.level = Level::new(0.65).unwrap();
        self.resize(Size::new(100, 30));
        if self.host.playback.is_hidden() {
            self.host.toggle_hidden(start);
        }
        self.host.restart(start);
        self.host.set_rate(start, PlaybackRate::new(1, 3).unwrap());
        self.host.replace_selected(start);
        let current = *self.host.selected();
        assert!(
            self.host.apply_update(
                DataUpdate {
                    id: current.id,
                    generation: current.generation,
                    progress: Progress::known(if self.host.only == Some(Design::Reveal) {
                        1.0
                    } else {
                        0.25
                    })
                    .unwrap(),
                },
                start
            )
        );
        self.host.refresh_animation(start, self.size);
        self.service(later(start, 16));
        let before = self.host.frames;
        let now = later(start, 137_123);
        self.service(now);
        assert_eq!(
            self.host.frames,
            before + 1,
            "stall coalesces to one current sample"
        );
        let expected = ClipTime::from_nanos(137_123_000_000 / 3);
        assert_eq!(self.host.playback.sample(now), expected);
        assert_eq!(self.host.reveal_playback.sample(now), expected);
        self.assert_fresh_and_protected(now);
        now
    }
    fn finish_current_generation(&mut self, mut now: HostTime) -> HostTime {
        self.host.producer_enabled = false;
        self.host.art_enabled = true;
        self.host.only = Some(Design::Reveal);
        self.host.profile = Profile::default();
        self.resize(Size::new(100, 30));
        if self.host.playback.is_hidden() {
            self.host.toggle_hidden(now);
        }
        if !self.host.playback.is_playing() {
            self.host.pause_or_resume(now);
        }
        self.host.set_rate(now, PlaybackRate::default());
        let obsolete = *self.host.selected();
        self.host.replace_selected(now);
        self.rejected_completion(
            DataUpdate {
                id: obsolete.id,
                generation: obsolete.generation,
                progress: Progress::known(1.0).unwrap(),
            },
            now,
        );
        let current = *self.host.selected();
        let completed = self.host.final_paints;
        assert!(self.host.apply_update(
            DataUpdate {
                id: current.id,
                generation: current.generation,
                progress: Progress::known(1.0).unwrap(),
            },
            now,
        ));
        let start = now;
        self.host.refresh_animation(now, self.size);
        for frame in 0..100 {
            let due = if self.style == HostStyle::Paced {
                Some(self.next_frame)
            } else {
                self.host.next_presentation()
            };
            if let Some(due) = due {
                now = now.max(due);
                self.service(now);
            }
            if self.host.final_paints == completed + 1 {
                break;
            }
            assert!(frame < 99, "finite final pose must actually be presented");
        }
        assert!(now >= later(start, 1200));
        assert!(now <= later(start, 1232));
        assert_eq!(self.host.final_paints, completed + 1);
        assert_eq!(self.host.next_wake(), None);
        let final_pose = self.last_frame.clone();
        let reveal_area = self.host.placements(final_pose.area)[5].1;
        for (offset, ch) in "Result ready. The host owns completion."
            .chars()
            .enumerate()
        {
            if offset < usize::from(reveal_area.width) {
                assert_eq!(
                    final_pose[(reveal_area.x + offset as u16, reveal_area.y)].symbol(),
                    ch.to_string(),
                    "terminal reveal text"
                );
            }
        }
        assert_eq!(final_pose, paint(&self.host, later(now, 90_000), self.size));
        now = later(now, 90_000);
        self.host.dirty = true; // Unrelated application redraw retains the final art.
        self.service(now);
        assert_eq!(self.host.final_paints, completed + 1);
        assert_eq!(self.host.next_wake(), None);
        assert_eq!(self.last_frame, final_pose);
        now
    }
}

#[test]
fn seeded_host_lifecycle_endurance_across_styles_layouts_and_designs() {
    let mut configurations = 0;
    let mut command_counts = [0_usize; 16];
    let mut presentations = 0;
    let mut longest_trace = HostTime::ZERO;
    for seed in SEEDS {
        for style in STYLES {
            for layout in ALL_LAYOUTS {
                for only in ALL_DESIGNS.into_iter().map(Some).chain([None]) {
                    configurations += 1;
                    let mut rng = Seed(seed);
                    let mut driver = Driver::new(style, layout, only);
                    let mut now = HostTime::ZERO;
                    driver.service(now);
                    for step in 0..CHECKPOINTS {
                        let delta = if step.is_multiple_of(47) {
                            60_000 + rng.next() % 240_000
                        } else {
                            rng.next() % 121
                        };
                        now = later(now, delta);
                        // Guaranteed coverage plus seeded ordering/interleaving.
                        let command = if step % 32 < 16 {
                            step % 16
                        } else {
                            rng.index(16)
                        };
                        command_counts[command] += 1;
                        match command {
                            0 => {
                                let old = *driver.host.selected();
                                driver.host.replace_selected(now);
                                assert_eq!(driver.host.selected().generation, old.generation + 1);
                                assert_eq!(driver.host.selected().progress, Progress::unknown());
                                driver.rejected_completion(
                                    DataUpdate {
                                        id: old.id,
                                        generation: old.generation,
                                        progress: Progress::known(1.0).unwrap(),
                                    },
                                    now,
                                );
                                assert!(!driver.host.active(Design::Reveal));
                            }
                            1 => {
                                driver.host.cancel_selected();
                                let old = *driver.host.selected();
                                driver.rejected_completion(
                                    DataUpdate {
                                        id: old.id,
                                        generation: old.generation,
                                        progress: Progress::known(1.0).unwrap(),
                                    },
                                    now,
                                );
                                assert!(!driver.host.active(Design::Reveal));
                            }
                            2 => {
                                let row = *driver.host.selected();
                                let update = DataUpdate {
                                    id: row.id,
                                    generation: row.generation,
                                    progress: Progress::known((rng.next() % 5) as f64 / 4.0)
                                        .unwrap(),
                                };
                                if row.state == OperationState::Cancelled {
                                    driver.rejected_completion(update, now);
                                } else {
                                    assert!(driver.host.apply_update(update, now));
                                    assert_eq!(driver.host.selected().progress, update.progress);
                                }
                            }
                            3 => {
                                let selected = *driver.host.selected();
                                driver.host.sort_rows();
                                assert_eq!(*driver.host.selected(), selected);
                            }
                            4 => driver.host.pause_or_resume(now),
                            5 => {
                                let policy = if rng.next() & 1 == 0 {
                                    HiddenPolicy::Freeze
                                } else {
                                    HiddenPolicy::CatchUp
                                };
                                driver.host.playback.set_hidden_policy(now, policy).unwrap();
                                driver
                                    .host
                                    .reveal_playback
                                    .set_hidden_policy(now, policy)
                                    .unwrap();
                                driver.host.toggle_hidden(now);
                            }
                            6 => driver.host.toggle_static(),
                            7 => {
                                let (numerator, denominator) =
                                    [(1, 3), (7, 2), (1, 1), (2, 7)][rng.index(4)];
                                let before = driver.host.playback.sample(now);
                                let reveal = driver.host.reveal_playback.sample(now);
                                driver.host.set_rate(
                                    now,
                                    PlaybackRate::new(numerator, denominator).unwrap(),
                                );
                                assert_eq!(driver.host.playback.sample(now), before);
                                assert_eq!(driver.host.reveal_playback.sample(now), reveal);
                            }
                            8 => driver.host.restart(now),
                            9 => driver.resize(SIZES[rng.index(SIZES.len())]),
                            10 => {
                                driver.host.layout = ALL_LAYOUTS[rng.index(ALL_LAYOUTS.len())];
                                driver.host.dirty = true;
                            }
                            11 => driver.burst(now),
                            12 => {
                                driver.host.art_enabled = !driver.host.art_enabled;
                                driver.host.dirty = true;
                            }
                            13 => {
                                driver.host.profile.motion =
                                    [Motion::Full, Motion::Reduced, Motion::Static][rng.index(3)];
                                driver.host.dirty = true;
                            }
                            14 => driver.rejected_completion(
                                DataUpdate {
                                    id: 999,
                                    generation: 0,
                                    progress: Progress::known(1.0).unwrap(),
                                },
                                now,
                            ),
                            _ => driver.host.dirty = true,
                        }
                        if driver.host.dirty {
                            driver.host.refresh_animation(now, driver.size);
                        }
                        driver.service(now);
                        if driver.host.last_presented == Some(now) && step.is_multiple_of(16) {
                            driver.assert_fresh_and_protected(now);
                        }
                        if step.is_multiple_of(13)
                            && (!driver.host.playback.is_playing()
                                || driver.host.playback.is_hidden()
                                || driver.host.profile.motion == Motion::Static)
                        {
                            driver.quiet_redraw(now);
                        }
                    }
                    now = driver.assert_current_after_stall(later(now, 16));
                    now = driver.finish_current_generation(later(now, 16));
                    longest_trace = longest_trace.max(now);
                    if style == HostStyle::Asynchronous {
                        assert_eq!(driver.maximum_channel, CHANNEL_BOUND);
                    }
                    presentations += driver.presentations;
                }
            }
        }
    }
    assert_eq!(configurations, 168);
    assert_eq!(command_counts.iter().sum::<usize>(), 32_256);
    assert!(
        command_counts
            .into_iter()
            .all(|count| count >= configurations * 6)
    );
    let invalidations = command_counts[11] * BURST;
    eprintln!(
        "endurance: {configurations} configurations, 32256 command checkpoints, {invalidations} burst invalidations, {presentations} actual presentations, longest fake-time trace {}ms",
        longest_trace.as_nanos() / 1_000_000
    );
}

#[test]
fn producer_endurance_is_independent_of_art_pacing_and_missed_frames() {
    let mut reference = None;
    for seed in SEEDS {
        for style in STYLES {
            for art in [false, true] {
                for motion in [Motion::Full, Motion::Static] {
                    let mut driver = Driver::new(style, LayoutMode::Dashboard, None);
                    driver.host.art_enabled = art;
                    driver.host.profile.motion = motion;
                    let mut rng = Seed(seed);
                    let mut now = HostTime::ZERO;
                    let mut observed = Vec::with_capacity(800);
                    for step in 0_usize..800 {
                        now = later(
                            now,
                            if step.is_multiple_of(97) {
                                120_000
                            } else {
                                rng.next() % 90 + 1
                            },
                        );
                        driver.service(now);
                        observed.push((driver.host.data_polls, driver.host.next_data));
                    }
                    let key = seed;
                    if let Some((prior_seed, prior)) = &reference
                        && *prior_seed == key
                    {
                        assert_eq!(&observed, prior, "art/style changes producer history");
                    }
                    reference = Some((key, observed));
                }
            }
        }
    }
}

#[test]
fn displayed_pose_survives_rate_change_then_pause_before_next_frame() {
    for design in [Design::Comet, Design::Reveal] {
        let mut driver = Driver::new(HostStyle::Synchronous, LayoutMode::Dashboard, Some(design));
        driver.host.producer_enabled = false;
        driver.service(ms(500));
        let before = driver.last_frame.clone();
        let shown = driver.host.playback.sample(ms(500));
        let shown_reveal = driver.host.reveal_playback.sample(ms(500));
        driver
            .host
            .set_rate(ms(510), PlaybackRate::new(7, 2).unwrap());
        driver.host.pause_or_resume(ms(511));
        assert_eq!(
            driver.host.playback.sample(ms(50_000)),
            shown,
            "pause must freeze displayed clip despite an unpresented reanchor"
        );
        assert_eq!(
            driver.host.reveal_playback.sample(ms(50_000)),
            shown_reveal,
            "reveal must freeze its displayed clip"
        );
        driver.service(ms(50_000));
        assert_eq!(driver.last_frame, before);
        assert_eq!(driver.host.next_wake(), None);
        assert_eq!(shown, ClipTime::from_nanos(500_000_000));
    }
}

#[test]
fn pause_after_explicit_reset_never_restores_a_displayed_obsolete_reveal() {
    // A stored displayed pose must be invalidated by explicit restart, operation
    // replacement, and fresh completion, including completion in the same ID/gen.
    for reset in 0..4 {
        let mut driver = Driver::new(
            HostStyle::Synchronous,
            LayoutMode::Request,
            Some(Design::Reveal),
        );
        driver.host.producer_enabled = false;
        driver.resize(Size::new(100, 30));
        driver.service(ms(1500));
        assert_eq!(driver.host.final_paints, 1);
        let old = *driver.host.selected();
        match reset {
            0 => driver.host.restart(ms(1510)),
            1 | 3 => driver.host.replace_selected(ms(1510)),
            _ => {
                assert!(driver.host.apply_update(
                    DataUpdate {
                        id: old.id,
                        generation: old.generation,
                        progress: Progress::known(0.25).unwrap()
                    },
                    ms(1510)
                ));
            }
        }
        if reset == 3 {
            // Pausing before a new completion must leave its restarted reveal paused.
            driver.host.pause_or_resume(ms(1510));
        }
        if reset != 0 {
            let current = *driver.host.selected();
            assert!(driver.host.apply_update(
                DataUpdate {
                    id: current.id,
                    generation: current.generation,
                    progress: Progress::known(1.0).unwrap()
                },
                ms(1510)
            ));
        }
        if driver.host.playback.is_playing() {
            driver.host.pause_or_resume(ms(1510));
        }
        assert_eq!(
            driver.host.reveal_playback.sample(ms(50_000)),
            ClipTime::ZERO,
            "reset case {reset} must not resurrect old shown art"
        );
        driver.service(ms(50_000));
        assert_eq!(
            driver.host.final_paints, 1,
            "new reveal is frozen at its start"
        );
        assert_eq!(driver.host.next_wake(), None);
        if reset == 1 || reset == 3 {
            assert_eq!(driver.host.selected().generation, old.generation + 1);
            driver.rejected_completion(
                DataUpdate {
                    id: old.id,
                    generation: old.generation,
                    progress: Progress::known(1.0).unwrap(),
                },
                ms(50_000),
            );
        }
    }
}
