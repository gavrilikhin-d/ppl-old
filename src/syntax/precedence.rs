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
type GroupIndex = usize;

/// Precedence groups of operators
pub struct PrecedenceGroups {
	/// Precedence groups
	groups: Vec<PrecedenceGroup>,
	/// Mapping of operators to group
	operators_mapping: HashMap<Operator, GroupIndex>,
}

impl PrecedenceGroups {
	/// Get precedence group index
	fn get_precedence_group_index(&self, op: &str) -> GroupIndex {
		self.operators_mapping.get(op).cloned().unwrap_or(0)
	}

	/// Check that next operator has greater precedence than previous
	pub fn has_greater_precedence(&self, next: &str, prev: &str) -> bool {
		dbg!(&prev);
		dbg!(&next);
		let next_group_index =
			self.get_precedence_group_index(next);
		let prev_group_index =
			self.get_precedence_group_index(prev);
		if next_group_index == prev_group_index {
			return
				next == prev &&
				self.groups[next_group_index].associativity == Associativity::Right;
		}
		next_group_index > prev_group_index
	}

	/// Check that next operator has less precedence than previous
	pub fn has_less_precedence(&self, next: &str, prev: &str) -> bool {
		dbg!(&prev);
		dbg!(&next);
		let next_group_index =
			self.get_precedence_group_index(next);
		let prev_group_index =
			self.get_precedence_group_index(prev);
		if next_group_index == prev_group_index {
			return
				next == prev &&
				self.groups[next_group_index].associativity == Associativity::Left;
		}
		next_group_index < prev_group_index
	}
}

impl Default for PrecedenceGroups {
	fn default() -> Self {
		Self {
			groups: vec![
				PrecedenceGroup {
					name: "DefaultPrecedence".to_string(),
					associativity: Associativity::Left,
				},
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
				("+".to_string(), 1),
				("-".to_string(), 1),
				("*".to_string(), 2),
				("/".to_string(), 2),
			].into_iter().collect(),
		}
	}
}