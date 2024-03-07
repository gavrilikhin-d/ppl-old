use crate::{Integer, String};

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
    pub name: String,
    pub size: Integer,
}
