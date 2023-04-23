use std::sync::Mutex;

use crate::Rule;

/// Current parsing context
static CONTEXT: Mutex<Context> = Mutex::new(Context { rules: vec![] });

/// Parsing context
#[derive(Default)]
pub struct Context {
    /// Parsing rules
    pub rules: Vec<Rule>,
}

/// Get the current parsing context
pub fn with_context<T>(f: impl FnOnce(&mut Context) -> T) -> T {
    let mut context = CONTEXT.lock().unwrap();
    f(&mut context)
}

/// Add a rule to the current parsing context
pub fn add_rule(rule: Rule) {
    with_context(|c| c.rules.push(rule))
}
