use std::borrow::Cow;
use std::env;
use std::ffi::OsString;
use std::io;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Component;
use std::path::Path;
use std::path::Prefix;
use std::path::PrefixComponent;
use std::ptr;

use windows_sys::Win32::Storage::FileSystem::GetFullPathNameW;

use crate::BasePath;
use crate::BasePathBuf;

macro_rules! static_assert {
    ( $condition:expr ) => {
        const _: () = assert!($condition, "static assertion failed");
    };
}

const SEPARATOR: u8 = b'\\';

pub(crate) fn is_base(path: &Path) -> bool {
    matches!(path.components().next(), Some(Component::Prefix(_)))
}

pub(crate) fn to_base(path: &Path) -> io::Result<BasePathBuf> {
    let base = env::current_dir()?;
    debug_assert!(is_base(&base));

    let mut base = BasePathBuf(base);
    base.push(path);
    Ok(base)
}

fn convert_separators(path: &Path, length: Option<usize>) -> Cow<'_, Path> {
    let path_bytes = path.as_os_str().as_encoded_bytes();
    let (prefix, suffix) = length
        .map(|x| path_bytes.split_at(x))
        .unwrap_or((path_bytes, &[]));
    let mut parts = prefix.split(|&x| x == b'/');

    let part = parts.next().expect("split iterator is empty");
    if part.len() == prefix.len() {
        debug_assert_eq!(prefix, part);
        debug_assert_eq!(None, parts.next());
        return Cow::Borrowed(path);
    }

    let mut path_bytes = Vec::with_capacity(path_bytes.len());
    path_bytes.extend(part);
    for part in parts {
        path_bytes.push(SEPARATOR);
        path_bytes.extend(part);
    }
    path_bytes.extend(suffix);

    // SAFETY: Only UTF-8 substrings were replaced.
    Cow::Owned(
        unsafe { OsString::from_encoded_bytes_unchecked(path_bytes) }.into(),
    )
}

fn normalize_verbatim(path: &Path) -> Cow<'_, BasePath> {
    // Normalizing more of a verbatim path can change its meaning. The part
    // changed here is required for the path to be verbatim.
    let path = convert_separators(path, Some(4));
    debug_assert!(matches!(
        path.components().next(),
        Some(Component::Prefix(prefix)) if prefix.kind().is_verbatim(),
    ));
    match path {
        Cow::Borrowed(path) => {
            Cow::Borrowed(BasePath::from_inner(path.as_os_str()))
        }
        Cow::Owned(path) => Cow::Owned(BasePathBuf(path)),
    }
}

pub(crate) fn normalize_virtually(
    initial_path: &Path,
) -> io::Result<BasePathBuf> {
    // [GetFullPathNameW] always converts separators.
    let path = convert_separators(initial_path, None);

    let mut wide_path: Vec<_> = path.as_os_str().encode_wide().collect();
    if wide_path.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "strings passed to WinAPI cannot contains NULs",
        ));
    }
    wide_path.push(0);

    match path.components().next() {
        // Verbatim paths should not be modified.
        Some(Component::Prefix(prefix)) if prefix.kind().is_verbatim() => {
            return Ok(normalize_verbatim(initial_path).into_owned());
        }
        Some(Component::RootDir) if wide_path[1] == SEPARATOR.into() => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "partial UNC prefixes are invalid",
            ));
        }
        _ => {}
    }

    let mut buffer = Vec::new();
    let mut capacity = 0;
    loop {
        capacity = unsafe {
            GetFullPathNameW(
                wide_path.as_ptr(),
                capacity,
                buffer.as_mut_ptr(),
                ptr::null_mut(),
            )
        };
        if capacity == 0 {
            break Err(io::Error::last_os_error());
        }

        let _: u32 = capacity;
        // This assertion should never fail.
        static_assert!(mem::size_of::<u32>() <= mem::size_of::<usize>());

        let length = capacity as usize;
        if let Some(mut additional_capacity) =
            length.checked_sub(buffer.capacity())
        {
            assert_ne!(0, additional_capacity);

            // WinAPI can recommend an insufficient capacity that causes it to
            // return incorrect results, so extra space is reserved as a
            // workaround.
            macro_rules! extra_capacity {
                () => {
                    2
                };
            }
            capacity =
                capacity.checked_add(extra_capacity!()).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "required path length is too large for WinAPI",
                    )
                })?;
            additional_capacity += extra_capacity!();

            buffer.reserve(additional_capacity);
            continue;
        }

        // SAFETY: These characters were initialized by the syscall.
        unsafe {
            buffer.set_len(length);
        }
        break Ok(BasePathBuf(OsString::from_wide(&buffer).into()));
    }
}

pub(crate) fn normalize(path: &Path) -> io::Result<BasePathBuf> {
    path.metadata().and_then(|_| normalize_virtually(path))
}

fn get_prefix(base: &BasePath) -> PrefixComponent<'_> {
    if let Some(Component::Prefix(prefix)) = base.components().next() {
        prefix
    } else {
        // Base paths should always have a prefix.
        panic!(
            "base path is missing a prefix: \"{}\"",
            base.as_path().display(),
        );
    }
}

fn push_separator(base: &mut BasePathBuf) {
    // Add a separator if necessary.
    base.0.push("");
}

pub(crate) fn push(base: &mut BasePathBuf, initial_path: &Path) {
    // [GetFullPathNameW] always converts separators.
    let path = convert_separators(initial_path, None);

    let mut components = path.components();
    let mut next_component = components.next();
    match next_component {
        Some(Component::Prefix(prefix)) => {
            // Verbatim paths should not be modified.
            if prefix.kind().is_verbatim() {
                *base = normalize_verbatim(initial_path).into_owned();
                return;
            }

            next_component = components.next();
            // Other prefixes are absolute, except drive-relative prefixes.
            if !matches!(prefix.kind(), Prefix::Disk(_))
                || prefix.kind() != get_prefix(base).kind()
                // Equivalent to [path.has_root()] but more efficient.
                || next_component == Some(Component::RootDir)
            {
                *base = BasePathBuf(path.into_owned());
                return;
            }
        }
        Some(Component::RootDir) => {
            let mut buffer = get_prefix(base).as_os_str().to_owned();
            buffer.push(&*path);
            *base = BasePathBuf(buffer.into());
            return;
        }
        _ => {
            while let Some(component) = next_component {
                match component {
                    Component::CurDir => {}
                    Component::ParentDir if base.pop().is_ok() => {}
                    _ => break,
                }
                next_component = components.next();
            }
        }
    }

    if let Some(component) = next_component {
        push_separator(base);
        base.0.as_mut_os_string().push(component);

        let components = components.as_path();
        if !components.as_os_str().is_empty() {
            push_separator(base);
            base.0.as_mut_os_string().push(components);
        }
    }

    let path_bytes = path.as_os_str().as_encoded_bytes();
    // At least one separator should be kept.
    if path_bytes.last() == Some(&SEPARATOR)
        || path_bytes.ends_with(&[SEPARATOR, b'.'])
    {
        push_separator(base);
    }
}
