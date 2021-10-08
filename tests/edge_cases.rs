mod common;

#[cfg(windows)]
#[test]
fn test_edge_cases() {
    use std::path::Path;

    use normpath::BasePath;
    use normpath::PathExt;

    // https://github.com/dylni/normpath/pull/4#issuecomment-938596259
    tj(r"X:\X:", r"ABC", r"X:\X:\ABC");

    #[track_caller]
    fn tj(base: &str, path: &str, joined_path: &str) {
        let joined_path = Path::new(joined_path);
        assert_eq!(joined_path, BasePath::try_new(base).unwrap().join(path));
        common::assert_eq(joined_path, joined_path.normalize_virtually());
    }
}
