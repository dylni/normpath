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
//! Additionally, these methods can be used for other enhancements:
//! - [`PathExt::localize_name`]
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file.
//!
//! ### Optional Features
//!
//! - **localization** -
//!   Provides [`PathExt::localize_name`] and [`BasePath::localize_name`].
//!
//! - **print_bytes** -
//!   Provides implementations of [`print_bytes::ToBytes`] for [`BasePath`] and
//!   [`BasePathBuf`].
//!
//! - **serde** -
//!   Provides implementations of [`serde::Deserialize`] and/or
//!   [`serde::Serialize`] for [`BasePath`] and [`BasePathBuf`].
//!
//! - **uniquote** -
//!   Provides implementations of [`uniquote::Quote`] for [`BasePath`] and
//!   [`BasePathBuf`].
//!
//! # Implementation
//!
//! All traits are [sealed], meaning that they can only be implemented by this
//! crate. Otherwise, backward compatibility would be more difficult to
//! maintain for new features.
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
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed

// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
#![cfg_attr(normpath_docs_rs, feature(doc_cfg))]
#![warn(unused_results)]

#[cfg(feature = "localization")]
use std::borrow::Cow;
#[cfg(feature = "localization")]
use std::ffi::OsStr;
use std::io;
#[cfg(feature = "localization")]
use std::path::Component;
use std::path::Path;

mod base;
pub use base::BasePath;
pub use base::BasePathBuf;

mod cmp;

pub mod error;

#[cfg(feature = "localization")]
#[cfg_attr(windows, path = "windows/localize.rs")]
#[cfg_attr(not(windows), path = "common/localize/mod.rs")]
mod localize;

#[cfg_attr(windows, path = "windows/normalize.rs")]
#[cfg_attr(not(windows), path = "common/normalize.rs")]
mod normalize;

/// Additional methods added to [`Path`].
pub trait PathExt: private::Sealed {
    /// Returns the localized simple name for this path.
    ///
    /// If the path does not exist or localization is not possible, the last
    /// component will be returned.
    ///
    /// The returned string should only be used for display to users. It will
    /// be as similar as possible to the name displayed by the system file
    /// manager for the path. However, nothing should be assumed about the
    /// result.
    ///
    /// # Implementation
    ///
    /// Currently, this method calls:
    /// - [`[NSFileManager displayNameAtPath:]`][displayNameAtPath] on MacOS
    ///   ([rust-lang/rfcs#845]).
    /// - [`SHGetFileInfoW`] on Windows.
    ///
    /// However, the implementation is subject to change. This section is only
    /// informative.
    ///
    /// <div style="background:rgba(255,181,77,0.16); padding:0.75em;">
    /// <strong>Warning</strong>: Documentation for <code>SHGetFileInfoW</code>
    /// has this note:
    /// <blockquote style="margin: 1em 2em 0.5em;">
    /// You should call this function from a background thread. Failure to do
    /// so could cause the UI to stop responding.
    /// </blockquote>
    /// </div>
    ///
    /// # Panics
    ///
    /// Panics if the path ends with a `..` component. In the future, this
    /// method might also panic for paths ending with `.` components, so they
    /// should not be given either. They currently cause a platform-dependent
    /// value to be returned.
    ///
    /// You should usually only call this method on [normalized] paths to avoid
    /// these panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use normpath::PathExt;
    ///
    /// assert_eq!("test.rs", &*Path::new("/foo/bar/test.rs").localize_name());
    /// ```
    ///
    /// [displayNameAtPath]: https://developer.apple.com/documentation/foundation/nsfilemanager/1409751-displaynameatpath
    /// [normalized]: Self::normalize
    /// [rust-lang/rfcs#845]: https://github.com/rust-lang/rfcs/issues/845
    /// [`SHGetFileInfoW`]: https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shgetfileinfow
    #[cfg(feature = "localization")]
    #[cfg_attr(normpath_docs_rs, doc(cfg(feature = "localization")))]
    #[must_use]
    fn localize_name(&self) -> Cow<'_, OsStr>;

    /// Normalizes `self` relative to the current directory.
    ///
    /// This method will access the file system to normalize the path. If the
    /// path might not exist, [`normalize_virtually`] can be used instead, but
    /// it is only available on Windows. Other platforms require file system
    /// access to perform normalization.
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
    /// Returns an error if `self` cannot be normalized or does not exist, even
    /// on Windows.
    ///
    /// This method is designed to give mostly consistent errors on different
    /// platforms, even when the functions it calls have different behavior. To
    /// normalize paths that might not exist, use [`normalize_virtually`].
    ///
    /// # Examples
    ///
    /// ```no_run
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
    /// [`normalize_virtually`]: Self::normalize_virtually
    /// [rust-lang/rust#42869]: https://github.com/rust-lang/rust/issues/42869
    /// [rust-lang/rust#49342]: https://github.com/rust-lang/rust/issues/49342
    /// [rust-lang/rust#52440]: https://github.com/rust-lang/rust/issues/52440
    /// [prefix]: ::std::path::Prefix
    /// [verbatim]: ::std::path::Prefix::is_verbatim
    fn normalize(&self) -> io::Result<BasePathBuf>;

    /// Equivalent to [`normalize`] but does not access the file system.
    ///
    /// # Errors
    ///
    /// Returns an error if `self` cannot be normalized or contains a null
    /// byte. Nonexistent paths will not cause an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use std::path::Path;
    ///
    /// use normpath::PathExt;
    ///
    /// #[cfg(windows)]
    /// assert_eq!(
    ///     Path::new(r"X:\foo\baz\test.rs"),
    ///     Path::new("X:/foo/bar/../baz/test.rs").normalize_virtually()?,
    /// );
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [`normalize`]: Self::normalize
    #[cfg(any(doc, windows))]
    #[cfg_attr(normpath_docs_rs, doc(cfg(windows)))]
    fn normalize_virtually(&self) -> io::Result<BasePathBuf>;
}

impl PathExt for Path {
    #[cfg(feature = "localization")]
    #[inline]
    fn localize_name(&self) -> Cow<'_, OsStr> {
        let name = if let Some(name) = self.components().next_back() {
            name
        } else {
            return Cow::Borrowed(OsStr::new(""));
        };
        assert_ne!(
            Component::ParentDir,
            name,
            "path ends with a `..` component: \"{}\"",
            self.display(),
        );

        localize::name(self)
            .map(Cow::Owned)
            .unwrap_or_else(|| Cow::Borrowed(name.as_os_str()))
    }

    #[inline]
    fn normalize(&self) -> io::Result<BasePathBuf> {
        normalize::normalize(self)
    }

    #[cfg(any(doc, windows))]
    #[inline]
    fn normalize_virtually(&self) -> io::Result<BasePathBuf> {
        normalize::normalize_virtually(self)
    }
}

mod private {
    use std::path::Path;

    pub trait Sealed {}
    impl Sealed for Path {}
}
