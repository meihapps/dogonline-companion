use crate::bravery::BraveryData;
use crate::bravery_tab::BraveryTab;
use crate::coordination::CoordinationData;
use crate::coordination_tab::CoordinationTab;
use crate::dailies_tab::{self, DailiesTab};
use crate::hardiness::HardinessData;
use crate::hardiness_tab::HardinessTab;
use crate::storage::{self, Theme};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Bravery,
    Coordination,
    Hardiness,
    Dailies,
}

#[component]
pub fn App(
    bravery_data: BraveryData,
    coordination_data: CoordinationData,
    hardiness_data: HardinessData,
) -> impl IntoView {
    // The data lives here (not in the tabs) so toggled progress survives switching tabs.
    let bravery_data = RwSignal::new(bravery_data);
    let coordination_data = RwSignal::new(coordination_data);
    let hardiness_data = RwSignal::new(hardiness_data);
    let dailies = RwSignal::new(storage::load_dailies(&dailies_tab::reset_day()));
    dailies_tab::start_reset_timer(dailies);
    let (selected_tab, set_selected_tab) = signal(Tab::Bravery);
    let theme = RwSignal::new(storage::load_theme());

    // Mirror the theme onto <html data-theme="..."> so the CSS can override color-scheme.
    Effect::new(move |_| {
        let theme = theme.get();
        if let Some(root) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
        {
            let _ = root.set_attribute("data-theme", theme.as_str());
        }
    });

    view! {
        <main class="page">
            <section class="card">
                <div class="content">
                    <header class="header">
                        <h1>"dogonline companion"</h1>
                        <div class="header-actions">
                            <div class="theme-toggle" role="group" aria-label="Colour theme">
                                <button
                                    class="theme-button"
                                    class:active=move || theme.get() == Theme::Device
                                    aria-pressed=move || theme.get() == Theme::Device
                                    aria-label="Use device theme"
                                    title="Use device theme"
                                    on:click=move |_| {
                                        theme.set(Theme::Device);
                                        storage::save_theme(Theme::Device);
                                    }
                                >
                                    <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="2" y="2.5" width="12" height="8" rx="1"/><path d="M6 14h4M8 10.5V14"/></svg>
                                </button>
                                <button
                                    class="theme-button"
                                    class:active=move || theme.get() == Theme::Light
                                    aria-pressed=move || theme.get() == Theme::Light
                                    aria-label="Light theme"
                                    title="Light theme"
                                    on:click=move |_| {
                                        theme.set(Theme::Light);
                                        storage::save_theme(Theme::Light);
                                    }
                                >
                                    <svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="2.8"/><path d="M8 1.5v1.7M8 12.8v1.7M1.5 8h1.7M12.8 8h1.7M3.4 3.4l1.2 1.2M11.4 11.4l1.2 1.2M12.6 3.4l-1.2 1.2M4.6 11.4l-1.2 1.2"/></svg>
                                </button>
                                <button
                                    class="theme-button"
                                    class:active=move || theme.get() == Theme::Dark
                                    aria-pressed=move || theme.get() == Theme::Dark
                                    aria-label="Dark theme"
                                    title="Dark theme"
                                    on:click=move |_| {
                                        theme.set(Theme::Dark);
                                        storage::save_theme(Theme::Dark);
                                    }
                                >
                                    <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M13.5 9.6A5.8 5.8 0 0 1 6.4 2.5a5.8 5.8 0 1 0 7.1 7.1z"/></svg>
                                </button>
                            </div>
                            <span class="header-divider" aria-hidden="true"></span>
                            <a
                                href="https://github.com/meihapps/dogonline-companion"
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                "GitHub"
                            </a>
                        </div>
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

                        <button
                            class="tab-button"
                            class:active=move || {
                                selected_tab.get() == Tab::Hardiness
                            }
                            aria-pressed=move || {
                                selected_tab.get() == Tab::Hardiness
                            }
                            on:click=move |_| {
                                set_selected_tab.set(Tab::Hardiness);
                            }
                        >
                            "Hardiness"
                        </button>

                        <button
                            class="tab-button"
                            class:active=move || {
                                selected_tab.get() == Tab::Dailies
                            }
                            aria-pressed=move || {
                                selected_tab.get() == Tab::Dailies
                            }
                            on:click=move |_| {
                                set_selected_tab.set(Tab::Dailies);
                            }
                        >
                            "Dailies"
                        </button>
                    </nav>

                    {move || match selected_tab.get() {
                        Tab::Bravery => view! {
                            <BraveryTab data=bravery_data />
                        }.into_any(),

                        Tab::Coordination => view! {
                            <CoordinationTab data=coordination_data />
                        }.into_any(),

                        Tab::Hardiness => view! {
                            <HardinessTab data=hardiness_data />
                        }.into_any(),

                        Tab::Dailies => view! {
                            <DailiesTab done=dailies />
                        }.into_any(),
                    }}
                </div>
            </section>
        </main>
    }
}
