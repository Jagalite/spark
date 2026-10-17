#![forbid(unsafe_code)]
//! Reproducible exact cells, actual captures, and explicitly scoped local evidence.
mod assets;
#[allow(dead_code)]
#[path = "../../apps/gallery/src/design.rs"]
mod design;
use design::{Designs, Selection};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::Block,
};
use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spark_catalog::SegmentedComet;
use spark_runtime::{ClipTime, Motion, Palette, Profile, SampleContext};
use std::{
    env,
    error::Error,
    fs,
    hint::black_box,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;
fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    env::set_current_dir(root)?;
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("validate") if args.len() == 1 => validate(),
        Some("snapshots") if args.len() == 1 || (args.len() == 2 && args[1] == "--check") => snapshots(args.len() == 2),
        Some("render") | Some("record") => {
            let selected = parse_selection(args.get(1))?;
            let fixture = match args.get(2).map(String::as_str) {
                None => "isolated",
                Some("--fixture") if args.len() == 4 => args[3].as_str(),
                _ => return Err("expected --fixture isolated|table-status|request-result|dashboard|reader".into()),
            };
            for selection in selected {
                render_design(selection, fixture)?;
                if args[0] == "record" { record(selection)?; }
            }
            Ok(())
        }
        Some("qualify") if args.len() == 2 => qualify(parse_selection(args.get(1))?),
        Some("generate-assets") if args.len() == 1 || (args.len() == 2 && args[1] == "--check") => assets::generate(args.len() == 2),
        Some("package-audit") if args.len() == 1 => package_audit(),
        Some("baseline") if args.len() == 1 => { baseline()?; foundation_baseline()?; allocation_baseline() },
        _ => Err("usage: cargo xtask {validate|snapshots [--check]|render <design-id|all> [--fixture <name>]|record <design-id|all>|qualify <design-id|all>|generate-assets [--check]|package-audit|baseline}".into()),
    }
}
fn parse_selection(arg: Option<&String>) -> Result<Vec<Selection>> {
    match arg.map(String::as_str) {
        Some("all") => Ok(Selection::ALL.to_vec()),
        Some(id) => Selection::from_id(id)
            .map(|design| vec![design])
            .ok_or_else(|| format!("unknown design {id}").into()),
        None => Err("a design ID or all is required".into()),
    }
}

