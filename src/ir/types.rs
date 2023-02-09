use inkwell::{AddressSpace, context::ContextRef, types::{VoidType, IntType, StructType, PointerType}};

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

    /// LLVM unsigned int type
    pub fn u(&self, bits: u32) -> IntType<'llvm> {
        self.i(bits)
    }

    /// Get LLVM opaque struct type or create it if it doesn't exist
    fn get_or_add_opaque_struct(&self, name: &str) -> StructType<'llvm> {
        if let Some(ty) = self.llvm.get_struct_type(name) {
            return ty;
        }

        self.llvm.opaque_struct_type(name)
    }

    /// LLVM IR for [`Class`](Type::Class) type
    pub fn opaque(&self, name: &str) -> PointerType<'llvm> {
        self.get_or_add_opaque_struct(name)
            .ptr_type(AddressSpace::Generic)
    }

    /// LLVM IR for [`None`](Type::None) type
    pub fn none(&self) -> VoidType<'llvm> {
        self.void()
    }

    /// LLVM IR for [`Integer`](Type::Integer) type
    pub fn integer(&self) -> PointerType<'llvm> {
        self.opaque("Integer")
    }

    /// LLVM IR for `Rational` type
    pub fn rational(&self) -> PointerType<'llvm> {
        self.opaque("Rational")
    }

    /// LLVM IR for [`String`](Type::String) type
    pub fn string(&self) -> PointerType<'llvm> {
        self.opaque("String")
    }

    /// LLVM IR for C string type
    pub fn c_string(&self) -> PointerType<'llvm> {
        self.llvm.i8_type().ptr_type(AddressSpace::Generic)
    }
}