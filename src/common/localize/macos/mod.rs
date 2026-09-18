#![expect(clashing_extern_declarations)]

use std::ops::Deref;
use std::os::raw::c_char;
use std::slice;
use std::str;

#[macro_use]
mod fruity;
use fruity::foundation::NSString;
use fruity::foundation::NSStringEncoding;
use fruity::objc::Class;
use fruity::objc::NSObject;
use fruity::objc::NSUInteger;
use fruity::objc::Object;
use fruity::objc::SEL;

impl NSString {
    fn is_empty(&self) -> bool {
        extern "C" {
            fn objc_msgSend(obj: &Object, sel: SEL) -> NSUInteger;
        }

        let sel = selector!(length);

        let length = unsafe { objc_msgSend(self, sel) };
        length == 0
    }

    fn utf8_length(&self) -> usize {
        extern "C" {
            fn objc_msgSend(
                obj: &Object,
                sel: SEL,
                enc: NSStringEncoding,
            ) -> NSUInteger;
        }

        let sel = selector!(lengthOfBytesUsingEncoding:);

        let length =
            unsafe { objc_msgSend(self, sel, NSStringEncoding::UTF8) };
        if length == 0 {
            assert!(self.is_empty());
        }
        length
    }

    fn to_utf8_ptr(&self) -> *const u8 {
        extern "C" {
            fn objc_msgSend(obj: &Object, sel: SEL) -> *const c_char;
        }

        let sel = selector!(UTF8String);

        unsafe { objc_msgSend(self, sel) }.cast()
    }

    unsafe fn to_str(&self) -> &str {
        let length = self.utf8_length();
        let ptr = self.to_utf8_ptr();
        if ptr.is_null() {
            return "";
        }
        // SAFETY: `ptr` is non-null and points to `length` UTF-8 encoded bytes.
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, length)) }
    }
}

impl ToString for NSString {
    fn to_string(&self) -> String {
        // SAFETY: The string has a short lifetime.
        unsafe { self.to_str() }.to_owned()
    }
}

#[repr(transparent)]
struct NSFileManager(NSObject);

impl NSFileManager {
    fn class() -> &'static Class {
        extern "C" {
            #[link_name = "OBJC_CLASS_$_NSFileManager"]
            static CLASS: Class;
        }
        unsafe { &CLASS }
    }

    fn default() -> Self {
        extern "C" {
            fn objc_msgSend(obj: &Class, sel: SEL) -> Option<&Object>;

            fn objc_retain(obj: &Object) -> Option<NSFileManager>;
        }

        let obj = Self::class();
        let sel = selector!(defaultManager);

        unsafe {
            let mgr = objc_msgSend(obj, sel)
                .expect("NSFileManager defaultManager returned nil");
            objc_retain(mgr)
                .expect("objc_retain returned nil for NSFileManager")
        }
    }
}

impl Deref for NSFileManager {
    type Target = NSObject;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(super) fn name(path: &str) -> String {
    extern "C" {
        fn objc_msgSend<'a>(
            obj: &'a Object,
            sel: SEL,
            path: &'a Object,
        ) -> Option<&'a Object>;

        fn objc_retain(obj: &Object) -> Option<NSString>;
    }

    let obj = NSFileManager::default();
    let sel = selector!(displayNameAtPath:);
    // SAFETY: This struct is dropped by the end of this method.
    let path = unsafe { NSString::from_str_no_copy(path) };

    let display_name = unsafe {
        let res = objc_msgSend(&obj, sel, &path)
            .expect("displayNameAtPath: returned nil");
        objc_retain(res)
            .expect("objc_retain returned nil for displayNameAtPath:")
    };
    display_name.to_string()
}
