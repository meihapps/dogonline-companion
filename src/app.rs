use crate::bravery::BraveryData;
use crate::bravery_tab::BraveryTab;
use crate::coordination::CoordinationData;
use crate::coordination_tab::CoordinationTab;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Bravery,
    Coordination,
}

#[component]
pub fn App(bravery_data: BraveryData, coordination_data: CoordinationData) -> impl IntoView {
    // The data lives here (not in the tabs) so toggled progress survives switching tabs.
    let bravery_data = RwSignal::new(bravery_data);
    let coordination_data = RwSignal::new(coordination_data);
    let (selected_tab, set_selected_tab) = signal(Tab::Bravery);

    view! {
        <main class="page">
            <section class="card">
                <div class="content">
                    <header class="header">
                        <h1>"dogonline companion"</h1>
                    </header>

                    <nav class="tabs" aria-label="Tracker tabs">
                        <button
                            class="tab-button"
                            class:active=move || {
                                selected_tab.get() == Tab::Bravery
                            }
                            aria-pressed=move || {
                                selected_tab.get() == Tab::Bravery
                            }
                            on:click=move |_| {
                                set_selected_tab.set(Tab::Bravery);
                            }
                        >
                            "Bravery"
                        </button>

                        <button
                            class="tab-button"
                            class:active=move || {
                                selected_tab.get() == Tab::Coordination
                            }
                            aria-pressed=move || {
                                selected_tab.get() == Tab::Coordination
                            }
                            on:click=move |_| {
                                set_selected_tab.set(Tab::Coordination);
                            }
                        >
                            "Coordination"
                        </button>
                    </nav>

                    {move || match selected_tab.get() {
                        Tab::Bravery => view! {
                            <BraveryTab data=bravery_data />
                        }.into_any(),

                        Tab::Coordination => view! {
                            <CoordinationTab data=coordination_data />
                        }.into_any(),
                    }}
                </div>
            </section>
        </main>
    }
}
