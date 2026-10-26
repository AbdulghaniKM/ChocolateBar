use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::{CF_DIB, CF_HDROP, CF_UNICODETEXT};
use windows::Win32::UI::Shell::{DragQueryFileW, DROPFILES, HDROP};

const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY_DELAY: Duration = Duration::from_millis(30);
const ALL_FILES: u32 = u32::MAX;

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

fn is_available(format: u32) -> bool {
    unsafe { IsClipboardFormatAvailable(format) }.is_ok()
}

pub fn has_image() -> bool {
    is_available(u32::from(CF_DIB.0))
}

pub fn is_own_write(sequence: u32) -> bool {
    OWN_WRITE_SEQUENCE.load(Ordering::Relaxed) == sequence
}

/// Password managers and other apps flag secrets with these formats so clipboard history skips them.
pub fn is_private() -> bool {
    let excluded = [w!("ExcludeClipboardContentFromMonitorProcessing"), w!("Clipboard Viewer Ignore")];
    if excluded.into_iter().any(|name| is_available(registered_format(name))) {
        return true;
    }
    let history_flag = registered_format(w!("CanIncludeInClipboardHistory"));
    is_available(history_flag) && read_bytes(history_flag).is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0))
}

fn registered_format(name: PCWSTR) -> u32 {
    unsafe { RegisterClipboardFormatW(name) }
}

fn read_bytes(format: u32) -> Option<Vec<u8>> {
    let _clipboard = OpenClipboardGuard::acquire(None)?;
    let handle = unsafe { GetClipboardData(format) }.ok()?;
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

pub fn read_image_bytes() -> Option<Vec<u8>> {
    read_bytes(u32::from(CF_DIB.0))
}

pub fn read_text() -> Option<String> {
    if !is_available(u32::from(CF_UNICODETEXT.0)) {
        return None;
    }
    let bytes = read_bytes(u32::from(CF_UNICODETEXT.0))?;
    let units: Vec<u16> = bytes.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
    let end = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
    Some(String::from_utf16_lossy(&units[..end]))
}

pub fn read_files() -> Option<Vec<String>> {
    if !is_available(u32::from(CF_HDROP.0)) {
        return None;
    }
    let _clipboard = OpenClipboardGuard::acquire(None)?;
    let handle = unsafe { GetClipboardData(u32::from(CF_HDROP.0)) }.ok()?;
    let drop = HDROP(handle.0);
    let count = unsafe { DragQueryFileW(drop, ALL_FILES, None) };
    let paths = (0..count)
        .filter_map(|index| {
            let length = unsafe { DragQueryFileW(drop, index, None) } as usize;
            let mut buffer = vec![0u16; length + 1];
            let copied = unsafe { DragQueryFileW(drop, index, Some(&mut buffer)) } as usize;
            (copied > 0).then(|| String::from_utf16_lossy(&buffer[..copied]))
        })
        .collect::<Vec<_>>();
    (!paths.is_empty()).then_some(paths)
}

pub fn write_text(text: &str) -> bool {
    let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes: Vec<u8> = units.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    write(u32::from(CF_UNICODETEXT.0), &bytes)
}

pub fn write_image_dib(dib: &[u8]) -> bool {
    write(u32::from(CF_DIB.0), dib)
}

pub fn write_files(paths: &[String]) -> bool {
    write(u32::from(CF_HDROP.0), &drop_files_payload(paths))
}

fn drop_files_payload(paths: &[String]) -> Vec<u8> {
    let header_size = std::mem::size_of::<DROPFILES>();
    let mut bytes = vec![0u8; header_size];
    bytes[0..4].copy_from_slice(&(header_size as u32).to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    for path in paths {
        bytes.extend(path.encode_utf16().chain(std::iter::once(0)).flat_map(u16::to_le_bytes));
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

fn write(format: u32, bytes: &[u8]) -> bool {
    let Some(clipboard) = OpenClipboardGuard::acquire(None) else {
        return false;
    };
    if unsafe { EmptyClipboard() }.is_err() {
        return false;
    }
    let stored = store(format, bytes);
    drop(clipboard);
    if stored {
        OWN_WRITE_SEQUENCE.store(current_sequence(), Ordering::Relaxed);
    }
    stored
}

fn store(format: u32, bytes: &[u8]) -> bool {
    let Ok(memory) = (unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len()) }) else {
        return false;
    };
    let locked = unsafe { GlobalLock(memory) };
    if locked.is_null() {
        return false;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), locked.cast::<u8>(), bytes.len()) };
    let _ = unsafe { GlobalUnlock(memory) };
    unsafe { SetClipboardData(format, Some(HANDLE(memory.0))) }.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_files_payload_is_a_wide_double_null_terminated_list() {
        let payload = drop_files_payload(&["C:\\a".into(), "D:\\b".into()]);
        assert_eq!(u32::from_le_bytes(payload[0..4].try_into().unwrap()), 20);
        assert_eq!(u32::from_le_bytes(payload[16..20].try_into().unwrap()), 1);
        let units: Vec<u16> = payload[20..].chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        assert_eq!(String::from_utf16_lossy(&units), "C:\\a\0D:\\b\0\0");
    }
}
