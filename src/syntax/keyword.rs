use super::{Ranged, Token};

/// A keyword in the language
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Keyword<const KEYWORD: &'static str> {
    /// Position of the keyword in the source code
    pub offset: usize,
}

impl<const KEYWORD: &'static str> Keyword<KEYWORD> {
    /// Create a new keyword
    pub fn at(offset: usize) -> Self {
        Self { offset }
    }

    /// Get the length of the keyword
    pub fn len(&self) -> usize {
        KEYWORD.len()
    }

    /// Convert the keyword to a token
    pub fn as_token() -> Token {
        match KEYWORD {
            "none" => Token::None,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "type" => Token::Type,
            "fn" => Token::Fn,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "true" => Token::True,
            "false" => Token::False,
            "loop" => Token::Loop,
            "while" => Token::While,
            "trait" => Token::Trait,
            "use" => Token::Use,
            _ => panic!("Unknown keyword: {}", KEYWORD),
        }
    }
}

impl<const KEYWORD: &'static str> AsRef<str> for Keyword<KEYWORD> {
    fn as_ref(&self) -> &str {
        KEYWORD
    }
}

impl<const KEYWORD: &'static str> Ranged for Keyword<KEYWORD> {
    fn start(&self) -> usize {
        self.offset
    }

    fn end(&self) -> usize {
        self.offset + self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_len() {
        let keyword = Keyword::<"loop">::at(0);
        assert_eq!(keyword.len(), 4);
    }

    #[test]
    fn keyword_as_ref() {
        let keyword = Keyword::<"loop">::at(0);
        assert_eq!(keyword.as_ref(), "loop");
    }

    #[test]
    fn keyword_range() {
        let keyword = Keyword::<"loop">::at(0);
        assert_eq!(keyword.range(), 0..4);
    }
}