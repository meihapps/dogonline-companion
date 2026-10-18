mod app;
mod bravery;
mod bravery_tab;
mod coordination;
mod coordination_tab;
mod dailies_tab;
mod formatting;
mod generate_bravery_data;
mod generate_coordination_data;
mod generate_hardiness_data;
mod hardiness;
mod hardiness_tab;
mod long_press;
mod regenerate;
mod storage;

use app::App;
use bravery::BraveryData;
use coordination::CoordinationData;
use hardiness::HardinessData;
use leptos::prelude::*;

const BRAVERY_DATA: &str = include_str!("../assets/bravery_data.json");
const COORDINATION_DATA: &str = include_str!("../assets/coordination_data.json");
const HARDINESS_DATA: &str = include_str!("../assets/hardiness_data.json");

fn main() {
    // To regenerate the JSON data: uncomment, `cargo run`, then comment it out again.
    // regenerate::regenerate_json_data();

    // Immutable data comes from the bundled JSON...
    let mut bravery_data =
        serde_json::from_str::<BraveryData>(BRAVERY_DATA).expect("invalid bravery_data.json");
    let mut coordination_data = serde_json::from_str::<CoordinationData>(COORDINATION_DATA)
        .expect("invalid coordination_data.json");

    let mut hardiness_data =
        serde_json::from_str::<HardinessData>(HARDINESS_DATA).expect("invalid hardiness_data.json");

    // ...and the mutable progress comes from browser storage.
    bravery_data.apply_progress(&storage::load_killed_enemies());
    coordination_data.apply_progress(&storage::load_caught_fish());
    hardiness_data.apply_progress(&storage::load_harvested_items());

    mount_to_body(move || {
        view! {
            <App
                bravery_data=bravery_data
                coordination_data=coordination_data
                hardiness_data=hardiness_data
            />
        }
    });
}
