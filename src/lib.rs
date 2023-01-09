#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(once_cell)]

pub mod mutability;
pub mod named;

pub mod ast;
pub mod syntax;

pub mod hir;
pub mod semantics;

pub mod ir;
