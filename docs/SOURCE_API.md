# Source API (frozen — report contract changes to Core)

Work only in `src-tauri/src/sources/<name>/**` and `src/features/<kind>/**`. Run `pnpm build` once before `cargo clippy` (tauri needs `dist/`).

## Rust (`crate::core::*`)
```
activity::{Activity, ActivityKind, Priority{Ambient,Normal,High,Critical}, CompactView, ActionSpec, ActionStyle{Primary,Neutral,Danger}, ActionRequest{activity_id,action,value:Option<Value>}, SourceCommand, now_ms()}
Activity::new(source:&str, key:&str, kind, CompactView) -> id "<source>:<key>"
  .priority(p) .expanded(Value) .action(ActionSpec) .actions(iter) .alert(bool) .ttl_ms(u64)
CompactView::new(icon, title) .subtitle(s) .progress(0..1) .accent("#hex") .badge(s)   // icon = emoji/short glyph
ActionSpec::button(id,label) | ::toggle(id,label,on) | ::slider(id,label,(min,max),step,value)  .icon(s) .style(ActionStyle)
SourceCommand { source, command, payload:Value, reply }  .respond_ok(Value) .respond_err(msg)
```
`source::{ActivitySource, SourceContext, SourceEvent{Action,Command,Idle,Shutdown}}`
```
trait ActivitySource: Send { fn name(&self)->&'static str; fn start(self: Box<Self>, ctx: SourceContext); }
```
Your struct MUST keep `#[derive(Default)]` (or `impl Default`); it is rebuilt via `Default::default()` on restart. `name()` must equal the folder name.

`SourceContext` (Clone; pub fields `name, app: AppHandle, hub: HubHandle, actions, commands, settings, shutdown`):
```
ctx.is_running() -> bool                       // loop while true
ctx.activity(key, kind, compact) -> Activity   // id prefixed with ctx.name
ctx.upsert(Activity)                           // replaces same id, restarts ttl
ctx.remove(key)                                // key without the "<name>:" prefix
ctx.emit(&impl Serialize)                      // -> event "stream://<ctx.name>"
ctx.current_settings() -> Settings             // snapshot
ctx.take_changed_settings(&mut self) -> Option<Settings>   // Some once per change (needs `mut ctx`)
ctx.next_event(Duration) -> SourceEvent        // Action | Command | Idle(timeout) | Shutdown
ctx.recv_action(Duration) -> Option<ActionRequest>
ctx.recv_command(Duration) -> Option<SourceCommand>
ctx.pause(Duration) -> bool                    // shutdown-aware sleep, returns is_running()
ctx.idle_until_shutdown()                      // stub body; replies "command not supported"
```
Ids: `"<source>:<key>"`, e.g. `game:download:42`; actions are routed by the text before the first `:`. A panic in `start` restarts the source once after 5s. COM (MTA) is initialised for you; do not call CoInitializeEx.
Settings: `crate::core::settings::Settings` (camelCase JSON, all fields `pub`, see `settings.rs`). Read-only from sources; the UI writes them.
Async work: use `tauri::async_runtime::spawn`/`block_on`, or your own tokio runtime (tokio has rt, macros, sync, time, net). Global shortcuts: `tauri_plugin_global_shortcut::GlobalShortcutExt` on `ctx.app`. Opening files/URLs: `tauri_plugin_opener`.
Errors: `crate::core::error::{AppError, AppResult}`. Log with `log::{info,warn,error}`.
Per-source Cargo deps are already in `Cargo.toml`; `windows` features are the full plan list. If you need another crate/feature, report it.

## Svelte (`src/features/<kind>/`)
Files `Compact.svelte`, `Expanded.svelte` take `let { activity }: { activity: Activity } = $props()`. `Settings.svelte` and `Launcher.svelte` (timer only) take no props. Extra files are yours; import them from your own Compact/Expanded. `registry.ts` already imports these exact filenames — keep them. `activity.expanded` is the `serde_json::Value` you set in Rust (cast it: `activity.expanded as MyShape`).
```ts
import type { Activity, ActionSpec, Settings } from "../../lib/contracts";
import { settingsState } from "../../lib/settings.svelte";   // settingsState.value: Settings (reactive); await settingsState.update({ timer: { workMin: 30 } })
import { useStream } from "../../lib/stream.svelte";          // const spectrum = useStream<number[]>("visualizer", []); spectrum.value   (call in component init)
import { onStream, sourceCommand, activityAction } from "../../lib/ipc";
```
- Stream events: Rust `ctx.emit(&payload)` -> `useStream<T>(source, initial)` or `onStream<T>(source, handler) => Promise<UnlistenFn>`.
- Commands: `await sourceCommand<T>("claude", "install_hooks", payload?)` -> your source receives `SourceEvent::Command` and answers with `respond_ok(Value)` / `respond_err(msg)` (30s timeout).
- Actions: `ActionSpec`s on the activity render automatically in `ActionBar` (button/toggle/slider -> `ActionRequest` with `value`). Call `activityAction(activity.id, "my-action", value)` for custom controls.
- Settings tab: rendered by `SettingsApp` from `registry[kind].Settings`; edit via `settingsState.update(patch)` (deep-merged, persisted, broadcast).
- Clock helper: `useClock()` from `lib/clock.svelte`. Drag-out (shelf): `@crabnebula/tauri-plugin-drag`.
- Island surface is `bg-island`; tokens: `text-island-text`, `text-island-muted`, `bg-island-raised`, `border-island-border`, `bg-accent`, `bg-danger`, `ease-island`. Compact area is 16rem x 2.5rem; expanded card is 28rem x 20rem (ActionBar sits at its bottom). No `px` anywhere (Tailwind scale or rem arbitrary values).
- Hit regions are tracked on the island surface only: anything you render must stay inside Compact/Expanded.

## Events (Rust -> JS)
`activity://upsert` Activity, `activity://remove` id, `island://focus` id|null, `island://visibility` bool, `settings://changed` Settings, `stream://<source>` payload.
