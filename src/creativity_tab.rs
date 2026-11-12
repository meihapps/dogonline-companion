use crate::creativity::CreativityData;
use crate::formatting::image_path;
use crate::long_press::LongPress;
use crate::progress_counts::{Progress, ProgressCounts};
use crate::storage::{self, Filter};
use leptos::prelude::*;

/// Flips `read` for the book at `index` and saves the result to browser storage.
fn toggle_read(data: RwSignal<CreativityData>, index: usize) {
    data.update(|data| {
        if let Some(book) = data.books.get_mut(index) {
            book.read = !book.read;
        }
    });

    storage::save_read_books(&data.with_untracked(CreativityData::read_names));
}

#[component]
pub fn CreativityTab(data: RwSignal<CreativityData>) -> impl IntoView {
    let (show_read, set_show_read) = signal(storage::load_filter_bool(Filter::ShowRead, true));

    // Remember the filter between visits.
    Effect::new(move |_| {
        storage::save_filter_bool(Filter::ShowRead, show_read.get());
    });

    let progress = Signal::derive(move || {
        data.with(|data| Progress::of(data.books.iter().map(|book| (book.read, book.creativity))))
    });

    view! {
        <section class="tab-content">
            <h2>"Creativity"</h2>

            <p class="hint">"Long-press (or right-click) a book to mark it as read"</p>

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || show_read.get()
                    on:change=move |ev| {
                        set_show_read.set(event_target_checked(&ev));
                    }
                />
                " Show read"
            </label>

            <ProgressCounts
                progress=progress
                show_done=show_read
                items="Books"
                done_label="read"
                stat="Creativity"
            />

            <div class="item-grid">
                {move || {
                    let show_read = show_read.get();

                    data.get()
                        .books
                        .into_iter()
                        // Enumerate before filtering so each card keeps its index in the full list.
                        .enumerate()
                        .filter(|(_, book)| show_read || !book.read)
                        .map(|(index, book)| {
                            let long_press = LongPress::new(move || toggle_read(data, index));

                            view! {
                                <div
                                    class="item-card"
                                    on:pointerdown={let lp = long_press.clone(); move |ev| lp.pointer_down(&ev)}
                                    on:pointerup={let lp = long_press.clone(); move |_| lp.cancel()}
                                    on:pointerleave={let lp = long_press.clone(); move |_| lp.cancel()}
                                    on:pointercancel={let lp = long_press.clone(); move |_| lp.cancel()}
                                    on:contextmenu={let lp = long_press.clone(); move |ev| lp.context_menu(&ev)}
                                >
                                    <div class="item-icon-wrap">
                                        <img
                                            class="item-icon"
                                            class:complete=book.read
                                            src=image_path("creativity", &book.name)
                                            alt=""
                                        />
                                    </div>

                                    <p class="item-name">{book.name.clone()}</p>

                                    <p class="item-value">
                                        {format!("+{} creativity", book.creativity)}
                                    </p>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </section>
    }
}
