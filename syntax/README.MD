# Todo
* [ ] Remove parse tree
* [ ] Support `["a", "b", "c"]` syntax
* [ ] Add a way to obtain syntax information from variable name
* [ ] Store source code in context
* [ ] Remove `on_parse` function from rule
* [x] Distinct types
* [x] Write all rules in bootstrap
* [x] Allow type variables in type cast
* [ ] Move logic to syntax
* [ ] Export/Import rules
* [ ] Change the way rules are added to parsing to be more explicit
* [ ] Add syntax highlighting
* [ ] Support left-recursion
* [ ] Packrat parsing
* [ ] Add documentation
* [ ] Make its own library (named *dp*. Reads as *Deep*. Means  **D**ynamic **P**arser)
* [ ] Copy rules on reference
* [ ] Add `&Rule` as a way to reference a rule without copying it.
* [ ] Simplify separated patterns
* [ ] Add `#[derive(Rule)]`
      * [ ] Add `#[rule]` attribute
	  * [ ] Add `Self::rule()` method
	  * [ ] Add `Self::parse(str: &str)` method
      * [ ] Implement deserialize