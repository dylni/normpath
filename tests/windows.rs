// https://googleprojectzero.blogspot.com/2016/02/the-definitive-guide-on-win32-to-nt.html

#![cfg(windows)]

use std::env;
use std::io;
use std::path::Path;

use normpath::PathExt;

#[macro_use]
mod common;

#[test]
fn test_drive_absolute() {
    test!(r"X:\ABC\DEF", SAME, SAME);
    test!(r"X:\", SAME, SAME);
    test!(r"X:\ABC\", SAME, SAME);
    test!(r"X:\ABC\DEF. .", SAME, r"X:\ABC\DEF");
    test!(r"X:/ABC/DEF", SAME, r"X:\ABC\DEF");
    test!(r"X:\ABC\..\XYZ", SAME, r"X:\XYZ");
    test!(r"X:\ABC\..\..\..", SAME, r"X:\");
}

#[test]
fn test_drive_relative() {
    test!(r"X:DEF\GHI", r"X:\ABC\DEF\GHI", SAME);
    test!(r"X:", r"X:\ABC", SAME);
    test!(r"X:DEF. .", r"X:\ABC\DEF. .", r"X:\ABC\DEF");
    test!(r"Y:", SAME, r"Y:\");
    test!(r"Z:", SAME, r"Z:\");
    test!(r"X:ABC\..\XYZ", r"X:\ABC\ABC\..\XYZ", r"X:\ABC\XYZ");
    test!(r"X:ABC\..\..\..", r"X:\ABC\ABC\..\..\..", r"X:\");
}

#[test]
fn test_rooted() {
    test!(r"\ABC\DEF", r"X:\ABC\DEF", SAME);
    test!(r"\", r"X:\", SAME);
    test!(r"\ABC\DEF. .", r"X:\ABC\DEF. .", r"X:\ABC\DEF");
    test!(r"/ABC/DEF", r"X:\ABC\DEF", SAME);
    test!(r"\ABC\..\XYZ", r"X:\ABC\..\XYZ", r"X:\XYZ");
    test!(r"\ABC\..\..\..", r"X:\ABC\..\..\..", r"X:\");
}

#[test]
fn test_relative() {
    test!(r"XYZ\DEF", r"X:\ABC\XYZ\DEF", SAME);
    test!(r".", r"X:\ABC", SAME);
    test!(r"XYZ\DEF. .", r"X:\ABC\XYZ\DEF. .", r"X:\ABC\XYZ\DEF");
    test!(r"XYZ/DEF", r"X:\ABC\XYZ\DEF", SAME);
    test!(r"..\XYZ", r"X:\XYZ", SAME);
    test!(r"XYZ\..\..\..", r"X:\ABC\XYZ\..\..\..", r"X:\");
}

#[test]
fn test_unc_absolute() {
    test!(r"\\server\share\ABC\DEF", SAME, SAME);
    test!(r"\\server\share", SAME, SAME);
    test!(r"\\server\share\ABC. .", SAME, r"\\server\share\ABC");
    test!(r"//server/share/ABC/DEF", SAME, r"\\server\share\ABC\DEF");
    test!(r"\\server\share\ABC\..\XYZ", SAME, r"\\server\share\XYZ");
    test!(r"\\server\share\ABC\..\..\..", SAME, r"\\server\share");

    assert_eq!(
        "partial UNC prefixes are invalid",
        Path::new(r"\\server")
            .normalize_virtually()
            .unwrap_err()
            .to_string(),
    );
}

#[test]
fn test_local_device() {
    test!(r"\\.\COM20", SAME, SAME);
    test!(r"\\.\pipe\mypipe", SAME, SAME);
    test!(r"\\.\X:\ABC\DEF. .", SAME, r"\\.\X:\ABC\DEF");
    test!(r"\\.\X:/ABC/DEF", SAME, r"\\.\X:\ABC\DEF");
    test!(r"\\.\X:\ABC\..\XYZ", SAME, r"\\.\X:\XYZ");
    test!(r"\\.\X:\ABC\..\..\C:\", SAME, r"\\.\C:\");
    test!(r"\\.\pipe\mypipe\..\notmine", SAME, r"\\.\pipe\notmine");

    test!(r"COM1", r"X:\ABC\COM1", r"\\.\COM1");
    test!(r"X:\COM1", SAME, r"\\.\COM1");
    test!(r"X:COM1", r"X:\ABC\COM1", r"\\.\COM1");
    test!(r"valid\COM1", r"X:\ABC\valid\COM1", r"\\.\COM1");
    test!(r"X:\notvalid\COM1", SAME, r"\\.\COM1");
    test!(r"X:\COM1.blah", SAME, r"\\.\COM1");
    test!(r"X:\COM1:blah", SAME, r"\\.\COM1");
    test!(r"X:\COM1  .blah", SAME, r"\\.\COM1");
    test!(r"\\.\X:\COM1", SAME, SAME);
    test!(r"\\abc\xyz\COM1", SAME, SAME);
}

#[test]
fn test_root_local_device() {
    test!(r"\\?\X:\ABC\DEF", SAME, SAME);
    test!(r"\\?\X:\", SAME, SAME);
    test!(r"\\?\X:", SAME, SAME);
    test!(r"\\?\X:\COM1", SAME, SAME);
    test!(r"\\?\X:\ABC\DEF. .", SAME, SAME);
    test!(r"\\?\X:/ABC/DEF", SAME, SAME);
    test!(r"\\?\X:\ABC\..\XYZ", SAME, SAME);
    test!(r"\\?\X:\ABC\..\..\..", SAME, SAME);

    // This prefix is not parsed by the standard library:
    // https://github.com/rust-lang/rust/issues/56030
    //
    // test!(r"\??\X:\ABC\DEF", SAME, SAME);
    // test!(r"\??\X:\", SAME, SAME);
    // test!(r"\??\X:", SAME, SAME);
    // test!(r"\??\X:\COM1", SAME, SAME);
    // test!(r"\??\X:\ABC\DEF. .", SAME, SAME);
    // test!(r"\??\X:/ABC/DEF", SAME, SAME);
    // test!(r"\??\X:\ABC\..\XYZ", SAME, SAME);
    // test!(r"\??\X:\ABC\..\..\..", SAME, SAME);
}

#[test]
fn test_edge_cases() {
    test!(r"//?/X:/ABC/DEF", SAME, r"\\?\X:\ABC\DEF");
    test!(r"//?/X:/", SAME, r"\\?\X:\");
    test!(r"//?/X:", SAME, r"\\?\X:");

    // This prefix is not parsed by the standard library:
    // https://github.com/rust-lang/rust/issues/56030
    //
    // test!(r"/??/X:/ABC/DEF", SAME, r"\??\X:\ABC\DEF");
    // test!(r"/??/X:/", SAME, r"\??\X:\");
    // test!(r"/??/X:", SAME, r"\??\X:");
}

// https://github.com/dylni/normpath/issues/5
#[test]
fn test_windows_bug() -> io::Result<()> {
    let initial_current_dir = env::current_dir()?;

    for current_dir in [r"C:\", r"C:\Users"] {
        let current_dir = Path::new(current_dir);
        env::set_current_dir(current_dir)?;
        common::assert_eq(current_dir, env::current_dir());

        common::assert_eq(current_dir, Path::new(".").normalize());
    }

    env::set_current_dir(initial_current_dir)
}
