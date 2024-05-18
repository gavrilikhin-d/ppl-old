use std::{ffi::c_char, io::Write, sync::Arc};

use crate::{decrement_strong_count, increment_strong_count};

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
pub struct String(pub *const std::string::String);

impl Clone for String {
    fn clone(&self) -> Self {
        increment_strong_count(self.0 as *const _);
        Self(self.0)
    }
}

impl Drop for String {
    fn drop(&mut self) {
        decrement_strong_count(self.0 as *const _);
    }
}

impl String {
    /// Get the inner value
    pub fn as_ref(&self) -> &std::string::String {
        unsafe { &*self.0 }
    }
}

impl<T> From<T> for String
where
    std::string::String: From<T>,
{
    fn from(x: T) -> Self {
        let this = Arc::new(std::string::String::from(x));
        Self(Arc::into_raw(this))
    }
}

/// Construct [`String`](ppl::semantics::Type::String) from a C string
/// and length
#[no_mangle]
pub extern "C" fn string_from_c_string_and_length(str: *const c_char, _len: u64) -> String {
    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    str.to_string().into()
}

/// Concatenate 2 string
///
/// # PPL
/// ```no_run
/// fn <:String> + <:String> -> None
/// ```
#[no_mangle]
pub extern "C" fn string_plus_string(x: String, y: String) -> String {
    let x = x.as_ref();
    let y = y.as_ref();

    format!("{x}{y}").into()
}

/// Print string to stdout
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn print <str: String> -> None
/// ```
#[no_mangle]
pub extern "C" fn print_string(str: String) {
    let str = str.as_ref();

    print!("{str}");
    std::io::stdout().flush().unwrap();
}

/// # PPL
/// ```no_run
/// fn destroy <:&mut String>
/// ```
#[no_mangle]
pub extern "C" fn destroy_string(x: &mut String) {
    decrement_strong_count(x.0 as *const _);
}

/// # PPL
/// ```no_run
/// @mangle_as("clone_string")
/// fn clone <:&String> -> String
/// ```
#[no_mangle]
pub extern "C" fn clone_string(x: &String) -> String {
    x.clone()
}
