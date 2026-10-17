//! Tooling-only complete-frame schema and deterministic Rust emission.
//! Source parsing is bounded separately from expansion; libraries never parse JSON.
use serde_json::{Map, Value};
use std::{
    collections::BTreeSet,
    error::Error,
    fmt::Write as _,
    fs::{self, File},
    io::Read as _,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Central schema-v1 resource ceilings. The runtime independently validates timing.
#[derive(Clone, Copy)]
struct Limits {
    source_bytes: usize,
    width: u64,
    height: u64,
    frames: usize,
    total_cells: u64,
    glyphs: usize,
    cells: usize,
    total_nanos: u64,
}
const LIMITS: Limits = Limits {
    source_bytes: 1_048_576,
    width: 128,
    height: 64,
    frames: 256,
    total_cells: 262_144,
    glyphs: 94,
    cells: 95,
    total_nanos: 86_400_000_000_000,
};
const SOURCE: &str = "catalog/pocket-mascot/frames.json";
const OUTPUT: &str = "crates/catalog/src/generated/mascot.rs";
const ROLES: [&str; 4] = ["foreground", "muted", "accent", "highlight"];

pub(crate) fn generate(check: bool) -> Result<()> {
    // `take` prevents a replaced or unexpectedly large input from causing an
    // unbounded read before the byte-size guard can run.
    let mut bytes = Vec::new();
    File::open(SOURCE)?
        .take(LIMITS.source_bytes as u64 + 1)
        .read_to_end(&mut bytes)?;
    let generated = compile(&bytes)?;
    if check {
        if fs::read_to_string(OUTPUT)? != generated {
            return Err(
                "authored asset drift; run cargo xtask generate-assets and review the diff".into(),
            );
        }
        println!("PASS: bounded schema-v1 asset and deterministic static Rust data");
    } else {
        fs::create_dir_all(Path::new(OUTPUT).parent().unwrap())?;
        fs::write(OUTPUT, generated)?;
        println!("Wrote {OUTPUT}; visual changes require independent review");
    }
    Ok(())
}

fn object<'a>(value: &'a Value, expected: &[&str], label: &str) -> Result<&'a Map<String, Value>> {
    let map = value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))?;
    if map.len() != expected.len() || !expected.iter().all(|name| map.contains_key(*name)) {
        return Err(format!("{label} has missing or unknown fields; expected {expected:?}").into());
    }
    Ok(map)
}
fn positive(value: &Value, limit: u64, label: &str) -> Result<u64> {
    let number = value
        .as_u64()
        .ok_or_else(|| format!("{label} must be an unsigned integer"))?;
    if number == 0 || number > limit {
        return Err(format!("{label} must be in 1..={limit}").into());
    }
    Ok(number)
}
fn index(value: &Value, count: usize, label: &str) -> Result<usize> {
    let number = value
        .as_u64()
        .ok_or_else(|| format!("{label} must be an unsigned index"))?;
    if number >= count as u64 {
        return Err(format!("{label} reference is out of range").into());
    }
    Ok(number as usize)
}
fn bounded_array<'a>(value: &'a Value, limit: usize, label: &str) -> Result<&'a [Value]> {
    let values = value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?;
    if values.is_empty() || values.len() > limit {
        return Err(format!("{label} count must be in 1..={limit}").into());
    }
    Ok(values)
}
fn printable(value: &Value, allow_space: bool, label: &str) -> Result<u8> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{label} must be a string"))?;
    let bytes = text.as_bytes();
    if bytes.len() != 1 || !(if allow_space { b' ' } else { b'!' }..=b'~').contains(&bytes[0]) {
        return Err(format!("{label} must be one printable ASCII cell").into());
    }
    Ok(bytes[0])
}
fn name(value: &Value, label: &str) -> Result<()> {
    let name = value
        .as_str()
        .ok_or_else(|| format!("{label} must be a string"))?;
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(
            format!("{label} must be 1–64 lowercase ASCII letters, digits, or hyphens").into(),
        );
    }
    Ok(())
}

