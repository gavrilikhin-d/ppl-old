use rug::Integer;

#[repr(C)]
pub struct MemoryAddress {
    pub value: *mut Integer,
}

#[no_mangle]
pub extern "C" fn memory_address_as_string(address: MemoryAddress) -> *mut String {
    assert!(!address.value.is_null());
    let value = unsafe { &*address.value };

    let hex = format!("0x{}", value.to_string_radix(16).to_uppercase());
    let boxed = Box::new(hex);
    Box::into_raw(boxed)
}
