use crate::String;

/// # PPL
/// ```no_run
/// @mangle_as("assert")
/// fn assert <condition: Bool> <message: &String>
/// ```
#[no_mangle]
pub extern "C" fn assert(condition: bool, message: &String) {
    let message = unsafe { message.data.as_ref().unwrap() };
    if !condition {
        println!("Assertion failed: {message}");
    }
    assert!(condition, "{message}");
}
