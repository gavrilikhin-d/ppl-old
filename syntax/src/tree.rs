use derive_more::From;
use miette::Diagnostic;

/// Parse tree consist from leaf tokens an subtrees
#[derive(Debug, From)]
pub enum ParseTree<'s> {
    /// Token
    Token(&'s str),
    /// Tree with children
    #[from(ignore)]
    Group {
        name: String,
        elements: Vec<ParseTree<'s>>,
    },
    /// Parsing error
    Error(Box<dyn Diagnostic>),
}

impl<'s> From<Vec<ParseTree<'s>>> for ParseTree<'s> {
    fn from(value: Vec<ParseTree<'s>>) -> Self {
        Self::Group {
            name: "".into(),
            elements: value,
        }
    }
}

impl PartialEq for ParseTree<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Token(a), Self::Token(b)) => a == b,
            (
                Self::Group { name, elements },
                Self::Group {
                    name: other_name,
                    elements: other_elements,
                },
            ) => name == other_name && elements == other_elements,
            (Self::Error(_), Self::Error(_)) => true,
            _ => false,
        }
    }
}
impl Eq for ParseTree<'_> {}

impl<'s> From<Vec<&'s str>> for ParseTree<'s> {
    fn from(v: Vec<&'s str>) -> Self {
        Self::Group {
            name: "".into(),
            elements: v.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl ParseTree<'_> {
    /// Create empty tree
    pub fn empty() -> Self {
        Self::Group {
            name: "".into(),
            elements: vec![],
        }
    }

    /// Create empty tree with a name
    pub fn named(name: impl Into<String>) -> Self {
        Self::Group {
            name: name.into(),
            elements: vec![],
        }
    }

    /// Get name of the tree or "", if no name
    pub fn name(&self) -> &str {
        match self {
            Self::Group { name, .. } => name,
            _ => "",
        }
    }

    /// Return tree with other name. If tree had no name, set it
    pub fn with_name(self, name: impl Into<String>) -> Self {
        match self {
            Self::Group { elements, .. } => Self::Group {
                name: name.into(),
                elements,
            },
            _ => Self::Group {
                name: name.into(),
                elements: vec![self],
            },
        }
    }

    /// Append another tree to this tree
    pub fn append(&mut self, tree: impl Into<Self>) -> &mut Self {
        let tree = tree.into();
        match self {
            Self::Group { elements, .. } => elements.push(tree),
            Self::Token(_) | Self::Error(_) => {
                let old = std::mem::replace(
                    self,
                    Self::Group {
                        name: "".into(),
                        elements: vec![],
                    },
                );
                match self {
                    Self::Group { elements, .. } => {
                        elements.push(old);
                        elements.push(tree);
                    }
                    _ => unreachable!(),
                }
            }
        };
        self
    }

    /// Return tree with element append to it
    pub fn with(mut self, tree: impl Into<Self>) -> Self {
        self.append(tree);
        self
    }

    /// Check if tree has errors
    pub fn has_errors(&self) -> bool {
        match self {
            Self::Error(_) => true,
            Self::Token(_) => false,
            Self::Group { elements, .. } => elements.iter().any(Self::has_errors),
        }
    }

    /// Check if tree has no errors
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }
}

/// Helper trait to convert errors to parse tree
pub trait IntoParseTree: Sized + Diagnostic + 'static {
    fn into_parse_tree(self) -> ParseTree<'static> {
        ParseTree::Error(Box::new(self))
    }
}

impl<'s, I: IntoParseTree + Diagnostic + 'static> From<I> for ParseTree<'s> {
    fn from(v: I) -> Self {
        v.into_parse_tree()
    }
}

#[cfg(test)]
mod test {
    use crate::{errors::Expected, ParseTree};

    #[test]
    fn append() {
        let mut tree = ParseTree::from("a");
        tree.append("b");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b"]));

        let mut tree = ParseTree::from(vec!["a", "b"]);
        tree.append("c");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b", "c"]));

        let mut tree = ParseTree::from(Expected {
            expected: "a".to_string(),
            at: 0.into(),
        });
        tree.append("b");
        assert!(tree.has_errors());
        assert_eq!(
            tree,
            ParseTree::from(vec![
                ParseTree::from(Expected {
                    expected: "a".to_string(),
                    at: 0.into(),
                }),
                ParseTree::from("b")
            ])
        );
    }
}