/// Preflight every bound and reference before expanding a single frame cell.
fn validate(value: &Value) -> Result<()> {
    object(
        value,
        &[
            "schema",
            "design",
            "visual_revision",
            "width",
            "height",
            "poster",
            "glyphs",
            "cells",
            "frames",
        ],
        "asset",
    )?;
    if value["schema"].as_u64() != Some(1) {
        return Err("unsupported asset schema; expected version 1".into());
    }
    name(&value["design"], "design")?;
    // This command owns exactly this generated asset, rather than a registry.
    if value["design"].as_str() != Some("pocket-mascot") {
        return Err("this generator expects pocket-mascot".into());
    }
    positive(
        &value["visual_revision"],
        u32::MAX as u64,
        "visual_revision",
    )?;
    let width = positive(&value["width"], LIMITS.width, "width")?;
    let height = positive(&value["height"], LIMITS.height, "height")?;
    let frames = bounded_array(&value["frames"], LIMITS.frames, "frames")?;
    let cells_per_frame = width.checked_mul(height).ok_or("cell dimension overflow")?;
    let total_cells = cells_per_frame
        .checked_mul(frames.len() as u64)
        .ok_or("total cell count overflow")?;
    if total_cells > LIMITS.total_cells {
        return Err("total expanded cells exceed the central limit".into());
    }
    index(&value["poster"], frames.len(), "poster")?;
    let glyphs = bounded_array(&value["glyphs"], LIMITS.glyphs, "glyphs")?;
    let mut glyph_set = BTreeSet::new();
    for glyph in glyphs {
        if !glyph_set.insert(printable(glyph, false, "glyph")?) {
            return Err("duplicate glyph".into());
        }
    }
    let cells = bounded_array(&value["cells"], LIMITS.cells, "cells")?;
    let mut keys = BTreeSet::new();
    for cell in cells {
        let fields: &[&str] = match cell["op"].as_str() {
            Some("keep") => &["key", "op"],
            Some("erase") => &["key", "op", "role"],
            Some("replace") => &["key", "op", "glyph", "role"],
            _ => return Err("cell op must be keep, erase, or replace".into()),
        };
        object(cell, fields, "cell")?;
        if !keys.insert(printable(&cell["key"], true, "cell key")?) {
            return Err("duplicate cell key".into());
        }
        if cell["op"] != "keep"
            && !cell["role"]
                .as_str()
                .is_some_and(|role| ROLES.contains(&role))
        {
            return Err("unknown palette role".into());
        }
        if cell["op"] == "replace" {
            index(&cell["glyph"], glyphs.len(), "glyph")?;
        }
    }
    let mut total_nanos = 0u64;
    let mut names = BTreeSet::new();
    for frame in frames {
        object(frame, &["name", "hold_ms", "rows"], "frame")?;
        name(&frame["name"], "frame name")?;
        if !names.insert(frame["name"].as_str().unwrap()) {
            return Err("duplicate frame name".into());
        }
        let hold_ms = positive(&frame["hold_ms"], u64::MAX, "frame hold_ms")?;
        let hold_ns = hold_ms
            .checked_mul(1_000_000)
            .ok_or("frame duration overflow")?;
        total_nanos = total_nanos
            .checked_add(hold_ns)
            .ok_or("cumulative frame duration overflow")?;
        if total_nanos > LIMITS.total_nanos {
            return Err("cumulative duration exceeds the central limit".into());
        }
        let rows = frame["rows"]
            .as_array()
            .ok_or("frame rows must be an array")?;
        if rows.len() as u64 != height {
            return Err("frame row count does not match height".into());
        }
        for row in rows {
            let row = row.as_str().ok_or("frame row must be a string")?;
            if !row.is_ascii() || row.len() as u64 != width {
                return Err("frame row must contain exactly width ASCII cell keys".into());
            }
            if !row.bytes().all(|key| keys.contains(&key)) {
                return Err("frame references an undefined cell key".into());
            }
        }
    }
    Ok(())
}

