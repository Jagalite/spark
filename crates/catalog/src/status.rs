//! A coordinated signal-seal theme for four explicitly host-selected states.
//!
//! The state is data, never a conclusion drawn from elapsed time. Success and
//! error animate only their arrival; neither their animation nor its completion
//! changes the supplied state or reports completion of a business operation.
use ratatui_core::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Modifier, Style},
    widgets::Widget,
};
use spark_runtime::{
    ChangeHint, ClipTime, FiniteClip, GlyphProfile, Motion, Period, SampleContext,
    surface::{CellOp, Glyph, OpaqueSurface},
};
use std::time::Duration;

/// Presentation state selected by the host. There is deliberately no automatic
/// mapping from progress, elapsed time, task adapters, or a previous sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusKind {
    Loading,
    Empty,
    Success,
    Error,
}
impl StatusKind {
    /// Literal label used by the full-size and sufficiently wide compact views.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Loading => "LOADING",
            Self::Empty => "EMPTY",
            Self::Success => "SUCCESS",
            Self::Error => "ERROR",
        }
    }
    const fn marker(self) -> char {
        match self {
            Self::Loading => '~',
            Self::Empty => '-',
            Self::Success => '+',
            Self::Error => '!',
        }
    }
    const fn finite(self) -> bool {
        matches!(self, Self::Success | Self::Error)
    }
}

/// An immutable, native 23×7 signal seal, centered without stretching.
///
/// A chamfered enclosure and two side rails link the family. Loading gathers
/// small packets through its core; empty has an unoccupied cradle; success joins
/// a check and closes the seal; error settles into a split seal and exclamation.
/// Literal state labels remain visible throughout every full-size transition.
///
/// Smaller requests use a static marker/label. Larger requests leave everything
/// outside the centered 23×7 composition untouched. Paint attempts are bounded
/// by 161 cells; scoped-writer row scans and exclusions still depend on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusTheme {
    loading_period: Period,
    transition: FiniteClip,
}
impl Default for StatusTheme {
    fn default() -> Self {
        Self {
            loading_period: Period::new(Duration::from_millis(2400)).unwrap(),
            transition: FiniteClip::new(Period::new(Duration::from_millis(960)).unwrap()),
        }
    }
}
impl StatusTheme {
    pub const WIDTH: u16 = 23;
    pub const HEIGHT: u16 = 7;

    pub fn new() -> Self {
        Self::default()
    }
    /// Full-motion loading period. Reduced motion uses four times this period,
    /// with fixed glyphs and only restrained palette-role changes at the core.
    pub const fn with_loading_period(mut self, period: Period) -> Self {
        self.loading_period = period;
        self
    }
    /// Arrival duration for success and error, identical for full and reduced
    /// motion. Static and compact views immediately use their final poses.
    pub const fn with_transition_duration(mut self, duration: Period) -> Self {
        self.transition = FiniteClip::new(duration);
        self
    }
    pub const fn loading_period(&self) -> Period {
        self.loading_period
    }
    pub const fn transition_duration(&self) -> Period {
        self.transition.duration()
    }
    /// `context.time` is the host's explicit visual age of the selected state.
    /// Reset it when an arrival should replay. A state change never resets it
    /// implicitly; a late success/error sample may already be visually settled.
    pub fn sample<'a>(
        &'a self,
        context: &SampleContext<'a>,
        status: StatusKind,
    ) -> StatusSample<'a> {
        StatusSample {
            design: self,
            context: *context,
            status,
            protected: &[],
        }
    }
}

