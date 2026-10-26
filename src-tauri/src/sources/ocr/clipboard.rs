use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::{CF_DIB, CF_UNICODETEXT};

const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY_DELAY: Duration = Duration::from_millis(30);

static OWN_WRITE_SEQUENCE: AtomicU32 = AtomicU32::new(0);

struct OpenClipboardGuard;

impl OpenClipboardGuard {
    fn acquire(owner: Option<HWND>) -> Option<Self> {
        for _ in 0..OPEN_ATTEMPTS {
            if unsafe { OpenClipboard(owner) }.is_ok() {
                return Some(Self);
            }
            thread::sleep(OPEN_RETRY_DELAY);
        }
        None
    }
}

impl Drop for OpenClipboardGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

pub fn current_sequence() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

pub fn has_image() -> bool {
    unsafe { IsClipboardFormatAvailable(u32::from(CF_DIB.0)) }.is_ok()
}

pub fn is_own_write(sequence: u32) -> bool {
    OWN_WRITE_SEQUENCE.load(Ordering::Relaxed) == sequence
}

pub fn read_image_bytes() -> Option<Vec<u8>> {
    let _clipboard = OpenClipboardGuard::acquire(None)?;
    let handle = unsafe { GetClipboardData(u32::from(CF_DIB.0)) }.ok()?;
    let memory = HGLOBAL(handle.0);
    let size = unsafe { GlobalSize(memory) };
    let locked = unsafe { GlobalLock(memory) };
    if locked.is_null() || size == 0 {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(locked.cast::<u8>(), size) }.to_vec();
    let _ = unsafe { GlobalUnlock(memory) };
    Some(bytes)
}

pub fn write_text(text: &str) -> bool {
    let Some(clipboard) = OpenClipboardGuard::acquire(None) else {
        return false;
    };
    if unsafe { EmptyClipboard() }.is_err() {
        return false;
    }
    let stored = store_unicode_text(text);
    drop(clipboard);
    if stored {
        OWN_WRITE_SEQUENCE.store(current_sequence(), Ordering::Relaxed);
    }
    stored
}

fn store_unicode_text(text: &str) -> bool {
    let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let byte_count = units.len() * std::mem::size_of::<u16>();
    let Ok(memory) = (unsafe { GlobalAlloc(GMEM_MOVEABLE, byte_count) }) else {
        return false;
    };
    let locked = unsafe { GlobalLock(memory) };
    if locked.is_null() {
        return false;
    }
    unsafe { std::ptr::copy_nonoverlapping(units.as_ptr().cast::<u8>(), locked.cast::<u8>(), byte_count) };
    let _ = unsafe { GlobalUnlock(memory) };
    unsafe { SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))) }.is_ok()
}
