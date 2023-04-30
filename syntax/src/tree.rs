use derive_more::From;
use miette::Diagnostic;

/// Parse tree consist from leaf tokens an subtrees
#[derive(Debug, From)]
pub enum ParseTree<'s> {
    /// Token
    Token(&'s str),
    /// Tree with children
    Tree(Vec<ParseTree<'s>>),
    /// Parsing error
    Error(Box<dyn Diagnostic>),
}

impl PartialEq for ParseTree<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Token(a), Self::Token(b)) => a == b,
            (Self::Tree(a), Self::Tree(b)) => a == b,
            (Self::Error(_), Self::Error(_)) => true,
            _ => false,
        }
    }
}
impl Eq for ParseTree<'_> {}

impl<'s> From<Vec<&'s str>> for ParseTree<'s> {
    fn from(v: Vec<&'s str>) -> Self {
        Self::Tree(v.into_iter().map(|s| s.into()).collect())
    }
}

impl ParseTree<'_> {
    /// Append another tree to this tree
    pub fn append(&mut self, tree: impl Into<Self>) -> &mut Self {
        let tree = tree.into();
        match self {
            Self::Tree(v) => v.push(tree),
            Self::Token(_) | Self::Error(_) => {
                let old = std::mem::replace(self, Self::Tree(vec![]));
                match self {
                    Self::Tree(v) => {
                        v.push(old);
                        v.push(tree);
                    }
                    _ => unreachable!(),
                }
            }
        };
        self
    }

    /// Check if tree has errors
    pub fn has_errors(&self) -> bool {
        match self {
            Self::Error(_) => true,
            Self::Token(_) => false,
            Self::Tree(v) => v.iter().any(Self::has_errors),
        }
    }

    /// Check if tree has no errors
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }
}

#[cfg(test)]
mod test {
    use crate::ParseTree;

    #[test]
    fn append() {
        let mut tree = ParseTree::from("a");
        tree.append("b");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b"]));

        let mut tree = ParseTree::from(vec!["a", "b"]);
        tree.append("c");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b", "c"]))
    }
}
