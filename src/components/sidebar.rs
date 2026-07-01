//! The left sidebar: brand, connection settings, and the thread list.
//!
//! Pure presentation — all state comes in as signals and all actions go out
//! as [`Callback`]s, so `App` owns the behaviour and this file owns the look.

use icons::{Check, Plus, RefreshCw, Trash2, X};
use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::input::Input;
use crate::model::ThreadInfo;
use crate::util::short_id;

#[component]
pub fn Sidebar(
    api_base: RwSignal<String>,
    tenant: RwSignal<String>,
    dark: RwSignal<bool>,
    collapsed: RwSignal<bool>,
    threads: RwSignal<Vec<ThreadInfo>>,
    threads_cursor: RwSignal<Option<String>>,
    current: RwSignal<Option<String>>,
    #[prop(into)] on_refresh: Callback<()>,
    #[prop(into)] on_new_thread: Callback<()>,
    #[prop(into)] on_load_thread: Callback<String>,
    #[prop(into)] on_load_more: Callback<()>,
    #[prop(into)] on_delete_thread: Callback<String>,
) -> impl IntoView {
    // Which thread (if any) is showing the inline delete confirmation.
    let confirm_id = RwSignal::new(None::<String>);
    view! {
        <aside class="flex-none overflow-hidden bg-background border-r border-border flex flex-col transition-[width] duration-150"
            style:width=move || if collapsed.get() { "0px".to_string() } else { "264px".to_string() }>
            <div class="w-[264px] flex flex-col h-full">
                // brand
                <div class="flex items-start justify-between px-4 pt-4 pb-3.5 border-b border-border">
                    <div class="flex items-center gap-2.5">
                        <img class="w-7 h-7 rounded-md bg-[#f5f1ea] object-contain p-0.5 ring-1 ring-border" src="runic-mark.png" alt="runic" />
                        <div>
                            <div class="font-bold text-[15px] leading-tight tracking-tight text-foreground">"runic"</div>
                            <div class="text-[10.5px] font-medium text-muted-foreground tracking-wider uppercase mt-0.5">"dev console"</div>
                        </div>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <Button variant=ButtonVariant::Outline size=ButtonSize::IconSm
                            attr:title="Toggle theme" on:click=move |_| dark.update(|d| *d = !*d)>
                            {move || if dark.get() { "☀".to_string() } else { "☾".to_string() }}
                        </Button>
                        <Button variant=ButtonVariant::Outline size=ButtonSize::IconSm
                            attr:title="Collapse sidebar" on:click=move |_| collapsed.set(true)>"«"</Button>
                    </div>
                </div>

                // connection
                <div class="px-4 pt-3.5 pb-4 border-b border-border">
                    <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground mb-2.5">"Connection"</div>
                    <label class="block text-[11px] text-muted-foreground mb-1">"server URL"</label>
                    <Input bind_value=api_base class="h-8 font-mono text-[12px]" />
                    <div class="flex items-center gap-1.5 mt-2.5 mb-1">
                        <span class="text-[11px] text-muted-foreground">"tenant"</span>
                        <span class="text-[10px] font-mono text-muted-foreground/70">"X-Runic-Tenant"</span>
                    </div>
                    <Input bind_value=tenant class="h-8 font-mono text-[12px]" />
                    <div class="flex items-center gap-1.5 mt-3 text-[11px] text-muted-foreground">
                        <span class="w-[7px] h-[7px] rounded-full bg-emerald-500 ring-[3px] ring-emerald-500/15"></span>
                        <span>"connected"</span>
                    </div>
                </div>

                // threads head
                <div class="flex items-center justify-between px-4 pt-3.5 pb-2">
                    <span class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">"Threads"</span>
                    <Button variant=ButtonVariant::Ghost size=ButtonSize::IconXs
                        attr:title="Refresh" on:click=move |_| on_refresh.run(())><RefreshCw /></Button>
                </div>
                // new thread — grouped with the list
                <div class="px-3 pb-2">
                    <Button variant=ButtonVariant::Outline class="w-full justify-start gap-2"
                        on:click=move |_| on_new_thread.run(())>
                        <Plus class="text-primary" />"New thread"
                    </Button>
                </div>
                // thread list
                <div class="flex-1 overflow-y-auto px-2 pb-2 flex flex-col gap-0.5">
                    {move || threads.get().into_iter().map(|t| {
                        let id = t.id;
                        let id_active = id.clone();
                        let id_click = id.clone();
                        let id_del = id.clone();
                        let short = short_id(&id);
                        let untitled = t.label.is_none();
                        let title = t.label.unwrap_or_else(|| "untitled".to_string());
                        let active = Memo::new(move |_| current.get().as_deref() == Some(id_active.as_str()));
                        let title_cls = if untitled {
                            "text-[12px] italic text-muted-foreground/60 mt-0.5"
                        } else {
                            "text-[12px] text-muted-foreground mt-0.5"
                        };
                        view! {
                            <div class="group relative rounded-md px-3 py-2 pl-3.5 cursor-pointer hover:bg-muted transition-colors"
                                class=("bg-accent", move || active.get())
                                on:click=move |_| on_load_thread.run(id_click.clone())>
                                <span class="absolute left-0 top-2 bottom-2 w-[3px] rounded-full bg-primary"
                                    class=("hidden", move || !active.get())></span>
                                <div class="flex items-center justify-between gap-2">
                                    <span class="font-mono text-[12px] text-muted-foreground truncate"
                                        class=("text-foreground", move || active.get())>{short}</span>
                                    <div class="flex items-center gap-0.5 shrink-0" on:click=move |ev| ev.stop_propagation()>
                                        {move || {
                                            if confirm_id.get().as_deref() == Some(id_del.as_str()) {
                                                let id_c = id_del.clone();
                                                view! {
                                                    <button type="button" class="text-destructive hover:opacity-70" title="Confirm delete"
                                                        on:click=move |ev| { ev.stop_propagation(); on_delete_thread.run(id_c.clone()); confirm_id.set(None); }>
                                                        <Check class="size-3.5" />
                                                    </button>
                                                    <button type="button" class="text-muted-foreground hover:text-foreground" title="Cancel"
                                                        on:click=move |ev| { ev.stop_propagation(); confirm_id.set(None); }>
                                                        <X class="size-3.5" />
                                                    </button>
                                                }.into_any()
                                            } else {
                                                let id_t = id_del.clone();
                                                view! {
                                                    <button type="button" class="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive transition-opacity" title="Delete thread"
                                                        on:click=move |ev| { ev.stop_propagation(); confirm_id.set(Some(id_t.clone())); }>
                                                        <Trash2 class="size-3.5" />
                                                    </button>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                                <div class=title_cls>{title}</div>
                            </div>
                        }
                    }).collect_view()}
                    {move || threads_cursor.get().map(|_| view! {
                        <Button variant=ButtonVariant::Outline class="w-full mt-1"
                            on:click=move |_| on_load_more.run(())>"Load more"</Button>
                    })}
                </div>

            </div>
        </aside>
    }
}
