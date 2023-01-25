use std::collections::{HashMap, HashSet};
use std::path::Path;

use derive_more::From;

use crate::ast;
use crate::hir::{FunctionDeclaration, Statement, TypeDeclaration, VariableDeclaration};
use crate::named::{HashByName, Named};
use crate::semantics::{ASTLoweringContext, ASTLoweringWithinContext};
use std::sync::{Arc, LazyLock};

use super::{Type, TraitDeclaration};

/// Class or trait
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ClassOrTrait {
	/// Class declaration
	Class(Arc<TypeDeclaration>),
	/// Trait declaration
	Trait(Arc<TraitDeclaration>),
}

impl From<ClassOrTrait> for Type {
	fn from(class_or_trait: ClassOrTrait) -> Self {
		match class_or_trait {
			ClassOrTrait::Class(c) => Type::Class(c),
			ClassOrTrait::Trait(t) => Type::Trait(t),
		}
	}
}

impl Named for ClassOrTrait {
	fn name(&self) -> &str {
		match self {
			ClassOrTrait::Class(c) => c.name(),
			ClassOrTrait::Trait(t) => t.name(),
		}
	}
}

/// Module with PPL code
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Module {
    /// Name of the module
    pub name: String,

	/// Filename of module
	pub filename: String,

	/// Is this a builtin module?
	pub is_builtin: bool,

    /// Variables, declared in this module
    pub variables: HashSet<HashByName<Arc<VariableDeclaration>>>,

    /// Types, declared in this module
    pub types: HashSet<HashByName<ClassOrTrait>>,

    /// Functions, declared in this module
    pub functions: HashMap<
        String, // Name format
        HashSet<HashByName<Arc<FunctionDeclaration>>>,
    >,

    /// Statements in this module
    pub statements: Vec<Statement>,
}

static BUILTIN: LazyLock<Arc<Module>> =
	LazyLock::new(|| Arc::new(Module::create_builtin()));

impl Module {
    /// Create an empty module
    pub fn new(name: &str, filename: &str) -> Self {
        Self {
            name: name.to_string(),
			filename: filename.to_string(),
			is_builtin: false,
            variables: HashSet::new(),
            types: HashSet::new(),
            functions: HashMap::new(),
            statements: vec![],
        }
    }

	/// Create module from file with providing builtin module
	fn from_file_with_builtin(path: &Path, is_builtin: bool)
		-> miette::Result<Self>
	{
        let content =
            std::fs::read_to_string(path)
			.expect(
				format!("Failed to read {}", path.to_str().unwrap()).as_str()
			);

		let ast = content.parse::<ast::Module>()?;

		let mut module = Module::new(
			path.file_stem().unwrap().to_str().unwrap(),
			path.to_str().unwrap()
		);
		module.is_builtin = is_builtin;
		let mut context = ASTLoweringContext::new(module);

		Ok(ast.lower_to_hir_within_context(&mut context)?)
	}

	/// Create module from file
	pub fn from_file(path: &Path) -> miette::Result<Self> {
		Module::from_file_with_builtin(path, false)
	}

    /// Create builtin module
    fn create_builtin_from_file(path: &Path) -> miette::Result<Self> {
       Module::from_file_with_builtin(path, true)
    }

    /// Create builtin module
    pub(crate) fn create_builtin() -> Self {
		let path = Path::new(
			concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime/ppl.ppl")
		);

        let module = Self::create_builtin_from_file(path);
        if let Err(err) = module {
            panic!(
                "Error in builtin module: {:?}",
                err.with_source_code(
					miette::NamedSource::new(
						path.file_name().unwrap().to_str().unwrap(),
						std::fs::read_to_string(path).unwrap()
					)
				)
            );
        }

        module.unwrap()
    }

    /// Get builtin module
    ///
    /// # Example
    /// ```
    /// use ppl::hir::Module;
    ///
    /// let module = Module::builtin();
    /// ```
    pub fn builtin() -> Arc<Self> {
        BUILTIN.clone()
    }

	/// Insert function to module
	pub fn insert_function(&mut self, function: Arc<FunctionDeclaration>) {
		let set =
			self.functions
				.entry(function.name_format().to_string())
				.or_insert_with(HashSet::new);
		set.insert(function.into());
	}

	/// Define previously declared function
	pub fn define_function(&mut self, function: Arc<FunctionDeclaration>) {
		self.functions
			.get_mut(function.name_format())
			.expect("function was not predeclared")
			.replace(function.clone().into());
	}

	/// Iterate over all functions with `n` name parts
	pub fn functions_with_n_name_parts(&self, n: usize) -> impl Iterator<Item = Arc<FunctionDeclaration>> + '_ {
		self.functions.values().flatten().map(|f| f.value.clone())
	}
}

impl Named for Module {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Default for Module {
	fn default() -> Self {
		Self::new("main", "")
	}
}
