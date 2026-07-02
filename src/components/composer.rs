//! The chat composer: a full-width message textarea on top, with a toolbar row
//! below (attach / voice / configurable on the left, send-or-stop on the right)
//! and the per-run "Configurable" popover. Presentation only — state in via
//! signals, actions out via callbacks.

use icons::{Braces, List, Mic, Paperclip, Plus, Send, SlidersHorizontal, Square};
use leptos::prelude::*;
use serde_json::{Map, Value};

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::model::Attachment;

const FIELD_CLS: &str = "w-full rounded-md border border-border bg-background px-3 py-2 text-[13px] font-mono text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring/50 placeholder:text-muted-foreground";

// ── context helpers: the key/value editor is a UI over the raw `context_json`
// object (an open map), so JSON stays the single source of truth and `send()`
// is unchanged. A value is parsed as JSON when it can be (so `true`, `42`,
// `["a"]` keep their type), otherwise it's stored as a plain string.

fn ctx_obj(s: &str) -> Map<String, Value> {
    match serde_json::from_str::<Value>(s) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}
fn ctx_write(ctx: RwSignal<String>, m: Map<String, Value>) {
    if m.is_empty() {
        ctx.set(String::new());
    } else {
        ctx.set(serde_json::to_string_pretty(&Value::Object(m)).unwrap_or_default());
    }
}
fn ctx_keys(ctx: RwSignal<String>) -> Vec<String> {
    ctx_obj(&ctx.get()).keys().cloned().collect()
}
fn entry_value_str(ctx: RwSignal<String>, key: &str) -> String {
    match ctx_obj(&ctx.get()).get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    }
}
fn set_kv(ctx: RwSignal<String>, key: &str, raw: &str) {
    let mut m = ctx_obj(&ctx.get_untracked());
    let val = serde_json::from_str::<Value>(raw).unwrap_or_else(|_| Value::String(raw.to_string()));
    m.insert(key.to_string(), val);
    ctx_write(ctx, m);
}
fn remove_key(ctx: RwSignal<String>, key: &str) {
    let mut m = ctx_obj(&ctx.get_untracked());
    m.remove(key);
    ctx_write(ctx, m);
}

