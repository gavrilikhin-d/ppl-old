use std::{thread::sleep, time::Duration};

use rug::Integer;

/// # PPL
/// ```no_run
/// fn sleep <ms: Integer> ms
/// ```
#[no_mangle]
pub extern "C" fn sleep_ms(ms: *const Integer) {
    debug_assert!(!ms.is_null());

    let ms = unsafe { &*ms };
    let ms = ms.to_u64();
    if ms.is_none() {
        return;
    }

    let ms = ms.unwrap();
    sleep(Duration::from_millis(ms));
}
