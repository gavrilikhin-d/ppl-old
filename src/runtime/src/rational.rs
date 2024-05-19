use std::ffi::c_char;

use rug::{ops::Pow, Integer};

use crate::String;

/// Rational number.
/// Wrapper around pointer to [`rug::Rational`].
///
/// # PPL
/// ```no_run
/// type RationalImpl
///
/// @builtin
/// type Rational:
///     impl: Reference<RationalImpl>
/// ```
#[repr(C)]
pub struct Rational {
    pub data: *mut rug::Rational,
}

impl Clone for Rational {
    fn clone(&self) -> Self {
        self.as_ref().into()
    }
}

impl Drop for Rational {
    fn drop(&mut self) {
        // let _ = unsafe { Box::from_raw(self.data) };
    }
}

impl Rational {
    /// Get the inner value
    pub fn as_ref(&self) -> &rug::Rational {
        unsafe { &*self.data }
    }
}

impl<T> From<T> for Rational
where
    rug::Rational: From<T>,
{
    fn from(x: T) -> Self {
        let this = Box::new(rug::Rational::from(x));
        Self {
            data: Box::into_raw(this),
        }
    }
}

/// Construct [`Rational`] from a C string
#[no_mangle]
pub extern "C" fn rational_from_c_string(str: *const c_char) -> Rational {
    debug_assert!(!str.is_null());

    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    str.parse::<rug::Rational>().unwrap().into()
}

/// # PPL
/// ```no_run
/// fn String from <:Rational> -> String
/// ```
#[no_mangle]
pub extern "C" fn rational_as_string(r: Rational) -> String {
    let value = r.as_ref();

    maybe_to_decimal_string(value).into()
}

/// Negates rational
///
/// # PPL
/// ```no_run
/// fn - <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn minus_rational(r: Rational) -> Rational {
    (-r.as_ref()).into()
}

/// Add 2 rationals
///
/// # PPL
/// ```no_run
/// fn <:Rational> + <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_plus_rational(x: Rational, y: Rational) -> Rational {
    let x = x.as_ref();
    let y = y.as_ref();

    (x + y).into()
}

/// Multiply 2 rationals
///
/// # PPL
/// ```no_run
/// fn <:Rational> * <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_star_rational(x: Rational, y: Rational) -> Rational {
    let x = x.as_ref();
    let y = y.as_ref();

    (x * y).into()
}

/// Divide 2 rationals
///
/// # PPL
/// ```no_run
/// fn <:Rational> / <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_slash_rational(x: Rational, y: Rational) -> Rational {
    let x = x.as_ref();
    let y = y.as_ref();

    (x / y).into()
}

/// Compare 2 rationals for equality
///
/// # PPL
/// ```no_run
/// fn <:Rational> == <:Rational> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn rational_eq_rational(x: Rational, y: Rational) -> bool {
    let x = x.as_ref();
    let y = y.as_ref();

    x == y
}

/// Is one rational less than another?
///
/// # PPL
/// ```no_run
/// fn <:Rational> < <:Rational> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn rational_less_rational(x: Rational, y: Rational) -> bool {
    let x = x.as_ref();
    let y = y.as_ref();

    x < y
}

/// # PPL
/// ```no_run
/// fn destroy <:&mut Rational>
/// ```
#[no_mangle]
pub extern "C" fn destroy_rational(x: &mut Rational) {
    let _ = unsafe { Box::from_raw(x.data) };
}

/// # PPL
/// ```no_run
/// @mangle_as("clone_rational")
/// fn clone <:&Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn clone_rational(x: &Rational) -> Rational {
    x.clone()
}

pub fn maybe_to_decimal_string(r: &rug::Rational) -> std::string::String {
    let mut denom = r.denom().clone();
    let pow2 = denom.remove_factor_mut(&Integer::from(2));
    let pow5 = denom.remove_factor_mut(&Integer::from(5));
    if denom != Integer::from(1) {
        return r.to_string();
    }

    let pow10 = pow2.max(pow5);
    let mut numer = r.numer().clone();
    numer *= Integer::from(2).pow(pow10 - pow2);
    numer *= Integer::from(5).pow(pow10 - pow5);

    let pow10 = pow10 as usize;
    let numer = format!("{numer:0>pow10$}");
    let dotpoint = numer.len() - pow10;
    let mut before_dot = &numer[..dotpoint];
    if before_dot.is_empty() {
        before_dot = "0";
    }
    let mut after_dot = &numer[dotpoint..];
    if after_dot.is_empty() {
        after_dot = "0";
    }
    format!("{before_dot}.{after_dot}")
}

#[cfg(test)]
mod test {
    #[test]
    fn to_decimal_string() {
        use super::maybe_to_decimal_string;
        use rug::Rational;

        let r = rug::Rational::from((1, 3));
        assert_eq!(maybe_to_decimal_string(&r), "1/3");

        let r = rug::Rational::from((1, 2));
        assert_eq!(maybe_to_decimal_string(&r), "0.5");

        let r = rug::Rational::from((5, 1));
        assert_eq!(maybe_to_decimal_string(&r), "5.0");

        let r = rug::Rational::from((5, 2));
        assert_eq!(maybe_to_decimal_string(&r), "2.5");

        let r = rug::Rational::from((1, 4));
        assert_eq!(maybe_to_decimal_string(&r), "0.25");

        let r = rug::Rational::from((1, 8));
        assert_eq!(maybe_to_decimal_string(&r), "0.125");

        let r = rug::Rational::from((1, 16));
        assert_eq!(maybe_to_decimal_string(&r), "0.0625");
    }
}
