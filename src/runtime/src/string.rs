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
