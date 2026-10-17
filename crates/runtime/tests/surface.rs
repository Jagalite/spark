use ratatui_core::{
    buffer::{Buffer, CellDiffOption},
    layout::Rect,
    style::{Color, Modifier, Style},
};
use spark_runtime::surface::{CellOp, Glyph, OpaqueSurface};
fn op() -> CellOp {
    CellOp::Replace(Glyph::new('*').unwrap(), Style::new().fg(Color::Cyan))
}
#[test]
fn validates_art_glyphs() {
    for c in ['\n', '\r', '\x1b', '界', '\u{0301}', '\u{200d}'] {
        assert!(Glyph::new(c).is_none(), "{c:?}");
    }
    for c in [' ', '*', '◆', '━', '·'] {
        assert!(Glyph::new(c).is_some());
    }
}
#[test]
fn clips_offset_protected_and_empty_regions() {
    let mut b = Buffer::with_lines(["abcdefgh", "ijklmnop", "qrstuvwx"]);
    let original = b.clone();
    let p = [Rect::new(3, 1, 1, 1)];
    let mut s = OpaqueSurface::new(&mut b, Rect::new(2, 1, 20, 20), &p);
    for y in 0..20 {
        for x in 0..20 {
            s.write(x, y, op());
        }
    }
    for y in 0..3 {
        for x in 0..8 {
            if y == 0 || x < 2 || (x == 3 && y == 1) {
                assert_eq!(b[(x, y)], original[(x, y)]);
            } else {
                assert_eq!(b[(x, y)].symbol(), "*");
            }
        }
    }
    for area in [Rect::new(0, 0, 0, 0), Rect::new(u16::MAX, u16::MAX, 10, 10)] {
        let before = b.clone();
        let mut s = OpaqueSurface::new(&mut b, area, &[]);
        for y in 0..10 {
            for x in 0..10 {
                assert!(!s.write(x, y, op()));
            }
        }
        assert_eq!(b, before);
    }
}
#[test]
fn keep_is_not_erase_and_replace_resets_modifiers() {
    let mut b = Buffer::with_lines(["a"]);
    b[(0, 0)].set_style(Style::new().bg(Color::Red).add_modifier(Modifier::REVERSED));
    let before = b.clone();
    let mut s = OpaqueSurface::new(&mut b, Rect::new(0, 0, 1, 1), &[]);
    assert!(!s.write(0, 0, CellOp::Keep));
    assert_eq!(b, before);
    let mut s = OpaqueSurface::new(&mut b, Rect::new(0, 0, 1, 1), &[]);
    assert!(s.write(0, 0, CellOp::Erase(Style::new())));
    assert_eq!(b[(0, 0)].symbol(), " ");
    assert_eq!(b[(0, 0)].bg, Color::Reset);
    assert_eq!(b[(0, 0)].modifier, Modifier::empty());
}
#[test]
fn wide_footprints_survive_both_edges_and_out_of_order_writes() {
    let mut b = Buffer::with_lines(["a界e\u{0301}界z"]);
    let before = b.clone();
    let mut s = OpaqueSurface::new(&mut b, Rect::new(2, 0, 4, 1), &[]);
    for x in [3, 0, 2, 1, 0] {
        s.write(x, 0, op());
    }
    for x in [0, 1, 2, 4, 5, 6] {
        assert_eq!(b[(x, 0)], before[(x, 0)], "x={x}");
    }
    assert_eq!(b[(3, 0)].symbol(), "*");
}
#[test]
fn forced_width_and_skip_cells_are_preserved() {
    let mut b = Buffer::with_lines(["abcdefgh"]);
    b[(0, 0)].set_diff_option(CellDiffOption::ForcedWidth(5.try_into().unwrap()));
    b[(6, 0)].set_diff_option(CellDiffOption::Skip);
    let before = b.clone();
    let mut s = OpaqueSurface::new(&mut b, Rect::new(2, 0, 6, 1), &[]);
    for x in 0..6 {
        s.write(x, 0, op());
    }
    for x in [0, 1, 2, 3, 4, 6] {
        assert_eq!(b[(x, 0)], before[(x, 0)]);
    }
    assert_eq!(b[(5, 0)].symbol(), "*");
    assert_eq!(b[(7, 0)].symbol(), "*");
}
#[test]
fn actual_buffer_can_have_nonzero_origin() {
    let mut b = Buffer::empty(Rect::new(10, 20, 3, 2));
    let mut s = OpaqueSurface::new(&mut b, Rect::new(9, 19, 4, 3), &[]);
    assert!(!s.write(0, 0, op()));
    assert!(s.write(1, 1, op()));
    assert_eq!(b[(10, 20)].symbol(), "*");
}
