use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_QWORD,
    REG_VALUE_TYPE,
};
use windows_core::{HSTRING, PWSTR};

const MAX_KEY_NAME_CHARS: usize = 256;

pub struct RegistryKey(HKEY);

impl RegistryKey {
    pub fn open_current_user(path: &str) -> Option<Self> {
        open_under(HKEY_CURRENT_USER, path)
    }

    pub fn open_subkey(&self, name: &str) -> Option<Self> {
        open_under(self.0, name)
    }

    pub fn subkey_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        while let Some(name) = self.subkey_name_at(names.len() as u32) {
            names.push(name);
        }
        names
    }

    pub fn read_qword(&self, name: &str) -> Option<u64> {
        let mut value = 0u64;
        let mut size = std::mem::size_of::<u64>() as u32;
        let mut kind = REG_VALUE_TYPE::default();
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                &HSTRING::from(name),
                None,
                Some(&mut kind),
                Some((&mut value as *mut u64).cast::<u8>()),
                Some(&mut size),
            )
        };
        (status == ERROR_SUCCESS && kind == REG_QWORD).then_some(value)
    }

    fn subkey_name_at(&self, index: u32) -> Option<String> {
        let mut buffer = [0u16; MAX_KEY_NAME_CHARS];
        let mut length = buffer.len() as u32;
        let status = unsafe {
            RegEnumKeyExW(self.0, index, Some(PWSTR(buffer.as_mut_ptr())), &mut length, None, None, None, None)
        };
        (status == ERROR_SUCCESS).then(|| String::from_utf16_lossy(&buffer[..length as usize]))
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

fn open_under(parent: HKEY, path: &str) -> Option<RegistryKey> {
    let mut handle = HKEY::default();
    let status = unsafe { RegOpenKeyExW(parent, &HSTRING::from(path), None, KEY_READ, &mut handle) };
    (status == ERROR_SUCCESS).then_some(RegistryKey(handle))
}
