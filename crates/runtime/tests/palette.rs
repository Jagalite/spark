use ratatui_core::style::{Color, Modifier, Style};
use spark_runtime::{ColorProfile, Palette};

#[test]
fn unknown_background_stays_unknown_and_overrides_survive() {
    let palette = Palette::default();
    let style = Style::new().fg(Color::Cyan).add_modifier(Modifier::ITALIC);
    assert_eq!(palette.resolve(style, ColorProfile::TrueColor), style);
    let palette = Palette {
        background: Some(Color::Blue),
        ..palette
    };
    assert_eq!(
        palette.resolve(style, ColorProfile::TrueColor).bg,
        Some(Color::Blue)
    );
    assert_eq!(
        palette
            .resolve(style.bg(Color::Red), ColorProfile::TrueColor)
            .bg,
        Some(Color::Red)
    );
}
#[test]
fn fallback_colors_are_real_fallbacks_and_keep_modifiers() {
    let palette = Palette::default();
    for color in (0..=255)
        .map(Color::Indexed)
        .chain([Color::Rgb(3, 44, 252), Color::Reset])
    {
        let style = Style::new()
            .fg(color)
            .bg(color)
            .add_modifier(Modifier::BOLD);
        let ansi = palette.resolve(style, ColorProfile::Ansi16);
        for resolved in [ansi.fg, ansi.bg] {
            assert!(!matches!(
                resolved,
                Some(Color::Rgb(..) | Color::Indexed(..))
            ));
        }
        let mono = palette.resolve(style, ColorProfile::Monochrome);
        assert_eq!(mono.fg, Some(Color::Reset));
        assert_eq!(mono.bg, Some(Color::Reset));
        assert_eq!(mono.add_modifier, Modifier::BOLD);
    }
}
