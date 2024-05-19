use std::{thread::sleep, time::Duration};

use crate::Integer;

/// # PPL
/// ```no_run
/// fn sleep <ms: Integer> ms
/// ```
#[no_mangle]
pub extern "C" fn sleep_ms(ms: Integer) {
    let ms = unsafe { ms.data.as_ref().unwrap() };
    let ms = ms.to_u64();
    if ms.is_none() {
        return;
    }

    let ms = ms.unwrap();
    sleep(Duration::from_millis(ms));
}
