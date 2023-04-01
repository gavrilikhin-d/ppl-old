mod r#match;
pub use r#match::*;

pub mod error;
pub use error::Error;

mod parser;
pub use parser::*;

pub mod patterns;
pub use patterns::Pattern;

mod rule;
pub use rule::*;

mod subslice;
pub use subslice::*;
