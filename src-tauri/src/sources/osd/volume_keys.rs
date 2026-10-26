use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HC_ACTION, KBDLLHOOKSTRUCT, MSG, PM_NOREMOVE, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN, WM_USER,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeKey {
    Up,
    Down,
    Mute,
}

static KEY_CHANNEL: OnceLock<(Sender<VolumeKey>, Receiver<VolumeKey>)> = OnceLock::new();
static SWALLOWING: AtomicBool = AtomicBool::new(false);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

fn key_channel() -> &'static (Sender<VolumeKey>, Receiver<VolumeKey>) {
    KEY_CHANNEL.get_or_init(unbounded)
}

pub fn key_receiver() -> Receiver<VolumeKey> {
    key_channel().1.clone()
}

pub struct VolumeKeyHook {
    thread: Option<JoinHandle<()>>,
}

impl VolumeKeyHook {
    pub fn install() -> Self {
        let spawned = thread::Builder::new().name("osd-volume-keys".into()).spawn(run_hook_thread);
        if let Err(error) = &spawned {
            log::warn!("volume key hook thread failed to start: {error}");
        }
        Self { thread: spawned.ok() }
    }

    pub fn set_swallowing(&self, swallowing: bool) {
        SWALLOWING.store(swallowing, Ordering::Relaxed);
    }
}

impl Drop for VolumeKeyHook {
    fn drop(&mut self) {
        SWALLOWING.store(false, Ordering::Relaxed);
        let thread_id = HOOK_THREAD_ID.swap(0, Ordering::Relaxed);
        if thread_id != 0 {
            let _ = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run_hook_thread() {
    let Some(hook) = install_keyboard_hook() else { return };
    create_message_queue();
    HOOK_THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {}
    let _ = unsafe { UnhookWindowsHookEx(hook) };
}

fn install_keyboard_hook() -> Option<windows::Win32::UI::WindowsAndMessaging::HHOOK> {
    let module = unsafe { GetModuleHandleW(None) }.ok()?;
    let installed =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), Some(HINSTANCE(module.0)), 0) };
    match installed {
        Ok(hook) => Some(hook),
        Err(error) => {
            log::warn!("volume key hook install failed: {error}");
            None
        }
    }
}

fn create_message_queue() {
    let mut message = MSG::default();
    let _ = unsafe { PeekMessageW(&mut message, None, WM_USER, WM_USER, PM_NOREMOVE) };
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && SWALLOWING.load(Ordering::Relaxed) {
        let event = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if let Some(key) = volume_key_of(event.vkCode) {
            if is_key_down(wparam) {
                let _ = key_channel().0.send(key);
            }
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn volume_key_of(virtual_key: u32) -> Option<VolumeKey> {
    match virtual_key {
        key if key == VK_VOLUME_UP.0 as u32 => Some(VolumeKey::Up),
        key if key == VK_VOLUME_DOWN.0 as u32 => Some(VolumeKey::Down),
        key if key == VK_VOLUME_MUTE.0 as u32 => Some(VolumeKey::Mute),
        _ => None,
    }
}

fn is_key_down(wparam: WPARAM) -> bool {
    matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN)
}
