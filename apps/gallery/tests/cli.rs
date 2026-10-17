//! Exercise real process output: export must not initialize a terminal or mix
//! help, progress logs, escape sequences, or UI cells into Rust on stdout.
use std::process::Command;

fn gallery(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spark-gallery"))
        .args(args)
        .output()
        .expect("run gallery binary")
}
#[test]
fn export_is_deterministic_source_only_without_terminal_or_clipboard() {
    let args = [
        "--export-rust",
        "--design",
        "assembly-progress",
        "--unknown",
        "--paused",
    ];
    let first = gallery(&args);
    let second = gallery(&args);
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let source = String::from_utf8(first.stdout).unwrap();
    assert!(source.starts_with("// Spark playground: assembly-progress\n"));
    assert!(source.contains("Progress::unknown()"));
    assert!(source.contains("playback.pause(host)"));
    assert!(source.contains("fn main()"));
    assert!(!source.contains("RUST / e exports"));
    assert!(source.chars().all(|c| c == '\n' || !c.is_control()));
}
#[test]
fn invalid_cli_and_unattached_live_mode_emit_no_partial_export_or_terminal_controls() {
    for args in [
        vec![],
        vec!["--export-rust", "--input", "101"],
        vec!["--export-rust", "--palette", "amber", "--not-an-option"],
        vec!["--smoke", "--paused"],
    ] {
        let output = gallery(&args);
        assert!(!output.status.success(), "accepted {args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.contains(&0x1b));
    }
}
#[test]
fn inspector_snapshot_is_visible_and_stays_bounded_in_tiny_terminals() {
    let output = gallery(&["--snapshot", "--inspect"]);
    assert!(output.status.success());
    let screen = String::from_utf8(output.stdout).unwrap();
    assert!(screen.contains("let design = SegmentedComet"));
    assert!(screen.contains("let palette = Palette::default()"));
    assert!(screen.contains("GlyphProfile::Unicode"));
    assert_eq!(screen.lines().count(), 32);
    for size in ["0x0", "1x1", "7x3", "24x8"] {
        let output = gallery(&["--snapshot", "--inspect", "--terminal", size]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
}
