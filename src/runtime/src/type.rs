use rug::Integer;

/// Runtime type information
///
/// # PPL
/// ```no_run
/// type Type<T>:
///     name: String
///     size: Integer
/// ```
#[repr(C)]
pub struct Type {
    pub name: *mut String,
    pub size: *mut Integer,
}
