use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum};

/// Convenience trait for inkwell
pub(crate) trait TryIntoBasicTypeEnum<'ctx>: TryInto<BasicTypeEnum<'ctx>> {
    /// Convert to [`BasicTypeEnum`](inkwell::types::BasicTypeEnum)
    fn try_into_basic_type(self) -> Result<BasicTypeEnum<'ctx>, Self::Error>;
}

impl<'ctx> TryIntoBasicTypeEnum<'ctx> for AnyTypeEnum<'ctx> {
    fn try_into_basic_type(self) -> Result<BasicTypeEnum<'ctx>, Self::Error> {
        self.try_into()
    }
}

/// Convenience trait for inkwell
pub(crate) trait FnType<'ctx> {
    /// Creates a `FunctionType` with this type for its return types
    fn fn_type(
        self,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
    ) -> inkwell::types::FunctionType<'ctx>;
}

impl<'ctx> FnType<'ctx> for AnyTypeEnum<'ctx> {
    fn fn_type(
        self,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
    ) -> inkwell::types::FunctionType<'ctx> {
        if self.is_void_type() {
            self.into_void_type().fn_type(param_types, is_var_args)
        } else {
            self.try_into_basic_type()
                .expect("Non-void and non basic return type")
                .fn_type(param_types, is_var_args)
        }
    }
}
