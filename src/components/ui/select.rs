// select.rs — Rust/UI Select, adapted to be signal-driven (open/close via an
// `RwSignal<bool>` in context) instead of the shipped inline-<script> +
// `window.ScrollLock`, which doesn't execute reliably under Leptos CSR.

use icons::{Check, ChevronDown, ChevronUp};
use leptos::context::Provider;
use leptos::prelude::*;
use leptos_ui::clx;
use strum::{AsRefStr, Display};
use tw_merge::*;

use crate::components::hooks::use_can_scroll_vertical::use_can_scroll_vertical;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, AsRefStr)]
pub enum SelectPosition {
    #[default]
    Below,
    Above,
}

mod components {
    use super::*;
    clx! {SelectLabel, span, "px-2 py-1.5 text-sm font-medium", "mb-1"}
    clx! {SelectItem, li, "inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm transition-colors text-popover-foreground hover:bg-accent hover:text-accent-foreground"}
}

pub use components::*;

#[derive(Clone, Copy)]
struct SelectContext {
    open: RwSignal<bool>,
    value_signal: RwSignal<Option<String>>,
    on_change: Option<Callback<Option<String>>>,
}

#[component]
pub fn Select(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] default_value: Option<String>,
    #[prop(optional)] on_change: Option<Callback<Option<String>>>,
) -> impl IntoView {
    let ctx = SelectContext {
        open: RwSignal::new(false),
        value_signal: RwSignal::new(default_value),
        on_change,
    };
    let merged_class = tw_merge!("relative w-fit", class);

    view! {
        <Provider value=ctx>
            <div data-name="Select" class=merged_class>
                {children()}
            </div>
        </Provider>
    }
}

#[component]
pub fn SelectTrigger(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let button_class = tw_merge!(
        "w-full p-2 h-9 inline-flex items-center justify-between text-sm font-medium whitespace-nowrap rounded-md transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4 border bg-background border-input hover:bg-accent hover:text-accent-foreground",
        class
    );

    view! {
        <button
            type="button"
            data-name="SelectTrigger"
            class=button_class
            tabindex="0"
            on:click=move |_| ctx.open.update(|o| *o = !*o)
        >
            {children()}
            <ChevronDown class="text-muted-foreground" />
        </button>
    }
}

#[component]
pub fn SelectValue(#[prop(optional, into)] placeholder: String) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    view! {
        <span data-name="SelectValue" class="text-sm text-foreground truncate">
            {move || ctx.value_signal.get().unwrap_or_else(|| placeholder.clone())}
        </span>
    }
}

#[component]
pub fn SelectGroup(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = "Select options".into(), into)] aria_label: String,
) -> impl IntoView {
    let merged_class = tw_merge!("group", class);
    view! {
        <ul data-name="SelectGroup" role="listbox" aria-label=aria_label class=merged_class>
            {children()}
        </ul>
    }
}

#[component]
pub fn SelectOption(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = false.into(), into)] aria_selected: Signal<bool>,
    #[prop(optional, into)] value: Option<String>,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let merged_class = tw_merge!(
        "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm cursor-pointer transition-colors text-popover-foreground hover:bg-accent hover:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
        class
    );
    let value_for_check = value.clone();
    let is_selected = move || aria_selected.get() || ctx.value_signal.get() == value_for_check;

    view! {
        <li
            data-name="SelectOption"
            class=merged_class
            role="option"
            tabindex="0"
            aria-selected=move || is_selected().to_string()
            on:click=move |_| {
                let val = value.clone();
                ctx.value_signal.set(val.clone());
                if let Some(on_change) = ctx.on_change {
                    on_change.run(val);
                }
                ctx.open.set(false);
            }
        >
            {children()}
            <Check class="ml-auto opacity-0 size-4 text-muted-foreground group-aria-selected:opacity-100" />
        </li>
    }
}

#[component]
pub fn SelectContent(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(default = SelectPosition::default())] position: SelectPosition,
) -> impl IntoView {
    let ctx = expect_context::<SelectContext>();
    let open = ctx.open;
    let (on_scroll, can_scroll_up, can_scroll_down) = use_can_scroll_vertical();

    let pos = match position {
        SelectPosition::Below => "top-[calc(100%+4px)] origin-top",
        SelectPosition::Above => "bottom-[calc(100%+4px)] origin-bottom",
    };
    let content_class = move || {
        tw_merge!(
            "absolute left-0 z-50 min-w-full w-max overflow-auto p-1 rounded-md border bg-card shadow-md max-h-[300px] transition-all duration-150 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            pos,
            if open.get() { "opacity-100 scale-100 pointer-events-auto" } else { "opacity-0 scale-95 pointer-events-none" },
            &class
        )
    };

    view! {
        // outside-click backdrop (only interactive while open)
        <div class=move || if open.get() { "fixed inset-0 z-40" } else { "hidden" }
            on:click=move |_| open.set(false)></div>

        <div data-name="SelectContent" class=content_class on:scroll=on_scroll>
            <div class=move || if can_scroll_up.get() { "sticky -top-1 z-10 flex items-center justify-center py-1 bg-card" } else { "hidden" }>
                <ChevronUp class="size-4 text-muted-foreground" />
            </div>
            {children()}
            <div class=move || if can_scroll_down.get() { "sticky -bottom-1 z-10 flex items-center justify-center py-1 bg-card" } else { "hidden" }>
                <ChevronDown class="size-4 text-muted-foreground" />
            </div>
        </div>
    }
}
