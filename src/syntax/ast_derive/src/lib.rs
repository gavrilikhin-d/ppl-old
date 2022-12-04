use proc_macro::TokenStream;
use quote::quote;
use syn;

/// Auto-implement `FromStr` for AST node
///
/// AST node must implement `Parse` trait
#[proc_macro_derive(AST)]
pub fn ast_macro_derive(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();

	impl_ast_macro(&ast)
}

fn impl_ast_macro(ast: &syn::DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let gen = quote! {
		impl FromStr for #name {
			type Err = <#name as Parse>::Err;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				let mut lexer = Token::lexer(s);

				let res = #name::parse(&mut lexer);

				let token = lexer.next();
				if token != None {
					return Err(
						UnexpectedToken {
							expected: vec![],
							got: token.unwrap(),
							at: lexer.span().into()
						}.into()
					);
				}
				res
			}
		}
	};

	gen.into()
}