use crate::generate_bravery_data::generate_bravery_data;
use crate::generate_coordination_data::generate_coordination_data;
use crate::generate_hardiness_data::generate_hardiness_data;
use serde::Serialize;

fn write_json(file_name: &str, data: &impl Serialize) {
    let path = format!("{}/assets/{file_name}", env!("CARGO_MANIFEST_DIR"));

    let json = serde_json::to_string_pretty(data)
        .unwrap_or_else(|err| panic!("failed to serialise {file_name}: {err}"));

    std::fs::write(&path, json).unwrap_or_else(|err| panic!("failed to write {path}: {err}"));

    println!("wrote {path}");
}

/// Rewrites every JSON file in `assets/` from the `generate_*_data` functions, then exits.
///
/// Only works in a native build, so run it with `cargo run`, not `trunk serve`:
/// add `regenerate_json_data();` at the top of `main`, `cargo run`, then remove the call.
/// It exits when done so the browser-only app code below it never runs.
#[allow(dead_code)]
pub fn regenerate_json_data() {
    write_json("bravery_data.json", &generate_bravery_data());
    write_json("coordination_data.json", &generate_coordination_data());
    write_json("hardiness_data.json", &generate_hardiness_data());

    println!("JSON data regenerated; run `trunk serve` to use it.");
    std::process::exit(0);
}
