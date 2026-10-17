use crate::design::{Designs, Selection};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Size,
};
use spark_runtime::{
    ClipTime, ColorProfile, GlyphProfile, Motion, Palette, Profile, SampleContext,
};
use std::time::Duration;

pub struct App {
    pub designs: Designs,
    pub selection: Selection,
    pub palette: Palette,
    pub profile: Profile,
    pub width: u16,
    pub height: u16,
    pub paused: bool,
    pub amber: bool,
    pub embedded: bool,
    pub inspector: bool,
    /// Signed scroll offset from the design configuration in the source.
    pub code_scroll: i16,
    pub code_column: u16,
    pub seed: u64,
    pub input: u8,
    pub unknown: bool,
    pub rate_milli: u32,
    pub displayed: ClipTime,
    clip_anchor: Duration,
    host_anchor: Duration,
}
impl Default for App {
    fn default() -> Self {
        Self {
            designs: Designs::new(7),
            selection: Selection::Comet,
            palette: Palette::default(),
            profile: Profile::default(),
            width: 40,
            height: 3,
            paused: false,
            amber: false,
            embedded: false,
            inspector: false,
            code_scroll: 0,
            code_column: 0,
            seed: 7,
            input: 65,
            unknown: false,
            rate_milli: 1000,
            displayed: ClipTime::ZERO,
            clip_anchor: Duration::ZERO,
            host_anchor: Duration::ZERO,
        }
    }
}
pub struct Outcome {
    pub quit: bool,
    pub export: bool,
    pub changed: bool,
}
impl App {
    pub fn clip_time(&self, host: Duration) -> ClipTime {
        let elapsed = if self.paused {
            0
        } else {
            host.saturating_sub(self.host_anchor)
                .as_nanos()
                .saturating_mul(u128::from(self.rate_milli))
                / 1000
        };
        ClipTime::from_nanos(
            self.clip_anchor
                .as_nanos()
                .saturating_add(elapsed)
                .min(u128::from(u64::MAX)) as u64,
        )
    }
    pub fn interval(&self, terminal: Size) -> Option<Duration> {
        if self.paused {
            return None;
        }
        let context = SampleContext::new(self.displayed, &self.palette, &self.profile);
        let area = crate::view::sample_area(terminal, self);
        self.designs
            .interval(
                self.selection,
                &context,
                self.input,
                self.unknown,
                area.as_size(),
            )
            .map(|duration| {
                Duration::from_nanos(
                    (duration.as_nanos().saturating_mul(1000) / u128::from(self.rate_milli))
                        .max(1)
                        .min(u128::from(u64::MAX)) as u64,
                )
            })
    }
    pub fn seek(&mut self, clip: ClipTime, now: Duration) {
        self.clip_anchor = clip.as_duration();
        self.host_anchor = now;
        self.displayed = clip;
    }
    pub fn select(&mut self, selection: Selection, now: Duration) {
        self.selection = selection;
        let size = selection.preferred();
        self.width = size.width;
        self.height = size.height;
        self.clip_anchor = Duration::ZERO;
        self.host_anchor = now;
        self.displayed = ClipTime::ZERO;
    }
    pub fn handle_key(&mut self, key: KeyEvent, now: Duration) -> Outcome {
        if key.kind != KeyEventKind::Press {
            return Outcome {
                quit: false,
                export: false,
                changed: false,
            };
        }
        let quit = key.code == KeyCode::Char('q')
            || key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL));
        if quit {
            return Outcome {
                quit: true,
                export: false,
                changed: false,
            };
        }
        if key.code == KeyCode::Char('e') {
            return Outcome {
                quit: true,
                changed: false,
                export: true,
            };
        }
        match key.code {
            KeyCode::Char('1'..='6') => {
                if let KeyCode::Char(c) = key.code {
                    self.select(Selection::ALL[c as usize - '1' as usize], now);
                }
            }
            KeyCode::Tab => self.select(self.selection.next(), now),
            KeyCode::Char(' ') => {
                if !self.paused {
                    self.clip_anchor = self.displayed.as_duration();
                }
                self.host_anchor = now;
                self.paused = !self.paused;
            }
            KeyCode::Char('r') => {
                self.clip_anchor = Duration::ZERO;
                self.host_anchor = now;
                self.displayed = ClipTime::ZERO;
            }
            KeyCode::Char('[') | KeyCode::Char(']') => {
                let clip = self.clip_time(now).as_duration();
                self.clip_anchor = if key.code == KeyCode::Char('[') {
                    clip.saturating_sub(Duration::from_millis(100))
                } else {
                    clip.saturating_add(Duration::from_millis(100))
                };
                self.host_anchor = now;
                self.displayed = ClipTime::saturating_from_duration(self.clip_anchor);
            }
            KeyCode::Char(',') | KeyCode::Char('.') => {
                self.clip_anchor = self.clip_time(now).as_duration();
                self.host_anchor = now;
                let rates = [250, 500, 1000, 2000, 4000];
                let index = rates.iter().position(|v| *v == self.rate_milli).unwrap();
                self.rate_milli = rates[if key.code == KeyCode::Char(',') {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(rates.len() - 1)
                }];
            }
            KeyCode::Char('s') => {
                self.profile.motion = if self.profile.motion == Motion::Static {
                    Motion::Full
                } else {
                    Motion::Static
                }
            }
            KeyCode::Char('p') => {
                self.profile.motion = if self.profile.motion == Motion::Reduced {
                    Motion::Full
                } else {
                    Motion::Reduced
                }
            }
            KeyCode::Char('a') => {
                self.profile.glyphs = if self.profile.glyphs == GlyphProfile::Ascii {
                    GlyphProfile::Unicode
                } else {
                    GlyphProfile::Ascii
                }
            }
            KeyCode::Char('m') => {
                self.profile.colors = match self.profile.colors {
                    ColorProfile::TrueColor => ColorProfile::Ansi16,
                    ColorProfile::Ansi16 => ColorProfile::Monochrome,
                    ColorProfile::Monochrome => ColorProfile::TrueColor,
                }
            }
            KeyCode::Char('c') => {
                self.amber = !self.amber;
                self.palette = if self.amber {
                    Palette::amber()
                } else {
                    Palette::default()
                };
            }
            KeyCode::Char('i') => self.inspector = !self.inspector,
            KeyCode::PageUp if self.inspector => {
                self.code_scroll = self.code_scroll.saturating_sub(8).max(-512)
            }
            KeyCode::PageDown if self.inspector => {
                self.code_scroll = self.code_scroll.saturating_add(8).min(512)
            }
            KeyCode::Home if self.inspector => {
                self.code_scroll = -512;
                self.code_column = 0;
            }
            KeyCode::End if self.inspector => self.code_scroll = 512,
            KeyCode::Char('h') if self.inspector => {
                self.code_column = self.code_column.saturating_sub(8)
            }
            KeyCode::Char('l') if self.inspector => {
                self.code_column = self.code_column.saturating_add(8).min(512)
            }
            KeyCode::Char('v') => self.embedded = !self.embedded,
            KeyCode::Char('n') => {
                self.seed = self.seed.wrapping_add(1);
                self.designs = Designs::new(self.seed);
            }
            KeyCode::Char('-') => self.input = self.input.saturating_sub(5),
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.input = self.input.saturating_add(5).min(100)
            }
            KeyCode::Char('u') => self.unknown = !self.unknown,
            KeyCode::Left => self.width = self.width.saturating_sub(4).max(1),
            KeyCode::Right => self.width = self.width.saturating_add(4).min(256),
            KeyCode::Down => self.height = self.height.saturating_sub(1).max(1),
            KeyCode::Up => self.height = self.height.saturating_add(1).min(64),
            _ => {
                return Outcome {
                    quit: false,
                    export: false,
                    changed: false,
                };
            }
        }
        Outcome {
            quit: false,
            export: false,
            changed: true,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    #[test]
    fn pause_preserves_displayed_pose_and_resume_reanchors() {
        let mut app = App {
            displayed: ClipTime::from_nanos(750_000_000),
            ..App::default()
        };
        app.handle_key(key(KeyCode::Char(' ')), Duration::from_millis(760));
        assert_eq!(app.clip_time(Duration::from_secs(100)), app.displayed);
        assert_eq!(app.interval(Size::new(100, 32)), None);
        app.handle_key(key(KeyCode::Char(' ')), Duration::from_secs(100));
        assert_eq!(
            app.clip_time(Duration::from_millis(100_100)).as_duration(),
            Duration::from_millis(850)
        );
    }
    #[test]
    fn controls_preserve_continuity_and_bound_inputs() {
        let mut app = App::default();
        app.handle_key(key(KeyCode::Char('.')), Duration::from_secs(1));
        assert_eq!(
            app.clip_time(Duration::from_secs(1)).as_duration(),
            Duration::from_secs(1)
        );
        assert_eq!(
            app.clip_time(Duration::from_secs(2)).as_duration(),
            Duration::from_secs(3)
        );
        app.handle_key(key(KeyCode::Char('[')), Duration::from_secs(2));
        assert_eq!(
            app.clip_time(Duration::from_secs(2)).as_duration(),
            Duration::from_millis(2900)
        );
        for _ in 0..100 {
            app.handle_key(key(KeyCode::Char('+')), Duration::ZERO);
        }
        assert_eq!(app.input, 100);
        for _ in 0..100 {
            app.handle_key(key(KeyCode::Char('-')), Duration::ZERO);
        }
        assert_eq!(app.input, 0);
        for selection in Selection::ALL {
            app.select(selection, Duration::from_secs(3));
            assert_eq!(app.selection, selection);
            assert_eq!(app.clip_time(Duration::from_secs(3)), ClipTime::ZERO);
        }
    }
    #[test]
    fn static_completed_and_empty_have_no_animation_deadline() {
        let mut app = App::default();
        for selection in Selection::ALL {
            app.select(selection, Duration::ZERO);
            assert_eq!(app.interval(Size::new(0, 0)), None);
            app.profile.motion = Motion::Static;
            assert_eq!(app.interval(Size::new(100, 32)), None);
            app.profile.motion = Motion::Full;
        }
        app.select(Selection::Prism, Duration::ZERO);
        app.displayed = ClipTime::MAX;
        assert_eq!(app.interval(Size::new(100, 32)), None);
    }
    #[test]
    fn repeated_inspector_controls_preserve_paused_seek_and_bound_state() {
        let mut app = App {
            paused: true,
            ..App::default()
        };
        app.seek(ClipTime::from_nanos(700_000_000), Duration::ZERO);
        for i in 0..1000 {
            let now = Duration::from_millis(i);
            app.handle_key(key(KeyCode::Char('i')), now);
            app.handle_key(key(KeyCode::Char(']')), now);
            app.handle_key(key(KeyCode::Char('[')), now);
            for code in [
                KeyCode::Right,
                KeyCode::Up,
                KeyCode::Char('+'),
                KeyCode::Char('.'),
                KeyCode::PageDown,
                KeyCode::Char('l'),
            ] {
                app.handle_key(key(code), now);
            }
        }
        assert!(!app.inspector);
        assert!(app.paused);
        assert_eq!(app.clip_time(Duration::MAX).as_nanos(), 700_000_000);
        assert_eq!(
            (app.width, app.height, app.input, app.rate_milli),
            (256, 64, 100, 4000)
        );
        assert_eq!((app.code_scroll, app.code_column), (512, 512));
        app.inspector = true;
        for _ in 0..1000 {
            for code in [
                KeyCode::Left,
                KeyCode::Down,
                KeyCode::Char('-'),
                KeyCode::Char(','),
                KeyCode::PageUp,
                KeyCode::Char('h'),
            ] {
                app.handle_key(key(code), Duration::from_secs(2));
            }
        }
        assert_eq!(
            (app.width, app.height, app.input, app.rate_milli),
            (1, 1, 0, 250)
        );
        assert_eq!((app.code_scroll, app.code_column), (-512, 0));
        app.handle_key(key(KeyCode::Char('r')), Duration::from_secs(2));
        assert!(app.paused);
        assert_eq!(app.clip_time(Duration::MAX), ClipTime::ZERO);
        app.handle_key(key(KeyCode::Char(' ')), Duration::from_secs(3));
        assert_eq!(
            app.clip_time(Duration::from_secs(4)).as_nanos(),
            250_000_000
        );
    }
    #[test]
    fn export_requires_an_explicit_press_and_quit_does_not_export() {
        let mut app = App::default();
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            let mut event = key(KeyCode::Char('e'));
            event.kind = kind;
            let outcome = app.handle_key(event, Duration::ZERO);
            assert!(!outcome.quit && !outcome.changed && !outcome.export);
        }
        for code in [KeyCode::Char('q'), KeyCode::Esc] {
            let outcome = app.handle_key(key(code), Duration::ZERO);
            assert!(outcome.quit && !outcome.export);
        }
        let outcome = app.handle_key(key(KeyCode::Char('e')), Duration::ZERO);
        assert!(outcome.quit && outcome.export && !outcome.changed);
    }
}
