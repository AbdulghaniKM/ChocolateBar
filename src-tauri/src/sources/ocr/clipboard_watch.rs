use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};

use crossbeam_channel::Sender;
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{AddClipboardFormatListener, RemoveClipboardFormatListener};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW,
    RegisterClassExW, TranslateMessage, HWND_MESSAGE, MSG, PM_REMOVE, QS_ALLINPUT, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLIPBOARDUPDATE, WNDCLASSEXW,
};

use super::clipboard;

const WAKE_INTERVAL_MS: u32 = 200;

static IMAGE_SINK: Mutex<Option<Sender<u32>>> = Mutex::new(None);

pub struct ClipboardWatcher {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ClipboardWatcher {
    pub fn start(images: Sender<u32>) -> Self {
        *IMAGE_SINK.lock().unwrap_or_else(PoisonError::into_inner) = Some(images);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let spawned = thread::Builder::new()
            .name("ocr-clipboard-watch".into())
            .spawn(move || run_message_loop(&thread_stop));
        let thread = spawned.map_err(|error| log::error!("clipboard watcher failed to spawn: {error}")).ok();
        Self { stop, thread }
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        *IMAGE_SINK.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

fn run_message_loop(stop: &AtomicBool) {
    let window = match create_message_window() {
        Ok(window) => window,
        Err(error) => return log::error!("clipboard message window failed: {error}"),
    };
    if let Err(error) = unsafe { AddClipboardFormatListener(window) } {
        log::error!("clipboard listener registration failed: {error}");
    }
    while !stop.load(Ordering::Relaxed) {
        unsafe { MsgWaitForMultipleObjects(None, false, WAKE_INTERVAL_MS, QS_ALLINPUT) };
        pump_pending_messages();
    }
    let _ = unsafe { RemoveClipboardFormatListener(window) };
    let _ = unsafe { DestroyWindow(window) };
}

fn pump_pending_messages() {
    let mut message = MSG::default();
    while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

fn create_message_window() -> windows::core::Result<HWND> {
    let instance: HINSTANCE = unsafe { GetModuleHandleW(None)? }.into();
    let class_name = w!("ChocolateBarOcrClipboardWatcher");
    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(window_procedure),
        hInstance: instance,
        lpszClassName: class_name,
        ..Default::default()
    };
    unsafe { RegisterClassExW(&class) };
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!(""),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance),
            None,
        )
    }
}

unsafe extern "system" fn window_procedure(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_CLIPBOARDUPDATE {
        report_image_if_present();
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

fn report_image_if_present() {
    let sequence = clipboard::current_sequence();
    if !clipboard::has_image() || clipboard::is_own_write(sequence) {
        return;
    }
    let sink = IMAGE_SINK.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(sender) = sink.as_ref() {
        let _ = sender.send(sequence);
    }
}
