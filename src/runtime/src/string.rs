use std::{ffi::c_char, io::Write};

/// PPL's String type.
/// Wrapper around pointer to [`std::string::String`].
///
/// # PPL
/// ```no_run
/// type StringImpl
///
/// @builtin
/// type String:
///     impl: Reference<StringImpl>
/// ```
#[repr(C)]
pub struct String {
    pub data: *mut std::string::String,
}

/// Construct [`String`](ppl::semantics::Type::String) from a C string
/// and length
#[no_mangle]
pub extern "C" fn string_from_c_string_and_length(str: *const c_char, _len: u64) -> String {
    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.to_string());
    String {
        data: Box::into_raw(boxed),
    }
}

/// Concatenate 2 string
///
/// # PPL
/// ```no_run
/// fn <:String> + <:String> -> None
/// ```
#[no_mangle]
pub extern "C" fn string_plus_string(x: String, y: String) -> String {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    let boxed = Box::new(format!("{x}{y}"));
    String {
        data: Box::into_raw(boxed),
    }
}

/// Print string to stdout
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn print <str: String> -> None
/// ```
#[no_mangle]
pub extern "C" fn print_string(str: String) {
    let str = unsafe { str.data.as_ref().unwrap() };

    print!("{str}");
    std::io::stdout().flush().unwrap();
}

/// # PPL
/// ```no_run
/// fn destroy <:String>
/// ```
#[no_mangle]
pub extern "C" fn destroy_string(x: String) {
    debug_assert!(!x.data.is_null());

    let _ = unsafe { Box::from_raw(x.data) };
}

/// # PPL
/// ```no_run
/// @mangle_as("clone_string")
/// fn clone <:String> -> String
/// ```
#[no_mangle]
pub extern "C" fn clone_string(x: String) -> String {
    let value = unsafe { x.data.as_ref() }.unwrap().clone();
    String {
        data: Box::into_raw(Box::new(value)),
    }
}
