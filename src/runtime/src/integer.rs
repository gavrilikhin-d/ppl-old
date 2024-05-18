use std::ffi::c_char;

use rug::ops::Pow;

use crate::{decrement_strong_count, increment_strong_count, Rational, String};

use std::sync::Arc;

/// Big integer number.
/// Wrapper around pointer to [`rug::Integer`].
///
/// # PPL
/// ```no_run
/// @builtin
/// type Integer
/// ```
#[repr(C)]
pub struct Integer(pub *const rug::Integer);

impl Clone for Integer {
    fn clone(&self) -> Self {
        increment_strong_count(self.0 as *const _);
        Self(self.0)
    }
}

impl Drop for Integer {
    fn drop(&mut self) {
        decrement_strong_count(self.0 as *const _);
    }
}

impl Integer {
    /// Get the inner value
    pub fn as_ref(&self) -> &rug::Integer {
        unsafe { &*self.0 }
    }
}

impl<T> From<T> for Integer
where
    rug::Integer: From<T>,
{
    fn from(x: T) -> Self {
        let this = Arc::new(rug::Integer::from(x));
        Self(Arc::into_raw(this))
    }
}

/// Construct [`Integer`] from [`i32`]
#[no_mangle]
pub extern "C" fn integer_from_i32(value: i32) -> Integer {
    integer_from_i64(value as i64)
}

/// Construct [`Integer`] from [`i64`]
#[no_mangle]
pub extern "C" fn integer_from_i64(value: i64) -> Integer {
    rug::Integer::from(value).into()
}

/// Construct [`Integer`] from [`u64`]
#[no_mangle]
pub extern "C" fn integer_from_u64(value: u64) -> Integer {
    rug::Integer::from(value).into()
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_c_string(str: *const c_char) -> Integer {
    debug_assert!(!str.is_null());

    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    str.parse::<rug::Integer>().unwrap().into()
}

/// Converts [`Integer`] to [`String`]
///
/// # PPL
/// ```no_run
/// fn String from <:Integer> -> String
/// ```
#[no_mangle]
pub extern "C" fn integer_as_string(i: Integer) -> String {
    let str = i.as_ref().to_string();
    let boxed = Box::new(str);
    String {
        data: Box::into_raw(boxed),
    }
}

/// Negates integer
///
/// # PPL
/// ```no_run
/// fn - <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn minus_integer(i: Integer) -> Integer {
    (-i.as_ref()).into()
}

/// Add 2 integers
///
/// # PPL
/// ```no_run
/// fn <:Integer> + <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_plus_integer(x: Integer, y: Integer) -> Integer {
    let x = x.as_ref();
    let y = y.as_ref();

    (x + y).into()
}

/// Multiply 2 integers
///
/// # PPL
/// ```no_run
/// fn <:Integer> * <:Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_star_integer(x: Integer, y: Integer) -> Integer {
    let x = x.as_ref();
    let y = y.as_ref();

    (x * y).into()
}

/// Divide 2 integers
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> / <:Integer> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn integer_slash_integer(x: Integer, y: Integer) -> Rational {
    let x = x.as_ref();
    let y = y.as_ref();

    let boxed = Box::new(rug::Rational::from(x) / y);
    Rational {
        data: Box::into_raw(boxed),
    }
}

/// Compare 2 integers for equality
///
/// # PPL
/// ```no_run
/// fn <:Integer> == <:Integer> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn integer_eq_integer(x: Integer, y: Integer) -> bool {
    let x = x.as_ref();
    let y = y.as_ref();

    x == y
}

/// Is one integer less than another?
///
/// # PPL
/// ```no_run
/// fn <:Integer> < <:Integer> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn integer_less_integer(x: Integer, y: Integer) -> bool {
    let x = x.as_ref();
    let y = y.as_ref();

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
    i.as_ref().clone().root(2).into()
}

/// Calculate `x` in `n`th power
///
/// # PPL
/// ```no_run
/// fn <x: Integer> ^ <n: Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_power_integer(x: Integer, n: Integer) -> Integer {
    let x = x.as_ref();
    let n = n.as_ref();

    // TODO: support other powers
    let res: rug::Integer = x.pow(n.to_u32().unwrap()).into();

    res.into()
}

/// # PPL
/// ```no_run
/// fn <x: Integer> % <y: Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn integer_mod_integer(x: Integer, y: Integer) -> Integer {
    let x = x.as_ref();
    let y = y.as_ref();

    let res = x.clone().modulo(y);
    res.into()
}

/// # PPL
/// ```no_run
/// fn destroy <:&mut Integer>
/// ```
#[no_mangle]
pub extern "C" fn destroy_integer(x: &mut Integer) {
    decrement_strong_count(x.0 as *const _);
}

/// # PPL
/// ```no_run
/// @mangle_as("clone_integer")
/// fn clone <:&Integer> -> Integer
/// ```
#[no_mangle]
pub extern "C" fn clone_integer(x: &Integer) -> Integer {
    x.clone()
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
    let integer = x.as_ref();
    integer
        .to_i32()
        .expect(&format!("Integer `{integer}` is too big to fit into i32"))
}
