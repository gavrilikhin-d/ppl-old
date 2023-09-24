use rug::{Integer, Rational};

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_i64(value: i64) -> *mut Integer {
    let boxed = Box::new(value.into());
    Box::into_raw(boxed)
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_c_string(str: *const i8) -> *mut Integer {
    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.parse::<Integer>().unwrap());
    Box::into_raw(boxed)
}

/// Construct [`Rational`](ppl::semantics::Type::Rational) from a C string
#[no_mangle]
pub extern "C" fn rational_from_c_string(str: *const i8) -> *mut Rational {
    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.parse::<Rational>().unwrap());
    Box::into_raw(boxed)
}

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
    if str.is_null() {
        panic!("null pointer passed to print_string");
    }
    println!("{}", unsafe { &*str });
}

/// Converts `Integer` to `String`
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> as String -> String
/// ```
#[no_mangle]
pub extern "C" fn integer_as_string(i: *const Integer) -> *mut String {
    if i.is_null() {
        panic!("null pointer passed to integer_as_string");
    }
    let boxed = Box::new(unsafe { &*i }.to_string());
    Box::into_raw(boxed)
}

/// Negates integer
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn - <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn minus_integer(i: *const Integer) -> *mut Integer {
    if i.is_null() {
        panic!("null pointer passed to integer_as_string");
    }
    let boxed = Box::new(-unsafe { &*i }.clone());
    Box::into_raw(boxed)
}

/// Add 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> + <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_plus_integer(x: *const Integer, y: *const Integer) -> *mut Integer {
    if x.is_null() || y.is_null() {
        panic!("null pointer passed to integer_as_string");
    }
    let boxed = Box::new(Integer::from(unsafe { &*x } + unsafe { &*y }));
    Box::into_raw(boxed)
}

/// Multiply 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> * <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_star_integer(x: *const Integer, y: *const Integer) -> *mut Integer {
    if x.is_null() || y.is_null() {
        panic!("null pointer passed to integer_as_string");
    }
    let boxed = Box::new(Integer::from(unsafe { &*x } * unsafe { &*y }));
    Box::into_raw(boxed)
}

/// Compare 2 integers for equality
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> == <:Integer> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn integer_eq_integer(x: *const Integer, y: *const Integer) -> bool {
    if x.is_null() || y.is_null() {
        panic!("null pointer passed to integer_as_string");
    }
    unsafe { *x == *y }
}

/// Is one integer less than another?
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> < <:Integer> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn integer_less_integer(x: *const Integer, y: *const Integer) -> bool {
    if x.is_null() || y.is_null() {
        panic!("null pointer passed to integer_as_string");
    }
    unsafe { *x < *y }
}

// IMPORTANT: don't forget to update global mapping after adding new function!!!