/// Cheap borrowed view; sampling and rendering never advance stored state.
#[derive(Debug, Clone, Copy)]
pub struct StatusSample<'a> {
    design: &'a StatusTheme,
    context: SampleContext<'a>,
    status: StatusKind,
    protected: &'a [Rect],
}
impl<'a> StatusSample<'a> {
    /// Absolute host-owned no-write regions, such as labels, carets, or media.
    pub fn with_protected(mut self, protected: &'a [Rect]) -> Self {
        self.protected = protected;
        self
    }
    pub const fn status(&self) -> StatusKind {
        self.status
    }
    /// Whether the success/error visual arrival has settled for this geometry.
    /// Loading and empty have no finite arrival and return `None`. Compact,
    /// empty-geometry, and static presentations bypass the arrival and return
    /// `Some(true)` for success/error. Reduced motion settles styles, not glyphs.
    /// This is a visual query only, never an operation-completion signal.
    pub fn transition_complete(&self, requested: Size) -> Option<bool> {
        self.status.finite().then(|| {
            !native_size(requested)
                || self.context.profile.motion == Motion::Static
                || self.design.transition.sample(self.context.time).complete
        })
    }
    /// Absolute next discrete visual boundary, with no clock or scheduling.
    /// The final transition boundary is requested before advice becomes idle.
    /// Static, empty, compact, and settled presentations request no time wake.
    /// An unrepresentable future boundary, including at `ClipTime::MAX`, is None.
    pub fn change_hint(&self, requested: Size) -> ChangeHint {
        if !native_size(requested)
            || self.context.profile.motion == Motion::Static
            || self.status == StatusKind::Empty
            || self.transition_complete(requested) == Some(true)
        {
            return ChangeHint::None;
        }
        let reduced = self.context.profile.motion == Motion::Reduced;
        let (period, steps, factor, looping) = if self.status == StatusKind::Loading {
            // A one-nanosecond loop can only ever select its first frame.
            if self.design.loading_period.as_nanos() == 1 {
                return ChangeHint::None;
            }
            (
                self.design.loading_period,
                if reduced { 4 } else { 8 },
                if reduced { 4 } else { 1 },
                true,
            )
        } else {
            (
                self.design.transition_duration(),
                if reduced { 2 } else { 4 },
                1,
                false,
            )
        };
        next_boundary(self.context.time, period, steps, factor, looping)
            .map_or(ChangeHint::None, ChangeHint::At)
    }
    fn paint(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let width = area.width.min(StatusTheme::WIDTH);
        let height = area.height.min(StatusTheme::HEIGHT);
        let ox = (area.width - width) / 2;
        let oy = (area.height - height) / 2;
        let native = native_size(area.as_size());
        let profile = self.context.profile;
        let palette = *self.context.palette;
        let styles = [
            palette.resolve(Style::new(), profile.colors),
            palette.resolve(palette.muted, profile.colors),
            palette.resolve(palette.foreground, profile.colors),
            palette.resolve(palette.accent, profile.colors),
            palette.resolve(
                palette.highlight.add_modifier(Modifier::BOLD),
                profile.colors,
            ),
            palette.resolve(
                palette.foreground.add_modifier(Modifier::BOLD),
                profile.colors,
            ),
        ];
        let frame = self.frame();
        let mut surface = OpaqueSurface::new(buffer, area, self.protected);
        for y in 0..height {
            for x in 0..width {
                let cell = if native {
                    self.native_cell(x, y, frame)
                } else {
                    compact_cell(self.status, x, y, width, height)
                };
                let style = styles[cell.role as usize];
                let symbol = if profile.glyphs == GlyphProfile::Ascii {
                    cell.ascii
                } else {
                    cell.unicode
                };
                let op = if symbol == ' ' {
                    CellOp::Erase(style)
                } else {
                    CellOp::Replace(Glyph::new(symbol).expect("one-column theme glyph"), style)
                };
                surface.write(ox + x, oy + y, op);
            }
        }
    }
    fn frame(&self) -> u32 {
        match self.status {
            StatusKind::Loading if self.context.profile.motion == Motion::Full => {
                self.design.loading_period.step(self.context.time, 8)
            }
            StatusKind::Loading if self.context.profile.motion == Motion::Reduced => self
                .design
                .loading_period
                .step(ClipTime::from_nanos(self.context.time.as_nanos() / 4), 4),
            StatusKind::Success | StatusKind::Error
                if self.context.profile.motion != Motion::Static =>
            {
                self.design.transition.step(
                    self.context.time,
                    if self.context.profile.motion == Motion::Reduced {
                        2
                    } else {
                        4
                    },
                )
            }
            _ => 4,
        }
    }
    fn native_cell(&self, x: u16, y: u16, frame: u32) -> ArtCell {
        if y == 6 {
            return label_cell(self.status.label(), x, StatusTheme::WIDTH);
        }
        let motion = self.context.profile.motion;
        let full_arrival = motion == Motion::Full && self.status.finite();
        let pose = if full_arrival { frame } else { 4 };
        let mut cell = shell(x, y);
        if self.status == StatusKind::Empty {
            if (y == 0 && x == 11) || (y == 2 && ((2..=4).contains(&x) || (18..=20).contains(&x))) {
                cell = if y == 0 {
                    ArtCell::BLANK
                } else {
                    art('.', Role::Muted)
                };
            }
            return match (x, y) {
                (9, 2) => art('\\', Role::Foreground),
                (13, 2) => art('/', Role::Foreground),
                (10..=12, 3) => art('_', Role::Foreground),
                _ => cell,
            };
        }
        if self.status == StatusKind::Success {
            if (y == 0 || y == 4) && (8..=14).contains(&x) {
                let inset = (x - 8).min(14 - x);
                if u32::from(inset) > pose {
                    cell = ArtCell::BLANK;
                }
            }
            let check = matches!((x, y), (10, 3) | (11, 3))
                || ((x, y) == (9, 2) && pose >= 1)
                || ((x, y) == (12, 2) && pose >= 2)
                || ((x, y) == (13, 1) && pose >= 3);
            if check {
                return art(if x <= 10 { '\\' } else { '/' }, self.core_role(frame));
            }
            if full_arrival && pose == 3 && matches!((x, y), (8, 1) | (15, 3)) {
                return art('+', Role::Accent);
            }
            return cell;
        }
        if self.status == StatusKind::Error {
            if (y == 0 || y == 4) && (8..=14).contains(&x) {
                let gap = if pose == 0 {
                    2
                } else if pose == 1 {
                    1
                } else {
                    0
                };
                if x.abs_diff(11) <= gap {
                    cell = ArtCell::BLANK;
                } else if pose >= 2 && (x == 10 || x == 12) {
                    cell = art(if (x == 10) == (y == 0) { '/' } else { '\\' }, Role::Accent);
                }
            }
            if x == 11 && (1..=3).contains(&y) {
                return art(if y < 3 { '|' } else { 'o' }, self.core_role(frame));
            }
            if full_arrival && pose == 3 && matches!((x, y), (3, 1) | (19, 3)) {
                return art('.', Role::Accent);
            }
            return cell;
        }
        // Loading: paired packets converge through the upper channel, then
        // release along its lower branch. No quantity or estimate is implied.
        let track = match (x, y) {
            (8 | 14, 1) => Some('.'),
            (9 | 13, 1 | 3) => Some('-'),
            (8 | 14, 3) => Some('\''),
            (10, 2) => Some('>'),
            (12, 2) => Some('<'),
            _ => None,
        };
        if let Some(symbol) = track {
            cell = art(symbol, Role::Muted);
        }
        if (x, y) == (11, 2) {
            cell = art('o', self.core_role(frame));
        }
        if motion == Motion::Full {
            let (px, py) = [
                (8, 1),
                (9, 1),
                (10, 2),
                (11, 2),
                (10, 2),
                (9, 3),
                (8, 3),
                (11, 2),
            ][frame as usize];
            if y == py && (x == px || x == 22 - px) {
                cell = art(
                    if frame == 3 {
                        '+'
                    } else if frame == 7 {
                        'o'
                    } else {
                        '='
                    },
                    Role::Bright,
                );
            }
        }
        cell
    }
    fn core_role(&self, frame: u32) -> Role {
        if self.context.profile.motion == Motion::Full && self.status.finite() && frame < 3 {
            return Role::Accent;
        }
        if self.context.profile.motion != Motion::Reduced {
            return Role::Bright;
        }
        if self.status == StatusKind::Loading {
            [Role::Foreground, Role::Accent, Role::Bright, Role::Accent][frame as usize]
        } else {
            [Role::Accent, Role::Foreground, Role::Bright][frame as usize]
        }
    }
}
impl Widget for StatusSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.paint(area, buffer);
    }
}
impl Widget for &StatusSample<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (*self).paint(area, buffer);
    }
}

