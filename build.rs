// Generates the JSON data bundled into the app from the `generate_*_data` functions.
// The data types also carry app-only methods, which go unused here.

#[allow(dead_code)]
#[path = "src/bravery.rs"]
mod bravery;
#[allow(dead_code)]
#[path = "src/coordination.rs"]
mod coordination;
#[path = "src/generate_bravery_data.rs"]
mod generate_bravery_data;
#[path = "src/generate_coordination_data.rs"]
mod generate_coordination_data;
#[path = "src/generate_hardiness_data.rs"]
mod generate_hardiness_data;
#[allow(dead_code)]
#[path = "src/hardiness.rs"]
mod hardiness;

use serde::Serialize;

fn write_json(file_name: &str, data: &impl Serialize) {
    let path = format!("{}/{file_name}", std::env::var("OUT_DIR").unwrap());

    let json = serde_json::to_string_pretty(data)
        .unwrap_or_else(|err| panic!("failed to serialise {file_name}: {err}"));

    std::fs::write(&path, json).unwrap_or_else(|err| panic!("failed to write {path}: {err}"));
}

fn main() {
    for file in [
        "bravery",
        "coordination",
        "hardiness",
        "generate_bravery_data",
        "generate_coordination_data",
        "generate_hardiness_data",
    ] {
        println!("cargo::rerun-if-changed=src/{file}.rs");
    }

    write_json(
        "bravery_data.json",
        &generate_bravery_data::generate_bravery_data(),
    );
    write_json(
        "coordination_data.json",
        &generate_coordination_data::generate_coordination_data(),
    );
    write_json(
        "hardiness_data.json",
        &generate_hardiness_data::generate_hardiness_data(),
    );
}
