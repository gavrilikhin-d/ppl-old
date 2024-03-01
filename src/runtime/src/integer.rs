use rug::{ops::Pow, Integer, Rational};

/// Construct [`Integer`](ppl::semantics::Type::Integer) from i32
#[no_mangle]
pub extern "C" fn integer_from_i32(value: i32) -> *mut Integer {
    let boxed = Box::new(value.into());
    Box::into_raw(boxed)
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from i64
#[no_mangle]
pub extern "C" fn integer_from_i64(value: i64) -> *mut Integer {
    let boxed = Box::new(value.into());
    Box::into_raw(boxed)
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from u64
#[no_mangle]
pub extern "C" fn integer_from_u64(value: u64) -> *mut Integer {
    let boxed = Box::new(value.into());
    Box::into_raw(boxed)
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_c_string(str: *const i8) -> *mut Integer {
    debug_assert!(!str.is_null());

    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.parse::<Integer>().unwrap());
    Box::into_raw(boxed)
}

/// Converts `Integer` to `String`
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> as String -> String
/// ```
#[no_mangle]
pub extern "C" fn integer_as_string(i: *const Integer) -> *mut String {
    debug_assert!(!i.is_null());

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
    debug_assert!(!i.is_null());

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
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

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
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    let boxed = Box::new(Integer::from(unsafe { &*x } * unsafe { &*y }));
    Box::into_raw(boxed)
}

/// Divide 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> / <:Integer> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn integer_slash_integer(x: *const Integer, y: *const Integer) -> *mut Rational {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    let boxed = Box::new(Rational::from(unsafe { &*x }) / unsafe { &*y });
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
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

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
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    unsafe { *x < *y }
}

/// Calculate square root of an integer with rounding
///
/// # PPL
/// ```no_run
/// fn sqrt <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn sqrt_integer(i: *const Integer) -> *mut Integer {
    debug_assert!(!i.is_null());

    let i = unsafe { &*i };

    let boxed = Box::new(i.clone().root(2));
    Box::into_raw(boxed)
}

/// Calculate `x` in `n`th power
///
/// # PPL
/// ```no_run
/// fn <x: Integer> ^ <n: Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_power_integer(x: *const Integer, n: *const Integer) -> *mut Integer {
    debug_assert!(!x.is_null());
    debug_assert!(!n.is_null());

    let x = unsafe { &*x };
    let n = unsafe { &*n };

    // TODO: support other powers
    let res: Integer = x.pow(n.to_u32().unwrap()).into();

    let boxed = Box::new(res);
    Box::into_raw(boxed)
}

/// # PPL
/// ```no_run
/// fn <x: Integer> % <y: Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_mod_integer(x: *const Integer, y: *const Integer) -> *mut Integer {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    let x = unsafe { &*x };
    let y = unsafe { &*y };

    let res = x.clone().modulo(y);

    let boxed = Box::new(res);
    Box::into_raw(boxed)
}

/// # PPL
/// ```no_run
/// fn destroy <:Integer>
/// ```
#[no_mangle]
pub extern "C" fn destroy_integer(x: *mut Integer) {
    debug_assert!(!x.is_null());

    unsafe {
        let _ = Box::from_raw(x);
    }
}

/// # PPL
/// ```no_run
/// fn - <:I32> -> I32
/// ```
#[no_mangle]
pub extern "C" fn minus_i32(x: i32) -> i32 {
    -x
}

/// # PPL
/// ```no_run
/// fn <:I32> + <:I32> -> I32
/// ```
#[no_mangle]
pub extern "C" fn i32_plus_i32(x: i32, y: i32) -> i32 {
    x + y
}

/// # PPL
/// ```no_run
/// @mangle_as("i32_as_string")
/// fn <:I32> as String -> String
/// ```
#[no_mangle]
pub extern "C" fn i32_as_string(x: i32) -> *mut String {
    let boxed = Box::new(x.to_string());
    Box::into_raw(boxed)
}
