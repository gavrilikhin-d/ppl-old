#![feature(is_some_and)]

mod r#match;
pub use r#match::*;

pub mod error;
pub use error::Error;

pub mod patterns;
pub use patterns::Pattern;

mod rule;
pub use rule::*;

mod subslice;
pub use subslice::*;
