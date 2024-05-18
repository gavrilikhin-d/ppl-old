use std::{thread::sleep, time::Duration};

use crate::Integer;

/// # PPL
/// ```no_run
/// fn sleep <ms: Integer> ms
/// ```
#[no_mangle]
pub extern "C" fn sleep_ms(ms: Integer) {
    let ms = ms.as_ref();
    let ms = ms.to_u64();
    if ms.is_none() {
        return;
    }

    let ms = ms.unwrap();
    sleep(Duration::from_millis(ms));
}
