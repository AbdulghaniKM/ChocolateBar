use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

pub struct ComGuard;

impl ComGuard {
    pub fn initialize_multithreaded() -> Option<Self> {
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        result.is_ok().then_some(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
