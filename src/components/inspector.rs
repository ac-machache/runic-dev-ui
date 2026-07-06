//! The thread inspector: a right-side slide-over drawer (Rust/UI `Sheet`) with
//! two tabs (`Tabs`) — **Timeline** (the Run → Turn → step tree from the event
//! stream) and **State** (system prompt + messages as sent to the model).
//!
//! Design ported from `fixtures/index.html`: collapsible run/turn cards,
//! color-coded role accents (user=blue, assistant=emerald, tool=amber), and a
//! mono badge system. NOTE: we have no per-step timing, so timing badges are
//! limited to a tool's `exec {ms}` (with a `slow` red treatment when long).

use icons::{ListTree, RefreshCw};
use leptos::prelude::*;
use serde_json::Value;

use crate::components::ui::button::{ButtonSize, ButtonVariant};
use crate::components::ui::sheet::{Sheet, SheetContent, SheetDirection, SheetTrigger};
use crate::components::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
use crate::events::cluster_runs;
use crate::model::{RunCluster, ToolView, TurnCluster};
use crate::util::{clean_tool_name, content_blocks, render_block_summary, short_id, truncate};

const PRE: &str =
    "mt-1 overflow-x-auto rounded bg-muted/50 p-2 font-mono text-[11px] text-foreground whitespace-pre-wrap";

