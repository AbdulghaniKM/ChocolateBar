use windows::core::PCWSTR;
use windows::Win32::Globalization::{GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SLOCALIZEDLANGUAGENAME};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

const LOCALE_NAME_CAPACITY: usize = 86;
const LANGUAGE_NAME_CAPACITY: usize = 80;
const LANGUAGE_ID_MASK: usize = 0xFFFF;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLayout {
    pub tag: String,
    pub name: String,
}

pub fn foreground_language_id() -> Option<u16> {
    let window = unsafe { GetForegroundWindow() };
    if window.is_invalid() {
        return None;
    }
    let thread_id = unsafe { GetWindowThreadProcessId(window, None) };
    if thread_id == 0 {
        return None;
    }
    let layout = unsafe { GetKeyboardLayout(thread_id) };
    Some((layout.0 as usize & LANGUAGE_ID_MASK) as u16)
}

pub fn describe_layout(language_id: u16) -> KeyboardLayout {
    let locale_name = locale_name_of(language_id);
    let tag = tag_of(&locale_name).unwrap_or_else(|| format!("{language_id:04X}"));
    let name = localized_language_name(&locale_name).unwrap_or_else(|| tag.clone());
    KeyboardLayout { tag, name }
}

fn locale_name_of(language_id: u16) -> Vec<u16> {
    let mut buffer = [0u16; LOCALE_NAME_CAPACITY];
    let length = unsafe { LCIDToLocaleName(u32::from(language_id), Some(&mut buffer), 0) };
    let used = usize::try_from(length).unwrap_or(0).saturating_sub(1);
    let mut name = buffer[..used].to_vec();
    name.push(0);
    name
}

fn tag_of(locale_name: &[u16]) -> Option<String> {
    let text = String::from_utf16_lossy(&locale_name[..locale_name.len() - 1]);
    let language = text.split('-').next().filter(|part| !part.is_empty())?;
    Some(language.to_uppercase())
}

fn localized_language_name(locale_name: &[u16]) -> Option<String> {
    if locale_name.len() <= 1 {
        return None;
    }
    let mut buffer = [0u16; LANGUAGE_NAME_CAPACITY];
    let length = unsafe {
        GetLocaleInfoEx(PCWSTR(locale_name.as_ptr()), LOCALE_SLOCALIZEDLANGUAGENAME, Some(&mut buffer))
    };
    let used = usize::try_from(length).ok()?.checked_sub(1).filter(|used| *used > 0)?;
    Some(String::from_utf16_lossy(&buffer[..used]))
}