fn validate() -> Result<()> {
    let metadata: Value = serde_json::from_slice(
        &Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version", "1"])
            .output()?
            .stdout,
    )?;
    if metadata["workspace_members"].as_array().unwrap().len() != 4 {
        return Err("expected exactly four workspace packages".into());
    }
    for name in ["spark-runtime", "spark-catalog"] {
        let pkg = metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == name)
            .unwrap();
        let allowed: &[&str] = if name == "spark-runtime" {
            &["ratatui-core", "unicode-width"]
        } else {
            &["ratatui-core", "spark-runtime"]
        };
        for dep in pkg["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["kind"].is_null())
        {
            if !allowed.contains(&dep["name"].as_str().unwrap()) {
                return Err(format!("unexpected normal dependency: {}", dep["name"]).into());
            }
        }
        let dir = PathBuf::from(pkg["manifest_path"].as_str().unwrap())
            .parent()
            .unwrap()
            .join("src");
        for path in source_files(&dir)? {
            let source = fs::read_to_string(&path)?;
            for forbidden in [
                "Instant::now",
                "SystemTime",
                "std::thread",
                "std::io",
                "std::fs",
                "std::net",
                "tokio::",
                "unsafe {",
            ] {
                if source.contains(forbidden) {
                    return Err(format!(
                        "renderer boundary violation: {}: {forbidden}",
                        path.display()
                    )
                    .into());
                }
            }
        }
    }
    validate_manifests()?;
    assets::generate(true)?;
    println!(
        "PASS: four packages; renderer dependency allowlist; source boundary scan (not a sandbox); six manifests; authored asset drift"
    );
    Ok(())
}
fn source_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut out = vec![];
    for entry in fs::read_dir(path)? {
        let p = entry?.path();
        if p.is_dir() {
            out.extend(source_files(&p)?);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}
fn source_hash() -> Result<String> {
    let mut hash = Sha256::new();
    for root in [
        "crates/runtime/src",
        "crates/runtime/tests",
        "crates/catalog/src",
        "crates/catalog/tests",
        "crates/catalog/examples",
        "fixtures/consumer/src",
        "fixtures/consumer-all/src",
        "fixtures/holdout/src",
        "fixtures/holdout/tests",
        "fixtures/extension-starter/src",
        "fixtures/extension-starter/tests",
        "apps/gallery/src",
        "apps/gallery/tests",
        "xtask/src",
    ] {
        for path in source_files(Path::new(root))? {
            hash.update(path.to_string_lossy().as_bytes());
            hash.update([0]);
            hash.update(fs::read(&path)?);
        }
    }
    for path in [
        "Cargo.lock",
        "Cargo.toml",
        "rust-toolchain.toml",
        "crates/runtime/Cargo.toml",
        "crates/catalog/Cargo.toml",
        "apps/gallery/Cargo.toml",
        "xtask/Cargo.toml",
        "fixtures/consumer/Cargo.toml",
        "fixtures/consumer/Cargo.lock",
        "fixtures/consumer-all/Cargo.toml",
        "fixtures/consumer-all/Cargo.lock",
        "fixtures/holdout/Cargo.toml",
        "fixtures/holdout/Cargo.lock",
        "fixtures/extension-starter/Cargo.toml",
        "fixtures/extension-starter/Cargo.lock",
        "fixtures/extension-starter/rust-toolchain.toml",
        "fixtures/extension-starter/copy.sh",
        "fixtures/extension-starter/preview.txt",
        "apps/gallery/tests/terminal.py",
        "fixtures/cells/segmented-comet.json",
        "fixtures/cells/foundation-six.json",
        ".cargo/config.toml",
        ".github/workflows/ci.yml",
    ] {
        hash.update(path.as_bytes());
        hash.update([0]);
        hash.update(fs::read(path)?);
    }
    for path in [
        "scripts/render_previews.py",
        "scripts/capture_terminal.py",
        "scripts/qualify_host_endurance.py",
        "catalog/pocket-mascot/frames.json",
    ]
    .into_iter()
    .map(PathBuf::from)
    .chain(
        Selection::ALL
            .into_iter()
            .map(|s| PathBuf::from(format!("catalog/{}/manifest.json", s.id()))),
    ) {
        hash.update(path.to_string_lossy().as_bytes());
        hash.update([0]);
        hash.update(fs::read(path)?);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn frame(time_ms: u64, area: Rect, profile: Profile, palette: Palette) -> Buffer {
    let mut buffer = Buffer::empty(area);
    let comet = SegmentedComet::new();
    let context = SampleContext::new(
        ClipTime::from_nanos(time_ms * 1_000_000),
        &palette,
        &profile,
    );
    comet.sample(&context).render(area, &mut buffer);
    buffer
}
fn cells(buffer: &Buffer) -> Value {
    let rows:Vec<_>=(buffer.area.y..buffer.area.bottom()).map(|y| {
        (buffer.area.x..buffer.area.right()).map(|x| {
            let c=&buffer[(x,y)];
            json!({"symbol":c.symbol(),"fg":format!("{:?}",c.fg),"bg":format!("{:?}",c.bg),"underline_color":format!("{:?}",c.underline_color),"modifiers":c.modifier.bits(),"diff":format!("{:?}",c.diff_option)})
        }).collect::<Vec<_>>()
    }).collect();
    json!({"area":[buffer.area.x,buffer.area.y,buffer.area.width,buffer.area.height],"rows":rows})
}
fn snapshot() -> Value {
    let mut frames = vec![];
    for t in [0, 1, 200, 400, 800, 1200, 1599, 1600, 1601] {
        frames.push(json!({"time_ms":t,"cells":cells(&frame(t,Rect::new(0,0,24,3),Profile::default(),Palette::default()))}));
    }
    for (name, profile) in [
        ("ascii-mono-static", Profile::quiet()),
        (
            "reduced",
            Profile {
                motion: Motion::Reduced,
                ..Profile::default()
            },
        ),
    ] {
        frames.push(json!({"profile":name,"time_ms":800,"cells":cells(&frame(800,Rect::new(0,0,16,1),profile,Palette::default()))}));
    }
    json!({"schema":1,"design":"segmented-comet","visual_revision":1,"seed":7,"period_ms":1600,"frames":frames})
}
fn snapshots(check: bool) -> Result<()> {
    // The reviewed F01-F06 golden is immutable in this workflow. New foundation
    // fixtures are separate, so review never silently replaces historical art.
    let original = "fixtures/cells/segmented-comet.json";
    let expected = serde_json::to_string_pretty(&snapshot())? + "\n";
    if fs::read_to_string(original)? != expected {
        return Err(
            "original R01 golden drift; inspect and review, never replace implicitly".into(),
        );
    }
    let path = "fixtures/cells/foundation-six.json";
    let expected = serde_json::to_string_pretty(&foundation_snapshot())? + "\n";
    if check {
        if fs::read_to_string(path)? != expected {
            return Err(
                "foundation exact-cell snapshot drift; inspect and review before updating".into(),
            );
        }
        println!("PASS: original R01 plus six-design exact glyph/style/diff/coverage fixtures");
    } else {
        fs::write(path, expected)?;
        println!("Wrote separate {path}; original R01 unchanged; independent review required");
    }
    Ok(())
}
fn foundation_snapshot() -> Value {
    let designs = Designs::new(7);
    let mut entries = Vec::new();
    for selection in Selection::ALL {
        let (width, height) = match selection {
            Selection::Comet | Selection::Assembly => (24, 3),
            Selection::Border => (20, 6),
            Selection::Embers => (24, 8),
            Selection::Mascot => (12, 6),
            Selection::Prism => (18, 1),
        };
        let area = Rect::new(0, 0, width, height);
        let end = designs.duration_ms(selection).unwrap_or(8000);
        let mut samples = Vec::new();
        for time in [0, 720, end - 1, end, end + 1, u64::MAX] {
            samples.push(exact_case(
                &designs,
                selection,
                "key-time",
                time,
                Profile::default(),
                area,
                area,
                65,
                false,
                false,
            ));
        }
        for (label, profile) in [
            ("quiet", Profile::quiet()),
            (
                "reduced",
                Profile {
                    motion: Motion::Reduced,
                    ..Profile::default()
                },
            ),
            (
                "ascii-ansi16",
                Profile {
                    glyphs: spark_runtime::GlyphProfile::Ascii,
                    colors: spark_runtime::ColorProfile::Ansi16,
                    ..Profile::default()
                },
            ),
        ] {
            samples.push(exact_case(
                &designs, selection, label, 720, profile, area, area, 65, false, false,
            ));
        }
        samples.push(exact_case(
            &designs,
            selection,
            "tiny",
            720,
            Profile::default(),
            Rect::new(0, 0, 3, 1),
            Rect::new(0, 0, 3, 1),
            65,
            false,
            false,
        ));
        samples.push(exact_case(
            &designs,
            selection,
            "offset-clipped",
            720,
            Profile::default(),
            Rect::new(9, 3, width, height),
            Rect::new(0, 0, 12, 5),
            65,
            false,
            false,
        ));
        samples.push(exact_case(
            &designs,
            selection,
            "protected-host-cell",
            720,
            Profile::default(),
            area,
            area,
            65,
            false,
            true,
        ));
        if matches!(selection, Selection::Assembly | Selection::Embers) {
            for input in [0, 100] {
                samples.push(exact_case(
                    &designs,
                    selection,
                    "input-boundary",
                    720,
                    Profile::default(),
                    area,
                    area,
                    input,
                    false,
                    false,
                ));
            }
        }
        if selection == Selection::Assembly {
            samples.push(exact_case(
                &designs,
                selection,
                "unknown-progress",
                720,
                Profile::default(),
                area,
                area,
                65,
                true,
                false,
            ));
        }
        entries.push(json!({"design":selection.id(),"visual_revision":1,"seed":if matches!(selection,Selection::Mascot|Selection::Prism){Value::Null}else{json!(7)},"samples":samples}));
    }
    json!({"schema":1,"scope":"Additional reviewed foundation cells; original R01 golden retained separately","palette":"default","coverage_note":"Changed-from-base is an observed cell difference mask, not a claim that unchanged cells were never written","designs":entries})
}
#[allow(clippy::too_many_arguments)]
fn exact_case(
    designs: &Designs,
    selection: Selection,
    label: &str,
    time_ms: u64,
    profile: Profile,
    requested: Rect,
    buffer_area: Rect,
    input: u8,
    unknown: bool,
    host_protection: bool,
) -> Value {
    let palette = Palette::default();
    let clip = ClipTime::from_nanos(time_ms.saturating_mul(1_000_000));
    let context = SampleContext::new(clip, &palette, &profile);
    let mut buffer = Buffer::empty(buffer_area);
    let mut protected = design::base(selection, requested, &context, &mut buffer);
    if host_protection && requested.width > 1 && !requested.is_empty() {
        let marker = Rect::new(requested.x + 1, requested.y, 1, 1);
        if buffer_area.contains((marker.x, marker.y).into()) {
            buffer.set_string(
                marker.x,
                marker.y,
                "P",
                Style::new()
                    .fg(Color::Magenta)
                    .bg(Color::Blue)
                    .add_modifier(ratatui_core::style::Modifier::ITALIC),
            );
            protected.push(marker);
        }
    }
    let before = buffer.clone();
    designs.render(
        selection,
        &context,
        input,
        unknown,
        requested,
        &protected,
        &mut buffer,
    );
    let mut outside_unchanged = true;
    let mut protected_unchanged = true;
    let changed: Vec<Vec<bool>> = (buffer_area.y..buffer_area.bottom())
        .map(|y| {
            (buffer_area.x..buffer_area.right())
                .map(|x| {
                    let different = buffer[(x, y)] != before[(x, y)];
                    if !requested.contains((x, y).into()) {
                        outside_unchanged &= !different;
                    }
                    if protected.iter().any(|rect| rect.contains((x, y).into())) {
                        protected_unchanged &= !different;
                    }
                    different
                })
                .collect()
        })
        .collect();
    assert!(
        outside_unchanged && protected_unchanged,
        "{} fixture violated requested/protected coverage",
        selection.id()
    );
    json!({"case":label,"clip_time_ns":clip.as_nanos(),"profile":{"glyphs":format!("{:?}",profile.glyphs),"colors":format!("{:?}",profile.colors),"motion":format!("{:?}",profile.motion)},"input":{"value":f64::from(input)/100.0,"unknown":unknown},"requested_area":[requested.x,requested.y,requested.width,requested.height],"protected":protected.iter().map(|rect| [rect.x,rect.y,rect.width,rect.height]).collect::<Vec<_>>(),"changed_from_base":changed,"outside_request_unchanged":outside_unchanged,"protected_unchanged":protected_unchanged,"cells":cells(&buffer)})
}
fn percentile(sorted: &[u128], p: usize) -> u128 {
    sorted[(sorted.len() - 1) * p / 100]
}
fn baseline() -> Result<()> {
    fs::create_dir_all("qualification/local")?;
    let mut timings = vec![];
    let palette = Palette::default();
    let profile = Profile::default();
    for (w, h) in [(32, 3), (60, 12), (120, 40)] {
        let area = Rect::new(0, 0, w, h);
        let mut buffer = Buffer::empty(area);
        let comet = SegmentedComet::new();
        let mut times = Vec::with_capacity(5000);
        for i in 0..5200u64 {
            let context =
                SampleContext::new(ClipTime::from_nanos(i * 33_000_000), &palette, &profile);
            let start = Instant::now();
            black_box(comet.sample(&context)).render(area, black_box(&mut buffer));
            let elapsed = start.elapsed().as_nanos();
            if i >= 200 {
                times.push(elapsed);
            }
        }
        times.sort();
        timings.push(json!({"requested_size":[w,h],"painted_max_rows":3,"samples":times.len(),"median_ns":percentile(&times,50),"p95_ns":percentile(&times,95),"p99_ns":percentile(&times,99)}));
    }
    let mut construction = vec![];
    for _ in 0..5000 {
        let start = Instant::now();
        black_box(SegmentedComet::new());
        construction.push(start.elapsed().as_nanos());
    }
    construction.sort();
    let mut output = vec![];
    for mode in ["no-art", "static", "animated"] {
        let area = Rect::new(0, 0, 32, 3);
        let mut previous = Buffer::empty(area);
        let mut bytes = Vec::<u8>::new();
        let mut counts = vec![];
        for i in 0..101 {
            let next = if mode == "no-art" {
                Buffer::empty(area)
            } else {
                frame(
                    i * 33,
                    area,
                    Profile {
                        motion: if mode == "static" {
                            Motion::Static
                        } else {
                            Motion::Full
                        },
                        ..Profile::default()
                    },
                    palette,
                )
            };
            let diff = previous.diff(&next);
            let changed = diff.len();
            bytes.clear();
            let start = Instant::now();
            {
                let mut backend = CrosstermBackend::new(&mut bytes);
                backend.draw(diff.into_iter())?;
                backend.flush()?;
            }
            let encode_ns = start.elapsed().as_nanos();
            if i > 0 {
                counts.push(json!({"changed_cells":changed,"backend_bytes":bytes.len(),"encode_ns":encode_ns}));
            }
            previous = next;
        }
        output.push(json!({"mode":mode,"frames":counts}));
    }
    let command = |program: &str, args: &[&str]| {
        Command::new(program)
            .args(args)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_else(|_| "unavailable".into())
    };
    let report = json!({"schema":1,"scope":"F06 first baseline, not release qualification","source_sha256":source_hash()?,"target":env::consts::ARCH,"os":env::consts::OS,"rustc":command("rustc",&["-Vv"]),"hardware":command("uname",&["-a"]),"cpu":fs::read_to_string("/proc/cpuinfo").ok().and_then(|s|s.lines().find(|l|l.starts_with("model name")).map(str::to_owned)),"build":if cfg!(debug_assertions){"debug"}else{"release, thin LTO, one codegen unit"},"configuration":{"seed":7,"period_ms":1600,"palette":"default","profile":"unicode-truecolor-full","sample_step_ms":33},"construction_p95_ns":percentile(&construction,95),"warmed_sample_and_paint":timings,"steady_output":output,"limitations":["Only R01; not six-design or scene qualification","Construction is trivial; nanosecond timing includes clock overhead","Output uses a Crossterm Vec<u8> backend, no terminal IO","Allocation counts, application CPU, terminal-emulator CPU, memory growth are not measured","Hidden/paused lifecycle helpers deferred beyond F06","Shared virtual host; no universal latency guarantee"]});
    fs::write(
        "qualification/local/baseline.json",
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report["warmed_sample_and_paint"])?
    );
    println!("Full report: qualification/local/baseline.json");
    Ok(())
}

fn validate_manifests() -> Result<()> {
    for selection in Selection::ALL {
        let path = format!("catalog/{}/manifest.json", selection.id());
        let document: Value = serde_json::from_slice(&fs::read(&path)?)?;
        if document["schema"] != 1
            || document["id"] != selection.id()
            || document["visual_revision"] != 1
        {
            return Err(format!("invalid schema, ID, or visual revision in {path}").into());
        }
        for field in ["title", "reference", "status", "driver"] {
            if document[field].as_str().is_none_or(|s| s.is_empty()) {
                return Err(format!("missing nonempty {field} in {path}").into());
            }
        }
        let implementation = document["implementation"]
            .as_str()
            .ok_or("missing implementation")?;
        if !implementation.starts_with("crates/catalog/src/")
            || implementation.contains("..")
            || !Path::new(implementation).is_file()
        {
            return Err(format!("invalid implementation path in {path}").into());
        }
        if document["surfaces"].as_array().is_none_or(|a| a.is_empty())
            || document["profiles"].as_array().is_none_or(|a| a.is_empty())
        {
            return Err(format!("missing surfaces/profiles in {path}").into());
        }
        for field in ["preferred", "minimum_full", "composition_cap"] {
            let pair = document["geometry"][field]
                .as_array()
                .ok_or("missing geometry")?;
            if pair.len() != 2
                || pair
                    .iter()
                    .any(|n| !n.as_u64().is_some_and(|v| (1..=256).contains(&v)))
            {
                return Err(format!("invalid {field} geometry in {path}").into());
            }
        }
        for field in ["code", "artwork", "license"] {
            if document["source_policy"][field]
                .as_str()
                .is_none_or(|s| s.is_empty())
            {
                return Err(format!("missing {field} provenance in {path}").into());
            }
        }
        for field in [
            "exact_cells",
            "fixed_font_preview",
            "qualification",
            "visual_review",
            "platforms",
        ] {
            if document["evidence"][field]
                .as_str()
                .is_none_or(|s| s.is_empty())
            {
                return Err(format!("missing evidence descriptor {field} in {path}").into());
            }
        }
    }
    Ok(())
}

fn sampled_frame(
    selection: Selection,
    time_ms: u64,
    size: ratatui_core::layout::Size,
    profile: Profile,
    input: u8,
    unknown: bool,
) -> Buffer {
    let area = Rect::new(0, 0, size.width, size.height);
    let mut buffer = Buffer::empty(area);
    let designs = Designs::new(7);
    let palette = Palette::default();
    let context = SampleContext::new(
        ClipTime::from_nanos(time_ms.saturating_mul(1_000_000)),
        &palette,
        &profile,
    );
    let protected = design::base(selection, area, &context, &mut buffer);
    designs.render(
        selection,
        &context,
        input,
        unknown,
        area,
        &protected,
        &mut buffer,
    );
    buffer
}
fn embedded_frame(selection: Selection, time_ms: u64, fixture: &str) -> Result<Buffer> {
    let (width, height) = match fixture {
        "table-status" => (76, 16),
        "request-result" => (72, 20),
        "dashboard" => (100, 32),
        "reader" => (80, 24),
        _ => return Err(format!("unknown fixture {fixture}").into()),
    };
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    let fg = Style::new().fg(Color::Rgb(155, 181, 208));
    Block::bordered()
        .title(format!(" SPARK / {fixture} / HOST CONTENT "))
        .style(fg)
        .render(buffer.area, &mut buffer);
    let designs = Designs::new(7);
    let palette = Palette::default();
    let profile = Profile::default();
    let context = SampleContext::new(
        ClipTime::from_nanos(time_ms.saturating_mul(1_000_000)),
        &palette,
        &profile,
    );
    let mut draw = |area, input, unknown, protected: &[Rect]| {
        let mut excluded = design::base(selection, area, &context, &mut buffer);
        excluded.extend_from_slice(protected);
        designs.render(
            selection,
            &context,
            input,
            unknown,
            area,
            &excluded,
            &mut buffer,
        );
    };
    match fixture {
        "table-status" => {
            for (i, input) in [25, 65, 100].into_iter().enumerate() {
                draw(Rect::new(36, 4 + i as u16 * 3, 36, 3), input, false, &[]);
            }
            buffer.set_string(
                3,
                2,
                "Operation       Host progress     Sample / fixed-width cell",
                fg,
            );
            for (i, (label, status)) in [
                ("Index docs", "25%"),
                ("Build search", "65%"),
                ("Export report", "100%"),
            ]
            .into_iter()
            .enumerate()
            {
                buffer.set_string(3, 5 + i as u16 * 3, label, fg);
                buffer.set_string(19, 5 + i as u16 * 3, status, fg);
            }
        }
        "request-result" => {
            draw(Rect::new(4, 6, 64, 10), 65, false, &[]);
            buffer.set_string(
                3,
                2,
                "REQUEST #0041 / generation 2 / semantic status: RUNNING",
                fg,
            );
            buffer.set_string(3, 4, selection.input_label(65, false), fg);
            buffer.set_string(
                3,
                17,
                "Host controls: edit | cancel | replace operation",
                fg,
            );
        }
        "dashboard" => {
            draw(Rect::new(32, 5, 64, 22), 65, false, &[]);
            buffer.set_string(
                3,
                2,
                "METRICS / caller level 0.65 / sample data fixed for this capture",
                fg,
            );
            buffer.set_string(3, 6, "Host samples: 0.65", fg);
            buffer.set_string(3, 8, "Producer rate: 1 Hz", fg);
            buffer.set_string(3, 10, "Independent UI cadence", fg);
            buffer.set_string(
                3,
                28,
                "A larger bounded widget uses the same sample API; no scene engine",
                fg,
            );
        }
        "reader" => {
            let protected = [
                Rect::new(5, 6, 22, 1),
                Rect::new(5, 8, 1, 1),
                Rect::new(5, 11, 18, 4),
            ];
            draw(Rect::new(30, 5, 46, 14), 65, false, &protected);
            buffer.set_string(
                3,
                2,
                "READER / protected text, selection, caret, reserved media",
                fg,
            );
            buffer.set_string(5, 6, "Selected host text 界", fg.bg(Color::Rgb(28, 48, 68)));
            buffer.set_string(5, 8, "| editable caret", fg);
            Block::bordered()
                .title(" MEDIA ")
                .style(fg)
                .render(Rect::new(5, 11, 18, 4), &mut buffer);
            buffer.set_string(
                3,
                20,
                "The host repaints its base before decoration; no accumulated trails",
                fg,
            );
        }
        _ => unreachable!(),
    }
    buffer.set_string(
        3,
        height - 2,
        "Synthetic fixture. Visual completion never changes business state.",
        fg,
    );
    Ok(buffer)
}
fn render_design(selection: Selection, fixture: &str) -> Result<()> {
    if ![
        "isolated",
        "table-status",
        "request-result",
        "dashboard",
        "reader",
    ]
    .contains(&fixture)
    {
        return Err(format!("unknown fixture {fixture}").into());
    }
    fs::create_dir_all("previews")?;
    let designs = Designs::new(7);
    let period = designs.duration_ms(selection);
    let (kind, duration) = match selection {
        Selection::Embers => ("observation-window", 8000),
        Selection::Prism => ("finite-replay", period.unwrap() + 600),
        _ => ("loop", period.unwrap()),
    };
    let size = selection.preferred();
    let mut frames = Vec::new();
    for time_ms in (0..duration).step_by(40) {
        let buffer = if fixture == "isolated" {
            sampled_frame(selection, time_ms, size, Profile::default(), 65, false)
        } else {
            embedded_frame(selection, time_ms, fixture)?
        };
        frames.push(json!({"time_ms":time_ms,"input":{"value":0.65,"unknown":false},"cells":cells(&buffer)}));
    }
    let boundary_times: Vec<u64> = period.map_or_else(
        || vec![0, 1, 7999, 8000, 8001],
        |p| vec![0, 1, p.saturating_sub(1), p, p + 1],
    );
    let boundaries: Vec<_> = boundary_times.iter().map(|t| json!({"time_ms":t,"cells":cells(&sampled_frame(selection,*t,size,Profile::default(),65,false))})).collect();
    let embeddings = ["table-status", "request-result", "dashboard", "reader"]
        .into_iter()
        .map(|name| {
            Ok((
                name.to_owned(),
                cells(&embedded_frame(selection, 720, name)?),
            ))
        })
        .collect::<Result<serde_json::Map<String, Value>>>()?;
    let manifest: Value = serde_json::from_slice(&fs::read(format!(
        "catalog/{}/manifest.json",
        selection.id()
    ))?)?;
    let minimum = ratatui_core::layout::Size::new(
        manifest["geometry"]["minimum_full"][0]
            .as_u64()
            .ok_or("invalid minimum width")? as u16,
        manifest["geometry"]["minimum_full"][1]
            .as_u64()
            .ok_or("invalid minimum height")? as u16,
    );
    let fixture_geometry = match fixture {
        "isolated" => json!([[0, 0, size.width, size.height]]),
        "table-status" => json!([[36, 4, 36, 3], [36, 7, 36, 3], [36, 10, 36, 3]]),
        "request-result" => json!([[4, 6, 64, 10]]),
        "dashboard" => json!([[32, 5, 64, 22]]),
        "reader" => json!([[30, 5, 46, 14]]),
        _ => unreachable!(),
    };
    let output = json!({"schema":2,"design":selection.id(),"title":selection.title(),"reference":format!("R{:02}",selection.number()),"visual_revision":1,"source_sha256":source_hash()?,"seed":if matches!(selection,Selection::Mascot|Selection::Prism){Value::Null}else{json!(7)},"period_ms":if kind=="loop"{json!(period)}else{Value::Null},"duration_ms":duration,"finite_duration_ms":if selection==Selection::Prism{json!(period)}else{Value::Null},"playback_kind":kind,"palette":"default","profile":"unicode-truecolor-full","fixture":fixture,"isolated_requested_size":[size.width,size.height],"requested_areas":fixture_geometry,"frame_duration_ms":40,"input_trace":{"kind":"constant caller-supplied example","value":0.65,"unknown":false},"capture":"exact Ratatui cells from consumer Rust implementation; offscreen, not a terminal recording","frames":frames,"boundaries":boundaries,"embedded":embeddings["request-result"],"embeddings":embeddings,"fallback":cells(&sampled_frame(selection,0,size,Profile::quiet(),65,false)),"smallest":cells(&sampled_frame(selection,0,ratatui_core::layout::Size::new(3,1),Profile::quiet(),65,false)),"minimum_full":cells(&sampled_frame(selection,720,minimum,Profile::default(),65,false)),"unknown_progress":if selection==Selection::Assembly{cells(&sampled_frame(selection,720,size,Profile::default(),65,true))}else{Value::Null},"limitations":["No cross-target cell equality claim","Fixed-font derivatives do not qualify terminal emulators","Telemetry capture is an observation window, not a seamless global loop","Finite reveal recordings replay a completed one-shot"]});
    let path = format!("previews/{}.cells.json", selection.id());
    fs::write(&path, serde_json::to_string_pretty(&output)? + "\n")?;
    println!("Wrote actual Rust styled cells: {path} ({kind}, {fixture})");
    Ok(())
}
fn record(selection: Selection) -> Result<()> {
    let status = Command::new("python3")
        .args(["scripts/render_previews.py", "--design", selection.id()])
        .status()?;
    if !status.success() {
        return Err("fixed-font renderer failed; exact Rust cells remain available".into());
    }
    println!(
        "Recorded offscreen fixed-font sequence for {}; this is not terminal-emulator evidence",
        selection.id()
    );
    Ok(())
}
fn check_command(program: &str, args: &[&str]) -> Value {
    let start = Instant::now();
    match Command::new(program).args(args).output() {
        Ok(output) => {
            json!({"command":std::iter::once(program).chain(args.iter().copied()).collect::<Vec<_>>(),"status":if output.status.success(){"passed"}else{"failed"},"exit_code":output.status.code(),"duration_ms":start.elapsed().as_millis(),"stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr)})
        }
        Err(error) => {
            json!({"command":program,"args":args,"status":"unavailable","error":error.to_string()})
        }
    }
}
fn qualify(selected: Vec<Selection>) -> Result<()> {
    fs::create_dir_all("qualification/local")?;
    let local_checks = vec![
        check_command(
            "cargo",
            &[
                "test",
                "-p",
                "spark-runtime",
                "-p",
                "spark-catalog",
                "--all-targets",
                "--locked",
            ],
        ),
        check_command(
            "cargo",
            &["test", "-p", "spark-gallery", "-p", "xtask", "--locked"],
        ),
        check_command("cargo", &["test", "--workspace", "--doc", "--locked"]),
        check_command(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        check_command("cargo", &["fmt", "--all", "--", "--check"]),
        check_command(
            "cargo",
            &[
                "run",
                "-p",
                "spark-catalog",
                "--example",
                "sync_host",
                "--locked",
                "--",
                "--smoke",
            ],
        ),
        check_command(
            "cargo",
            &[
                "run",
                "-p",
                "spark-catalog",
                "--example",
                "async_host",
                "--locked",
                "--",
                "--smoke",
            ],
        ),
        check_command(
            "cargo",
            &[
                "run",
                "-p",
                "spark-catalog",
                "--example",
                "paced_host",
                "--locked",
                "--",
                "--smoke",
            ],
        ),
        check_command(
            "cargo",
            &["run", "-p", "spark-gallery", "--locked", "--", "--smoke"],
        ),
    ];
    let checks_passed = local_checks.iter().all(|check| check["status"] == "passed");
    let snapshots_ok = snapshots(true).is_ok();
    let assets_ok = assets::generate(true).is_ok();
    let manifests_ok = validate_manifests().is_ok();
    let omissions = [
        "Independent visual review of all six designs is not implied by test success",
        "macOS and Windows build/test/terminal runs not performed by this command",
        "Named terminal/font, tmux and remote-session qualification not performed",
        "WASM compilation and cross-target sample equality not measured here",
        "Allocation counts, memory growth and emulator CPU not measured here",
        "Package/license publication is not authorized by this report",
    ];
    let mut all_passed = checks_passed && snapshots_ok && assets_ok && manifests_ok;
    for selection in selected {
        render_design(selection, "isolated")?;
        let reproducible = deterministic_samples(selection);
        let passed = checks_passed && snapshots_ok && assets_ok && manifests_ok && reproducible;
        all_passed &= passed;
        let report = json!({"schema":1,"design":selection.id(),"source_sha256":source_hash()?,"scope":"Local exact-cell and host checks; not full release qualification","status":if passed{"local-checks-passed-with-unmeasured-gates"}else{"failed"},"target":{"architecture":env::consts::ARCH,"os":env::consts::OS},"checks":local_checks,"existing_r01_goldens":if snapshots_ok{"passed"}else{"failed"},"foundation_six_exact_fixture":if snapshots_ok{"passed"}else{"failed"},"generated_asset_drift":if assets_ok{"passed"}else{"failed"},"manifest_validation":if manifests_ok{"passed"}else{"failed"},"repeated_and_shuffled_sample_equality":if reproducible{"passed"}else{"failed"},"capture":format!("previews/{}.cells.json",selection.id()),"unmeasured":omissions});
        fs::write(
            format!("qualification/local/{}.json", selection.id()),
            serde_json::to_string_pretty(&report)? + "\n",
        )?;
        let summary = format!(
            "# {}: local qualification\n\nStatus: {}\n\nSource SHA-256: `{}`\n\nExact capture: `previews/{}.cells.json`\n\n## Unmeasured gates\n{}\n",
            selection.title(),
            report["status"].as_str().unwrap(),
            report["source_sha256"].as_str().unwrap(),
            selection.id(),
            omissions
                .iter()
                .map(|item| format!("- {item}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        fs::write(
            format!("qualification/local/{}.md", selection.id()),
            summary,
        )?;
        println!("{}: {}", selection.id(), report["status"].as_str().unwrap());
    }
    if !all_passed {
        return Err("one or more local checks failed; inspect qualification/local reports".into());
    }
    Ok(())
}
fn deterministic_samples(selection: Selection) -> bool {
    for profile in [Profile::default(), Profile::quiet()] {
        for size in [
            ratatui_core::layout::Size::new(0, 0),
            ratatui_core::layout::Size::new(1, 1),
            selection.preferred(),
        ] {
            let timestamps = [
                0,
                1,
                720,
                1199,
                1200,
                1599,
                1600,
                2399,
                2400,
                8000,
                u64::MAX,
            ];
            let expected: Vec<_> = timestamps
                .iter()
                .map(|t| sampled_frame(selection, *t, size, profile, 65, false))
                .collect();
            for index in [10, 3, 1, 5, 9, 0, 8, 2, 6, 4, 7, 3, 0] {
                if sampled_frame(selection, timestamps[index], size, profile, 65, false)
                    != expected[index]
                {
                    return false;
                }
            }
        }
    }
    true
}
fn package_audit() -> Result<()> {
    fs::create_dir_all("qualification/local")?;
    let checks = vec![
        check_command(
            "cargo",
            &["metadata", "--locked", "--no-deps", "--format-version", "1"],
        ),
        check_command(
            "cargo",
            &[
                "tree",
                "-p",
                "spark-runtime",
                "--edges",
                "normal",
                "--locked",
            ],
        ),
        check_command(
            "cargo",
            &[
                "tree",
                "-p",
                "spark-catalog",
                "--edges",
                "normal",
                "--locked",
            ],
        ),
        check_command(
            "cargo",
            &[
                "package",
                "-p",
                "spark-runtime",
                "--list",
                "--allow-dirty",
                "--offline",
            ],
        ),
        check_command(
            "cargo",
            &[
                "package",
                "-p",
                "spark-catalog",
                "--list",
                "--allow-dirty",
                "--offline",
            ],
        ),
    ];
    let mut consumers = Vec::new();
    for (name, manifest, binary) in [
        (
            "one-design",
            "fixtures/consumer/Cargo.toml",
            "fixtures/consumer/target/release/spark-external-consumer",
        ),
        (
            "all-six",
            "fixtures/consumer-all/Cargo.toml",
            "fixtures/consumer-all/target/release/spark-all-design-consumer",
        ),
    ] {
        let build = check_command(
            "cargo",
            &[
                "build",
                "--manifest-path",
                manifest,
                "--release",
                "--locked",
            ],
        );
        let run = if build["status"] == "passed" {
            check_command(binary, &[])
        } else {
            json!({"status":"not-run","reason":"build failed"})
        };
        let dependencies = check_command(
            "cargo",
            &[
                "tree",
                "--manifest-path",
                manifest,
                "--edges",
                "normal",
                "--locked",
            ],
        );
        let dependency_text = dependencies["stdout"].as_str().unwrap_or("");
        let excluded = [
            "spark-gallery",
            "xtask",
            "tokio",
            "serde_json",
            "sha2",
            "crossterm",
        ]
        .into_iter()
        .filter(|name| dependency_text.contains(name))
        .collect::<Vec<_>>();
        let symbols = check_command("nm", &["-C", binary]);
        let catalog_symbols = symbols["stdout"]
            .as_str()
            .unwrap_or("")
            .lines()
            .filter(|line| line.contains("spark_catalog::"))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mascot_symbols = catalog_symbols
            .iter()
            .filter(|line| line.contains("mascot"))
            .cloned()
            .collect::<Vec<_>>();
        let metadata = fs::metadata(binary).ok();
        consumers.push(json!({"name":name,"manifest":manifest,"settings":"cargo build --release --locked; each standalone manifest's default release settings; dependencies may be warm","build":build,"run":run,"normal_dependencies":dependencies,"unwanted_dependency_matches":excluded,"binary":binary,"bytes":metadata.map(|v| v.len()),"binary_sha256":fs::read(binary).ok().map(|bytes| format!("{:x}",Sha256::digest(bytes))),"symbol_tool_status":symbols["status"],"catalog_symbols":catalog_symbols,"mascot_symbol_matches":mascot_symbols,"asset_elimination_conclusion":"Symbol and size observations only; const/inlined/anonymous data may not have recognizable symbols"}));
    }
    let passed = checks.iter().all(|c| c["status"] == "passed")
        && consumers.iter().all(|c| {
            c["build"]["status"] == "passed"
                && c["run"]["status"] == "passed"
                && c["normal_dependencies"]["status"] == "passed"
                && c["unwanted_dependency_matches"]
                    .as_array()
                    .is_some_and(Vec::is_empty)
        });
    let report = json!({"schema":2,"source_sha256":source_hash()?,"scope":"Local dependencies, package contents, downstream execution and measured release binary comparison; no publication","status":if passed{"passed-with-limitations"}else{"failed"},"checks":checks,"consumers":consumers,"limitations":["Package listing does not publish or select a license","Build wall times may include cached dependencies; not a controlled cold-build benchmark","Exact fixture binaries compared: the one-design host also contains the original public-API TravelingMarker extension; the all-six host has a different layout. Their size delta is not an ideal stripped catalogue-only marginal cost","Symbol absence cannot prove elimination of anonymous/constant asset bytes","No remote registry-name or release-readiness claim"]});
    fs::write(
        "qualification/local/package-audit.json",
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    println!(
        "Package audit: {}; qualification/local/package-audit.json",
        report["status"].as_str().unwrap()
    );
    for consumer in &consumers {
        println!(
            "{} release binary: {} bytes; build {} ms",
            consumer["name"], consumer["bytes"], consumer["build"]["duration_ms"]
        );
    }
    if !passed {
        return Err("package audit command failed; inspect the report".into());
    }
    Ok(())
}

fn foundation_baseline() -> Result<()> {
    let designs = Designs::new(7);
    let palette = Palette::default();
    let scenarios = [
        "no-art",
        "one-comet",
        "one-large-embers",
        "mixed-six",
        "grid-24",
        "static-six",
        "hidden-six",
        "completed-prism",
    ];
    let mut results = Vec::new();
    for scenario in scenarios {
        let area = if scenario == "one-comet" {
            Rect::new(0, 0, 32, 3)
        } else {
            Rect::new(0, 0, 120, 42)
        };
        let placements: Vec<(Selection, Rect)> = match scenario {
            "one-comet" => vec![(Selection::Comet, area)],
            "one-large-embers" => vec![(Selection::Embers, Rect::new(0, 0, 120, 40))],
            "completed-prism" => vec![(Selection::Prism, Rect::new(2, 2, 32, 3))],
            "mixed-six" | "static-six" => Selection::ALL
                .into_iter()
                .enumerate()
                .map(|(i, d)| {
                    (
                        d,
                        Rect::new((i % 3) as u16 * 40, (i / 3) as u16 * 21, 40, 20),
                    )
                })
                .collect(),
            "grid-24" => (0..24)
                .map(|i| {
                    (
                        Selection::ALL[i % 6],
                        Rect::new((i % 4) as u16 * 30, (i / 4) as u16 * 7, 30, 7),
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
        let profile = Profile {
            motion: if scenario == "static-six" {
                Motion::Static
            } else {
                Motion::Full
            },
            ..Profile::default()
        };
        let mut buffer = Buffer::empty(area);
        let mut previous = Buffer::empty(area);
        let mut prepare_times = Vec::new();
        let mut paint_times = Vec::new();
        let mut diff_times = Vec::new();
        let mut encode_times = Vec::new();
        let mut changed_total = 0usize;
        let mut bytes_total = 0usize;
        let mut encoded = Vec::new();
        let mut quiet_hints = true;
        for index in 0..1200u64 {
            let clip = if scenario == "completed-prism" {
                ClipTime::MAX
            } else {
                ClipTime::from_nanos(index * 33_000_000)
            };
            let context = SampleContext::new(clip, &palette, &profile);
            let start = Instant::now();
            buffer.reset();
            let exclusions: Vec<_> = placements
                .iter()
                .map(|(design, rect)| design::base(*design, *rect, &context, &mut buffer))
                .collect();
            let preparation = start.elapsed().as_nanos();
            let start = Instant::now();
            for ((design, rect), protected) in placements.iter().zip(&exclusions) {
                designs.render(
                    *design,
                    &context,
                    65,
                    false,
                    *rect,
                    protected,
                    black_box(&mut buffer),
                );
            }
            let painting = start.elapsed().as_nanos();
            let start = Instant::now();
            let difference = previous.diff(black_box(&buffer));
            let difference_ns = start.elapsed().as_nanos();
            let changed = difference.len();
            encoded.clear();
            let start = Instant::now();
            {
                let mut backend = CrosstermBackend::new(&mut encoded);
                backend.draw(difference.into_iter())?;
                backend.flush()?;
            }
            let encoding = start.elapsed().as_nanos();
            if index >= 200 {
                prepare_times.push(preparation);
                paint_times.push(painting);
                diff_times.push(difference_ns);
                encode_times.push(encoding);
                changed_total += changed;
                bytes_total += encoded.len();
            }
            if matches!(scenario, "static-six" | "hidden-six" | "completed-prism") {
                quiet_hints &= placements.iter().all(|(design, rect)| {
                    designs
                        .interval(*design, &context, 65, false, rect.as_size())
                        .is_none()
                });
            }
            std::mem::swap(&mut previous, &mut buffer);
        }
        let summary = |mut samples: Vec<u128>| {
            samples.sort_unstable();
            json!({"samples":samples.len(),"p50_ns":percentile(&samples,50),"p95_ns":percentile(&samples,95),"p99_ns":percentile(&samples,99)})
        };
        results.push(json!({"scenario":scenario,"buffer_size":[area.width,area.height],"visible_instances":placements.len(),"requested_areas":placements.iter().map(|(d,r)| json!({"design":d.id(),"area":[r.x,r.y,r.width,r.height]})).collect::<Vec<_>>(),"host_base_preparation":summary(prepare_times),"warmed_sample_and_paint":summary(paint_times),"buffer_diff":summary(diff_times),"backend_encoding_to_memory":summary(encode_times),"mean_changed_cells":changed_total as f64/1000.0,"mean_backend_bytes":bytes_total as f64/1000.0,"quiet_cases_have_no_animation_advice":if matches!(scenario,"static-six"|"hidden-six"|"completed-prism"){json!(quiet_hints)}else{Value::Null}}));
    }
    let report = json!({"schema":1,"source_sha256":source_hash()?,"scope":"Six-design local performance observations; not universal budgets","build":if cfg!(debug_assertions){"debug"}else{"release; workspace thin LTO, one codegen unit"},"target":{"arch":env::consts::ARCH,"os":env::consts::OS},"rustc":check_command("rustc",&["-Vv"]),"hardware":check_command("uname",&["-a"]),"cpu":fs::read_to_string("/proc/cpuinfo").ok().and_then(|s|s.lines().find(|l|l.starts_with("model name")).map(str::to_owned)),"configuration":{"seed":7,"input":0.65,"time_step_ms":33,"warmup_samples":200,"measured_samples":1000,"base":"freshly reset buffer; decorator bases prepared outside paint timer","output":"Crossterm encoding into Vec<u8>, no terminal IO"},"scenarios":results,"unmeasured":["Cold construction/preparation of all designs is not isolated here","Allocator calls, allocation growth, process CPU and terminal emulator CPU","Native emulator rendering, macOS/Windows performance, tmux and remote sessions","Shared virtual hardware; no universal latency guarantee", "stats_alloc 0.1.10 global allocator instrumentation is enabled in the measured tooling binary","Hidden case intentionally skips all samples; host wakeup traces are in separate tests"]});
    fs::write(
        "qualification/local/foundation-baseline.json",
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    for result in results {
        println!(
            "{}: paint p95={} ns; diff p95={} ns; mean changed={} / bytes={}",
            result["scenario"],
            result["warmed_sample_and_paint"]["p95_ns"],
            result["buffer_diff"]["p95_ns"],
            result["mean_changed_cells"],
            result["mean_backend_bytes"]
        );
    }
    Ok(())
}

fn allocation_value(stats: stats_alloc::Stats) -> Value {
    json!({"allocations":stats.allocations,"deallocations":stats.deallocations,"reallocations":stats.reallocations,"bytes_allocated":stats.bytes_allocated,"bytes_deallocated":stats.bytes_deallocated,"bytes_reallocated":stats.bytes_reallocated,"net_live_bytes_delta":stats.bytes_allocated as i128-stats.bytes_deallocated as i128})
}
fn allocation_baseline() -> Result<()> {
    let palette = Palette::default();
    let profile = Profile::default();
    let designs = Designs::new(7);
    let mut reports = Vec::new();
    for selection in Selection::ALL {
        let size = selection.preferred();
        let area = Rect::new(0, 0, size.width, size.height);
        let mut buffer = Buffer::empty(area);
        let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
        let exclusions = design::base(selection, area, &context, &mut buffer);
        // Warm all cell representations before measuring. No formatting, result
        // storage, cloning or buffer/base creation occurs inside these regions.
        for i in 0..200u64 {
            let context =
                SampleContext::new(ClipTime::from_nanos(i * 33_000_000), &palette, &profile);
            designs.render(
                selection,
                &context,
                65,
                false,
                area,
                &exclusions,
                &mut buffer,
            );
        }
        let mut counts = Vec::new();
        for samples in [1000u64, 100_000] {
            let region = stats_alloc::Region::new(ALLOCATOR);
            for i in 0..samples {
                let context =
                    SampleContext::new(ClipTime::from_nanos(i * 33_000_000), &palette, &profile);
                designs.render(
                    selection,
                    black_box(&context),
                    65,
                    false,
                    area,
                    &exclusions,
                    black_box(&mut buffer),
                );
            }
            let stats = region.change();
            counts.push(json!({"samples":samples,"totals":allocation_value(stats)}));
        }
        reports.push(json!({"design":selection.id(),"requested_size":[size.width,size.height],"scope":"Warmed direct sample and paint only. Same preallocated buffer; base/protection/configuration created outside the region. Allocation observation, not a fresh-base visual or output benchmark.","runs":counts}));
    }
    let area = Rect::new(0, 0, 32, 3);
    let mut left = Buffer::empty(area);
    let mut right = Buffer::empty(area);
    let context = SampleContext::new(ClipTime::ZERO, &palette, &profile);
    let comet = SegmentedComet::new();
    comet.sample(&context).render(area, &mut left);
    let later = SampleContext::new(ClipTime::from_nanos(400_000_000), &palette, &profile);
    comet.sample(&later).render(area, &mut right);
    let measure = |mut work: Box<dyn FnMut()>, samples: u32| {
        for _ in 0..10 {
            work();
        }
        let region = stats_alloc::Region::new(ALLOCATOR);
        for _ in 0..samples {
            work();
        }
        let stats = region.change();
        allocation_value(stats)
    };
    let buffer_stats = measure(
        Box::new(|| {
            drop(black_box(Buffer::empty(area)));
        }),
        1000,
    );
    let diff_stats = measure(
        Box::new(|| {
            drop(black_box(left.diff(&right)));
        }),
        1000,
    );
    let inline_stats = measure(
        Box::new(|| {
            drop(black_box(comet.sample(&later).inline(32)));
        }),
        1000,
    );
    let mut base = Buffer::empty(Rect::new(0, 0, 40, 9));
    let base_stats = measure(
        Box::new(|| {
            drop(black_box(design::base(
                Selection::Border,
                Rect::new(0, 0, 40, 9),
                &context,
                &mut base,
            )));
        }),
        1000,
    );
    let difference = left.diff(&right);
    let mut bytes = Vec::with_capacity(4096);
    let backend_stats = measure(
        Box::new(|| {
            bytes.clear();
            let mut backend = CrosstermBackend::new(&mut bytes);
            backend
                .draw(difference.iter().copied())
                .expect("memory encoding");
            backend.flush().expect("memory flush");
            black_box(&bytes);
        }),
        1000,
    );
    let report = json!({"schema":1,"source_sha256":source_hash()?,"target":{"arch":env::consts::ARCH,"os":env::consts::OS},"build":if cfg!(debug_assertions){"debug"}else{"release; thin LTO, one codegen unit"},"allocator":"stats_alloc 0.1.10 over System; safe external instrumented wrapper; single-threaded measured regions","paint":reports,"separate_host_and_adapter_stages":{"samples_each":1000,"buffer_creation_32x3":buffer_stats,"host_border_base_40x9":base_stats,"buffer_diff_32x3":diff_stats,"backend_encoding_preallocated_memory":backend_stats,"owned_comet_inline_32_columns":inline_stats},"limitations":["Measured allocation requests, not process RSS, fragmentation, or allocator-reserved memory","Existing buffer and configuration memory is excluded from warmed paint counts","Zero observed allocation in this matrix is not a universal zero-allocation guarantee","Long run observes100000 direct samples and net live-byte delta; it is not an endurance test of a real terminal host","Instrumentation changes timing; final latency reports use the same instrumented binary and disclose that"]});
    fs::write(
        "qualification/local/allocations.json",
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    for design in reports {
        println!(
            "{} allocations (100000 paints): {}",
            design["design"], design["runs"][1]["totals"]
        );
    }
    println!("Inline32 allocation totals /1000 samples: {inline_stats}");
    Ok(())
}

#[cfg(test)]
mod tooling_tests {
    use super::*;
    #[test]
    fn all_six_tool_samples_are_reproducible_and_fixtures_keep_labels() {
        for selection in Selection::ALL {
            assert!(deterministic_samples(selection), "{}", selection.id());
            for fixture in ["table-status", "request-result", "dashboard", "reader"] {
                let first = embedded_frame(selection, 720, fixture).unwrap();
                assert_eq!(first, embedded_frame(selection, 720, fixture).unwrap());
                let content: String = first.content.iter().map(|cell| cell.symbol()).collect();
                assert!(content.contains("HOST CONTENT"));
                assert!(content.contains("Visual completion never changes business state"));
            }
        }
    }
    #[test]
    fn captures_do_not_treat_nonloops_as_periods() {
        let designs = Designs::new(7);
        assert_eq!(designs.duration_ms(Selection::Embers), None);
        assert_eq!(designs.duration_ms(Selection::Prism), Some(1200));
        assert_eq!(designs.duration_ms(Selection::Mascot), Some(2400));
        assert!(embedded_frame(Selection::Comet, 0, "made-up").is_err());
    }
}
