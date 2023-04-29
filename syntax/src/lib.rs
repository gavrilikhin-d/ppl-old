#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(assert_matches)]
#![feature(is_some_and)]
#![feature(const_trait_impl)]

mod tree;
pub use tree::*;

pub mod patterns;
pub use patterns::Pattern;

mod rule;
pub use rule::*;

pub mod parsers;

pub mod context;
pub use context::Context;

pub mod errors;
