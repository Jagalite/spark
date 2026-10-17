//! Explicit capabilities, never terminal detection.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GlyphProfile {
    Ascii,
    #[default]
    Unicode,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ColorProfile {
    #[default]
    TrueColor,
    Ansi16,
    Monochrome,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    #[default]
    Full,
    Reduced,
    Static,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub glyphs: GlyphProfile,
    pub colors: ColorProfile,
    pub motion: Motion,
}
impl Profile {
    /// Deliberately quiet, terminal-independent fallback.
    pub const fn quiet() -> Self {
        Self {
            glyphs: GlyphProfile::Ascii,
            colors: ColorProfile::Monochrome,
            motion: Motion::Static,
        }
    }
}
