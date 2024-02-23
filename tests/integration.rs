use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

use normpath::BasePath;
use normpath::PathExt;

use tempfile::tempdir;

#[macro_use]
mod common;

#[cfg(windows)]
#[rustfmt::skip]
use windows_sys::Win32::Foundation::ERROR_INVALID_NAME;
#[cfg(not(windows))]
use libc::ENOENT as ERROR_INVALID_NAME;

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

#[cfg(feature = "serde")]
#[test]
fn test_serde() -> io::Result<()> {
    use normpath::BasePathBuf;

    // https://doc.rust-lang.org/std/ffi/struct.OsStr.html#examples-2
    let path = {
        #[cfg(windows)]
        {
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;

            OsString::from_wide(&[0x66, 0x66, 0xD800, 0x6F])
        }
        #[cfg(not(windows))]
        {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            OsStr::from_bytes(&[0x66, 0x66, 0x80, 0x6F]).to_owned()
        }
    };

    let base = BasePathBuf::new(path)?;
    let bytes = bincode::serialize(&base).unwrap();
    assert_eq!(base, bincode::deserialize::<BasePathBuf>(&bytes).unwrap());

    Ok(())
}
