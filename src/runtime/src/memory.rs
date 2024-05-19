use std::sync::Arc;

use libc::{c_void, malloc, memcpy, size_t};

use crate::{integer_from_i64, integer_from_u64, Integer, String, Type};

#[repr(C)]
pub struct MemoryAddress {
    pub value: Integer,
}

/// # PPL
/// ```no_run
/// fn String from <address: MemoryAddress> -> String
/// ```
#[no_mangle]
pub extern "C" fn memory_address_as_string(address: MemoryAddress) -> String {
    let value = address.value.as_ref();

    let hex = format!("0x{}", value.to_string_radix(16).to_uppercase());
    hex.into()
}

/// # PPL
/// ```no_run
/// fn allocate <n: Integer> bytes -> MemoryAddress
/// ```
#[no_mangle]
pub extern "C" fn allocate_n_bytes(n: Integer) -> MemoryAddress {
    let n = n.as_ref();

    let n = n.to_usize();
    if n.is_none() {
        return MemoryAddress {
            value: integer_from_i64(0),
        };
    }
    let n = n.unwrap();

    let address = unsafe { malloc(n) } as u64;

    MemoryAddress {
        value: integer_from_u64(address),
    }
}

/// # PPL
/// ```no_run
/// fn free <address: &MemoryAddress>
/// ```
#[no_mangle]
pub extern "C" fn free_memory(address: &MemoryAddress) {
    let address = address.value.as_ref();

    let address = address.to_u64();
    if address.is_none() {
        return;
    }
    let address = address.unwrap();

    unsafe {
        libc::free(address as *mut libc::c_void);
    }
}

/// # PPL
/// ```no_run
/// fn<T> <ty: Type<T>> at <address: &MemoryAddress> -> Reference<T>
/// ```
#[no_mangle]
pub extern "C" fn read_memory(_ty: Type, address: &MemoryAddress) -> *mut c_void {
    read_memory_impl(address)
}

fn read_memory_impl(address: &MemoryAddress) -> *mut c_void {
    let address = address.value.as_ref();

    let address = address.to_u64().unwrap();

    address as *mut libc::c_void
}

/// # PPL
/// ```no_run
/// /// Get memory address of a reference
/// @mangle_as("address_of")
/// fn<T> address of <ref: &T> -> MemoryAddress
/// ```
#[no_mangle]
pub extern "C" fn address_of(ptr: *const c_void) -> MemoryAddress {
    let address = ptr as usize;

    MemoryAddress {
        value: integer_from_u64(address as u64),
    }
}

/// # PPL
/// ```no_run
/// /// Copy `n` bytes from `src` to `dst`
/// @mangle_as("copy_bytes")
/// fn copy <n: &Integer> bytes from <src: &MemoryAddress> to <dst: &MemoryAddress>
/// ```
#[no_mangle]
pub extern "C" fn copy_bytes(n: &Integer, src: &MemoryAddress, dst: &MemoryAddress) {
    let dest = read_memory_impl(dst);
    let src = read_memory_impl(src);
    let n = n.as_ref().to_usize().unwrap() as size_t;
    unsafe { memcpy(dest, src, n) };
}

#[no_mangle]
pub extern "C" fn create_arc(bytes: usize) -> *const c_void {
    let bytes = unsafe { Arc::<[u8]>::new_zeroed_slice(bytes).assume_init() };
    Arc::into_raw(bytes) as *const c_void
}

#[no_mangle]
pub extern "C" fn increment_strong_count(ptr: *const c_void) {
    unsafe { Arc::increment_strong_count(ptr) }
}

#[no_mangle]
pub extern "C" fn decrement_strong_count(ptr: *const c_void) {
    unsafe { Arc::decrement_strong_count(ptr) }
}
