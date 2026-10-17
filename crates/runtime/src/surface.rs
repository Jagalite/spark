//! Permission-limited one-column writes. Existing wide glyphs are left intact.
use ratatui_core::{
    buffer::{Buffer, CellDiffOption, CellWidth},
    layout::Rect,
    style::Style,
};
use unicode_width::UnicodeWidthChar;

/// A non-control scalar with an agreed one-column width. No arbitrary strings/ANSI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph(char);
impl Glyph {
    pub fn new(value: char) -> Option<Self> {
        (!value.is_control() && value.width() == Some(1)).then_some(Self(value))
    }
    pub const fn as_char(self) -> char {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellOp {
    /// Transparency: preserve the entire current cell.
    Keep,
    /// Explicitly replace with a space and the supplied complete style.
    Erase(Style),
    /// Replace a cell, resetting prior attributes before applying this style.
    Replace(Glyph, Style),
}

/// An owned region with optional protected rectangles, all in buffer coordinates.
/// No mutable buffer escape hatch. Local coordinates retain `requested` origin.
///
/// Writes skip every part of existing multi-column glyphs (even when fully owned),
/// special diff cells, and supplied exclusions. Base buffers must be well formed.
/// Row-major writes are linear in the visited row prefix; out-of-order writes are
/// safe but can rescan it. No memory is allocated here.
pub struct OpaqueSurface<'a> {
    buffer: &'a mut Buffer,
    requested: Rect,
    protected: &'a [Rect],
    scan_y: Option<u16>,
    scan_x: u32,
    covered_until: u32,
}
impl<'a> OpaqueSurface<'a> {
    pub fn new(buffer: &'a mut Buffer, requested: Rect, protected: &'a [Rect]) -> Self {
        Self {
            buffer,
            requested,
            protected,
            scan_y: None,
            scan_x: 0,
            covered_until: 0,
        }
    }
    pub const fn requested(&self) -> Rect {
        self.requested
    }
    /// Returns true only when an actual replacement occurred.
    pub fn write(&mut self, x: u16, y: u16, operation: CellOp) -> bool {
        if matches!(operation, CellOp::Keep)
            || x >= self.requested.width
            || y >= self.requested.height
        {
            return false;
        }
        let Some(x) = self.requested.x.checked_add(x) else {
            return false;
        };
        let Some(y) = self.requested.y.checked_add(y) else {
            return false;
        };
        if !contains(self.buffer.area, x, y) || self.protected.iter().any(|r| contains(*r, x, y)) {
            return false;
        }
        if !self.single_owned_cell(x, y) {
            return false;
        }
        let Some(cell) = self.buffer.cell_mut((x, y)) else {
            return false;
        };
        let (glyph, style) = match operation {
            CellOp::Keep => return false,
            CellOp::Erase(style) => (' ', style),
            CellOp::Replace(glyph, style) => (glyph.0, style),
        };
        cell.reset();
        cell.set_char(glyph).set_style(style);
        true
    }
    fn single_owned_cell(&mut self, x: u16, y: u16) -> bool {
        if self.scan_y != Some(y) || self.scan_x > u32::from(x) {
            self.scan_y = Some(y);
            self.scan_x = u32::from(self.buffer.area.x);
            self.covered_until = self.scan_x;
        }
        let mut safe = false;
        while self.scan_x <= u32::from(x) {
            let Some(cell) = self.buffer.cell((self.scan_x as u16, y)) else {
                return false;
            };
            // Control strings violate Ratatui's base-buffer invariant; reject safely.
            let width = match cell.diff_option {
                CellDiffOption::ForcedWidth(width) => width.get(),
                _ if cell.symbol().chars().any(char::is_control) => 0,
                _ => cell.cell_width(),
            };
            #[allow(deprecated)]
            let normal = cell.diff_option == CellDiffOption::None && !cell.skip;
            safe = self.scan_x >= self.covered_until && width == 1 && normal;
            self.covered_until = self.covered_until.max(self.scan_x + u32::from(width));
            self.scan_x += 1;
        }
        safe
    }
}
fn contains(rect: Rect, x: u16, y: u16) -> bool {
    u32::from(x) >= u32::from(rect.x)
        && u32::from(x) < u32::from(rect.x) + u32::from(rect.width)
        && u32::from(y) >= u32::from(rect.y)
        && u32::from(y) < u32::from(rect.y) + u32::from(rect.height)
}

/// Small explicit masks, expressed in requested-local coordinates.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PaintMask {
    #[default]
    Region,
    Border,
}
impl PaintMask {
    fn allows(self, area: Rect, x: u16, y: u16) -> bool {
        x < area.width
            && y < area.height
            && match self {
                Self::Region => true,
                Self::Border => x == 0 || y == 0 || x + 1 == area.width || y + 1 == area.height,
            }
    }
}

