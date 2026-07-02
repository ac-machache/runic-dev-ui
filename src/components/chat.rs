//! The chat pane: the top bar, the scrolling transcript (user/assistant/
//! thinking/tool/warning items plus the live streaming tail), and the HITL
//! "agent is asking" card. Presentation only — state in via signals, the
//! answer action out via a callback.

use icons::Webhook;
use leptos::prelude::*;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::model::{Item, LiveKind, ToolView};
use crate::util::{clean_tool_name, md_to_html};

/// Group a token count with thousands separators: `1234` -> `1,234`.
fn commas(n: u64) -> String {
    let s = n.to_string();
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in b.iter().enumerate() {
        if i > 0 && (b.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*c as char);
    }
    out
}

/// Dot + pill colour for a tool's status.
fn status_colors(status: &str) -> (&'static str, &'static str) {
    match status {
        "running" => ("bg-amber-500", "text-amber-500"),
        "error" => ("bg-destructive", "text-destructive"),
        _ => ("bg-emerald-500", "text-emerald-500"),
    }
}

/// A tool-call card in the transcript — secondary to the conversation.
fn render_tool_card(t: ToolView) -> AnyView {
    let (dot, pill) = status_colors(&t.status);
    let dur = if t.duration_ms > 0 { format!("{}ms", t.duration_ms) } else { String::new() };
    let label = clean_tool_name(&t.name);
    let has_input = !t.input.is_empty();
    let input = t.input.clone();
    let has_result = !t.result.is_empty();
    let is_err = t.status == "error";
    let result = t.result.clone();
    let result_cls = if is_err {
        "mt-1 overflow-x-auto rounded bg-background p-2 font-mono text-[11px] text-destructive whitespace-pre-wrap"
    } else {
        "mt-1 overflow-x-auto rounded bg-background p-2 font-mono text-[11px] text-foreground whitespace-pre-wrap"
    };
    let sources = t.sources.clone();
    view! {
        <div class="mx-4 my-2 rounded-lg border border-border bg-muted/40 text-[12px]">
            <div class="flex items-center gap-2 px-3 py-2">
                <span class=format!("w-1.5 h-1.5 rounded-full {dot}")></span>
                <span class="font-mono font-medium text-foreground">{label}</span>
                <span class=format!("text-[10px] uppercase tracking-wide {pill}")>{t.status.clone()}</span>
                <span class="ml-auto font-mono text-[10px] text-muted-foreground">{dur}</span>
            </div>
            {has_input.then(|| view! {
                <details class="border-t border-border px-3 py-1.5">
                    <summary class="text-[10px] uppercase tracking-wide text-muted-foreground cursor-pointer select-none">"args"</summary>
                    <pre class="mt-1 overflow-x-auto rounded bg-background p-2 font-mono text-[11px] text-foreground whitespace-pre-wrap">{input}</pre>
                </details>
            })}
            {has_result.then(move || view! {
                <details class="border-t border-border px-3 py-1.5">
                    <summary class="text-[10px] uppercase tracking-wide text-muted-foreground cursor-pointer select-none">"result"</summary>
                    <pre class=result_cls>{result}</pre>
                    {(!sources.is_empty()).then(|| view! {
                        <div class="flex flex-wrap gap-1.5 mt-2">
                            {sources.iter().map(|s| {
                                let url = s.url.clone();
                                let title = if s.title.is_empty() { s.url.clone() } else { s.title.clone() };
                                view! {
                                    <a class="inline-flex items-center gap-1 rounded-md border border-border bg-background px-2 py-1 text-[11px] text-foreground hover:border-ring"
                                        href=url target="_blank">{title}<span class="text-muted-foreground">"🔗"</span></a>
                                }
                            }).collect_view()}
                        </div>
                    })}
                </details>
            })}
        </div>
    }.into_any()
}

/// One transcript entry.
fn render_item(item: Item) -> AnyView {
    match item {
        Item::User(text) => view! {
            <div class="flex justify-end px-4 py-2">
                <div class="max-w-[82%] rounded-2xl rounded-br-md bg-primary text-primary-foreground px-3.5 py-2 text-[13px] leading-relaxed whitespace-pre-wrap break-words">{text}</div>
            </div>
        }.into_any(),
        Item::Assistant(text) => view! {
            <div class="flex gap-2.5 px-4 py-2">
                <img class="w-6 h-6 rounded-md bg-[#f5f1ea] object-contain p-0.5 ring-1 ring-border shrink-0 mt-0.5" src="runic-mark.png" alt="runic" />
                <div class="prose min-w-0 flex-1 text-[13px] text-foreground leading-relaxed" inner_html=md_to_html(&text)></div>
            </div>
        }.into_any(),
        Item::Thinking(text) => view! {
            <div class="flex gap-2.5 px-4 py-2">
                <img class="w-6 h-6 rounded-md bg-[#f5f1ea] object-contain p-0.5 ring-1 ring-border shrink-0 mt-0.5" src="runic-mark.png" alt="runic" />
                <details class="min-w-0 flex-1">
                    <summary class="text-[12px] text-muted-foreground italic cursor-pointer select-none">"thinking"</summary>
                    <div class="mt-1 text-[12px] text-muted-foreground/80 whitespace-pre-wrap font-mono">{text}</div>
                </details>
            </div>
        }.into_any(),
        Item::Warning(text) => view! {
            <div class="mx-4 my-2 flex items-center gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-[12px] text-amber-600 dark:text-amber-400">
                <span>"⚠"</span><span>{text}</span>
            </div>
        }.into_any(),
        Item::Tool(t) => render_tool_card(t),
        Item::Usage { input, output } => view! {
            <div class="flex gap-2.5 px-4 pb-2 -mt-1">
                <div class="w-6 shrink-0"></div>
                <span class="font-mono text-[10.5px] text-muted-foreground">
                    {format!("↑ {} · ↓ {} tokens", commas(input), commas(output))}
                </span>
            </div>
        }.into_any(),
        Item::Hook(h) => {
            let kind = if h.kind.is_empty() { String::new() } else { format!("· {}", h.kind) };
            view! {
                <div class="mx-4 my-1.5 flex items-center gap-2 rounded-md border border-border bg-muted/30 px-3 py-1.5 text-[11px]">
                    <Webhook class="size-3.5 text-amber-500 shrink-0" />
                    <span class="font-mono font-medium text-foreground">{h.name}</span>
                    <span class="text-muted-foreground">{h.lifecycle}{kind}</span>
                    <span class="rounded px-1.5 py-0.5 text-[10px] uppercase tracking-wide bg-amber-500/15 text-amber-600 dark:text-amber-400 shrink-0">{h.outcome}</span>
                    {h.note.map(|n| view! { <span class="text-muted-foreground truncate">{n}</span> })}
                </div>
            }.into_any()
        }
    }
}

