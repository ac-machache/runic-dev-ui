// sheet.rs — Rust/UI Sheet, adapted to be signal-driven (open/close via an
// `RwSignal<bool>` in context) instead of the shipped inline-<script> + global
// `window.ScrollLock`, which doesn't execute reliably under Leptos CSR.

use icons::X;
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::*;

use super::button::ButtonSize;
use crate::components::ui::button::{Button, ButtonVariant};

mod components {
    use super::*;
    clx! {SheetHeader, div, "flex flex-col gap-0.5 p-4"}
    clx! {SheetTitle, h2, "font-bold text-2xl"}
    clx! {SheetDescription, p, "text-muted-foreground"}
    clx! {SheetBody, div, "flex flex-col gap-4"}
    clx! {SheetFooter, footer, "mt-auto flex flex-col gap-2 p-4"}
}

pub use components::*;

/* ========================================================== */
/*                     ✨ CONTEXT ✨                          */
/* ========================================================== */

#[derive(Clone, Copy)]
pub struct SheetContext {
    pub open: RwSignal<bool>,
}

/* ========================================================== */
/*                     ✨ FUNCTIONS ✨                        */
/* ========================================================== */

pub type SheetVariant = ButtonVariant;
pub type SheetSize = ButtonSize;

#[component]
pub fn Sheet(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let open = RwSignal::new(false);
    let merged_class = tw_merge!("", class);

    view! {
        <Provider value=SheetContext { open }>
            <div data-name="Sheet" class=merged_class>
                {children()}
            </div>
        </Provider>
    }
}

#[component]
pub fn SheetTrigger(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = ButtonVariant::Outline)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Default)] size: ButtonSize,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();

    view! {
        <Button class=class on:click=move |_| ctx.open.set(true) variant=variant size=size>
            {children()}
        </Button>
    }
}

#[component]
pub fn SheetClose(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = ButtonVariant::Outline)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Default)] size: ButtonSize,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();

    view! {
        <Button
            class=class
            attr:aria-label="Close sheet"
            on:click=move |_| ctx.open.set(false)
            variant=variant
            size=size
        >
            {children()}
        </Button>
    }
}

#[component]
pub fn SheetContent(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = SheetDirection::Right)] direction: SheetDirection,
    #[prop(default = true)] show_close_button: bool,
) -> impl IntoView {
    let ctx = expect_context::<SheetContext>();
    let open = ctx.open;

    // Reactive panel class: slide in (translate-*-0) when open, else off-screen.
    let panel_class = move || {
        tw_merge!(
            "fixed z-[100] bg-card shadow-lg p-6 transition-transform duration-300 overflow-y-auto overscroll-y-contain",
            direction.initial_position(),
            if open.get() { "translate-x-0 translate-y-0 pointer-events-auto" } else { direction.closed_class() },
            if open.get() { "" } else { "pointer-events-none" },
            &class
        )
    };

    view! {
        // backdrop
        <div
            data-name="SheetBackdrop"
            class=move || tw_merge!(
                "fixed inset-0 z-[60] bg-black/50 transition-opacity duration-200",
                if open.get() { "opacity-100 pointer-events-auto" } else { "opacity-0 pointer-events-none" }
            )
            on:click=move |_| open.set(false)
        />

        // panel
        <div data-name="SheetContent" class=panel_class>
            {show_close_button
                .then(|| {
                    view! {
                        <button
                            type="button"
                            class="absolute top-4 right-4 p-1 rounded-sm focus:ring-2 focus:ring-offset-2 focus:outline-none [&_svg:not([class*='size-'])]:size-4 focus:ring-ring"
                            aria-label="Close sheet"
                            on:click=move |_| open.set(false)
                        >
                            <span class="hidden">"Close Sheet"</span>
                            <X />
                        </button>
                    }
                })}
            {children()}
        </div>
    }
}

/* ========================================================== */
/*                     ✨ ENUM ✨                             */
/* ========================================================== */

#[derive(Clone, Copy, strum::AsRefStr, strum::Display)]
pub enum SheetDirection {
    Right,
    Left,
    Top,
    Bottom,
}

impl SheetDirection {
    fn closed_class(self) -> &'static str {
        match self {
            SheetDirection::Right => "translate-x-full",
            SheetDirection::Left => "-translate-x-full",
            SheetDirection::Top => "-translate-y-full",
            SheetDirection::Bottom => "translate-y-full",
        }
    }

    fn initial_position(self) -> &'static str {
        match self {
            SheetDirection::Right => "top-0 right-0 h-full w-[400px]",
            SheetDirection::Left => "top-0 left-0 h-full w-[400px]",
            SheetDirection::Top => "top-0 left-0 w-full h-[400px]",
            SheetDirection::Bottom => "bottom-0 left-0 w-full h-[400px]",
        }
    }
}
