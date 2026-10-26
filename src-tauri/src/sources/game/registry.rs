use windows::core::PCWSTR;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};

pub fn read_current_user_string(subkey: PCWSTR, value: PCWSTR) -> Option<String> {
    let mut byte_length = 0u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey,
            value,
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut byte_length),
        )
    }
    .ok()
    .ok()?;
    let mut buffer = vec![0u16; byte_length as usize / 2 + 1];
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey,
            value,
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut byte_length),
        )
    }
    .ok()
    .ok()?;
    Some(String::from_utf16_lossy(&buffer).trim_end_matches('\0').to_string())
}