#[component]
pub fn ChatPane(
    collapsed: RwSignal<bool>,
    current: RwSignal<Option<String>>,
    items: RwSignal<Vec<Item>>,
    live: RwSignal<crate::model::LiveBuf>,
    transcript_ref: NodeRef<leptos::html::Div>,
    pending: RwSignal<Option<crate::model::PendingAsk>>,
    has_pending: Memo<bool>,
    ask_answer: RwSignal<String>,
    #[prop(into)] on_submit_answer: Callback<()>,
) -> impl IntoView {
    view! {
        // ── floating "open sidebar" affordance (only when collapsed) ───
        {move || collapsed.get().then(|| view! {
            <div class="absolute top-2 left-2 z-10">
                <Button variant=ButtonVariant::Ghost size=ButtonSize::IconSm
                    attr:title="Open sidebar" on:click=move |_| collapsed.set(false)>"»"</Button>
            </div>
        })}

        // ── transcript ─────────────────────────────────────────────────
        <div class="flex-1 overflow-y-auto min-h-0" node_ref=transcript_ref>
            <div class="max-w-3xl mx-auto py-3">
                {move || (items.get().is_empty() && live.get().text.is_empty()).then(|| {
                    let msg = if current.get().is_some() {
                        "Send a message to start the conversation."
                    } else {
                        "Create or select a thread to begin."
                    };
                    view! { <div class="text-center text-muted-foreground italic text-[13px] py-16">{msg}</div> }
                })}
                {move || items.get().into_iter().map(render_item).collect_view()}
                {move || {
                    let lb = live.get();
                    if lb.text.is_empty() {
                        ().into_any()
                    } else if matches!(lb.kind, LiveKind::Thinking) {
                        view! {
                            <div class="flex gap-2.5 px-4 py-2">
                                <img class="w-6 h-6 rounded-md bg-[#f5f1ea] object-contain p-0.5 ring-1 ring-border shrink-0 mt-0.5" src="runic-mark.png" alt="runic" />
                                <div class="min-w-0 flex-1 text-[12px] text-muted-foreground/80 whitespace-pre-wrap font-mono">{lb.text}</div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex gap-2.5 px-4 py-2">
                                <img class="w-6 h-6 rounded-md bg-[#f5f1ea] object-contain p-0.5 ring-1 ring-border shrink-0 mt-0.5" src="runic-mark.png" alt="runic" />
                                <div class="min-w-0 flex-1 text-[13px] text-foreground leading-relaxed whitespace-pre-wrap">
                                    {lb.text}<span class="inline-block w-1.5 h-4 bg-foreground/70 ml-0.5 align-middle animate-pulse"></span>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>

        // ── HITL ask card ──────────────────────────────────────────────
        {move || has_pending.get().then(|| {
            let p = pending.get_untracked().expect("has_pending implies Some");
            view! {
                <div class="mx-4 mb-3 rounded-xl border border-amber-500/40 bg-amber-500/5 p-3">
                    <div class="flex items-center gap-2 mb-2">
                        <span class="text-amber-500">"⏸"</span>
                        <span class="text-[13px] font-semibold text-foreground">"agent is asking"</span>
                        <span class="ml-auto text-[10px] uppercase tracking-wide text-amber-600 dark:text-amber-400 border border-amber-500/40 rounded px-1.5 py-0.5">"input needed"</span>
                    </div>
                    <div class="text-[13px] text-foreground mb-2 whitespace-pre-wrap">{p.question.clone()}</div>
                    {p.context.clone().map(|c| view! { <div class="text-[12px] text-muted-foreground font-mono mb-2 whitespace-pre-wrap">{c}</div> })}
                    <textarea class="w-full h-20 rounded-md border border-border bg-background px-2.5 py-2 text-[13px] text-foreground outline-none resize-none focus-visible:ring-2 focus-visible:ring-ring/50 placeholder:text-muted-foreground"
                        spellcheck="false" placeholder="Your answer…"
                        prop:value=move || ask_answer.get()
                        on:input=move |e| ask_answer.set(event_target_value(&e))></textarea>
                    <div class="flex justify-end mt-2">
                        <Button variant=ButtonVariant::Default on:click=move |_| on_submit_answer.run(())>"Send answer"</Button>
                    </div>
                </div>
            }
        })}
    }
}