fn native_size(size: Size) -> bool {
    size.width >= StatusTheme::WIDTH && size.height >= StatusTheme::HEIGHT
}

// Wide intermediates preserve exact ceil boundaries, even with a maximum period
// and the reduced-motion scale. This helper owns no clock and mutates nothing.
fn next_boundary(
    time: ClipTime,
    period: Period,
    steps: u32,
    factor: u64,
    looping: bool,
) -> Option<ClipTime> {
    let scaled = u128::from(time.as_nanos() / factor);
    let period = u128::from(period.as_nanos());
    let phase = if looping { scaled % period } else { scaled };
    let step = phase * u128::from(steps) / period;
    let end = ((step + 1) * period).div_ceil(u128::from(steps));
    let cycle = if looping { scaled - phase } else { 0 };
    let next = (cycle + end) * u128::from(factor);
    u64::try_from(next)
        .ok()
        .filter(|&next| next > time.as_nanos())
        .map(ClipTime::from_nanos)
}

#[derive(Clone, Copy)]
enum Role {
    Blank,
    Muted,
    Foreground,
    Accent,
    Bright,
    Label,
}
#[derive(Clone, Copy)]
struct ArtCell {
    ascii: char,
    unicode: char,
    role: Role,
}
impl ArtCell {
    const BLANK: Self = art(' ', Role::Blank);
}
const fn pair(ascii: char, unicode: char, role: Role) -> ArtCell {
    ArtCell {
        ascii,
        unicode,
        role,
    }
}
const fn art(ascii: char, role: Role) -> ArtCell {
    pair(
        ascii,
        match ascii {
            '-' | '_' => '─',
            '=' => '━',
            '/' => '╱',
            '\\' => '╲',
            '|' => '│',
            '.' | '\'' => '·',
            'o' => '●',
            '+' => '✦',
            _ => ascii,
        },
        role,
    )
}
fn shell(x: u16, y: u16) -> ArtCell {
    match (x, y) {
        (7, 0) => pair('.', '╭', Role::Muted),
        (15, 0) => pair('.', '╮', Role::Muted),
        (7, 4) => pair('`', '╰', Role::Muted),
        (15, 4) => pair('\'', '╯', Role::Muted),
        (8..=14, 0 | 4) => art('-', Role::Muted),
        (6, 1) | (16, 3) => art('/', Role::Muted),
        (16, 1) | (6, 3) => art('\\', Role::Muted),
        (5, 2) => art('<', Role::Accent),
        (17, 2) => art('>', Role::Accent),
        (2..=4 | 18..=20, 2) => art('-', Role::Muted),
        _ => ArtCell::BLANK,
    }
}
fn label_cell(label: &str, x: u16, width: u16) -> ArtCell {
    let start = (usize::from(width) - label.len()) / 2;
    usize::from(x)
        .checked_sub(start)
        .and_then(|i| label.as_bytes().get(i))
        .map_or(ArtCell::BLANK, |&symbol| {
            pair(char::from(symbol), char::from(symbol), Role::Label)
        })
}
fn compact_cell(status: StatusKind, x: u16, y: u16, width: u16, height: u16) -> ArtCell {
    if y != height / 2 {
        return ArtCell::BLANK;
    }
    let label = status.label();
    if width >= 11 {
        let start = (width - label.len() as u16 - 4) / 2;
        if let Some(local) = x.checked_sub(start) {
            let symbol = match local {
                0 => '[',
                1 => status.marker(),
                2 => ']',
                4.. => label
                    .as_bytes()
                    .get(usize::from(local - 4))
                    .map_or(' ', |&c| char::from(c)),
                _ => ' ',
            };
            return pair(
                symbol,
                symbol,
                if local <= 2 {
                    Role::Accent
                } else {
                    Role::Label
                },
            );
        }
    } else if width >= 7 {
        return label_cell(label, x, width);
    } else if width >= 3 {
        let start = (width - 3) / 2;
        let symbol = match x.checked_sub(start) {
            Some(0) => '[',
            Some(1) => status.marker(),
            Some(2) => ']',
            _ => ' ',
        };
        return pair(symbol, symbol, Role::Label);
    } else if x == width / 2 {
        let marker = status.marker();
        return pair(marker, marker, Role::Label);
    }
    ArtCell::BLANK
}
