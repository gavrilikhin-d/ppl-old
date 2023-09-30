use rug::Rational;

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

    let boxed = Box::new(unsafe { &*r }.to_string());
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
