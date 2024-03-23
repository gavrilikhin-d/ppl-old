#![feature(trait_upcasting)]
#![feature(debug_closure_helpers)]

pub mod diagnostic;
pub mod parser;
pub mod source;

mod db;
pub use db::*;

#[cfg(test)]
mod tests;
