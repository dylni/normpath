#![allow(clashing_extern_declarations)]

use std::ops::Deref;

use fruity::foundation::NSString;
use fruity::objc::Class;
use fruity::objc::NSObject;
use fruity::objc::Object;
use fruity::objc::SEL;
use fruity::selector;

#[repr(C)]
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
            fn objc_msgSend(obj: &Class, sel: SEL) -> &Object;

            fn objc_retain(obj: &Object) -> NSFileManager;
        }

        let obj = Self::class();
        let sel = selector!(defaultManager);

        unsafe { objc_retain(objc_msgSend(obj, sel)) }
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
        fn objc_msgSend(obj: &Object, sel: SEL, path: NSString) -> &Object;

        fn objc_retain(obj: &Object) -> NSString;
    }

    let obj = NSFileManager::default();
    let sel = selector!(displayNameAtPath:);
    // SAFETY: This struct is dropped by the end of this method.
    let path = unsafe { NSString::from_str_no_copy(path) };

    unsafe { objc_retain(&objc_msgSend(&obj, sel, path)) }.to_string()
}
