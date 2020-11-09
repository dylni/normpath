use std::env;
use std::fs::File;
use std::io;
use std::os::raw::c_int;
use std::path::Path;

use normpath::BasePath;
use normpath::PathExt;

use tempfile::tempdir;

#[macro_use]
mod common;

const ERROR_INVALID_NAME: c_int = {
    #[cfg(windows)]
    {
        123
    }
    #[cfg(not(windows))]
    {
        use libc::ENOENT;

        ENOENT
    }
};

#[test]
fn test_empty() -> io::Result<()> {
    assert_eq!(
        Some(ERROR_INVALID_NAME),
        Path::new("").normalize().unwrap_err().raw_os_error(),
    );

    let base = env::current_dir()?;
    assert_eq!(&base, &BasePath::try_new(&base).unwrap().join(""));

    Ok(())
}

#[test]
fn test_created() -> io::Result<()> {
    let dir = tempdir()?;
    let dir = dir.path().normalize().unwrap();

    let file = dir.as_path().join("foo");
    let _ = File::create(&file)?;

    assert_eq!(file, dir.join("foo"));
    common::assert_eq(&file, file.normalize());

    Ok(())
}