/// Human duration: `840ms` / `1.6s`.
fn fmt_dur(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

/// A left-accent "step" block: a colored bar, a small role label, and content.
fn step(accent: &str, label: &str, label_color: &str, body: AnyView) -> AnyView {
    let wrap = format!("border-l-2 {accent} pl-3 py-0.5");
    let cap = format!("text-[10px] uppercase tracking-wide {label_color}");
    view! {
        <div class=wrap>
            <span class=cap>{label.to_string()}</span>
            {body}
        </div>
    }
    .into_any()
}

// ── timeline: run → turn → tool ── flat activity feed, left-accent bars ──

fn render_turn_tool(t: &ToolView) -> AnyView {
    let status = t.status.clone();
    let status_cls = match t.status.as_str() {
        "error" => "text-destructive",
        "running" => "text-amber-500",
        _ => "text-emerald-500",
    };
    let label = clean_tool_name(&t.name);
    let slow = t.duration_ms > 3000;
    let dur = if t.duration_ms > 0 { format!("exec {}ms", t.duration_ms) } else { String::new() };
    let dur_cls = if slow {
        "ml-auto font-mono text-[11px] text-destructive"
    } else {
        "ml-auto font-mono text-[11px] text-muted-foreground"
    };
    let has_input = !t.input.is_empty();
    let input = t.input.clone();
    let has_result = !t.result.is_empty();
    let is_err = t.status == "error";
    let result = t.result.clone();
    let result_cls = if is_err {
        "mt-1 overflow-x-auto rounded bg-muted/50 p-2 font-mono text-[11px] text-destructive whitespace-pre-wrap"
    } else {
        PRE
    };
    let sources = t.sources.clone();
    view! {
        <div class="border-l-2 border-l-amber-500 pl-3 py-0.5">
            <div class="flex items-center gap-2 text-[12px]">
                <span class="font-mono font-medium text-foreground">{label}</span>
                <span class=format!("text-[10px] uppercase tracking-wide {status_cls}")>{status}</span>
                <span class=dur_cls>{dur}</span>
            </div>
            {has_input.then(|| view! {
                <details class="mt-0.5">
                    <summary class="text-[10px] uppercase tracking-wide text-muted-foreground cursor-pointer select-none">"args"</summary>
                    <pre class=PRE>{input}</pre>
                </details>
            })}
            {has_result.then(move || view! {
                <details class="mt-0.5">
                    <summary class="text-[10px] uppercase tracking-wide text-muted-foreground cursor-pointer select-none">"result"</summary>
                    <pre class=result_cls>{result}</pre>
                    {(!sources.is_empty()).then(|| view! {
                        <div class="flex flex-wrap gap-1.5 mt-1.5">
                            {sources.iter().map(|s| {
                                let url = s.url.clone();
                                let title = if s.title.is_empty() { s.url.clone() } else { s.title.clone() };
                                view! { <a class="rounded border border-border bg-background px-1.5 py-0.5 text-[10px] text-foreground hover:border-ring" href=url target="_blank">{title}</a> }
                            }).collect_view()}
                        </div>
                    })}
                </details>
            })}
        </div>
    }.into_any()
}

fn render_turn(idx: usize, t: TurnCluster, show_thinking: bool) -> AnyView {
    let running = !t.closed;
    let calls = t.tool_calls.max(t.tools.len() as u32);
    let has_text = !t.text.is_empty();
    let text = t.text.clone();
    let show_think = show_thinking && !t.thinking.is_empty();
    let thinking = t.thinking.clone();
    let tool_views = t.tools.iter().map(render_turn_tool).collect_view();
    let hook_views = t.hooks.iter().map(|h| {
        let meta = if h.kind.is_empty() { h.lifecycle.clone() } else { format!("{} · {}", h.lifecycle, h.kind) };
        let note = h.note.clone();
        view! {
            <div class="border-l-2 border-l-rose-400 pl-3 py-0.5">
                <div class="flex items-center gap-2 text-[12px]">
                    <span class="font-mono font-medium text-foreground">{h.name.clone()}</span>
                    <span class="text-[10px] uppercase tracking-wide text-rose-400">{h.outcome.clone()}</span>
                    <span class="ml-auto font-mono text-[10px] text-muted-foreground/70">{meta}</span>
                </div>
                {note.map(|n| view! { <div class="text-[11px] text-muted-foreground">{n}</div> })}
            </div>
        }
    }).collect_view();
    let meta = if running { "streaming".to_string() } else { format!("{calls} call{}", if calls == 1 { "" } else { "s" }) };
    view! {
        <div class="mt-3 first:mt-1.5">
            <div class="flex items-center gap-1.5 mb-1.5">
                <span class="text-[10px] uppercase tracking-wider text-muted-foreground/50">{format!("Turn {}", idx + 1)}</span>
                {running.then(|| view! { <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> })}
                <span class="text-[10px] text-muted-foreground/50">{meta}</span>
            </div>
            <div class="space-y-2">
                {show_think.then(|| step(
                    "border-l-violet-400", "thinking", "text-violet-400/80",
                    view! { <div class="text-[12px] text-muted-foreground/80 whitespace-pre-wrap font-mono">{thinking}</div> }.into_any(),
                ))}
                {has_text.then(|| step(
                    "border-l-emerald-500", "assistant", "text-emerald-500/80",
                    view! { <div class="text-[12px] text-foreground whitespace-pre-wrap">{text}</div> }.into_any(),
                ))}
                {hook_views}
                {tool_views}
            </div>
        </div>
    }.into_any()
}

fn render_run(idx: usize, total: usize, r: RunCluster, show_thinking: bool) -> AnyView {
    let dot = if r.running {
        "bg-amber-500 animate-pulse"
    } else if r.errored {
        "bg-destructive"
    } else {
        "bg-emerald-500"
    };
    let has_prompt = !r.prompt.is_empty();
    let prompt = r.prompt.clone();
    let label = if has_prompt {
        truncate(r.prompt.lines().next().unwrap_or(""), 54)
    } else if r.id.is_empty() {
        format!("run {}", idx + 1)
    } else {
        format!("run · {}", short_id(&r.id))
    };
    let n = r.turns.len();
    let turns_label = format!("{n} turn{}", if n == 1 { "" } else { "s" });
    let stop = r.stop_reason.clone();
    let usage = r.usage;
    let duration = r.duration_ms;
    let agent = r.agent.clone();
    let open = r.running || idx + 1 == total;
    let turn_views = r
        .turns
        .into_iter()
        .enumerate()
        .map(|(i, t)| render_turn(i, t, show_thinking))
        .collect_view();
    view! {
        <details class="border-b border-border/70 last:border-0" open=open>
            <summary class="flex items-center gap-2.5 py-2.5 cursor-pointer select-none">
                <span class=format!("w-2 h-2 rounded-full shrink-0 {dot}")></span>
                <span class="font-mono text-[13px] font-medium text-foreground truncate flex-1">{label}</span>
                <span class="flex items-center gap-3 font-mono text-[11px] text-muted-foreground shrink-0">
                    {agent.map(|a| view! { <span class="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">{a}</span> })}
                    {duration.map(|d| { let slow = d > 10_000; view! { <span class=if slow { "text-destructive" } else { "text-foreground/80" }>{fmt_dur(d)}</span> } })}
                    {usage.map(|(i, o)| view! { <span class="text-foreground/80">{format!("↑{i} ↓{o}")}</span> })}
                    <span>{turns_label}</span>
                    {stop.map(|s| view! { <span class="text-muted-foreground/70">{s}</span> })}
                </span>
            </summary>
            <div class="pb-3 pl-4">
                {has_prompt.then(|| step(
                    "border-l-blue-400", "user", "text-blue-400/80",
                    view! { <div class="text-[12px] text-foreground whitespace-pre-wrap">{prompt}</div> }.into_any(),
                ))}
                {turn_views}
            </div>
        </details>
    }.into_any()
}

// ── state tab ────────────────────────────────────────────────────────────

fn render_state(s: &Value) -> AnyView {
    let busy = s.get("busy").and_then(|v| v.as_bool()).unwrap_or(false);
    let prompt = s
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let event_count = s.get("event_count").and_then(|v| v.as_u64()).unwrap_or(0);
    // `run_count` was replaced by a nested `stats` object (ThreadStatsView).
    let stat = |k: &str| s.pointer(&format!("/stats/{k}")).and_then(|v| v.as_u64());
    let runs = stat("runs").map(|r| r.to_string()).unwrap_or_else(|| "—".into());
    let turns = stat("turns");
    let tok_in = stat("input_tokens");
    let tok_out = stat("output_tokens");
    let tool_calls = stat("total_tool_calls");
    let tasks_spawned = stat("tasks_spawned").unwrap_or(0);
    let has_prompt = !prompt.trim().is_empty();
    let messages = s
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let msg_count = messages.len();

    let msg_views = messages.into_iter().map(|m| {
        let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let body = content_blocks(&m).iter().map(render_block_summary).collect::<Vec<_>>().join("\n");
        let preview = truncate(body.lines().next().unwrap_or(""), 90);
        view! {
            <details class="border-b border-border/50 last:border-0 py-1">
                <summary class="flex items-center gap-2 cursor-pointer select-none text-[12px] py-0.5">
                    <span class="font-mono text-[10px] uppercase tracking-wide text-muted-foreground w-16 shrink-0">{role}</span>
                    <span class="text-muted-foreground truncate">{preview}</span>
                </summary>
                <pre class="mt-1 pl-[4.5rem] font-mono text-[11px] text-foreground whitespace-pre-wrap">{body}</pre>
            </details>
        }
    }).collect_view();

    view! {
        <div class="space-y-4">
            <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-muted-foreground">
                <span><strong class="text-foreground">{runs}</strong>" runs"</span>
                {turns.map(|t| view! { <span>"· "<strong class="text-foreground">{t.to_string()}</strong>" turns"</span> })}
                <span>"· "<strong class="text-foreground">{msg_count.to_string()}</strong>" messages"</span>
                <span>"· "<strong class="text-foreground">{event_count.to_string()}</strong>" events"</span>
                {tool_calls.map(|c| view! { <span>"· "<strong class="text-foreground">{c.to_string()}</strong>" tool calls"</span> })}
                {(tasks_spawned > 0).then(|| view! { <span>"· "<strong class="text-foreground">{tasks_spawned.to_string()}</strong>" tasks"</span> })}
                {(tok_in.is_some() || tok_out.is_some()).then(|| view! {
                    <span class="font-mono text-foreground/80">{format!("↑{} ↓{}", tok_in.unwrap_or(0), tok_out.unwrap_or(0))}</span>
                })}
                {busy.then(|| view! { <span class="flex items-center gap-1.5 text-emerald-500"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>"busy"</span> })}
            </div>

            {has_prompt.then(|| view! {
                <div>
                    <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">"System prompt"</div>
                    <pre class="rounded-md border border-border bg-muted/30 p-3 font-mono text-[12px] text-foreground whitespace-pre-wrap max-h-80 overflow-auto">{prompt}</pre>
                </div>
            })}

            <div>
                <div class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">{format!("Messages · {msg_count} as sent to model")}</div>
                <div class="space-y-1">{msg_views}</div>
            </div>
        </div>
    }.into_any()
}

// ── the drawer ───────────────────────────────────────────────────────────

#[component]
pub fn Inspector(
    events: RwSignal<Vec<Value>>,
    state_json: RwSignal<Option<Value>>,
    show_thinking: RwSignal<bool>,
    #[prop(into)] on_refresh_state: Callback<()>,
) -> impl IntoView {
    view! {
        <Sheet>
            <SheetTrigger
                variant=ButtonVariant::Outline
                size=ButtonSize::IconSm
                class="absolute top-2 right-2 z-10"
                attr:title="Inspect thread"
                on:click=move |_| on_refresh_state.run(())>
                <ListTree />
            </SheetTrigger>

            <SheetContent direction=SheetDirection::Right class="w-[min(880px,94vw)] p-0 flex flex-col">
                <div class="px-4 py-3 border-b border-border shrink-0">
                    <span class="text-[13px] font-semibold text-foreground">"Thread inspector"</span>
                </div>

                <Tabs default_value="timeline" class="flex flex-col flex-1 min-h-0 px-4 py-3">
                    <TabsList class="shrink-0">
                        <TabsTrigger value="timeline">"Timeline"</TabsTrigger>
                        <TabsTrigger value="state">"State"</TabsTrigger>
                    </TabsList>

                    <TabsContent value="timeline" class="overflow-y-auto">
                        <div class="flex items-center justify-end pb-2">
                            <button class="text-[11px] text-muted-foreground hover:text-foreground"
                                on:click=move |_| show_thinking.update(|t| *t = !*t)>
                                {move || if show_thinking.get() { "hide thinking" } else { "show thinking" }}
                            </button>
                        </div>
                        {move || {
                            let st = show_thinking.get();
                            let runs = cluster_runs(&events.get());
                            let total = runs.len();
                            if total == 0 {
                                view! { <div class="text-center text-muted-foreground italic text-[13px] py-12">"No runs yet."</div> }.into_any()
                            } else {
                                runs.into_iter().enumerate().rev()
                                    .map(|(i, r)| render_run(i, total, r, st))
                                    .collect_view().into_any()
                            }
                        }}
                    </TabsContent>

                    <TabsContent value="state" class="overflow-y-auto">
                        <div class="flex items-center justify-end pb-2">
                            <button class="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground"
                                on:click=move |_| on_refresh_state.run(())>
                                <RefreshCw class="size-3" />"refresh"
                            </button>
                        </div>
                        {move || match state_json.get() {
                            None => view! { <div class="text-center text-muted-foreground italic text-[13px] py-12">"Open or refresh to load state."</div> }.into_any(),
                            Some(s) => render_state(&s),
                        }}
                    </TabsContent>
                </Tabs>
            </SheetContent>
        </Sheet>
    }
}
