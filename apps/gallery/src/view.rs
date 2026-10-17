use crate::{
    app::App,
    design::{self, Selection},
};
use ratatui::{
    Frame,
    layout::{Rect, Size},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};
use spark_runtime::{ClipTime, ColorProfile, GlyphProfile, Motion, SampleContext};

const BACKGROUND: Color = Color::Rgb(10, 16, 27);
const PANEL: Color = Color::Rgb(14, 23, 37);
const INK: Color = Color::Rgb(213, 225, 236);
const MUTED: Color = Color::Rgb(126, 149, 174);
const EDGE: Color = Color::Rgb(40, 60, 82);
#[derive(Clone, Copy)]
struct Regions {
    header: Rect,
    sidebar: Rect,
    panel: Rect,
    footer: Rect,
    art: Rect,
    compact: bool,
}
fn inset(rect: Rect, x: u16, y: u16) -> Rect {
    let x = x.min(rect.width / 2);
    let y = y.min(rect.height / 2);
    Rect::new(
        rect.x.saturating_add(x),
        rect.y.saturating_add(y),
        rect.width.saturating_sub(x * 2),
        rect.height.saturating_sub(y * 2),
    )
}
fn row(rect: Rect, offset: u16) -> Rect {
    if offset >= rect.height {
        Rect::default()
    } else {
        Rect::new(rect.x, rect.y.saturating_add(offset), rect.width, 1)
    }
}
fn fit(rect: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(rect.width);
    let height = height.min(rect.height);
    Rect::new(
        rect.x + (rect.width - width) / 2,
        rect.y + (rect.height - height) / 2,
        width,
        height,
    )
}
fn regions(size: Size, width: u16, height: u16, embedded: bool) -> Regions {
    let full = Rect::new(0, 0, size.width, size.height);
    if size.width < 44 || size.height < 17 {
        let panel = Rect::new(0, 0, size.width, size.height.saturating_sub(1));
        let inner = if panel.width >= 3 && panel.height >= 3 {
            inset(panel, 1, 1)
        } else {
            panel
        };
        return Regions {
            header: Rect::default(),
            sidebar: Rect::default(),
            panel,
            footer: row(full, size.height.saturating_sub(1)),
            art: fit(inner, width, height),
            compact: true,
        };
    }
    let full = inset(full, if size.width >= 70 { 2 } else { 1 }, 1);
    let header = Rect::new(full.x, full.y, full.width, 4);
    let footer = Rect::new(full.x, full.bottom().saturating_sub(3), full.width, 3);
    let body = Rect::new(
        full.x,
        header.bottom(),
        full.width,
        full.height.saturating_sub(7),
    );
    let side_width = if size.width >= 96 { 24 } else { 0 };
    let gap = if side_width > 0 { 3 } else { 0 };
    let sidebar = Rect::new(body.x, body.y, side_width, body.height);
    let panel = Rect::new(
        body.x + side_width + gap,
        body.y,
        body.width.saturating_sub(side_width + gap),
        body.height,
    );
    let inner = inset(panel, 3, 2);
    let reserve = if embedded { 4 } else { 2 };
    let art_region = Rect::new(
        inner.x,
        inner.y.saturating_add(reserve.min(inner.height)),
        inner.width,
        inner.height.saturating_sub(reserve + 2),
    );
    Regions {
        header,
        sidebar,
        panel,
        footer,
        art: fit(art_region, width, height),
        compact: false,
    }
}
pub fn sample_area(size: Size, width: u16, height: u16, embedded: bool) -> Rect {
    regions(size, width, height, embedded).art
}
fn style(app: &App, color: Color) -> Style {
    if app.profile.colors == ColorProfile::Monochrome {
        Style::default()
    } else {
        app.palette
            .resolve(Style::default().fg(color), app.profile.colors)
    }
}
fn text(frame: &mut Frame, area: Rect, content: impl Into<Line<'static>>, style: Style) {
    frame.render_widget(Paragraph::new(content.into()).style(style), area);
}
fn panel(frame: &mut Frame, app: &App, area: Rect, title: String) {
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(style(app, EDGE))
        .title(Span::styled(title, style(app, MUTED)))
        .style(if app.profile.colors == ColorProfile::Monochrome {
            Style::default()
        } else {
            app.palette
                .resolve(Style::default().bg(PANEL), app.profile.colors)
        });
    if app.profile.glyphs == GlyphProfile::Ascii {
        block = block.border_set(ratatui::symbols::border::Set {
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        });
    }
    frame.render_widget(block, area);
}
pub fn render(frame: &mut Frame, app: &App, clip: ClipTime) {
    let full = frame.area();
    let regions = regions(full.as_size(), app.width, app.height, app.embedded);
    let accent = app.palette.resolve(app.palette.accent, app.profile.colors);
    let background = if app.profile.colors == ColorProfile::Monochrome {
        Style::default()
    } else {
        app.palette
            .resolve(Style::default().bg(BACKGROUND).fg(INK), app.profile.colors)
    };
    frame.render_widget(Block::default().style(background), full);
    let context = SampleContext::new(clip, &app.palette, &app.profile);
    if regions.compact {
        if regions.panel.width >= 3 && regions.panel.height >= 3 {
            panel(
                frame,
                app,
                regions.panel,
                format!(" R0{} / {} ", app.selection.number(), app.selection.title()),
            );
        }
    } else {
        let state = if app.profile.motion == Motion::Static {
            "STATIC"
        } else if app.paused {
            "PAUSED"
        } else if app.designs.complete(app.selection, &context) {
            "SETTLED"
        } else {
            "PLAYING"
        };
        text(
            frame,
            row(regions.header, 0),
            Line::from(vec![
                Span::styled("SPARK", accent.add_modifier(Modifier::BOLD)),
                Span::styled("    /    SIX TERMINAL MOTION STUDIES", style(app, MUTED)),
            ]),
            Style::default(),
        );
        text(
            frame,
            row(regions.header, 2),
            Line::from(vec![
                Span::styled(
                    app.selection.title(),
                    style(app, INK).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("     R0{}  /  {state}", app.selection.number()),
                    accent,
                ),
            ]),
            Style::default(),
        );
        if !regions.sidebar.is_empty() {
            let side = regions.sidebar;
            text(frame, row(side, 0), "THE COLLECTION", style(app, MUTED));
            for selection in Selection::ALL {
                text(
                    frame,
                    row(side, selection.number() as u16 + 1),
                    format!(
                        "{}{:02}  {}",
                        if selection == app.selection { ">" } else { " " },
                        selection.number(),
                        selection.title()
                    ),
                    if selection == app.selection {
                        accent.add_modifier(Modifier::BOLD)
                    } else {
                        style(app, MUTED)
                    },
                );
            }
            text(frame, row(side, 9), "DRAWING CONTRACT", style(app, MUTED));
            text(
                frame,
                row(side, 10),
                app.selection.capability().split(';').next().unwrap(),
                style(app, INK),
            );
            text(
                frame,
                row(side, 11),
                app.selection.capability().split(';').nth(1).unwrap().trim(),
                style(app, INK),
            );
            text(
                frame,
                row(side, 12),
                app.selection.geometry_label(),
                style(app, MUTED),
            );
            text(
                frame,
                row(side, 13),
                app.selection.fit_label(),
                style(app, MUTED),
            );
            text(frame, row(side, 15), "LOCAL EVIDENCE", style(app, MUTED));
            text(
                frame,
                row(side, 17),
                "Fixed-font review done",
                style(app, INK),
            );
            text(
                frame,
                row(side, 18),
                "Exact cells + test hosts",
                style(app, MUTED),
            );
            text(
                frame,
                row(side, 19),
                "Native fonts unqualified",
                style(app, MUTED),
            );
            text(
                frame,
                row(side, 21),
                "See qualification/",
                style(app, MUTED),
            );
        }
        panel(
            frame,
            app,
            regions.panel,
            if app.embedded {
                " IN A SYNTHETIC APPLICATION ".into()
            } else {
                " ISOLATED / LIVE SAMPLE ".into()
            },
        );
        let inner = inset(regions.panel, 3, 1);
        text(
            frame,
            row(inner, 0),
            format!(
                "{} x {} / {:?} / {:?}",
                regions.art.width, regions.art.height, app.profile.glyphs, app.profile.colors
            ),
            style(app, MUTED),
        );
        if app.embedded {
            text(
                frame,
                row(inner, 2),
                "REQUEST #0041  /  HOST-OWNED STATUS",
                style(app, INK).add_modifier(Modifier::BOLD),
            );
            text(
                frame,
                row(inner, 3),
                app.selection.input_label(app.input, app.unknown),
                accent,
            );
        }
        if inner.height >= 3 {
            text(
                frame,
                row(inner, inner.height - 2),
                format!(
                    "clip {:.3}s / speed {:.2}x / seed {}",
                    clip.as_duration().as_secs_f64(),
                    f64::from(app.rate_milli) / 1000.0,
                    if matches!(app.selection, Selection::Mascot | Selection::Prism) {
                        "n/a".to_owned()
                    } else {
                        app.seed.to_string()
                    }
                ),
                style(app, MUTED),
            );
            text(
                frame,
                row(inner, inner.height - 1),
                if app.embedded {
                    "Motion never completes the host request".into()
                } else {
                    app.selection.input_label(app.input, app.unknown)
                },
                style(app, MUTED),
            );
        }
    }
    // Base is freshly drawn every time; only the selected sample is animated.
    let protected = design::base(app.selection, regions.art, &context, frame.buffer_mut());
    app.designs.render(
        app.selection,
        &context,
        app.input,
        app.unknown,
        regions.art,
        &protected,
        frame.buffer_mut(),
    );
    if regions.compact {
        text(
            frame,
            regions.footer,
            "1-6 design  space pause  q quit",
            style(app, MUTED),
        );
    } else {
        text(
            frame,
            row(regions.footer, 0),
            "1-6/tab design  space play/pause  r restart  [ ] scrub  v view  q quit",
            style(app, MUTED),
        );
        text(
            frame,
            row(regions.footer, 1),
            "arrows size  c palette  a ASCII  m colors  p reduced  s static",
            style(app, MUTED),
        );
        text(
            frame,
            row(regions.footer, 2),
            ", . speed  n seed  - + input  u unknown progress",
            style(app, MUTED),
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    #[test]
    fn all_six_preserve_host_status_and_quiet_profile() {
        let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
        let mut app = App::default();
        app.embedded = true;
        for selection in Selection::ALL {
            app.select(selection, std::time::Duration::ZERO);
            for quiet in [false, true] {
                app.profile = if quiet {
                    spark_runtime::Profile::quiet()
                } else {
                    spark_runtime::Profile::default()
                };
                terminal
                    .draw(|frame| render(frame, &app, ClipTime::from_nanos(750_000_000)))
                    .unwrap();
                let buffer = terminal.backend().buffer();
                let content: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
                assert!(content.contains("HOST-OWNED STATUS"), "{}", selection.id());
                assert!(content.contains("Motion never completes"));
                if quiet {
                    for cell in &buffer.content {
                        assert!(
                            cell.symbol().is_ascii(),
                            "{}: {:?}",
                            selection.id(),
                            cell.symbol()
                        );
                        assert_eq!(cell.fg, Color::Reset);
                        assert_eq!(cell.bg, Color::Reset);
                    }
                }
            }
        }
    }
    #[test]
    fn resized_presentations_are_safe_and_advice_geometry_matches() {
        let mut app = App::default();
        for selection in Selection::ALL {
            app.select(selection, std::time::Duration::ZERO);
            for width in [0, 1, 3, 7, 24, 43, 44, 95, 96, 100, 256] {
                for height in [0, 1, 2, 3, 8, 16, 17, 24, 32] {
                    for embedded in [false, true] {
                        app.embedded = embedded;
                        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                        terminal
                            .draw(|frame| render(frame, &app, ClipTime::MAX))
                            .unwrap();
                        let art =
                            sample_area(Size::new(width, height), app.width, app.height, embedded);
                        if !art.is_empty() {
                            assert!(
                                art.right() <= width && art.bottom() <= height,
                                "{art:?} exceeds {width}x{height}"
                            );
                        }
                    }
                }
            }
        }
    }
}
