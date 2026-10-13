mod app;
mod bravery;
mod bravery_tab;
mod coordination;
mod coordination_tab;
mod formatting;
mod generate_bravery_data;
mod generate_coordination_data;
mod storage;

use app::App;
use bravery::BraveryData;
use coordination::CoordinationData;
use leptos::prelude::*;

const BRAVERY_DATA: &str = include_str!("../assets/bravery_data.json");
const COORDINATION_DATA: &str = include_str!("../assets/coordination_data.json");

fn main() {
    // Immutable data comes from the bundled JSON...
    let mut bravery_data =
        serde_json::from_str::<BraveryData>(BRAVERY_DATA).expect("invalid bravery_data.json");
    let mut coordination_data = serde_json::from_str::<CoordinationData>(COORDINATION_DATA)
        .expect("invalid coordination_data.json");

    // ...and the mutable progress comes from browser storage.
    bravery_data.apply_progress(&storage::load_killed_enemies());
    coordination_data.apply_progress(&storage::load_caught_fish());

    mount_to_body(move || {
        view! {
            <App
                bravery_data=bravery_data
                coordination_data=coordination_data
            />
        }
    });
}