fn compile(source: &[u8]) -> Result<String> {
    if source.len() > LIMITS.source_bytes {
        return Err("asset source exceeds the central byte limit".into());
    }
    // serde_json's default nesting-depth limit remains enabled.
    let asset: Value = serde_json::from_slice(source)?;
    validate(&asset)?;
    // From here on all allocations/loops are bounded by the validated limits.
    let width = asset["width"].as_u64().unwrap();
    let height = asset["height"].as_u64().unwrap();
    let frames = asset["frames"].as_array().unwrap();
    let cells = asset["cells"].as_array().unwrap();
    let mut output = format!(
        "// @generated by cargo xtask generate-assets; do not edit.\n// Schema 1; pocket-mascot visual revision {}.\nuse super::{{AssetCell, Role}};\n\npub(super) const WIDTH: u16 = {width};\npub(super) const HEIGHT: u16 = {height};\npub(super) const POSTER: usize = {};\n\n",
        asset["visual_revision"], asset["poster"]
    );
    let mut keys = [0usize; 128];
    for (i, cell) in cells.iter().enumerate() {
        keys[cell["key"].as_str().unwrap().as_bytes()[0] as usize] = i;
        let role = match cell["role"].as_str() {
            Some("foreground") => "Role::Foreground",
            Some("muted") => "Role::Muted",
            Some("accent") => "Role::Accent",
            Some("highlight") => "Role::Highlight",
            _ => "", // Keep has no palette reference.
        };
        let operation = match cell["op"].as_str().unwrap() {
            "keep" => "AssetCell::Keep".to_owned(),
            "erase" => format!("AssetCell::Erase({role})"),
            "replace" => {
                let glyph = asset["glyphs"][cell["glyph"].as_u64().unwrap() as usize]
                    .as_str()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap();
                format!("AssetCell::Replace({glyph:?}, {role})")
            }
            _ => unreachable!("validated operation"),
        };
        writeln!(&mut output, "const C{i}: AssetCell = {operation};")?;
    }
    writeln!(
        &mut output,
        "\n#[rustfmt::skip]\npub(super) const ENDS_NS: [u64; {}] = [",
        frames.len()
    )?;
    let mut end = 0u64;
    for frame in frames {
        end += frame["hold_ms"].as_u64().unwrap() * 1_000_000;
        writeln!(&mut output, "    {end},")?;
    }
    writeln!(
        &mut output,
        "];\n\n#[rustfmt::skip]\npub(super) static FRAMES: [[AssetCell; {}]; {}] = [",
        width * height,
        frames.len()
    )?;
    for frame in frames {
        writeln!(
            &mut output,
            "    // {}: {} ms",
            frame["name"].as_str().unwrap(),
            frame["hold_ms"]
        )?;
        output.push_str("    [\n");
        for row in frame["rows"].as_array().unwrap() {
            output.push_str("        ");
            for key in row.as_str().unwrap().bytes() {
                write!(&mut output, "C{}, ", keys[key as usize])?;
            }
            // Do not create trailing spaces; exact output is stable under rustfmt.
            output.pop();
            output.push('\n');
        }
        output.push_str("    ],\n");
    }
    output.push_str("];\n");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid() -> Value {
        json!({"schema":1,"design":"pocket-mascot","visual_revision":1,
            "width":2,"height":1,"poster":0,"glyphs":["o"],
            "cells":[{"key":".","op":"keep"},{"key":" ","op":"erase","role":"muted"},
                {"key":"o","op":"replace","glyph":0,"role":"highlight"}],
            "frames":[{"name":"idle","hold_ms":800,"rows":["o."]},
                {"name":"blink","hold_ms":100,"rows":[" o"]}]})
    }
    fn check(value: &Value) -> Result<String> {
        compile(&serde_json::to_vec(value).unwrap())
    }
    fn rejects(edit: impl FnOnce(&mut Value)) {
        let mut value = valid();
        edit(&mut value);
        assert!(check(&value).is_err(), "accepted invalid source: {value}");
    }

    #[test]
    fn deterministic_complete_frames_and_unequal_holds() {
        let a = check(&valid()).unwrap();
        assert_eq!(a, check(&valid()).unwrap());
        assert!(a.contains("800000000,\n    900000000,"));
        assert!(a.contains("AssetCell::Keep"));
        assert!(a.contains("AssetCell::Erase(Role::Muted)"));
        assert!(a.contains("AssetCell::Replace('o', Role::Highlight)"));
        assert!(a.contains("C2, C0,"));
        assert!(a.contains("C1, C2,"));
    }

    #[test]
    fn rejects_source_and_structural_failures() {
        assert!(compile(&vec![b' '; LIMITS.source_bytes + 1]).is_err());
        for bytes in [b"".as_slice(), b"{", b"null", b"[]", b"{\"schema\":1}"] {
            assert!(compile(bytes).is_err());
        }
        rejects(|v| v["schema"] = json!(2));
        rejects(|v| v["extra"] = json!(0));
        rejects(|v| v["frames"][0]["extra"] = json!(0));
        rejects(|v| v["design"] = json!("not-this-asset"));
        rejects(|v| v["visual_revision"] = json!(0));
        rejects(|v| v["visual_revision"] = json!(u64::MAX));
        rejects(|v| v["poster"] = json!(2));
        rejects(|v| v["frames"][1]["name"] = json!("idle"));
        rejects(|v| v["frames"][0]["name"] = json!("unsafe\nname"));
    }

    #[test]
    fn rejects_bounds_before_expansion() {
        for value in [json!(0), json!(-1), json!(1.5), json!(u64::MAX)] {
            rejects(|v| v["width"] = value.clone());
            rejects(|v| v["height"] = value);
        }
        rejects(|v| v["width"] = json!(LIMITS.width + 1));
        rejects(|v| v["height"] = json!(LIMITS.height + 1));
        rejects(|v| v["frames"] = json!([]));
        rejects(|v| v["frames"] = Value::Array(vec![v["frames"][0].clone(); LIMITS.frames + 1]));
        rejects(|v| {
            v["width"] = json!(LIMITS.width);
            v["height"] = json!(LIMITS.height);
            v["frames"] = Value::Array(vec![v["frames"][0].clone(); LIMITS.frames]);
        });
        rejects(|v| v["glyphs"] = Value::Array(vec![json!("o"); LIMITS.glyphs + 1]));
        rejects(|v| v["cells"] = Value::Array(vec![v["cells"][0].clone(); LIMITS.cells + 1]));
    }

    #[test]
    fn rejects_invalid_glyphs_palette_and_coverage() {
        for glyph in [
            "", "ab", " ", "\n", "\u{1b}", "\u{7f}", "界", "é", "\u{301}",
        ] {
            rejects(|v| v["glyphs"][0] = json!(glyph));
        }
        rejects(|v| v["glyphs"] = json!(["o", "o"]));
        rejects(|v| v["glyphs"] = json!([]));
        rejects(|v| v["cells"] = json!([]));
        rejects(|v| v["cells"][2]["glyph"] = json!(1));
        rejects(|v| v["cells"][2]["glyph"] = json!(-1));
        rejects(|v| v["cells"][2]["role"] = json!("background"));
        rejects(|v| v["cells"][0]["role"] = json!("muted"));
        rejects(|v| v["cells"][2]["op"] = json!("transparent"));
        rejects(|v| v["cells"][2]["key"] = json!("."));
        rejects(|v| v["cells"][2]["key"] = json!("界"));
        rejects(|v| v["frames"][0]["rows"] = json!([]));
        rejects(|v| v["frames"][0]["rows"] = json!(["o.", "o."]));
        rejects(|v| v["frames"][0]["rows"] = json!(["o"]));
        rejects(|v| v["frames"][0]["rows"] = json!(["oo."]));
        rejects(|v| v["frames"][0]["rows"] = json!(["界."]));
        rejects(|v| v["frames"][0]["rows"] = json!(["x."]));
        rejects(|v| v["frames"][0]["rows"] = json!([12]));
    }

    #[test]
    fn rejects_zero_fractional_and_overflowing_holds() {
        for duration in [json!(0), json!(-1), json!(0.5), json!(u64::MAX)] {
            rejects(|v| v["frames"][0]["hold_ms"] = duration);
        }
        rejects(|v| v["frames"][0]["hold_ms"] = json!(LIMITS.total_nanos / 1_000_000));
        let mut maximum = valid();
        maximum["frames"] =
            json!([{"name":"long","hold_ms":LIMITS.total_nanos/1_000_000,"rows":["o."]}]);
        assert!(check(&maximum).is_ok());
        maximum["frames"][0]["hold_ms"] = json!(LIMITS.total_nanos / 1_000_000 + 1);
        assert!(check(&maximum).is_err());
    }

    #[test]
    fn malformed_byte_mutations_never_panic() {
        let bytes = serde_json::to_vec(&valid()).unwrap();
        for i in 0..bytes.len() {
            for byte in [0, b'\n', b'"', b'[', b'}', 255] {
                let mut changed = bytes.clone();
                changed[i] = byte;
                let _ = compile(&changed);
            }
        }
        for end in 0..bytes.len() {
            assert!(compile(&bytes[..end]).is_err());
        }
    }
}
