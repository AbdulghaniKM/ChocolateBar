use windows::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

pub fn has_package_identity() -> bool {
    let mut length = 0u32;
    let result = unsafe { GetCurrentPackageFullName(&mut length, None) };
    result != APPMODEL_ERROR_NO_PACKAGE
}
