use crate::ColorProfile;
use ratatui_core::style::{Color, Style};

/// Remappable roles. `None` means the terminal background is unknown; no blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub foreground: Style,
    pub muted: Style,
    pub accent: Style,
    pub highlight: Style,
    pub background: Option<Color>,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            foreground: Style::new().fg(Color::Rgb(155, 181, 208)),
            muted: Style::new().fg(Color::Rgb(49, 70, 97)),
            accent: Style::new().fg(Color::Rgb(46, 207, 225)),
            highlight: Style::new().fg(Color::Rgb(230, 255, 246)),
            background: None,
        }
    }
}
impl Palette {
    pub fn amber() -> Self {
        Self {
            accent: Style::new().fg(Color::Rgb(250, 167, 62)),
            highlight: Style::new().fg(Color::Rgb(255, 240, 174)),
            ..Self::default()
        }
    }
    /// Resolve a caller's style without inventing an unknown background.
    pub fn resolve(self, mut style: Style, profile: ColorProfile) -> Style {
        if style.bg.is_none() {
            style.bg = self.background;
        }
        match profile {
            ColorProfile::TrueColor => style,
            ColorProfile::Ansi16 => {
                style.fg = style.fg.map(ansi16);
                style.bg = style.bg.map(ansi16);
                style.underline_color = style.underline_color.map(ansi16);
                style
            }
            ColorProfile::Monochrome => {
                style.fg = Some(Color::Reset);
                style.bg = Some(Color::Reset);
                style.underline_color = Some(Color::Reset);
                style
            }
        }
    }
}

fn ansi16(color: Color) -> Color {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(i) if i < 16 => return ANSI[i as usize].0,
        Color::Indexed(i) if i >= 232 => {
            let v = 8 + (i - 232) * 10;
            (v, v, v)
        }
        Color::Indexed(i) => {
            let i = i - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            (level(i / 36), level((i / 6) % 6), level(i % 6))
        }
        other => return other,
    };
    ANSI.iter()
        .min_by_key(|(_, (ar, ag, ab))| {
            (r as i32 - *ar).pow(2) + (g as i32 - *ag).pow(2) + (b as i32 - *ab).pow(2)
        })
        .unwrap()
        .0
}
const ANSI: [(Color, (i32, i32, i32)); 16] = [
    (Color::Black, (0, 0, 0)),
    (Color::Red, (128, 0, 0)),
    (Color::Green, (0, 128, 0)),
    (Color::Yellow, (128, 128, 0)),
    (Color::Blue, (0, 0, 128)),
    (Color::Magenta, (128, 0, 128)),
    (Color::Cyan, (0, 128, 128)),
    (Color::Gray, (192, 192, 192)),
    (Color::DarkGray, (128, 128, 128)),
    (Color::LightRed, (255, 0, 0)),
    (Color::LightGreen, (0, 255, 0)),
    (Color::LightYellow, (255, 255, 0)),
    (Color::LightBlue, (0, 0, 255)),
    (Color::LightMagenta, (255, 0, 255)),
    (Color::LightCyan, (0, 255, 255)),
    (Color::White, (255, 255, 255)),
];
