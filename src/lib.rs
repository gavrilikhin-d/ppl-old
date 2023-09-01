#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]
#![feature(lazy_cell)]

pub mod mutability;
pub mod named;

pub mod ast;
pub mod syntax;

pub mod hir;
pub mod semantics;

pub mod ir;

pub mod compilation;

pub mod from_decimal;
