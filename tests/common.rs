#![allow(dead_code)]
#![allow(unused_macros)]

use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::io;
use std::path::Path;

use normpath::BasePath;
use normpath::BasePathBuf;
use normpath::PathExt;

// https://github.com/rust-lang/rust/issues/76483
#[track_caller]
pub(crate) fn assert_eq<P>(expected: &Path, result: io::Result<P>)
where
    P: AsRef<Path>,
{
    struct Wrapper<'a>(&'a Path);

    impl Debug for Wrapper<'_> {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            formatter.debug_tuple("Ok").field(&self.0).finish()
        }
    }

    impl PartialEq<Result<&Path, &io::Error>> for Wrapper<'_> {
        fn eq(&self, other: &Result<&Path, &io::Error>) -> bool {
            other
                .map(|x| self.0.as_os_str() == x.as_os_str())
                .unwrap_or(false)
        }
    }

    assert_eq!(Wrapper(expected), result.as_ref().map(AsRef::as_ref));
}

pub(crate) fn normalize<P>(path: P) -> io::Result<BasePathBuf>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    #[cfg(windows)]
    {
        path.normalize_virtually()
    }
    #[cfg(not(windows))]
    {
        path.normalize()
    }
}

#[track_caller]
pub(crate) fn test(path: &str, joined_path: &str, normalized_path: &str) {
    let joined_path = Path::new(joined_path);
    let normalized_path = Path::new(normalized_path);

    let base =
        BasePath::try_new(if cfg!(windows) { r"X:\ABC" } else { "/tmp" })
            .unwrap();
    assert_eq!(joined_path, base.join(path));

    assert_eq(normalized_path, normalize(joined_path));
    assert_eq(normalized_path, normalize(normalized_path));
}

macro_rules! test {
    ( $path:literal , $joined_path:literal , $normalized_path:literal ) => {
        $crate::common::test($path, $joined_path, $normalized_path);
    };
    ( $path:literal , SAME, $normalized_path:literal ) => {
        test!($path, $path, $normalized_path);
    };
    ( $path:literal , $joined_path:literal , SAME ) => {
        test!($path, $joined_path, $joined_path);
    };
    ( $path:literal , SAME, SAME ) => {
        test!($path, $path, $path);
    };
}
