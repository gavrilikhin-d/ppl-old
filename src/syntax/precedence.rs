use std::collections::HashMap;

/// Associativity of operators
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Associativity {
	Left,
	Right,
}

pub struct PrecedenceGroup {
	/// Name of precedence group
	name: String,
	/// Associativity of operators in group
	associativity: Associativity,
}

type Operator = String;
type GroupName = String;

/// Precedence groups of operators
pub struct PrecedenceGroups {
	/// Precedence groups
	groups: Vec<PrecedenceGroup>,
	/// Mapping of operators to group
	operators_mapping: HashMap<Operator, GroupName>,
}

impl Default for PrecedenceGroups {
	fn default() -> Self {
		Self {
			groups: vec![
				PrecedenceGroup {
					name: "AdditionPrecedence".to_string(),
					associativity: Associativity::Left,
				},
				PrecedenceGroup {
					name: "MultiplicationPrecedence".to_string(),
					associativity: Associativity::Left,
				},
			],
			operators_mapping: vec![
				("+".to_string(), "AdditionPrecedence".to_string()),
				("-".to_string(), "AdditionPrecedence".to_string()),
				("*".to_string(), "MultiplicationPrecedence".to_string()),
				("/".to_string(), "MultiplicationPrecedence".to_string()),
			].into_iter().collect(),
		}
	}
}