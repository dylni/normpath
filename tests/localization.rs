#![cfg(feature = "localization")]

use std::path::Path;

use normpath::PathExt;

#[track_caller]
fn test(result: &str, path: &str) {
    assert_eq!(result, &*Path::new(path).localize_name());
}

#[test]
fn test_same() {
    test("Applications", "Applications");
    test("applications", "/foo/applications");
    test("applications", "/foo/applications/");
    test("applications", "/foo/applications//");

    test("test\0.rs", "/foo/bar/test\0.rs");
    if cfg!(unix) {
        test("test\\.rs", "/foo/bar/test\\.rs");
    }
}

#[should_panic = "path ends with a `..` component: \"/foo/bar/..\""]
#[test]
fn test_parent() {
    let _ = Path::new("/foo/bar/..").localize_name();
}

#[cfg(windows)]
#[should_panic = r#"path ends with a `..` component: "X:\foo\bar\.."#]
#[test]
fn test_windows_parent() {
    let _ = Path::new(r"X:\foo\bar\..").localize_name();
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
#[test]
fn test_localized() {
    test("Applications", "/applications");
    test("foo/bar", "foo:bar");
    test("foo/bar", "/foo:bar");
}

#[cfg(not(windows))]
#[test]
fn test_invalid() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = OsStr::from_bytes(&[0x66, 0x6F, 0x80, 0x6F]);
    assert_eq!(path, Path::new(path).localize_name());
}
