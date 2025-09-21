//! This crate provides procedural macros for [normpath].
//!
//! **Do not add this crate as a dependency.** It has no backward compatibility
//! guarantees. It is only used for development of [normpath].
//!
//! [normpath]: https://crates.io/crates/normpath

#![cfg(windows)]
#![warn(unused_results)]

use std::io;
use std::iter;
use std::mem::MaybeUninit;
use std::result;

use proc_macro::Delimiter;
use proc_macro::Group;
use proc_macro::Ident;
use proc_macro::Literal;
use proc_macro::Punct;
use proc_macro::Spacing;
use proc_macro::Span;
use proc_macro::TokenStream;
use proc_macro::TokenTree;

use windows_sys::Wdk::System::SystemServices::RtlGetVersion;
use windows_sys::Win32::Foundation::STATUS_SUCCESS;
use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;

trait TokenStreamExt {
    fn push<T>(&mut self, token: T)
    where
        T: Into<TokenTree>;
}

impl TokenStreamExt for TokenStream {
    fn push<T>(&mut self, token: T)
    where
        T: Into<TokenTree>,
    {
        self.extend(iter::once(token.into()));
    }
}

fn macro_path(module: &str, name: &str) -> impl Iterator<Item = TokenTree> {
    [
        Punct::new(':', Spacing::Joint).into(),
        Punct::new(':', Spacing::Alone).into(),
        Ident::new(module, Span::call_site()).into(),
        Punct::new(':', Spacing::Joint).into(),
        Punct::new(':', Spacing::Alone).into(),
        Ident::new(name, Span::call_site()).into(),
        Punct::new('!', Spacing::Alone).into(),
    ]
    .into_iter()
}

// https://docs.rs/syn/1.0/syn/struct.Error.html
struct Error {
    start: Span,
    end: Span,
    message: String,
}

impl Error {
    const fn new(span: Span, message: String) -> Self {
        Self {
            start: span,
            end: span,
            message,
        }
    }

    fn new_spanned<T>(tokens: T, message: &'static str) -> Self
    where
        T: Into<TokenStream>,
    {
        let mut tokens = tokens.into().into_iter();
        let start = tokens
            .next()
            .map(|x| x.span())
            .unwrap_or_else(Span::call_site);
        Self {
            start,
            end: tokens.last().map(|x| x.span()).unwrap_or(start),
            message: message.to_owned(),
        }
    }

    fn into_compile_error(self) -> TokenStream {
        let mut result: TokenStream = macro_path("std", "compile_error")
            .map(|mut token| {
                token.set_span(self.start);
                token
            })
            .collect();

        let mut literal = Literal::string(&self.message);
        literal.set_span(self.end);

        let mut group =
            Group::new(Delimiter::Brace, TokenTree::Literal(literal).into());
        group.set_span(self.end);

        result.push(group);
        result
    }
}

// https://docs.rs/syn/1.0/syn/type.Result.html
type Result<T> = result::Result<T, Error>;

fn get_windows_version() -> Result<OSVERSIONINFOW> {
    let mut version_info = MaybeUninit::uninit();
    let result = unsafe { RtlGetVersion(version_info.as_mut_ptr()) };
    if result == STATUS_SUCCESS {
        Ok(unsafe { version_info.assume_init() })
    } else {
        let error = io::Error::last_os_error();
        Err(Error::new(
            Span::call_site(),
            format!("failed syscall: {}", error),
        ))
    }
}

#[proc_macro_attribute]
pub fn cfg_supports_joined_device_paths(
    args: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let Ok(enable) = args.to_string().parse::<bool>() else {
        return Error::new_spanned(args, "argument must be a boolean")
            .into_compile_error();
    };

    let supported = match get_windows_version() {
        Ok(version_info) => version_info.dwBuildNumber < 21000,
        Err(error) => return error.into_compile_error(),
    };

    if enable == supported {
        item
    } else {
        TokenStream::new()
    }
}
