use std::ffi::OsString;
use std::mem;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;

use windows_sys::Win32::UI::Shell::SHGetFileInfoW;
use windows_sys::Win32::UI::Shell::SHGFI_DISPLAYNAME;

pub(super) fn name(path: &Path) -> Option<OsString> {
    let mut path: Vec<_> = path.as_os_str().encode_wide().collect();
    if path.contains(&0) {
        return None;
    }
    path.push(0);

    let mut path_info = MaybeUninit::uninit();
    let result = unsafe {
        SHGetFileInfoW(
            path.as_ptr(),
            0,
            path_info.as_mut_ptr(),
            mem::size_of_val(&path_info) as _,
            SHGFI_DISPLAYNAME,
        )
    };
    if result == 0 {
        return None;
    }

    // SAFETY: This struct was initialized by the syscall.
    let display_name = unsafe { path_info.assume_init() }.szDisplayName;
    // The display name buffer has a fixed length, so it must be truncated at
    // the first null character.
    Some(OsString::from_wide(
        display_name
            .split(|&x| x == 0)
            .next()
            .expect("missing null byte in display name"),
    ))
}
