//! Touch-friendly alternative to right-click: hold a card for half a second to trigger an action.
//! Mouse users keep using right-click.

use leptos::ev::{MouseEvent, PointerEvent};
use leptos::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

const HOLD_TIME: Duration = Duration::from_millis(500);

thread_local! {
    /// Set when a hold completes, so the click that follows the release is ignored.
    /// Global rather than per-card: the hold toggles progress, which re-renders the grid and
    /// replaces the card, so the click can arrive at a brand-new card with fresh state.
    static LONG_PRESS_FIRED: Cell<bool> = const { Cell::new(false) };
}

#[derive(Default)]
struct State {
    timer: Cell<Option<TimeoutHandle>>,
    /// Whether the latest press was touch/pen rather than mouse.
    touch: Cell<bool>,
}

#[derive(Clone)]
pub struct LongPress {
    state: Rc<State>,
    action: Rc<dyn Fn()>,
}

impl LongPress {
    pub fn new(action: impl Fn() + 'static) -> Self {
        Self {
            state: Rc::new(State::default()),
            action: Rc::new(action),
        }
    }

    /// Hook up to `on:pointerdown`.
    pub fn pointer_down(&self, ev: PointerEvent) {
        self.cancel();
        LONG_PRESS_FIRED.set(false);

        let touch = ev.pointer_type() != "mouse";
        self.state.touch.set(touch);

        // Mouse users use right-click instead.
        if !touch {
            return;
        }

        let state = self.state.clone();
        let action = self.action.clone();

        if let Ok(handle) = set_timeout_with_handle(
            move || {
                state.timer.set(None);
                LONG_PRESS_FIRED.set(true);
                action();
            },
            HOLD_TIME,
        ) {
            self.state.timer.set(Some(handle));
        }
    }

    /// Hook up to `on:pointerup`, `on:pointerleave` and `on:pointercancel`
    /// (the browser cancels the press when the user starts scrolling).
    pub fn cancel(&self) {
        if let Some(handle) = self.state.timer.take() {
            handle.clear();
        }
    }

    /// Hook up to `on:contextmenu`. Always hides the browser menu; right-click triggers the
    /// action, while a touch long-press is handled by the timer instead.
    pub fn context_menu(&self, ev: MouseEvent) {
        ev.prevent_default();

        if !self.state.touch.get() {
            (self.action)();
        }
    }

    /// Use at the top of `on:click`: returns true if the click is just the end of a long press.
    pub fn take_fired(&self) -> bool {
        LONG_PRESS_FIRED.replace(false)
    }
}
