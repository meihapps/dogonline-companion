use crate::bravery::{BraveryData, Enemy};
use crate::formatting::format_enum;
use crate::long_press::LongPress;
use crate::storage::{self, Filter};
use leptos::prelude::*;

/// Flips `killed` for the enemy at `index` and saves the result to browser storage.
fn toggle_killed(data: RwSignal<BraveryData>, index: usize) {
    data.update(|data| {
        if let Some(enemy) = data.enemies.get_mut(index) {
            enemy.killed = !enemy.killed;
        }
    });

    storage::save_killed_enemies(&data.with_untracked(BraveryData::killed_names));
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn BraveryTab(data: RwSignal<BraveryData>) -> impl IntoView {
    let (selected_enemy, set_selected_enemy) = signal(None::<Enemy>);
    let (show_killed, set_show_killed) =
        signal(storage::load_filter_bool(Filter::ShowKilled, true));

    Effect::new(move |_| {
        storage::save_filter_bool(Filter::ShowKilled, show_killed.get());
    });

    view! {
        <section class="tab-content">
            <h2>"Bravery"</h2>

            <p class="hint">"Long-press (or right-click) an enemy to mark it as killed"</p>

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || show_killed.get()
                    on:change=move |ev| {
                        set_show_killed.set(event_target_checked(&ev));
                    }
                />
                " Show killed"
            </label>

            <div class="item-grid">
                {move || {
                    let show_killed = show_killed.get();

                    data.get()
                        .enemies
                        .into_iter()
                        // Enumerate before filtering so each card keeps its index in the full list.
                        .enumerate()
                        .filter(|(_, enemy)| show_killed || !enemy.killed)
                        .map(|(index, enemy)| {
                    let enemy_for_click = enemy.clone();
                    let long_press = LongPress::new(move || toggle_killed(data, index));

                    view! {
                        <button
                            class="item-card"
                            on:click=move |_| {
                                if LongPress::take_fired() {
                                    return;
                                }
                                set_selected_enemy.set(Some(enemy_for_click.clone()));
                            }
                            on:pointerdown={let lp = long_press.clone(); move |ev| lp.pointer_down(&ev)}
                            on:pointerup={let lp = long_press.clone(); move |_| lp.cancel()}
                            on:pointerleave={let lp = long_press.clone(); move |_| lp.cancel()}
                            on:pointercancel={let lp = long_press.clone(); move |_| lp.cancel()}
                            on:contextmenu={let lp = long_press.clone(); move |ev| lp.context_menu(&ev)}
                        >
                            <div class="item-icon-wrap">
                                <div
                                    class="item-icon-placeholder"
                                    class:caught=enemy.killed
                                ></div>
                            </div>

                            <p class="item-name">
                                {enemy.name.clone()}
                            </p>

                            <p class="item-value">
                                {format!("+{} bravery", enemy.bravery)}
                            </p>
                        </button>
                    }
                        })
                        .collect_view()
                }}
            </div>

            {move || selected_enemy.get().map(|enemy| {
                view! {
                    <div
                        class="modal-backdrop"
                        on:click=move |_| {
                            set_selected_enemy.set(None);
                        }
                    >
                        <div
                            class="enemy-modal"
                            on:click=|ev| ev.stop_propagation()
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
                                        <strong>"Type: "</strong>
                                        {format_enum(&enemy.damage_type)}
                                    </p>

                                    <p>
                                        <strong>"HP: "</strong>
                                        {enemy.hp}
                                    </p>

                                    <p>
                                        <strong>"Bravery: "</strong>
                                        {format!("+{}", enemy.bravery)}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}
        </section>
    }
}
