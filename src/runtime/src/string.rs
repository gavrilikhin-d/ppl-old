use std::io::Write;

/// Construct [`String`](ppl::semantics::Type::String) from a C string
/// and length
#[no_mangle]
pub extern "C" fn string_from_c_string_and_length(str: *const i8, _len: u64) -> *mut String {
    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.to_string());
    Box::into_raw(boxed)
}

/// Concatenate 2 string
///
/// # PPL
/// ```no_run
/// fn <:String> + <:String> -> None
/// ```
#[no_mangle]
pub extern "C" fn string_plus_string(a: *const String, b: *const String) -> *mut String {
    debug_assert!(!a.is_null());
    debug_assert!(!b.is_null());

    let a = unsafe { &*a };
    let b = unsafe { &*b };

    let boxed = Box::new(format!("{a}{b}"));
    Box::into_raw(boxed)
}

/// Prints none value
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn print <str: String> -> None
/// ```
#[no_mangle]
pub extern "C" fn print_string(str: *const String) {
    debug_assert!(!str.is_null());

    print!("{}", unsafe { &*str });
    std::io::stdout().flush().unwrap();
}

/// # PPL
/// ```no_run
/// fn destroy <:String>
/// ```
#[no_mangle]
pub extern "C" fn destroy_string(x: *mut String) {
    debug_assert!(!x.is_null());

    unsafe {
        let _ = Box::from_raw(x);
    }
}
