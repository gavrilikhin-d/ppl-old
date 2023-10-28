#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]
#![feature(lazy_cell)]
#![feature(assert_matches)]
#![feature(trait_upcasting)]

pub mod mutability;
pub mod named;

pub mod ast;
pub mod syntax;

pub mod hir;
pub mod semantics;

pub mod ir;

pub mod compilation;

pub mod from_decimal;

pub mod driver;

mod source_file;
pub use source_file::*;

mod source_location;
pub use source_location::*;

pub(crate) mod test_compilation_result;

mod reporter;
pub use reporter::*;

mod err_vec;
pub use err_vec::*;
