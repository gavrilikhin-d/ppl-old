use std::{
    any::Any,
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;

use crate::{ParseTree, Rule, RuleName};

/// Current parsing context
static CONTEXT: Lazy<Mutex<Context>> = Lazy::new(|| {
    Mutex::new(Context {
        rules: vec![],
        on_parse: HashMap::new(),
    })
});

pub trait OnParseAction = Sync + Send + FnMut(&ParseTree, &dyn Any) -> Result<(), Box<dyn Error>>;

/// Parsing context
#[derive(Default)]
pub struct Context {
    /// Parsing rules
    pub rules: Vec<Arc<Mutex<Rule>>>,
    /// Actions to perform after parsing a rule
    pub on_parse: HashMap<RuleName, Box<dyn OnParseAction>>,
}

/// Get the current parsing context
pub fn with_context<T>(f: impl FnOnce(&mut Context) -> T) -> T {
    let mut context = CONTEXT.lock().unwrap();
    f(&mut context)
}

/// Add a rule to the current parsing context
pub fn add_rule(rule: Rule) {
    with_context(|c| c.rules.push(Arc::new(Mutex::new(rule))))
}

/// Find rule by name in the current parsing context
pub fn find_rule(name: &str) -> Option<Arc<Mutex<Rule>>> {
    with_context(|c| {
        c.rules
            .iter()
            .find(|r| r.lock().unwrap().name == name)
            .map(|r| r.clone())
    })
}

/// Add an action to perform after parsing a rule
pub fn on_parse(name: &str, action: impl OnParseAction + 'static) {
    with_context(|c| {
        c.on_parse.insert(name.to_string(), Box::new(action));
    })
}
