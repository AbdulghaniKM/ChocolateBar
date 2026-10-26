use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL};

const TOGGLED_BIT: i16 = 1;

pub fn is_caps_lock_on() -> bool {
    let state = unsafe { GetKeyState(i32::from(VK_CAPITAL.0)) };
    state & TOGGLED_BIT != 0
}
