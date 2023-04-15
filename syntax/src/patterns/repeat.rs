use crate::{GroupMatch, Match, Parser, Pattern};

/// Repeat pattern
pub struct Repeat {
    /// Minimum number of times to repeat
    pub at_least: usize,
    /// Maximum number of times to repeat
    pub at_most: Option<usize>,
    /// Pattern to repeat
    pub pattern: Box<Pattern>,
}

impl Repeat {
    /// Repeat pattern zero times or once
    pub fn zero_or_once(pattern: Pattern) -> Self {
        Self {
            at_least: 0,
            at_most: Some(1),
            pattern: Box::new(pattern),
        }
    }

    /// Repeat pattern zero or more times
    pub fn zero_or_more(pattern: Pattern) -> Self {
        Self {
            at_least: 0,
            at_most: None,
            pattern: Box::new(pattern),
        }
    }

    /// Repeat pattern zero or more times
    pub fn once_or_more(pattern: Pattern) -> Self {
        Self {
            at_least: 1,
            at_most: None,
            pattern: Box::new(pattern),
        }
    }

    /// Repeat up to `at_most` times
    pub fn up_to(at_most: usize, pattern: Pattern) -> Self {
        Self {
            at_least: 0,
            at_most: Some(at_most),
            pattern: Box::new(pattern),
        }
    }

    /// Repeat at least `at_least` times
    pub fn at_least(at_least: usize, pattern: Pattern) -> Self {
        Self {
            at_least,
            at_most: None,
            pattern: Box::new(pattern),
        }
    }

    /// Repeat pattern exactly `n` times
    pub fn n_times(n: usize, pattern: Pattern) -> Self {
        Self {
            at_least: n,
            at_most: Some(n),
            pattern: Box::new(pattern),
        }
    }
}

impl Repeat {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut (impl Iterator<Item = &'source str> + Clone),
        parser: &mut Parser,
    ) -> GroupMatch<'source> {
        debug_assert!(self.at_most == None || self.at_most.unwrap() >= self.at_least);

        let mut matched = Vec::new();

        for _ in 0..self.at_least {
            let m = self.pattern.apply(source, tokens, parser);
            if m.has_error() {
                unimplemented!("error in repeat")
            }
            matched.push(m);
        }

        while self.at_most.is_none() || matched.len() < self.at_most.unwrap() {
            let tokens_copy = tokens.clone();
            let m = self.pattern.apply(source, tokens, parser);
            if m.has_error() {
                *tokens = tokens_copy;
                break;
            }

            matched.push(m);
        }

        GroupMatch {
            name: String::new(),
            matched,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{patterns::Repeat, Match, Parser};

    #[test]
    fn not_enough() {
        let mut parser = Parser::default();
        let pattern = Repeat::n_times(2, "a".try_into().unwrap());
        let mut tokens = vec!["a"].into_iter();
        let m = pattern.apply("a", &mut tokens, &mut parser);
        assert!(m.has_error());
        assert_eq!(m.matched.len(), 2);
        assert_eq!(m.matched[0].as_token(), "a");
    }
}
