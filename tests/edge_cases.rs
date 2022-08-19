mod common;

#[cfg(windows)]
#[test]
fn test_windows() {
    use std::path::Path;

    use normpath::PathExt;

    #[track_caller]
    fn test(base: &str, path: &str, result: &str) {
        common::test_join(base, path, result);

        let result = Path::new(result);
        common::assert_eq(result, result.normalize_virtually());
    }

    // https://github.com/dylni/normpath/pull/4#issuecomment-938596259
    test(r"X:\X:", r"ABC", r"X:\X:\ABC");
    test(r"\\?\X:\X:", r"ABC", r"\\?\X:\X:\ABC");
}