/// Glyph replacement rights restricted to a simple mask and explicit exclusions.
pub struct MaskedSurface<'a> {
    inner: OpaqueSurface<'a>,
    mask: PaintMask,
}
impl<'a> MaskedSurface<'a> {
    pub fn new(
        buffer: &'a mut Buffer,
        requested: Rect,
        protected: &'a [Rect],
        mask: PaintMask,
    ) -> Self {
        Self {
            inner: OpaqueSurface::new(buffer, requested, protected),
            mask,
        }
    }
    pub fn write(&mut self, x: u16, y: u16, operation: CellOp) -> bool {
        self.mask.allows(self.inner.requested, x, y) && self.inner.write(x, y, operation)
    }
}

/// Only explicitly enabled style fields may be patched. Symbols are never changed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StylePermissions {
    pub foreground: bool,
    pub background: bool,
    pub modifiers: bool,
    pub underline_color: bool,
}
impl StylePermissions {
    pub const fn foreground() -> Self {
        Self {
            foreground: true,
            background: false,
            modifiers: false,
            underline_color: false,
        }
    }
    pub const fn foreground_and_modifiers() -> Self {
        Self {
            modifiers: true,
            ..Self::foreground()
        }
    }
}

/// Field-limited styling over freshly drawn host content. Conservatively skips
/// complete wide/special-cell footprints, just like glyph-writing surfaces.
pub struct StyleSurface<'a> {
    inner: OpaqueSurface<'a>,
    mask: PaintMask,
    permissions: StylePermissions,
}
impl<'a> StyleSurface<'a> {
    pub fn new(
        buffer: &'a mut Buffer,
        requested: Rect,
        protected: &'a [Rect],
        mask: PaintMask,
        permissions: StylePermissions,
    ) -> Self {
        Self {
            inner: OpaqueSurface::new(buffer, requested, protected),
            mask,
            permissions,
        }
    }
    pub fn patch(&mut self, x: u16, y: u16, style: Style) -> bool {
        if !self.mask.allows(self.inner.requested, x, y) {
            return false;
        }
        let Some(x) = self.inner.requested.x.checked_add(x) else {
            return false;
        };
        let Some(y) = self.inner.requested.y.checked_add(y) else {
            return false;
        };
        if !contains(self.inner.buffer.area, x, y)
            || self.inner.protected.iter().any(|r| contains(*r, x, y))
            || !self.inner.single_owned_cell(x, y)
        {
            return false;
        }
        let Some(cell) = self.inner.buffer.cell_mut((x, y)) else {
            return false;
        };
        if self.permissions.foreground
            && let Some(fg) = style.fg
        {
            cell.fg = fg;
        }
        if self.permissions.background
            && let Some(bg) = style.bg
        {
            cell.bg = bg;
        }
        if self.permissions.underline_color
            && let Some(color) = style.underline_color
        {
            cell.underline_color = color;
        }
        if self.permissions.modifiers {
            cell.modifier.insert(style.add_modifier);
            cell.modifier.remove(style.sub_modifier);
        }
        true
    }
}
