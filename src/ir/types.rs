use inkwell::{
    context::ContextRef,
    types::{FloatType, IntType, PointerType, StructType, VoidType},
    AddressSpace,
};

/// LLVM IR for PPL's types
pub struct Types<'llvm> {
    /// LLVM context
    llvm: ContextRef<'llvm>,
}

impl<'llvm> Types<'llvm> {
    /// Initialize LLVM IR for PPL's types
    pub(crate) fn new(llvm: ContextRef<'llvm>) -> Self {
        Self { llvm }
    }

    /// LLVM void type
    pub fn void(&self) -> VoidType<'llvm> {
        self.llvm.void_type()
    }

    /// LLVM bool type
    pub fn bool(&self) -> IntType<'llvm> {
        self.llvm.bool_type()
    }

    /// LLVM int type
    pub fn i(&self, bits: u32) -> IntType<'llvm> {
        self.llvm.custom_width_int_type(bits)
    }

    /// LLVM 32-bit int type
    pub fn i32(&self) -> IntType<'llvm> {
        self.i(32)
    }

    /// LLVM 64-bit int type
    pub fn i64(&self) -> IntType<'llvm> {
        self.i(64)
    }

    /// LLVM unsigned int type
    pub fn u(&self, bits: u32) -> IntType<'llvm> {
        self.i(bits)
    }

    /// LLVM 32-bit unsigned int type
    pub fn u32(&self) -> IntType<'llvm> {
        self.u(32)
    }

    /// LLVM 64-bit unsigned int type
    pub fn u64(&self) -> IntType<'llvm> {
        self.u(64)
    }

    /// LLVM 64-bit float type
    pub fn f64(&self) -> FloatType<'llvm> {
        self.llvm.f64_type()
    }

    /// Get LLVM opaque struct type or create it if it doesn't exist
    fn get_or_add_opaque_struct(&self, name: &str) -> StructType<'llvm> {
        if let Some(ty) = self.llvm.get_struct_type(name) {
            return ty;
        }

        self.llvm.opaque_struct_type(name)
    }

    /// Get wrapper around pointer to opaque impl
    fn with_impl(&self, name: &str) -> StructType<'llvm> {
        if let Some(ty) = self.llvm.get_struct_type(name) {
            return ty;
        }

        let ty = self.llvm.opaque_struct_type(name);
        ty.set_body(&[self.opaque(&format!("{name}Impl")).into()], false);
        ty
    }

    /// LLVM IR for [`Class`](Type::Class) type
    pub fn opaque(&self, name: &str) -> PointerType<'llvm> {
        self.get_or_add_opaque_struct(name);
        self.llvm.ptr_type(AddressSpace::default())
    }

    /// LLVM IR for [`None`](Type::None) type
    pub fn none(&self) -> VoidType<'llvm> {
        self.void()
    }

    /// LLVM IR for [`Integer`](Type::Integer) type
    pub fn integer(&self) -> StructType<'llvm> {
        self.with_impl("Integer")
    }

    /// LLVM IR for `Rational` type
    pub fn rational(&self) -> StructType<'llvm> {
        self.with_impl("Rational")
    }

    /// LLVM IR for [`String`](Type::String) type
    pub fn string(&self) -> StructType<'llvm> {
        self.with_impl("String")
    }

    /// LLVM IR for C string type
    pub fn c_string(&self) -> PointerType<'llvm> {
        self.llvm.ptr_type(AddressSpace::default())
    }
}
