//! This crate provides methods to normalize paths in the recommended way for
//! the operating system.
//!
//! It was made to fix a recurring bug caused by using [`fs::canonicalize`] on
//! Windows: [#45067], [#48249], [#52440], [#55812], [#58613], [#59107],
//! [#74327]. Normalization is usually a better choice unless you specifically
//! need a canonical path.
//!
//! Using these replacement methods will usually fix those issues, but see
//! their documentation for more information:
//! - [`PathExt::normalize`] (*usually* replaces [`Path::canonicalize`])
//! - [`BasePath::join`] (replaces [`Path::join`])
//! - [`BasePath::parent`] (replaces [`Path::parent`])
//! - [`BasePathBuf::pop`] (replaces [`PathBuf::pop`])
//! - [`BasePathBuf::push`] (replaces [`PathBuf::push`])
//!
//! # Sponsorship
//!
//! If this crate has been useful for your project, let me know with a
//! [sponsorship](https://github.com/sponsors/dylni)! Sponsorships help me
//! create and maintain my open source libraries, and they are always very
//! appreciated.
//!
//! # Examples
//!
//! ```
//! use std::io;
//! use std::path::Path;
//!
//! use normpath::BasePathBuf;
//! use normpath::PathExt;
//!
//! fn find_target_dir(path: &Path) -> io::Result<Option<BasePathBuf>> {
//!     let mut path = path.normalize()?;
//!     while !path.ends_with("target") {
//!         match path.pop() {
//!             Ok(true) => continue,
//!             Ok(false) => {}
//!             Err(_) => {
//!                 eprintln!("Some components could not be normalized.");
//!             }
//!         }
//!         return Ok(None);
//!     }
//!     Ok(Some(path))
//! }
//! ```
//!
//! [#45067]: https://github.com/rust-lang/rust/issues/45067
//! [#48249]: https://github.com/rust-lang/rust/issues/48249
//! [#52440]: https://github.com/rust-lang/rust/issues/52440
//! [#55812]: https://github.com/rust-lang/rust/issues/55812
//! [#58613]: https://github.com/rust-lang/rust/issues/58613
//! [#59107]: https://github.com/rust-lang/rust/issues/59107
//! [#74327]: https://github.com/rust-lang/rust/issues/74327
//! [`fs::canonicalize`]: ::std::fs::canonicalize
//! [`PathBuf::pop`]: ::std::path::PathBuf::pop
//! [`PathBuf::push`]: ::std::path::PathBuf::push

#![doc(html_root_url = "https://docs.rs/normpath/*")]
#![warn(unused_results)]

use std::io;
use std::path::Path;

macro_rules! matches {
    ( $value:expr , $($pattern:pat)|+ ) => {{
        #[allow(clippy::match_like_matches_macro)]
        match $value {
            $($pattern)|+ => true,
            _ => false,
        }
    }};
}

mod cmp;

pub mod error;

#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(not(windows), path = "common.rs")]
mod imp;

mod base;
pub use base::BasePath;
pub use base::BasePathBuf;

/// Additional methods added to [`Path`].
pub trait PathExt: private::Sealed {
    /// Normalizes `self` relative to the current directory.
    ///
    /// # Unix Behavior
    ///
    /// On Unix, normalization is equivalent to canonicalization.
    ///
    /// # Windows Behavior
    ///
    /// On Windows, normalization is similar to canonicalization, but:
    /// - the [prefix] of the path is rarely changed. Canonicalization would
    ///   always return a [verbatim] path, which can be difficult to use.
    ///   ([rust-lang/rust#42869])
    /// - the result is more consistent. ([rust-lang/rust#49342])
    /// - shared partition paths do not cause an error.
    ///   ([rust-lang/rust#52440])
    ///
    /// However, [verbatim] paths will not be modified, so they might still
    /// contain `.` or `..` components. [`BasePath::join`] and
    /// [`BasePathBuf::push`] can normalize them before they become part of the
    /// path.
    ///
    /// # Implementation
    ///
    /// Currently, this method calls:
    /// - [`fs::canonicalize`] on Unix.
    /// - [`GetFullPathNameW`] on Windows.
    ///
    /// However, the implementation is subject to change. This section is only
    /// informative.
    ///
    /// # Errors
    ///
    /// Returns an error if `self` cannot be normalized or contains a null
    /// byte. On Unix, only existing paths can be normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use std::path::Path;
    ///
    /// use normpath::PathExt;
    ///
    /// if cfg!(windows) {
    ///     assert_eq!(
    ///         Path::new(r"X:\foo\baz\test.rs"),
    ///         Path::new("X:/foo/bar/../baz/test.rs").normalize()?,
    ///     );
    /// }
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [`fs::canonicalize`]: ::std::fs::canonicalize
    /// [`GetFullPathNameW`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew
    /// [rust-lang/rust#42869]: https://github.com/rust-lang/rust/issues/42869
    /// [rust-lang/rust#49342]: https://github.com/rust-lang/rust/issues/49342
    /// [rust-lang/rust#52440]: https://github.com/rust-lang/rust/issues/52440
    /// [prefix]: ::std::path::Prefix
    /// [verbatim]: ::std::path::Prefix::is_verbatim
    fn normalize(&self) -> io::Result<BasePathBuf>;
}

impl PathExt for Path {
    #[inline]
    fn normalize(&self) -> io::Result<BasePathBuf> {
        imp::normalize(self)
    }
}

mod private {
    use std::path::Path;

    pub trait Sealed {}
    impl Sealed for Path {}
}
