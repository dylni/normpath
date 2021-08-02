use std::ffi::OsString;
use std::path::Path;

#[cfg(any(target_os = "ios", target_os = "macos"))]
mod macos;

pub(super) fn name(path: &Path) -> Option<OsString> {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    if let Some(path) = path.to_str() {
        return Some(macos::name(path).into());
    }
    None
}
