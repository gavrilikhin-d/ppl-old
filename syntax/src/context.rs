use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use miette::Diagnostic;
use once_cell::sync::Lazy;

use crate::{ParseTree, Rule, RuleName};

/// Current parsing context
static CONTEXT: Lazy<Mutex<Context>> = Lazy::new(|| Mutex::new(Context::default()));

pub trait OnParseAction =
    Sync + Send + FnMut(&ParseTree, Box<dyn Any>) -> Result<Box<dyn Any>, Box<dyn Diagnostic>>;

/// Parsing context
pub struct Context {
    /// Parsing rules
    pub rules: Vec<Arc<Rule>>,
    /// Actions to perform after parsing a rule
    pub on_parse: HashMap<RuleName, Box<dyn OnParseAction>>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            rules: vec![Arc::new(Rule {
                name: "Regex".to_string(),
                patterns: vec![r"[^\s]+".into()],
            })],
            on_parse: HashMap::new(),
        }
    }
}

/// Get the current parsing context
pub fn with_context<T>(f: impl FnOnce(&mut Context) -> T) -> T {
    let mut context = CONTEXT.lock().unwrap();
    f(&mut context)
}

/// Add a rule to the current parsing context
pub fn add_rule(rule: Rule) {
    with_context(|c| c.rules.push(Arc::new(rule)))
}

/// Find rule by name in the current parsing context
pub fn find_rule(name: &str) -> Option<Arc<Rule>> {
    with_context(|c| c.rules.iter().find(|r| r.name == name).map(|r| r.clone()))
}

/// Add an action to perform after parsing a rule
pub fn on_parse(name: &str, action: impl OnParseAction + 'static) {
    with_context(|c| {
        c.on_parse.insert(name.to_string(), Box::new(action));
    })
}
