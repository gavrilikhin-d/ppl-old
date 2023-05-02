use std::ops::{Deref, Index};

use derive_more::From;
use miette::Diagnostic;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTree<'s> {
    /// Name of the tree. Empty string for anonymous trees
    pub name: String,
    /// Children of the subtree
    pub children: Vec<ParseTreeNode<'s>>,
}

impl<'s> ParseTree<'s> {
    /// Create empty tree
    pub fn empty() -> Self {
        Self {
            name: "".into(),
            children: vec![],
        }
    }

    /// Create empty tree with a name
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: vec![],
        }
    }

    /// Return this tree with another name
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: self.children,
        }
    }

    /// Push a node to the end of tree
    pub fn push(&mut self, node: impl Into<ParseTreeNode<'s>>) -> &mut Self {
        self.children.push(node.into());
        self
    }

    /// Return tree with element append to it
    pub fn with(mut self, node: impl Into<ParseTreeNode<'s>>) -> Self {
        self.push(node);
        self
    }

    /// Check if tree has errors
    pub fn has_errors(&self) -> bool {
        self.children.iter().any(|c| c.has_errors())
    }

    /// Check if tree has no errors
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Flatten one level of the tree,
    /// moving all children of subtrees without name to the root
    pub fn flatten(mut self) -> Self {
        let mut children = Vec::new();
        for child in self.children.drain(..) {
            match child {
                ParseTreeNode::Tree(tree) if tree.name.is_empty() => children.extend(tree.children),
                _ => children.push(child),
            }
        }
        self.children = children;
        self
    }

    /// Get subtree by name
    pub fn get(&self, name: &str) -> Option<&ParseTree<'s>> {
        self.children.iter().find_map(|c| match c {
            ParseTreeNode::Tree(tree) if tree.name == name => Some(tree),
            _ => None,
        })
    }

    /// Iterate over errors
    pub fn errors(&self) -> Box<dyn Iterator<Item = &dyn Error> + '_> {
        Box::new(self.children.iter().flat_map(|c| c.errors()))
    }

    /// Iterate over tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'s str> + '_> {
        Box::new(self.children.iter().flat_map(|c| c.tokens()))
    }
}

impl Serialize for ParseTree<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.name.is_empty() {
            if self.children.len() == 1 {
                self.children[0].serialize(serializer)
            } else {
                self.children.serialize(serializer)
            }
        } else {
            let mut map = serializer.serialize_map(Some(1))?;
            if self.children.len() == 1 {
                map.serialize_entry(&self.name, &self.children[0])?;
            } else {
                map.serialize_entry(&self.name, &self.children)?;
            }
            map.end()
        }
    }
}

impl<'s> Index<&str> for ParseTree<'s> {
    type Output = ParseTree<'s>;

    fn index(&self, name: &str) -> &Self::Output {
        self.get(name)
            .expect(format!("No subtree with name '{name}'").as_str())
    }
}

impl<'s> From<&'s str> for ParseTree<'s> {
    fn from(child: &'s str) -> Self {
        Self {
            name: "".into(),
            children: vec![child.into()],
        }
    }
}

impl<'s> From<ParseTreeNode<'s>> for ParseTree<'s> {
    fn from(child: ParseTreeNode<'s>) -> Self {
        Self {
            name: "".into(),
            children: vec![child.into()],
        }
    }
}

impl<'s, I: IntoParseTreeNode> From<I> for ParseTree<'s> {
    fn from(child: I) -> Self {
        Self {
            name: "".into(),
            children: vec![child.into()],
        }
    }
}

impl<'s> From<Vec<ParseTreeNode<'s>>> for ParseTree<'s> {
    fn from(children: Vec<ParseTreeNode<'s>>) -> Self {
        Self {
            name: "".into(),
            children,
        }
    }
}

impl<'s> From<Vec<ParseTree<'s>>> for ParseTree<'s> {
    fn from(children: Vec<ParseTree<'s>>) -> Self {
        Self {
            name: "".into(),
            children: children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl<'s> From<Vec<&'s str>> for ParseTree<'s> {
    fn from(children: Vec<&'s str>) -> Self {
        Self {
            name: "".into(),
            children: children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'s> {
    /// Token's string value
    pub value: &'s str,
    /// Spaces between this token and the previous one
    pub trivia: &'s str,
}

impl<'s> Token<'s> {
    /// Get token's value
    pub fn as_str(&self) -> &'s str {
        self.value
    }
}

impl Deref for Token<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'s> From<&'s str> for Token<'s> {
    fn from(value: &'s str) -> Self {
        Self { value, trivia: "" }
    }
}

impl<'s> From<&'s str> for ParseTreeNode<'s> {
    fn from(value: &'s str) -> Self {
        Self::Token(value.into())
    }
}

