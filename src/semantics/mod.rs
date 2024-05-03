mod to_hir;
pub use to_hir::*;

pub mod clone;

mod contexts;
pub use contexts::*;

mod monomorphize;
pub use monomorphize::*;

mod declare;
pub use declare::*;

pub mod error;

mod find_declaration;
pub use find_declaration::*;

mod add_declaration;
pub use add_declaration::*;

mod implements;
pub use implements::*;

mod convert;
pub use convert::*;

mod implicit;
pub use implicit::*;

mod destructors;
pub use destructors::*;

mod tmp;
pub use tmp::*;

mod unnamed;
pub use unnamed::*;

mod replace_self;
pub use replace_self::*;

mod link_impls;
pub use link_impls::*;
