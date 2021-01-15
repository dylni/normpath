use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

use normpath::BasePath;
use normpath::PathExt;

use tempfile::tempdir;

#[macro_use]
mod common;

#[cfg(not(windows))]
use libc::ENOENT as ERROR_INVALID_NAME;
#[cfg(windows)]
use winapi::shared::winerror::ERROR_INVALID_NAME;

#[test]
fn test_empty() -> io::Result<()> {
    assert_eq!(
        Some(Ok(ERROR_INVALID_NAME)),
        common::normalize("")
            .unwrap_err()
            .raw_os_error()
            .map(TryInto::try_into),
    );
    assert_eq!(
        io::ErrorKind::NotFound,
        Path::new("").normalize().unwrap_err().kind(),
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
