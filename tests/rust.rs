//! Tests copied and modified from The Rust Programming Language.
//!
//! Sources:
//! - <https://github.com/rust-lang/rust/pull/47363/files#diff-2fddd6220cb2006f2e025a3a682366ec85198a4c192e4e8c479bf72be6f2aa5aR4237-R4264>
//! - <https://github.com/rust-lang/rust/blob/b1277d04db0dc8009037e872a1be7cdc2bd74a43/library/std/src/path/tests.rs#L1004-L1037>
//!
//! Copyrights:
//! - Copyrights in the Rust project are retained by their contributors. No
//!   copyright assignment is required to contribute to the Rust project.
//!
//!   Some files include explicit copyright notices and/or license notices.
//!   For full authorship information, see the version control history or
//!   <https://thanks.rust-lang.org>
//!
//!   <https://github.com/rust-lang/rust/blob/b1277d04db0dc8009037e872a1be7cdc2bd74a43/COPYRIGHT>
//! - Modifications copyright (c) 2020 dylni (<https://github.com/dylni>)<br>
//!   <https://github.com/dylni/normpath/blob/master/COPYRIGHT>

#[macro_use]
mod common;

#[test]
fn test_simple() {
    if cfg!(windows) {
        test!(r"a\b\c", r"X:\ABC\a\b\c", SAME);
        test!(r"a/b\c", r"X:\ABC\a\b\c", SAME);
        test!(r"a/b\c\", r"X:\ABC\a\b\c\", SAME);
        test!(r"a/b\c/", r"X:\ABC\a\b\c\", SAME);
        test!(r"\\", r"X:\\", r"X:\");
        test!(r"/", r"X:\", SAME);
        test!(r"//", r"X:\\", r"X:\");

        test!(r"C:\a\b", SAME, SAME);
        test!(r"C:\", SAME, SAME);
        test!(r"C:\.", SAME, r"C:\");
        test!(r"C:\..", SAME, r"C:\");

        test!(r"\\server\share\a\b", SAME, SAME);
        test!(r"\\server\share\a\.\b", SAME, r"\\server\share\a\b");
        test!(r"\\server\share\a\..\b", SAME, r"\\server\share\b");
        test!(r"\\server\share\a\b\", SAME, SAME);

        test!(r"\\?\a\b", SAME, SAME);
        test!(r"\\?\a/\\b\", SAME, SAME);
        test!(r"\\?\a/\\b/", SAME, SAME);
        test!(r"\\?\a\b", SAME, SAME);
    } else {
        test!("/", SAME, SAME);
        test!("//", SAME, "/");

        test!("/.", SAME, "/");
        test!("/..", SAME, "/");
        test!("/../../", SAME, "/");

        if cfg!(target_os = "macos") {
            test!(".", "/tmp/.", "/private/tmp");
            test!("..", "/tmp/..", "/private");
            test!("/tmp", SAME, "/private/tmp");
            test!("//tmp", SAME, "/private/tmp");
            test!("../tmp/", "/tmp/../tmp/", "/private/tmp");
            test!("../tmp/../tmp/../", "/tmp/../tmp/../tmp/../", "/private");
        } else {
            test!(".", "/tmp/.", "/tmp");
            test!("..", "/tmp/..", "/");
            test!("/tmp", SAME, SAME);
            test!("//tmp", SAME, "/tmp");
            test!("../tmp/", "/tmp/../tmp/", "/tmp");
            test!("../tmp/../tmp/../", "/tmp/../tmp/../tmp/../", "/");
        }
    }
}

#[cfg(windows)]
#[test]
fn test_complex() {
    use common::test_join;

    test_join(r"c:\", r"windows", r"c:\windows");
    test_join(r"c:", r"windows", r"c:windows");

    test_join(r"C:\a", r"C:\b.txt", r"C:\b.txt");
    test_join(r"C:\a\b\c", "C:d", r"C:\a\b\c\d");
    test_join(r"C:a\b\c", "C:d", r"C:a\b\c\d");
    test_join(r"C:", r"a\b\c", r"C:a\b\c");
    test_join(r"C:", r"..\a", r"C:..\a");
    test_join(r"\\server\share\foo", "bar", r"\\server\share\foo\bar");
    test_join(r"\\server\share\foo", "C:baz", "C:baz");
    test_join(r"\\?\C:\a\b", r"C:c\d", r"C:c\d");
    test_join(r"\\?\C:a\b", r"C:c\d", r"C:c\d");
    test_join(r"\\?\C:\a\b", r"C:\c\d", r"C:\c\d");
    test_join(r"\\?\foo\bar", "baz", r"\\?\foo\bar\baz");
    test_join(
        r"\\?\UNC\server\share\foo",
        "bar",
        r"\\?\UNC\server\share\foo\bar",
    );
    test_join(r"\\?\UNC\server\share", r"C:\a", r"C:\a");
    test_join(r"\\?\UNC\server\share", "C:a", "C:a");

    test_join(r"\\?\UNC\server", "foo", r"\\?\UNC\server\foo");

    test_join(r"C:\a", r"\\?\UNC\server\share", r"\\?\UNC\server\share");
    test_join(r"\\.\foo\bar", "baz", r"\\.\foo\bar\baz");
    test_join(r"\\.\foo\bar", "C:a", "C:a");
    test_join(r"\\.\foo", r"..\bar", r"\\.\foo\bar");
}
