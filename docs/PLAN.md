# ChocolateBar — Build Plan

Windows 10 (1809+) / 11 Dynamic Island bar. Tauri v2 + Svelte 5 (runes, TS, plain Vite, no SvelteKit) + Tailwind v4 + Rust (windows-rs).

## Code rules (every worker)
- Clean Code: small functions, intention-revealing names, SRP, DRY.
- NO comments in code.
- Tailwind v4 only; never `px` — use scale utilities or rem arbitrary values (`w-[20rem]`).
- Svelte 5 runes only. pnpm. No interactive CLIs.
- Pin `tauri` and every Tauri plugin to major "2" (crates) / "^2" (JS) — tauri 3 alphas exist on crates.io.
- Versions below are targets; if a version does not resolve, use the latest compatible release.

## Ownership
- **Core** owns and then FREEZES: `package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig*.json`, `index.html`, `src/app.css`, `src/main.ts`, `src/App.svelte`, `src/lib/**`, `src/shell/**`, `src/features/registry.ts`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/**`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/core/**`, `src-tauri/src/window/**`, `src-tauri/src/sources/mod.rs`.
- Feature workers own only `src-tauri/src/sources/<name>/**` and `src/features/<kind>/**` for their features. A contract change goes back through Core (report it, don't edit frozen files).

## Frontend setup (Core)
1. Vite + svelte-ts layout written by hand; identifier `com.chocolatebar.app`.
2. deps: `@tauri-apps/api ^2`, `@tauri-apps/plugin-store ^2`, `@tauri-apps/plugin-autostart ^2`, `@crabnebula/tauri-plugin-drag ^2`.
3. devDeps: `svelte ^5`, `vite`, `@sveltejs/vite-plugin-svelte`, `tailwindcss ^4`, `@tailwindcss/vite ^4`, `typescript`, `svelte-check`, `@tauri-apps/cli ^2`.
4. vite plugins `[tailwindcss(), svelte()]`, `server.port 1420 strictPort`, `clearScreen false`; scripts dev/build/check/tauri.
5. `app.css`: `@import "tailwindcss"`; `@theme` island tokens (radius, colors, easing); html/body `bg-transparent overflow-hidden select-none`.

## Cargo (Core)
- `tauri "2"` [tray-icon, image-png], `tauri-build "2"`, `serde 1` [derive], `serde_json 1`, `thiserror 2`, `log 0.4`, `tauri-plugin-log "2"`.
- Plugins "2": store, global-shortcut, single-instance, autostart, opener; `tauri-plugin-drag` (CrabNebula) "2".
- `tokio 1` [rt, macros, sync, time, net], `crossbeam-channel 0.5`, `axum 0.8`, `reqwest` [json, rustls-tls], `dirs`, `chrono` [serde].
- `cpal`, `rustfft 6`, `wmi`, `notify`, `sysinfo`, `keyvalues-parser`, `base64`.
- `windows 0.62` features: Foundation, Foundation_Collections, Storage_Streams, Media_Control, Media_Ocr, Globalization, Graphics_Imaging, UI_Notifications, UI_Notifications_Management, Devices_Enumeration, Devices_Bluetooth, Devices_Bluetooth_Rfcomm, Devices_Bluetooth_GenericAttributeProfile, Networking_Sockets, Networking_Connectivity, System_Power, Devices_Power, Win32_Foundation, Win32_UI_WindowsAndMessaging, Win32_UI_Shell, Win32_UI_HiDpi, Win32_UI_Accessibility, Win32_UI_Input_KeyboardAndMouse, Win32_Graphics_Gdi, Win32_Graphics_Dwm, Win32_System_Com, Win32_System_Com_StructuredStorage, Win32_System_Variant, Win32_System_Registry, Win32_System_Threading, Win32_System_LibraryLoader, Win32_System_DataExchange, Win32_System_Memory, Win32_System_Ole, Win32_System_Power, Win32_Storage_Packaging_Appx, Win32_Media_Audio, Win32_Media_Audio_Endpoints, Win32_Devices_FunctionDiscovery, Win32_UI_Shell_PropertiesSystem, Win32_NetworkManagement_IpHelper, Win32_NetworkManagement_Ndis, Win32_Networking_WinSock, Win32_Globalization, implement. Add `windows-core` matching version for `#[implement]`.

## tauri.conf.json (Core)
- main window: transparent, decorations/shadow/resizable false, alwaysOnTop, skipTaskbar, focus false, visible false, 480x360, dragDropEnabled true.
- settings window created at runtime (decorated) at `index.html#settings`.
- bundle target nsis, WebView2 embedBootstrapper. Icons generated via `pnpm tauri icon`.
- capability windows [main, settings]: core:default, store:default, autostart:default, drag:default (opener used from Rust only).

## Shared contracts (Core) — `src-tauri/src/core/`
- `activity.rs`: `Priority {Ambient, Normal, High, Critical}` (Ord, serde lowercase). `ActivityKind`: 17 variants, one per feature folder (media, osd, privacy, buds, bluetooth, battery, keyboard, download, network, claude, notification, ocr, shelf, sports, timer, weather, game).
- `Activity {id, kind, priority, compact: CompactView, expanded: serde_json::Value, actions: Vec<ActionSpec>, alert: bool, ttl_ms: Option<u64>, updated_at_ms: u64}`; all structs `serde(rename_all = "camelCase")`.
- `CompactView {icon, title, subtitle: Option, progress: Option<f32 0..1>, accent: Option, badge: Option}`.
- `ActionSpec {id, label, icon: Option, style: ActionStyle{Primary,Neutral,Danger}, control: ActionControl}`; `enum ActionControl {Button, Toggle{on}, Slider{min,max,step,value}}` serde tag="type".
- `ActionRequest {activity_id, action, value: Option<Value>}`; `SourceCommand {source, command, payload: Value, reply: oneshot}`.
- `source.rs`: `trait ActivitySource: Send { fn name(&self) -> &'static str; fn start(self: Box<Self>, ctx: SourceContext); }` blocking loop.
- `SourceContext {app: AppHandle, hub: HubHandle, actions: Receiver<ActionRequest>, commands: Receiver<SourceCommand>, settings: watch::Receiver<Settings>, shutdown: Arc<AtomicBool>}` + `ctx.is_running()`.
- `spawn_source`: one std::thread per source, `CoInitializeEx(MTA)` RAII guard (`com.rs`), catch_unwind, restart once after 5s.
- `hub.rs`: `HubHandle` (Clone, crossbeam): `upsert(Activity)`, `remove(id)`, `stream(name, payload)`; ids are `"<source name>:<key>"`. Hub thread: HashMap by id, ordered by (priority, alert, updated_at) desc; ttl expiry; tracks focused id + cycle index; routes ActionRequest by id prefix and SourceCommand by name; while fullscreen-hidden only Critical becomes focused.
- `events.rs`: `activity://upsert`, `activity://remove`, `island://focus`, `island://visibility`, `settings://changed`, `stream://<source>`.
- `commands.rs`: `list_activities`, `activity_action{id,action,value}`, `source_command{source,command,payload}`, `cycle_activity{direction}`, `set_hit_regions{rects}`, `set_expanded{expanded}`, `get_settings`, `update_settings{patch}` (deep-merge, save, broadcast), `open_settings`.
- `settings.rs`: `Settings {general{hide_in_fullscreen, autostart}, hotkeys{mic_mute "Ctrl+Alt+M", ocr "Ctrl+Alt+O"}, sources: BTreeMap<String,bool>, media{lyrics, visualizer}, osd{replace_volume: true, brightness}, claude{port 47653, token}, sports{leagues ["eng.1","esp.1","uefa.champions","nba"], favorite_teams: Vec<TeamRef{sport,league,id,name}>}, timer{work_min 25, short_min 5, long_min 15}, weather{lat, lon, city: Option, units}, ocr{auto_clipboard, language}, network{show_speed}}`, serde default. `SettingsStore`: tauri-plugin-store `settings.json` key `settings`, tokio watch::Sender broadcast, managed state.
- `tray.rs`: menu Settings / Pause island / Quit.
- `lib.rs`: single-instance first, other plugins, manage Hub + SettingsStore, invoke_handler, `window::setup`, `sources::spawn_all`.

## Window (Core) — `src-tauri/src/window/`
- `styles.rs`: add `WS_EX_TOOLWINDOW|WS_EX_NOACTIVATE|WS_EX_TOPMOST`, remove `WS_EX_APPWINDOW`; show via `ShowWindow(SW_SHOWNOACTIVATE)`; Win11 `DWMWA_BORDER_COLOR = DWMWA_COLOR_NONE`.
- `no_activate.rs`: `SetWindowSubclass` top HWND, `WM_MOUSEACTIVATE → MA_NOACTIVATE`; if foreground becomes island, restore last external foreground.
- `placement.rs`: primary monitor rcMonitor/rcWork + `SHAppBarMessage(ABM_GETTASKBARPOS)` edge; bottom-center; `GetDpiForMonitor`, gap 8 DIP above taskbar, physical coords; autohide (`ABM_GETSTATE`) uses rcMonitor; re-place on WM_DISPLAYCHANGE, WM_DPICHANGED, WM_SETTINGCHANGE "TraySettings", ScaleFactorChanged.
- `click_through.rs`: 60Hz thread `GetCursorPos` vs hit rects + inner_position; `set_ignore_cursor_events` only on change.
- `zorder.rs`: `SetWinEventHook(EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT)` on a msg-loop thread; re-assert `SetWindowPos(HWND_TOPMOST, NOMOVE|NOSIZE|NOACTIVATE)` on event + 2s timer.
- `fullscreen.rs`: `SHQueryUserNotificationState` BUSY / RUNNING_D3D_FULL_SCREEN / PRESENTATION_MODE, plus fg rect == monitor rect (ignore Progman/WorkerW/Shell_TrayWnd); emit `island://visibility`; hide unless Critical focused; respect `general.hide_in_fullscreen`.
- One fixed 480x360 transparent window; the pill↔card morph is CSS only.

## Sources registry (Core)
- `sources/mod.rs`: `all_sources()` boxes 18 sources (media, visualizer, osd, privacy, buds, bluetooth, battery, keyboard, download, network, claude, notification, ocr, shelf, sports, timer, weather, game); `spawn_all` skips `settings.sources[name] == false`.
- Each stub = `sources/<name>/mod.rs` with struct + `start()` idling until shutdown.

## Frontend shell (Core)
- `src/lib/contracts.ts`: TS mirror of all contracts. `src/lib/ipc.ts`: typed invoke wrappers + listen helpers for every event.
- `src/lib/island.svelte.ts`: `$state` activities Map, focusedId, expanded, hidden; `$derived` ordered list; `cycle(dir)`.
- `src/lib/hitRegions.ts`: ResizeObserver + rAF while animating → `set_hit_regions` (CSS rect × devicePixelRatio), union pill+card.
- `src/shell/Island.svelte`: pill↔card morph via CSS transitions; wheel → `cycle_activity`; click → `set_expanded`; collapse 3s after pointerleave.
- `Pill.svelte` renders registry Compact or generic CompactView; `Card.svelte` renders Expanded + `ActionBar.svelte` (Button/Toggle/Slider → `activity_action`).
- `IdleCard.svelte`: clock + registry Launchers when no activity.
- `src/features/registry.ts`: `Record<ActivityKind, {Compact?, Expanded, Settings?, Launcher?}>` importing `src/features/<kind>/`; stub Compact/Expanded/Settings.svelte in each of 17 folders + `timer/Launcher.svelte`.
- `SettingsApp.svelte`: general/hotkeys tab + one tab per registry Settings; `App.svelte` routes on `location.hash`.

## W-A audio/media (features 1, 2, 11, 12)
- `media/session.rs`: GSMTC RequestAsync; CurrentSessionChanged, MediaProperties/PlaybackInfo/TimelineProperties events.
- `media/artwork.rs`: thumbnail OpenReadAsync → DataReader bytes → base64 data URL, cached per track.
- `media/lyrics.rs`: LRCLIB `/api/get` (artist, track, album, duration) then `/api/search`; parse LRC; User-Agent; cache.
- `media/mod.rs`: `media:current` Normal; actions prev/toggle/next → Try*Async; position + timestamp for client interpolation.
- `visualizer/capture.rs`: cpal default output device loopback input stream (fallback raw `IAudioClient` `AUDCLNT_STREAMFLAGS_LOOPBACK`), mono ring buffer, active only while playing. `spectrum.rs`: 1024 Hann FFT → 12 log bands, dB normalize + decay; `stream://visualizer` at 30fps.
- `osd/endpoint.rs`: IAudioEndpointVolume + `#[implement]` IAudioEndpointVolumeCallback; IMMNotificationClient for default change. `osd/volume_keys.rs`: WH_KEYBOARD_LL thread swallows VK_VOLUME_UP/DOWN/MUTE when replace_volume; hook only posts to a channel; 2% steps. `osd/brightness.rs`: wmi `WmiMonitorBrightnessEvent` in root\WMI. `osd/mod.rs`: `osd:volume` / `osd:brightness` High ttl 1.5s with Slider.
- `privacy/consent_store.rs`: ConsentStore microphone/webcam + NonPackaged subkeys; `LastUsedTimeStop == 0` → in use; 1s poll. `privacy/mic_mute.rs`: default capture endpoint (eConsole + eCommunications) SetMute; global-shortcut `hotkeys.mic_mute`. `privacy/mod.rs`: `privacy:mic` / `privacy:camera` Ambient with app names; mute Toggle; mute change → High ttl 2s.
- Svelte: `features/media/{Compact,Visualizer,Expanded,Settings}.svelte`, `features/osd/*`, `features/privacy/*`.

## W-B system/devices (features 7, 8, 10, 15, 16, 17)
- `buds/rfcomm.rs`: paired "Galaxy Buds*", RfcommDeviceService for SPP UUID, StreamSocket read loop. `buds/protocol.rs`: SOM 0xFD / EOM 0xDD, len+flags header, msg id, CRC16; parse 0x60 STATUS_UPDATED, 0x61 EXTENDED_STATUS; build NOISE_CONTROLS 0x78 Off/ANC/Ambient; Buds2/2 Pro/FE/3 offsets. `buds/mod.rs`: `buds:<addr>` L/R/case %, ANC actions, reconnect with backoff.
- `bluetooth/mod.rs`: DeviceWatcher on BluetoothDevice + BluetoothLEDevice selectors; ConnectionStatusChanged → High ttl 3s. `bluetooth/battery.rs`: classic DEVPKEY `{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2`; BLE GATT 0x180F/0x2A19 notify.
- `battery/mod.rs`: PowerManager events; plug-in → High ttl 3s (charging animation); ≤15% → High.
- `keyboard/mod.rs`: 200ms poll GetKeyboardLayout(fg thread) → LCIDToLocaleName short tag; GetKeyState(VK_CAPITAL); change → ttl 1.2s.
- `download/mod.rs`: FOLDERID_Downloads + notify for .crdownload/.part/.partial; 1s size sampling → speed; done on rename.
- `network/throughput.rs`: GetIfTable2 1s In/OutOctets delta (up, non-loopback, non-tunnel). `network/vpn.rs`: GetAdaptersAddresses PPP/TUNNEL/PROP_VIRTUAL or desc WireGuard/TAP/OpenVPN/Tailscale, OperStatus Up. `network/wifi.rs`: NetworkStatusChanged + GetInternetConnectionProfile WLAN SSID (SSID may need location consent → show "Wi-Fi connected"); drop → High. `network/mod.rs`: `network:status` Ambient speed + `network:event` alerts.
- Svelte: `features/{buds,bluetooth,battery,keyboard,download,network}/*` (battery has charging keyframes, buds gauges + ANC segmented).

## W-C claude/notifications/OCR/shelf (features 3, 13, 14, 18)
- `claude/server.rs`: axum on 127.0.0.1:port in own current-thread tokio runtime; `POST /hook/{event}`; token header check; non-permission events → 200 `{}` immediately + update session state.
- `claude/permission.rs`: PermissionRequest parks a oneshot per request; Allow/Deny resolves; 590s timeout → `{}` (CLI prompts normally). Reply `hookSpecificOutput {hookEventName:"PermissionRequest", decision:{behavior:"allow"|"deny"}}` — verify against current Claude Code hooks docs; unknown → `{}`.
- `claude/hooks_install.rs`: merge 7 hook events (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, Notification, Stop, PermissionRequest) into `~/.claude/settings.json`, tag entries by our URL, backup `.chocolatebar.bak`, uninstall removes only ours. Command: `curl.exe -s --max-time 600 -H <token> --data-binary @- http://127.0.0.1:<port>/hook/<Event>`, timeout 600. Never auto-install: Settings button only.
- `claude/mod.rs`: `claude:<session_id>` states thinking / tool+arg summary / waiting / done; permission → Critical alert; source_commands install_hooks / uninstall_hooks / hooks_status.
- `notification/mod.rs`: `GetCurrentPackageFullName == APPMODEL_ERROR_NO_PACKAGE` → idle, status "needs package identity". `notification/listener.rs`: packaged → RequestAccessAsync, poll GetNotificationsAsync(Toast) 2s diff → High ttl 5s.
- `ocr/clipboard_watch.rs`: message-only window + AddClipboardFormatListener; CF_DIB present and not self-written → trigger. `ocr/capture.rs`: `hotkeys.ocr` opens `ms-screenclip:`; arms 60s to OCR next clipboard image. `ocr/recognize.rs`: GetClipboardData(CF_DIB) → BGRA → SoftwareBitmap::CreateCopyFromBuffer; OcrEngine "ar" if supported & chosen else TryCreateFromUserProfileLanguages. `ocr/mod.rs`: `ocr:last` High ttl 8s with preview + Copy; hotkey path auto-copies; mark own writes.
- `shelf/mod.rs`: main window DragDrop paths → `shelf:files`; actions open (opener), remove, clear. Svelte drag-out via `@crabnebula/tauri-plugin-drag` startDrag; drop-hover glow.
- Svelte: `features/{claude,notification,ocr,shelf}/*` (claude Allow/Deny + tool input preview + Settings install/uninstall).

## W-D web data/timers/games (features 4, 5, 6, 9)
- `sports/espn.rs`: `site.api.espn.com/apis/site/v2/sports/{soccer/<league>, basketball/nba}/scoreboard`, Option-heavy serde. `sports/mod.rs`: favorites filter; poll 30s while live else 10min; score change → High ttl 6s.
- `timer/mod.rs`: countdown + stopwatch on Instant; source_commands start/pause/reset/lap. `timer/pomodoro.rs`: work → short ×3 → long from settings; phase end → Critical + MessageBeep.
- `weather/geolocate.rs`: settings lat/lon else ipapi.co/json cached 6h. `weather/open_meteo.rs`: current temperature_2m, weather_code, precipitation, wind_gusts_10m + minutely_15; 15min poll. `weather/mod.rs`: alerts codes 95–99, 61–67/80–82 soon, gusts ≥60 km/h, temp ≥42/≤−5 → High once per event.
- `game/steam.rs`: SteamPath HKCU\Software\Valve\Steam, libraryfolders.vdf, appmanifest StateFlags + BytesDownloaded/ToDownload. `game/epic.rs`: Manifests\*.item install dirs; progress from launcher log tail else indeterminate. `game/playtime.rs`: sysinfo 5s scan, exe under install dirs → session; totals in store `playtime.json`. `game/mod.rs`: `game:download:<id>` Normal progress, `game:playing` Ambient elapsed.
- Svelte: `features/{sports,timer,weather,game}/*` (timer ring/controls/Launcher, sports scorebug).

## Risks / degraded in MVP
- WebView2 may take focus on click → MA_NOACTIVATE subclass + restore prior foreground.
- Win11 native volume flyout can't be hidden by window tricks → swallow volume keys in LL hook; brightness island shows alongside native.
- LL hook unhooked if callback slow → hook only posts to channel.
- Notification mirror disabled when unpackaged (sparse MSIX deferred).
- Galaxy Buds offsets differ per model; Wearable app may hold RFCOMM → BT battery fallback + "in use".
- Epic progress and Chrome .crdownload totals → indeterminate + speed.
- ESPN/LRCLIB/ipapi.co unofficial/rate-limited → Option fields, backoff, caching.
- Out of scope: Explorer injection, MSIX packaging, auto-update, multi-monitor islands (primary only).

## Verify
`cd src-tauri && cargo clippy --all-targets -- -D warnings`, then `pnpm check`, then `pnpm build`. Manual: pill above taskbar, click-through, no focus steal, each card fires.
