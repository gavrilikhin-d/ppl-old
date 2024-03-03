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
pub extern "C" fn string_plus_string(x: *const String, y: *const String) -> *mut String {
    let x = unsafe { x.as_ref().unwrap() };
    let y = unsafe { y.as_ref().unwrap() };

    let boxed = Box::new(format!("{x}{y}"));
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
    let str = unsafe { str.as_ref().unwrap() };

    print!("{str}");
    std::io::stdout().flush().unwrap();
}

/// # PPL
/// ```no_run
/// fn destroy <:String>
/// ```
#[no_mangle]
pub extern "C" fn destroy_string(x: *mut String) {
    debug_assert!(!x.is_null());

    let _ = unsafe { Box::from_raw(x) };
}
