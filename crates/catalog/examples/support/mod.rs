//! Shared synthetic application, used unchanged by three host-owned event loops.
//! This is example code, never a runtime/catalog dependency or public registry.
#![allow(dead_code)] // Each independently compiled host uses a different subset.

use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
};
use spark_catalog::{
    AssemblyProgress, BorderRunner, Level, PocketMascot, PrismReveal, Progress, SegmentedComet,
    TelemetryEmbers,
};
use spark_runtime::{
    ChangeHint, ClipTime, HostTime, Motion, Palette, Playback, PlaybackRate, Profile,
    RepaintPlanner, SampleContext,
};
use std::time::Duration;

pub const FRAME_CAP: Duration = Duration::from_millis(16);
pub const DATA_INTERVAL: Duration = Duration::from_millis(500);
pub const MIN_WAIT: Duration = Duration::from_millis(1);
pub const ALL_DESIGNS: [Design; 6] = [
    Design::Comet,
    Design::Assembly,
    Design::Border,
    Design::Embers,
    Design::Mascot,
    Design::Reveal,
];
pub const ALL_LAYOUTS: [LayoutMode; 4] = [
    LayoutMode::Table,
    LayoutMode::Request,
    LayoutMode::Dashboard,
    LayoutMode::Reader,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Design {
    Comet,
    Assembly,
    Border,
    Embers,
    Mascot,
    Reveal,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Table,
    Request,
    Dashboard,
    Reader,
}
impl LayoutMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Table => "Table / stable entity rows",
            Self::Request => "Request / replace and cancel",
            Self::Dashboard => "Dashboard / independent data cadence",
            Self::Reader => "Reader / protected selection and media",
        }
    }
    pub fn next(self) -> Self {
        ALL_LAYOUTS[(ALL_LAYOUTS
            .iter()
            .position(|&layout| layout == self)
            .unwrap()
            + 1)
            % 4]
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Working,
    Completed,
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Operation {
    pub id: u64,
    pub generation: u64,
    pub progress: Progress,
    pub state: OperationState,
}
#[derive(Debug, Clone, Copy)]
pub struct DataUpdate {
    pub id: u64,
    pub generation: u64,
    pub progress: Progress,
}

pub struct Host {
    comet: SegmentedComet,
    assembly: AssemblyProgress,
    border: BorderRunner,
    embers: TelemetryEmbers,
    mascot: PocketMascot,
    reveal: PrismReveal,
    pub palette: Palette,
    pub profile: Profile,
    pub playback: Playback,
    pub reveal_playback: Playback,
    pub layout: LayoutMode,
    pub only: Option<Design>,
    pub art_enabled: bool,
    pub producer_enabled: bool,
    pub rows: Vec<Operation>,
    pub selected_id: u64,
    pub level: Level,
    pub data_polls: u64,
    pub frames: u64,
    pub rejected_updates: u64,
    pub final_paints: u64,
    pub dirty: bool,
    pub last_presented: Option<HostTime>,
    pub animation_due: Option<HostTime>,
    pub next_data: HostTime,
    final_generation: Option<(u64, u64)>,
    displayed_clip: ClipTime,
    displayed_reveal: ClipTime,
}
impl Default for Host {
    fn default() -> Self {
        Self {
            comet: SegmentedComet::new().with_seed(7),
            assembly: AssemblyProgress::new().with_seed(13),
            border: BorderRunner::new(),
            embers: TelemetryEmbers::new().with_seed(29),
            mascot: PocketMascot::new(),
            reveal: PrismReveal::new("Result ready. The host owns completion.").unwrap(),
            palette: Palette::amber(),
            profile: Profile::default(),
            playback: Playback::new(HostTime::ZERO),
            reveal_playback: Playback::new(HostTime::ZERO),
            layout: LayoutMode::Dashboard,
            only: None,
            art_enabled: true,
            producer_enabled: true,
            rows: vec![
                Operation {
                    id: 42,
                    generation: 1,
                    progress: Progress::known(0.25).unwrap(),
                    state: OperationState::Working,
                },
                Operation {
                    id: 7,
                    generation: 3,
                    progress: Progress::unknown(),
                    state: OperationState::Working,
                },
            ],
            selected_id: 42,
            level: Level::new(0.65).unwrap(),
            data_polls: 0,
            frames: 0,
            rejected_updates: 0,
            final_paints: 0,
            dirty: true,
            last_presented: None,
            animation_due: None,
            next_data: host_time(DATA_INTERVAL),
            final_generation: None,
            displayed_clip: ClipTime::ZERO,
            displayed_reveal: ClipTime::ZERO,
        }
    }
}
impl Host {
    pub fn new(layout: LayoutMode) -> Self {
        Self {
            layout,
            ..Self::default()
        }
    }
    pub fn for_design(design: Design, layout: LayoutMode) -> Self {
        let mut host = Self {
            only: Some(design),
            layout,
            ..Self::default()
        };
        if design == Design::Reveal {
            host.rows[0].progress = Progress::known(1.0).unwrap();
            host.rows[0].state = OperationState::Completed;
        }
        host
    }
    pub fn selected(&self) -> &Operation {
        self.rows
            .iter()
            .find(|row| row.id == self.selected_id)
            .expect("stable selected entity exists")
    }
    pub fn apply_update(&mut self, update: DataUpdate, now: HostTime) -> bool {
        let Some(row) = self.rows.iter_mut().find(|row| row.id == update.id) else {
            self.rejected_updates += 1;
            return false;
        };
        if row.generation != update.generation || row.state == OperationState::Cancelled {
            self.rejected_updates += 1;
            return false;
        }
        let was_complete = row.state == OperationState::Completed;
        row.progress = update.progress;
        row.state = if update.progress.value() == Some(1.0) {
            OperationState::Completed
        } else {
            OperationState::Working
        };
        if row.id == self.selected_id && row.state == OperationState::Completed && !was_complete {
            self.reveal_playback.restart(now).unwrap();
            // A new visual lifecycle supersedes the old shown reveal, even if
            // pause arrives before the replacement's first presentation.
            self.displayed_reveal = ClipTime::ZERO;
            if !self.playback.is_playing() {
                self.reveal_playback.pause(now).unwrap();
            }
            self.final_generation = None;
        }
        self.dirty = true;
        true
    }
    pub fn replace_selected(&mut self, now: HostTime) {
        let row = self
            .rows
            .iter_mut()
            .find(|row| row.id == self.selected_id)
            .unwrap();
        row.generation = row
            .generation
            .checked_add(1)
            .expect("fixture generation exhausted");
        row.progress = Progress::unknown();
        row.state = OperationState::Working;
        self.reveal_playback.restart(now).unwrap();
        self.displayed_reveal = ClipTime::ZERO;
        if !self.playback.is_playing() {
            self.reveal_playback.pause(now).unwrap();
        }
        self.final_generation = None;
        self.dirty = true;
    }
    pub fn cancel_selected(&mut self) {
        self.rows
            .iter_mut()
            .find(|row| row.id == self.selected_id)
            .unwrap()
            .state = OperationState::Cancelled;
        self.dirty = true;
    }
    pub fn sort_rows(&mut self) {
        self.rows.reverse(); // Identity/selection follows IDs, never row positions.
        self.dirty = true;
    }
    /// This is an explicitly synthetic host data producer, not artwork sampling.
    pub fn poll_data(&mut self, now: HostTime) {
        self.data_polls += 1;
        self.level = Level::new((self.data_polls % 9) as f64 / 8.0).unwrap();
        let selected = *self.selected();
        if selected.state == OperationState::Working {
            let value = (selected.progress.value().unwrap_or(0.0) + 0.125).min(1.0);
            self.apply_update(
                DataUpdate {
                    id: selected.id,
                    generation: selected.generation,
                    progress: Progress::known(value).unwrap(),
                },
                now,
            );
        }
        // Late data wakes coalesce too. No backlog of missed measurements.
        self.next_data = add(now, DATA_INTERVAL);
        self.dirty = true;
    }
    pub fn poll_data_if_due(&mut self, now: HostTime) -> bool {
        if self.producer_enabled && now >= self.next_data {
            self.poll_data(now);
            true
        } else {
            false
        }
    }
    pub fn pause_or_resume(&mut self, now: HostTime) {
        if self.playback.is_playing() {
            // Freeze the last presented pose, not an unseen next frame.
            // A rate/visibility re-anchor can no longer reconstruct a previous
            // pose from its host timestamp. Remember the actual submitted clips.
            let (clip, reveal) = if self.last_presented.is_some() {
                (self.displayed_clip, self.displayed_reveal)
            } else {
                (self.playback.sample(now), self.reveal_playback.sample(now))
            };
            self.playback.pause(now).unwrap();
            self.playback.seek(now, clip).unwrap();
            self.reveal_playback.pause(now).unwrap();
            self.reveal_playback.seek(now, reveal).unwrap();
        } else {
            self.playback.resume(now).unwrap();
            self.reveal_playback.resume(now).unwrap();
        }
        self.dirty = true;
    }
    pub fn restart(&mut self, now: HostTime) {
        self.playback.restart(now).unwrap();
        self.reveal_playback.restart(now).unwrap();
        self.displayed_clip = ClipTime::ZERO;
        self.displayed_reveal = ClipTime::ZERO;
        self.final_generation = None;
        self.dirty = true;
    }
    pub fn toggle_hidden(&mut self, now: HostTime) {
        let hidden = !self.playback.is_hidden();
        self.playback.set_hidden(now, hidden).unwrap();
        self.reveal_playback.set_hidden(now, hidden).unwrap();
        self.dirty = true;
    }
    pub fn toggle_static(&mut self) {
        self.profile.motion = if self.profile.motion == Motion::Static {
            Motion::Full
        } else {
            Motion::Static
        };
        self.dirty = true;
    }
    pub fn set_rate(&mut self, now: HostTime, rate: PlaybackRate) {
        self.playback.set_rate(now, rate).unwrap();
        self.reveal_playback.set_rate(now, rate).unwrap();
        self.dirty = true;
    }
    pub fn active(&self, design: Design) -> bool {
        self.art_enabled
            && !self.playback.is_hidden()
            && self.only.is_none_or(|only| only == design)
            && match design {
                Design::Reveal => self.selected().state == OperationState::Completed,
                Design::Comet => {
                    let row = if self.layout == LayoutMode::Table {
                        self.rows
                            .iter()
                            .find(|row| row.id != self.selected_id)
                            .unwrap_or(self.selected())
                    } else {
                        self.selected()
                    };
                    row.state == OperationState::Working
                }
                Design::Assembly => self.selected().state != OperationState::Cancelled,
                _ => true,
            }
    }
    /// Table art follows stable entity IDs through sorting, not fixed row offsets.
    pub fn placements(&self, area: Rect) -> [(Design, Rect); 6] {
        let mut result = placements(self.layout, area);
        if self.layout == LayoutMode::Table {
            let inner = Block::bordered().inner(area);
            let selected_row = self
                .rows
                .iter()
                .position(|row| row.id == self.selected_id)
                .unwrap();
            let secondary_row = self
                .rows
                .iter()
                .position(|row| row.id != self.selected_id)
                .unwrap_or(selected_row);
            result[0].1 = region(inner, 30, 2 + secondary_row as u16 * 4, 24, 3);
            result[1].1 = region(inner, 30, 2 + selected_row as u16 * 4, 24, 3);
        }
        result
    }
    fn hint(&self, design: Design, now: HostTime, size: Size) -> ChangeHint {
        let playback = if design == Design::Reveal {
            &self.reveal_playback
        } else {
            &self.playback
        };
        let context = SampleContext::new(playback.sample(now), &self.palette, &self.profile);
        match design {
            Design::Comet => self.comet.sample(&context).change_hint(size),
            Design::Assembly => self
                .assembly
                .sample(&context, self.selected().progress)
                .change_hint(size),
            Design::Border => self.border.sample(&context).change_hint(size),
            Design::Embers => self.embers.sample(&context, self.level).change_hint(size),
            Design::Mascot => self.mascot.sample(&context).change_hint(size),
            Design::Reveal => self.reveal.sample(&context).change_hint(size),
        }
    }
    pub fn refresh_animation(&mut self, now: HostTime, size: Size) {
        let planner = RepaintPlanner::new(FRAME_CAP);
        let last = self.last_presented.unwrap_or(now);
        let placements = self.placements(Rect::new(0, 0, size.width, size.height));
        self.animation_due = RepaintPlanner::earliest(
            placements
                .into_iter()
                .filter(|(design, _)| self.active(*design))
                .map(|(design, area)| {
                    let playback = if design == Design::Reveal {
                        &self.reveal_playback
                    } else {
                        &self.playback
                    };
                    planner.next_deadline(
                        now,
                        last,
                        playback,
                        self.hint(design, now, area.as_size()),
                    )
                }),
        );
    }
    pub fn presented(&mut self, now: HostTime, size: Size) {
        self.frames += 1;
        self.last_presented = Some(now);
        self.displayed_clip = self.playback.sample(now);
        self.displayed_reveal = self.reveal_playback.sample(now);
        self.dirty = false;
        if self.active(Design::Reveal) {
            let context = SampleContext::new(
                self.reveal_playback.sample(now),
                &self.palette,
                &self.profile,
            );
            let area = self
                .placements(Rect::new(0, 0, size.width, size.height))
                .into_iter()
                .find(|(design, _)| *design == Design::Reveal)
                .unwrap()
                .1;
            if !area.is_empty()
                && self.reveal.sample(&context).complete()
                && self.final_generation != Some((self.selected().id, self.selected().generation))
            {
                self.final_generation = Some((self.selected().id, self.selected().generation));
                self.final_paints += 1;
            }
        }
        self.refresh_animation(now, size);
    }
    pub fn next_presentation(&self) -> Option<HostTime> {
        if self.dirty {
            Some(
                self.last_presented
                    .map_or(HostTime::ZERO, |last| add(last, FRAME_CAP)),
            )
        } else {
            self.animation_due
        }
    }
    pub fn next_wake(&self) -> Option<HostTime> {
        RepaintPlanner::earliest([
            self.next_presentation(),
            self.producer_enabled.then_some(self.next_data),
        ])
    }
}

pub fn host_time(duration: Duration) -> HostTime {
    HostTime::saturating_from_duration(duration)
}
pub fn add(time: HostTime, duration: Duration) -> HostTime {
    HostTime::from_nanos(
        time.as_nanos()
            .saturating_add(duration.as_nanos().min(u128::from(u64::MAX)) as u64),
    )
}
pub fn wait_until(deadline: HostTime, now: HostTime) -> Duration {
    Duration::from_nanos(deadline.as_nanos().saturating_sub(now.as_nanos())).max(MIN_WAIT)
}
fn region(root: Rect, x: u16, y: u16, width: u16, height: u16) -> Rect {
    Rect::new(
        root.x.saturating_add(x.min(root.width)),
        root.y.saturating_add(y.min(root.height)),
        width.min(root.width.saturating_sub(x)),
        height.min(root.height.saturating_sub(y)),
    )
}
pub fn placements(layout: LayoutMode, area: Rect) -> [(Design, Rect); 6] {
    let inner = Block::bordered().inner(area);
    let right = inner.width.saturating_sub(27);
    let bottom = inner.height.saturating_sub(2);
    let (comet, assembly, embers, mascot, reveal) = match layout {
        LayoutMode::Table => (
            region(inner, 30, 2, 24, 1),
            region(inner, 30, 4, 24, 3),
            region(inner, 30, 11, 24, 8),
            region(inner, 2, 11, 12, 6),
            region(inner, 2, bottom, 42, 1),
        ),
        LayoutMode::Request => (
            region(inner, 2, 3, 24, 3),
            region(inner, 2, 7, 24, 3),
            region(inner, 34, 3, inner.width.saturating_sub(36), 8),
            region(inner, 2, 12, 12, 6),
            region(inner, 18, 14, 42, 1),
        ),
        LayoutMode::Dashboard => (
            region(inner, 1, 2, 24, 3),
            region(inner, 1, 7, 24, 3),
            region(
                inner,
                30,
                2,
                inner.width.saturating_sub(32),
                inner.height.saturating_sub(5),
            ),
            region(inner, 4, 12, 12, 6),
            region(inner, 2, bottom, 42, 1),
        ),
        LayoutMode::Reader => (
            region(inner, right, 1, 24, 3),
            region(inner, right, 6, 24, 3),
            region(inner, right, 10, 24, 8),
            region(inner, 2, inner.height.saturating_sub(8), 12, 6),
            region(inner, 2, 2, inner.width.saturating_sub(32), 1),
        ),
    };
    [
        (Design::Comet, comet),
        (Design::Assembly, assembly),
        (Design::Border, area),
        (Design::Embers, embers),
        (Design::Mascot, mascot),
        (Design::Reveal, reveal),
    ]
}
/// Fresh base content is drawn first. Every decorator uses the same explicit
/// no-write rectangles, so titles, selections, carets, and media retain precedence.
pub fn protected_regions(layout: LayoutMode, area: Rect) -> Vec<Rect> {
    let inner = Block::bordered().inner(area);
    let mut protected = vec![
        region(area, 2, 0, 55, 1),
        region(inner, 0, 0, inner.width, 1),
    ];
    if layout == LayoutMode::Reader {
        protected.extend([
            region(inner, 2, 4, 22, 1),
            region(inner, 2, 5, inner.width.saturating_sub(4), 1),
            region(inner, 2, 7, 1, 1),
            region(inner, 5, 10, 12, 4),
        ]);
    }
    protected
}
pub fn draw(frame: &mut Frame, host: &Host, now: HostTime) {
    let area = frame.area();
    let block = Block::bordered()
        .title(format!(" {} ", host.layout.name()))
        .style(Style::new().bg(Color::Rgb(10, 16, 27)))
        .border_style(host.palette.muted);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let selected = host.selected();
    let status = match selected.state {
        OperationState::Working => "working",
        OperationState::Completed => "complete",
        OperationState::Cancelled => "cancelled",
    };
    frame.render_widget(
        Paragraph::new(format!(
            "id={} gen={} {status} | data polls={} | host owns data",
            selected.id, selected.generation, host.data_polls
        ))
        .style(host.palette.foreground),
        region(inner, 0, 0, inner.width, 1),
    );
    match host.layout {
        LayoutMode::Table => {
            for (index, row) in host.rows.iter().enumerate() {
                let progress = row.progress.value().map_or_else(
                    || "unknown".to_owned(),
                    |value| format!("{:>3.0}%", value * 100.0),
                );
                frame.render_widget(
                    Paragraph::new(format!("#{} / g{}  {progress}", row.id, row.generation))
                        .style(host.palette.foreground),
                    region(inner, 2, 3 + index as u16 * 4, 26, 1),
                );
            }
            frame.render_widget(
                Paragraph::new("Rows sort; IDs stay attached"),
                region(inner, 2, 10, 27, 1),
            );
        }
        LayoutMode::Request => {
            frame.render_widget(
                Paragraph::new("Request: render / editable host form"),
                region(inner, 2, 1, 45, 1),
            );
            frame.render_widget(
                Paragraph::new("n replace  c cancel"),
                region(inner, 2, 11, 27, 1),
            );
        }
        LayoutMode::Dashboard => {
            frame.render_widget(
                Paragraph::new("Transfer / 2 Hz data"),
                region(inner, 1, 6, 27, 1),
            );
            frame.render_widget(
                Paragraph::new(format!("Synthetic level {:.3}", host.level.value())),
                region(inner, 30, 1, 35, 1),
            );
        }
        LayoutMode::Reader => {
            frame.render_widget(
                Paragraph::new("Wide text: 界 e\u{301}"),
                region(inner, 2, 4, 22, 1),
            );
            frame.render_widget(
                Paragraph::new("Selected host row: keep exact colors").style(
                    Style::new()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                region(inner, 2, 5, inner.width.saturating_sub(4), 1),
            );
            frame.render_widget(
                Paragraph::new("|").style(Style::new().fg(Color::LightRed)),
                region(inner, 2, 7, 1, 1),
            );
            frame.render_widget(
                Block::bordered().title("media"),
                region(inner, 5, 10, 12, 4),
            );
        }
    }
    let protected = protected_regions(host.layout, area);
    for (design, requested) in host.placements(area) {
        if !host.active(design) {
            continue;
        }
        let playback = if design == Design::Reveal {
            &host.reveal_playback
        } else {
            &host.playback
        };
        let context = SampleContext::new(playback.sample(now), &host.palette, &host.profile);
        match design {
            Design::Comet => frame.render_widget(
                host.comet.sample(&context).with_protected(&protected),
                requested,
            ),
            Design::Assembly => frame.render_widget(
                host.assembly
                    .sample(&context, selected.progress)
                    .with_protected(&protected),
                requested,
            ),
            Design::Border => frame.render_widget(
                host.border.sample(&context).with_protected(&protected),
                requested,
            ),
            Design::Embers => frame.render_widget(
                host.embers
                    .sample(&context, host.level)
                    .with_protected(&protected),
                requested,
            ),
            Design::Mascot => frame.render_widget(
                host.mascot.sample(&context).with_protected(&protected),
                requested,
            ),
            Design::Reveal => {
                frame.render_widget(
                    Paragraph::new(host.reveal.text()).style(host.palette.foreground),
                    requested,
                );
                frame.render_widget(
                    host.reveal.sample(&context).with_protected(&protected),
                    requested,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStyle {
    Synchronous,
    Asynchronous,
    Paced,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct SmokeReport {
    pub frames: u64,
    pub data_polls: u64,
    pub animation_wakes: u64,
    pub final_paints: u64,
}
/// Fake time exercises the same production model and draw path without terminal IO.
/// Asynchronous mode injects bursty invalidations; paced mode always presents at
/// the host's fixed cadence. Neither changes the independently scheduled producer.
pub fn smoke(style: HostStyle) -> SmokeReport {
    let mut report = SmokeReport::default();
    for layout in ALL_LAYOUTS {
        for design in ALL_DESIGNS {
            let mut host = Host::for_design(design, layout);
            let size = Size::new(100, 30);
            let mut terminal = Terminal::new(TestBackend::new(size.width, size.height)).unwrap();
            let end = host_time(Duration::from_secs(2));
            let mut now = HostTime::ZERO;
            let mut next_burst = host_time(Duration::from_millis(125));
            loop {
                host.poll_data_if_due(now);
                if style == HostStyle::Asynchronous && now >= next_burst {
                    for _ in 0..64 {
                        host.dirty = true;
                    }
                    next_burst = add(now, Duration::from_millis(125));
                }
                if style == HostStyle::Paced
                    || host.next_presentation().is_some_and(|due| now >= due)
                {
                    if !host.dirty && style != HostStyle::Paced {
                        report.animation_wakes += 1;
                    }
                    terminal.draw(|frame| draw(frame, &host, now)).unwrap();
                    host.presented(now, size);
                }
                if now >= end {
                    break;
                }
                let next = if style == HostStyle::Paced {
                    add(now, FRAME_CAP).min(host.next_data)
                } else if style == HostStyle::Asynchronous {
                    host.next_wake().unwrap_or(end).min(next_burst)
                } else {
                    host.next_wake().unwrap_or(end)
                };
                assert!(next > now, "host must wait rather than spin");
                now = next.min(end);
            }
            assert_eq!(host.data_polls, 4, "art must not change the 2 Hz producer");
            host.producer_enabled = false;
            if host.playback.is_playing() {
                host.pause_or_resume(now);
            }
            terminal.draw(|frame| draw(frame, &host, now)).unwrap();
            host.presented(now, size);
            assert_eq!(
                host.next_wake(),
                None,
                "inactive artwork cannot request wakes"
            );
            report.frames += host.frames;
            report.data_polls += host.data_polls;
            report.final_paints += host.final_paints;
        }
    }
    assert!(
        report.final_paints >= 4,
        "each layout paints the final finite reveal"
    );
    report
}

pub const CONTROLS: &str = "q quit | space pause | r restart | s static | h hide | l layout | n new generation | c cancel | o sort | d data on/off";
pub fn handle_key(
    host: &mut Host,
    key: ratatui::crossterm::event::KeyEvent,
    now: HostTime,
) -> bool {
    use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
    if key.kind != KeyEventKind::Press {
        return false;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Char(' ') => host.pause_or_resume(now),
        KeyCode::Char('r') => host.restart(now),
        KeyCode::Char('s') => host.toggle_static(),
        KeyCode::Char('h') => host.toggle_hidden(now),
        KeyCode::Char('l') => {
            host.layout = host.layout.next();
            host.dirty = true;
        }
        KeyCode::Char('n') => host.replace_selected(now),
        KeyCode::Char('c') => host.cancel_selected(),
        KeyCode::Char('o') => host.sort_rows(),
        KeyCode::Char('d') => {
            host.producer_enabled = !host.producer_enabled;
            host.next_data = add(now, DATA_INTERVAL);
            host.dirty = true;
        }
        _ => {}
    }
    false
}
