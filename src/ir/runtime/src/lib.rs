/// Runtime for PPL's builtin functions
#[repr(C)]
pub struct None {
    _data: [u8; 0],
    _marker:
        core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Default constructor for PPL's [`None`](ppl::semantics::Type::None) type
#[no_mangle]
pub extern "C" fn none() -> *const None {
	core::ptr::null::<None>()
}