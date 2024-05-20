use crate::String;

/// # PPL
/// ```no_run
/// @mangle_as("env")
/// fn env <:&String> -> String
/// ```
#[no_mangle]
pub extern "C" fn env(name: &String) -> String {
    std::env::var(name.as_ref()).unwrap_or_default().into()
}
