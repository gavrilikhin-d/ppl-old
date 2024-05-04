use std::ffi::c_char;

use rug::ops::Pow;

use crate::{Rational, String};

/// Big integer number.
/// Wrapper around pointer to [`rug::Integer`].
///
/// # PPL
/// ```no_run
/// type IntegerImpl
///
/// @builtin
/// type Integer:
///     impl: Reference<IntegerImpl>
/// ```
#[repr(C)]
pub struct Integer {
    pub data: *mut rug::Integer,
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from i32
#[no_mangle]
pub extern "C" fn integer_from_i32(value: i32) -> Integer {
    let boxed = Box::new(value.into());
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from i64
#[no_mangle]
pub extern "C" fn integer_from_i64(value: i64) -> Integer {
    let boxed = Box::new(value.into());
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from u64
#[no_mangle]
pub extern "C" fn integer_from_u64(value: u64) -> Integer {
    let boxed = Box::new(value.into());
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_c_string(str: *const c_char) -> Integer {
    debug_assert!(!str.is_null());

    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.parse::<rug::Integer>().unwrap());
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Converts `Integer` to `String`
///
/// # PPL
/// ```no_run
/// fn String from <:Integer> -> String
/// ```
#[no_mangle]
pub extern "C" fn integer_as_string(i: Integer) -> String {
    let i = unsafe { i.data.as_ref().unwrap() };
    let str = i.to_string();
    let boxed = Box::new(str);
    String {
        data: Box::into_raw(boxed),
    }
}

/// Negates integer
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn - <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn minus_integer(i: Integer) -> Integer {
    let i = unsafe { i.data.as_ref().unwrap() };
    let boxed = Box::new(rug::Integer::from(-i));
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Add 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> + <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_plus_integer(x: Integer, y: Integer) -> Integer {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    let boxed = Box::new(rug::Integer::from(x + y));
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Multiply 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> * <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_star_integer(x: Integer, y: Integer) -> Integer {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    let boxed = Box::new(rug::Integer::from(x * y));
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Divide 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> / <:Integer> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn integer_slash_integer(x: Integer, y: Integer) -> Rational {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    let boxed = Box::new(rug::Rational::from(x) / y);
    Rational {
        data: Box::into_raw(boxed),
    }
}

/// Compare 2 integers for equality
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> == <:Integer> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn integer_eq_integer(x: Integer, y: Integer) -> bool {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    x == y
}

/// Is one integer less than another?
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> < <:Integer> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn integer_less_integer(x: Integer, y: Integer) -> bool {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    x < y
}

/// Calculate square root of an integer with rounding
///
/// # PPL
/// ```no_run
/// fn sqrt <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn sqrt_integer(i: Integer) -> Integer {
    let i = unsafe { i.data.as_ref().unwrap() };

    let boxed = Box::new(i.clone().root(2));
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// Calculate `x` in `n`th power
///
/// # PPL
/// ```no_run
/// fn <x: Integer> ^ <n: Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_power_integer(x: Integer, n: Integer) -> Integer {
    let x = unsafe { x.data.as_ref().unwrap() };
    let n = unsafe { n.data.as_ref().unwrap() };

    // TODO: support other powers
    let res: rug::Integer = x.pow(n.to_u32().unwrap()).into();

    let boxed = Box::new(res);
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// # PPL
/// ```no_run
/// fn <x: Integer> % <y: Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_mod_integer(x: Integer, y: Integer) -> Integer {
    let x = unsafe { x.data.as_ref().unwrap() };
    let y = unsafe { y.data.as_ref().unwrap() };

    let res = x.clone().modulo(y);

    let boxed = Box::new(res);
    Integer {
        data: Box::into_raw(boxed),
    }
}

/// # PPL
/// ```no_run
/// fn destroy <:&mut Integer>
/// ```
#[no_mangle]
pub extern "C" fn destroy_integer(x: *mut Integer) {
    let _ = unsafe { Box::from_raw(x.as_ref().unwrap().data) };
}

/// # PPL
/// ```no_run
/// @mangle_as("clone_integer")
/// fn clone <:&Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn clone_integer(x: &Integer) -> Integer {
    let value = unsafe { x.data.as_ref() }.unwrap().clone();
    Integer {
        data: Box::into_raw(Box::new(value)),
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
/// fn String from <:I32> -> String
/// ```
#[no_mangle]
pub extern "C" fn i32_as_string(x: i32) -> String {
    let boxed = Box::new(x.to_string());
    String {
        data: Box::into_raw(boxed),
    }
}

/// # PPL
/// ```no_run
/// /// Convert `Integer` to `I32
/// @mangle_as("integer_as_i32")
/// fn <:Integer> as I32 -> I32
/// ```
#[no_mangle]
pub extern "C" fn integer_as_i32(x: Integer) -> i32 {
    debug_assert!(!x.data.is_null());

    let integer = unsafe { Box::from_raw(x.data) };
    integer
        .to_i32()
        .expect(&format!("Integer `{integer}` is too big to fit into i32"))
}
