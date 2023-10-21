mod token;
pub use token::*;

mod lexer;
pub use lexer::*;

pub mod error;

mod with_offset;
pub use with_offset::*;

mod ranged;
pub use ranged::*;

mod parse;
pub use parse::*;

mod precedence;
pub use precedence::*;

#[cfg(test)]
mod tests {
    use crate::test_compilation_result;

    test_compilation_result!(consume_greater);
    test_compilation_result!(multiple_errors);
}
