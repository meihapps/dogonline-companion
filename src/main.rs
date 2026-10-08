mod bravery;
mod generate_bravery_data;

use bravery::{BraveryData, Enemy};
use leptos::prelude::*;

const BRAVERY_DATA: &str = include_str!("../assets/bravery_data.json");

fn main() {
    let bravery_data: BraveryData =
        serde_json::from_str(BRAVERY_DATA).expect("invalid bravery_data.json");

    mount_to_body(move || {
        view! {
            <App bravery_data=bravery_data />
        }
    });
}

#[component]
fn App(bravery_data: BraveryData) -> impl IntoView {
    let (selected_enemy, set_selected_enemy) = signal(None::<Enemy>);

    view! {
        <main class="page">
            <section class="card">
                <div class="content">
                    <header class="header">
                        <h1>"bravery"</h1>
                    </header>

                    <div class="enemy-grid">
                        {bravery_data
                            .enemies
                            .into_iter()
                            .map(|enemy| {
                                let enemy_for_click = enemy.clone();

                                view! {
                                    <button
                                        class="enemy-card"
                                        on:click=move |_| {
                                            set_selected_enemy.set(
                                                Some(enemy_for_click.clone())
                                            );
                                        }
                                    >
                                        <div class="enemy-icon-wrap">
                                            <div class="enemy-icon-placeholder"></div>
                                        </div>

                                        <p class="enemy-name">
                                            {enemy.name.clone()}
                                        </p>
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </section>

            {move || {
                selected_enemy.get().map(|enemy| {
                    view! {
                        <div
                            class="modal-backdrop"
                            on:click=move |_| {
                                set_selected_enemy.set(None);
                            }
                        >
                            <div
                                class="enemy-modal"
                                on:click=|ev| {
                                    ev.stop_propagation();
                                }
                            >
                                <button
                                    class="modal-close"
                                    aria-label="Close"
                                    on:click=move |_| {
                                        set_selected_enemy.set(None);
                                    }
                                >
                                    "×"
                                </button>

                                <div class="modal-body">
                                    <div class="modal-image"></div>

                                    <h2>{enemy.name.clone()}</h2>

                                    <div class="enemy-stats">
                                        <p>
                                            <strong>"Type:"</strong>
                                            " "
                                            {format!("{:?}", enemy.damage_type)}
                                        </p>

                                        <p>
                                            <strong>"HP:"</strong>
                                            " "
                                            {enemy.hp}
                                        </p>

                                        <p>
                                            <strong>"Bravery:"</strong>
                                            " +"
                                            {enemy.bravery}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                })
            }}
        </main>
    }
}