impl Serialize for Token<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.trivia.is_empty() {
            return serializer.serialize_str(self.value);
        }

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("value", self.value)?;
        map.serialize_entry("trivia", &self.trivia)?;
        map.end()
    }
}

impl<'de: 's, 's> Deserialize<'de> for Token<'s> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TokenDTO<'s> {
            Value(&'s str),
            ValueWithTrivia { value: &'s str, trivia: &'s str },
        }

        let dto = TokenDTO::deserialize(deserializer)?;
        Ok(match dto {
            TokenDTO::Value(value) => Self { value, trivia: "" },
            TokenDTO::ValueWithTrivia { value, trivia } => Self { value, trivia },
        })
    }
}

/// Parse tree consist from leaf tokens an subtrees
#[derive(Debug, From)]
pub enum ParseTreeNode<'s> {
    /// Token
    Token(Token<'s>),
    /// Subtree
    Tree(ParseTree<'s>),
    /// Parsing error
    Error(Box<dyn Error>),
}

impl<'s> ParseTreeNode<'s> {
    /// Check if tree node has errors
    pub fn has_errors(&self) -> bool {
        match self {
            Self::Token(_) => false,
            Self::Tree(tree) => tree.has_errors(),
            Self::Error(_) => true,
        }
    }

    /// Check if tree node has no errors
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Iterate over errors
    pub fn errors(&self) -> Box<dyn Iterator<Item = &dyn Error> + '_> {
        match self {
            Self::Token(_) => Box::new(std::iter::empty()),
            Self::Tree(tree) => tree.errors(),
            Self::Error(err) => Box::new(std::iter::once(err.as_ref())),
        }
    }

    /// Iterate over tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'s str> + '_> {
        match self {
            Self::Token(token) => Box::new(std::iter::once(token.as_str())),
            Self::Tree(tree) => tree.tokens(),
            Self::Error(_) => Box::new(std::iter::empty()),
        }
    }
}

impl PartialEq for ParseTreeNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Token(a), Self::Token(b)) => a == b,
            (Self::Tree(a), Self::Tree(b)) => a == b,
            (Self::Error(_), Self::Error(_)) => true,
            _ => false,
        }
    }
}
impl Eq for ParseTreeNode<'_> {}

/// Helper trait to convert errors to parse tree
pub trait IntoParseTreeNode: Sized + Error {
    fn into_parse_tree_node(self) -> ParseTreeNode<'static> {
        ParseTreeNode::Error(Box::new(self))
    }
}

impl<'s, I: IntoParseTreeNode + Diagnostic + 'static> From<I> for ParseTreeNode<'s> {
    fn from(v: I) -> Self {
        v.into_parse_tree_node()
    }
}

impl Serialize for ParseTreeNode<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Token(token) => token.serialize(serializer),
            Self::Tree(tree) => tree.serialize(serializer),
            Self::Error(err) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("error", err)?;
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{errors::Expected, ParseTree, Token};

    #[test]
    fn create() {
        let tree = ParseTree::from("a").with("b");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b"]));

        let tree = ParseTree::from(vec!["a", "b"]).with("c");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b", "c"]));

        let tree = ParseTree::from(Expected {
            expected: "a".to_string(),
            at: 0,
        })
        .with("b");
        assert!(tree.has_errors());
        assert_eq!(
            tree,
            ParseTree::from(Expected {
                expected: "a".to_string(),
                at: 0,
            })
            .with("b")
        );
    }

    #[test]
    fn name() {
        let tree = ParseTree::from(vec![ParseTree::named("name").with("a")]);
        assert_eq!(tree["name"], ParseTree::named("name").with("a"));
        assert_eq!(tree.get("invalid"), None);
    }

    #[test]
    fn serialize() {
        let tree = ParseTree::named("A")
            .with("a")
            .with(ParseTree::named("B").with(Token {
                value: "b",
                trivia: " ",
            }))
            .with(Expected {
                expected: "c".to_string(),
                at: 2,
            });
        assert_eq!(
            serde_json::to_value(&tree).unwrap(),
            json!({
                "A": [
                    "a",
                    {"B": {"value": "b", "trivia": " "}},
                    {"error": {"expected": "c", "at": 2}}
                ]
            })
        )
    }

    #[test]
    fn deserialize() {
        let token: Token = serde_json::from_str("\"a\"").unwrap();
        assert_eq!(token, Token::from("a"));

        let source = json!({"value": "a", "trivia": " "}).to_string();
        let token: Token = serde_json::from_str(&source).unwrap();
        assert_eq!(
            token,
            Token {
                value: "a",
                trivia: " "
            }
        )
    }
}