#[component]
pub fn Composer(
    current: RwSignal<Option<String>>,
    input: RwSignal<String>,
    streaming: RwSignal<bool>,
    cancelling: RwSignal<bool>,
    uploading: RwSignal<usize>,
    recording: RwSignal<bool>,
    config_open: RwSignal<bool>,
    context_json: RwSignal<String>,
    attachments: RwSignal<Vec<Attachment>>,
    file_input_ref: NodeRef<leptos::html::Input>,
    #[prop(into)] on_send: Callback<()>,
    #[prop(into)] on_stop: Callback<()>,
    #[prop(into)] on_toggle_record: Callback<()>,
    #[prop(into)] on_pick_files: Callback<()>,
) -> impl IntoView {
    let raw_mode = RwSignal::new(false);
    let draft_key = RwSignal::new(String::new());
    let draft_val = RwSignal::new(String::new());

    let commit_draft = move || {
        let k = draft_key.get_untracked();
        if k.trim().is_empty() { return; }
        set_kv(context_json, k.trim(), &draft_val.get_untracked());
        draft_key.set(String::new());
        draft_val.set(String::new());
    };

    view! {
        <div class="border-t border-border p-3">
            <div class="relative rounded-xl border border-border bg-card shadow-sm px-3 pt-2.5 pb-2 focus-within:border-ring/60 transition-colors">

                // ── Configurable popover ───────────────────────────────
                {move || config_open.get().then(|| view! {
                    <div class="absolute bottom-full left-0 mb-2 w-[620px] max-w-[calc(100vw-2rem)] rounded-xl border border-border bg-popover shadow-md p-4">
                        <div class="flex items-center gap-2 mb-3">
                            <span class="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">"Context"</span>
                            <span class="flex-1"></span>
                            <button type="button" class="flex items-center gap-1.5 text-[12px] text-muted-foreground hover:text-foreground"
                                on:click=move |_| raw_mode.update(|r| *r = !*r)>
                                {move || if raw_mode.get() {
                                    view! { <List class="size-3.5" />"Form" }.into_any()
                                } else {
                                    view! { <Braces class="size-3.5" />"Raw" }.into_any()
                                }}
                            </button>
                            <Button variant=ButtonVariant::Ghost size=ButtonSize::IconXs
                                on:click=move |_| config_open.set(false)>"✕"</Button>
                        </div>

                        {move || if raw_mode.get() {
                            // ── raw JSON (power users) ─────────────────────
                            view! {
                                <div class="flex items-center gap-2 mb-1 text-[10px]">
                                    <span class="font-mono text-muted-foreground/70">"sent verbatim"</span>
                                    {move || {
                                        let t = context_json.get();
                                        if t.trim().is_empty() {
                                            ().into_any()
                                        } else if serde_json::from_str::<Value>(&t).is_ok() {
                                            view! { <span class="ml-auto text-emerald-500">"● valid"</span> }.into_any()
                                        } else {
                                            view! { <span class="ml-auto text-destructive">"● invalid"</span> }.into_any()
                                        }
                                    }}
                                </div>
                                <textarea class="w-full h-52 rounded-md border border-border bg-background px-3 py-2.5 font-mono text-[13px] text-foreground outline-none resize-none focus-visible:ring-2 focus-visible:ring-ring/50 placeholder:text-muted-foreground"
                                    spellcheck="false"
                                    placeholder=r#"{ "user_id": "u1", "allow_web_search": true }"#
                                    prop:value=move || context_json.get()
                                    on:input=move |e| context_json.set(event_target_value(&e))></textarea>
                            }.into_any()
                        } else {
                            // ── key/value editor (default) ─────────────────
                            view! {
                                <div class="space-y-2">
                                    <For each=move || ctx_keys(context_json) key=|k| k.clone()
                                        children=move |k: String| {
                                            let k_get = k.clone();
                                            let k_set = k.clone();
                                            let k_rm = k.clone();
                                            let k_title = k.clone();
                                            view! {
                                                <div class="flex items-center gap-1.5">
                                                    <span class="font-mono text-[13px] text-foreground shrink-0 w-32 truncate" title=k_title>{k}</span>
                                                    <input class=FIELD_CLS spellcheck="false" placeholder="value"
                                                        prop:value=move || entry_value_str(context_json, &k_get)
                                                        on:input=move |e| set_kv(context_json, &k_set, &event_target_value(&e)) />
                                                    <button type="button" class="text-muted-foreground hover:text-destructive shrink-0 px-1"
                                                        title="Remove" on:click=move |_| remove_key(context_json, &k_rm)>"✕"</button>
                                                </div>
                                            }
                                        } />
                                    {move || ctx_keys(context_json).is_empty().then(|| view! {
                                        <div class="text-[11px] text-muted-foreground/70 italic py-1">"No context keys — add one below."</div>
                                    })}
                                </div>

                                <div class="flex items-center gap-1.5 mt-2 pt-2 border-t border-border">
                                    <input class="w-32 shrink-0 rounded-md border border-border bg-background px-3 py-2 text-[13px] font-mono text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring/50 placeholder:text-muted-foreground"
                                        spellcheck="false" placeholder="key"
                                        prop:value=move || draft_key.get()
                                        on:input=move |e| draft_key.set(event_target_value(&e)) />
                                    <input class=FIELD_CLS spellcheck="false" placeholder="value · true · 42 · {…}"
                                        prop:value=move || draft_val.get()
                                        on:input=move |e| draft_val.set(event_target_value(&e))
                                        on:keydown=move |e| if e.key() == "Enter" { e.prevent_default(); commit_draft(); } />
                                    <Button variant=ButtonVariant::Default
                                        on:click=move |_| commit_draft()><Plus />"Add"</Button>
                                </div>
                            }.into_any()
                        }}
                    </div>
                })}

                // ── attachment chips ───────────────────────────────────
                {move || {
                    let a = attachments.get();
                    (!a.is_empty()).then(|| view! {
                        <div class="flex flex-wrap gap-1.5 pb-2">
                            {a.into_iter().enumerate().map(|(i, att)| view! {
                                <span class="inline-flex items-center gap-1.5 rounded-md border border-border bg-muted px-2 py-1 text-[11px]"
                                    title=att.media_type.clone()>
                                    <span class="font-mono text-muted-foreground">{att.name}</span>
                                    <button class="text-muted-foreground hover:text-destructive"
                                        on:click=move |_| attachments.update(|v| { if i < v.len() { v.remove(i); } })>"✕"</button>
                                </span>
                            }).collect_view()}
                        </div>
                    })
                }}

                // ── message textarea (full width, on top) ──────────────
                <textarea class="w-full resize-none bg-transparent text-[13px] leading-relaxed text-foreground outline-none placeholder:text-muted-foreground min-h-[40px] max-h-48 disabled:opacity-50"
                    spellcheck="false"
                    rows=move || input.get().lines().count().clamp(1, 8).to_string()
                    prop:value=move || input.get()
                    on:input=move |e| input.set(event_target_value(&e))
                    on:keydown=move |e| {
                        if e.key() == "Enter" && !e.shift_key() {
                            e.prevent_default();
                            on_send.run(());
                        }
                    }
                    prop:disabled=move || current.get().is_none() || streaming.get()
                    placeholder=move || if current.get().is_some() {
                        "Message the agent…".to_string()
                    } else {
                        "Create or pick a thread first".to_string()
                    }></textarea>

                // ── toolbar row (icons left · send right) ───────────────
                <div class="flex items-center gap-0.5 pt-1.5">
                    <Button variant=ButtonVariant::Ghost size=ButtonSize::IconSm
                        attr:title="Attach files"
                        attr:disabled=move || current.get().is_none() || streaming.get()
                        on:click=move |_| { if let Some(el) = file_input_ref.get() { el.click(); } }>
                        <Paperclip />
                    </Button>
                    <input type="file" multiple node_ref=file_input_ref class="hidden"
                        on:change=move |_| on_pick_files.run(()) />

                    <Button
                        variant=Signal::derive(move || if recording.get() { ButtonVariant::Destructive } else { ButtonVariant::Ghost })
                        size=ButtonSize::IconSm
                        attr:title=move || if recording.get() { "Stop recording" } else { "Record voice" }
                        attr:disabled=move || current.get().is_none() || streaming.get()
                        on:click=move |_| on_toggle_record.run(())>
                        {move || if recording.get() {
                            view! { <Square class="fill-current" /> }.into_any()
                        } else {
                            view! { <Mic /> }.into_any()
                        }}
                    </Button>

                    <Button variant=ButtonVariant::Ghost size=ButtonSize::IconSm class="relative"
                        attr:title="Configurable" on:click=move |_| config_open.update(|c| *c = !*c)>
                        <SlidersHorizontal />
                        {move || (!context_json.get().trim().is_empty()).then(|| view! {
                            <span class="absolute top-1 right-1 w-1.5 h-1.5 rounded-full bg-primary"></span>
                        })}
                    </Button>

                    <span class="flex-1"></span>

                    {move || if streaming.get() {
                        view! {
                            <Button variant=ButtonVariant::Destructive
                                attr:disabled=move || cancelling.get()
                                on:click=move |_| on_stop.run(())>
                                <Square class="fill-current" />
                                {move || if cancelling.get() { "Cancelling…" } else { "Stop" }}
                            </Button>
                        }.into_any()
                    } else {
                        view! {
                            <Button variant=ButtonVariant::Default
                                attr:disabled={move || current.get().is_none() || uploading.get() > 0}
                                on:click=move |_| on_send.run(())>
                                {move || if uploading.get() > 0 {
                                    view! { "Uploading…" }.into_any()
                                } else {
                                    view! { "Send"<Send /> }.into_any()
                                }}
                            </Button>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
