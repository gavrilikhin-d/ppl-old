use rug::{ops::Pow, Integer, Rational};

/// Construct [`Rational`](ppl::semantics::Type::Rational) from a C string
#[no_mangle]
pub extern "C" fn rational_from_c_string(str: *const i8) -> *mut Rational {
    debug_assert!(!str.is_null());

    let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
    let str = c_str.to_str().unwrap();
    let boxed = Box::new(str.parse::<Rational>().unwrap());
    Box::into_raw(boxed)
}

/// Converts `Rational` to `String`
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Rational> as String -> String
/// ```
#[no_mangle]
pub extern "C" fn rational_as_string(r: *const Rational) -> *mut String {
    debug_assert!(!r.is_null());

    let value = unsafe { &*r };

    let boxed = Box::new(maybe_to_decimal_string(value));
    Box::into_raw(boxed)
}

/// Negates rational
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn - <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn minus_rational(r: *const Rational) -> *mut Rational {
    debug_assert!(!r.is_null());

    let boxed = Box::new(-unsafe { &*r }.clone());
    Box::into_raw(boxed)
}

/// Add 2 rationals
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Rational> + <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_plus_rational(x: *const Rational, y: *const Rational) -> *mut Rational {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    let boxed = Box::new(Rational::from(unsafe { &*x } + unsafe { &*y }));
    Box::into_raw(boxed)
}

/// Multiply 2 rationals
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Rational> * <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_star_rational(x: *const Rational, y: *const Rational) -> *mut Rational {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    let boxed = Box::new(Rational::from(unsafe { &*x } * unsafe { &*y }));
    Box::into_raw(boxed)
}

/// Divide 2 rationals
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Rational> / <:Rational> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_slash_rational(x: *const Rational, y: *const Rational) -> *mut Rational {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    let boxed = Box::new(Rational::from(unsafe { &*x } / unsafe { &*y }));
    Box::into_raw(boxed)
}

/// Compare 2 rationals for equality
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Rational> == <:Rational> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn rational_eq_rational(x: *const Rational, y: *const Rational) -> bool {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    unsafe { *x == *y }
}

/// Is one rational less than another?
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Rational> < <:Rational> -> Bool
/// ```
#[no_mangle]
pub extern "C" fn rational_less_rational(x: *const Rational, y: *const Rational) -> bool {
    debug_assert!(!x.is_null());
    debug_assert!(!y.is_null());

    unsafe { *x < *y }
}

pub fn maybe_to_decimal_string(r: &Rational) -> String {
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

        let r = Rational::from((1, 3));
        assert_eq!(maybe_to_decimal_string(&r), "1/3");

        let r = Rational::from((1, 2));
        assert_eq!(maybe_to_decimal_string(&r), "0.5");

        let r = Rational::from((5, 1));
        assert_eq!(maybe_to_decimal_string(&r), "5.0");

        let r = Rational::from((5, 2));
        assert_eq!(maybe_to_decimal_string(&r), "2.5");

        let r = Rational::from((1, 4));
        assert_eq!(maybe_to_decimal_string(&r), "0.25");

        let r = Rational::from((1, 8));
        assert_eq!(maybe_to_decimal_string(&r), "0.125");

        let r = Rational::from((1, 16));
        assert_eq!(maybe_to_decimal_string(&r), "0.0625");
    }
}
