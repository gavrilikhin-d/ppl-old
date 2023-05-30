# Todo
* [ ] Remove parse tree
* [ ] Support integers
* [ ] Support `["a", "b", "c"]` syntax
* [x] Actions in sequence
* [x] Print errors only after whole rule failed
* [ ] Add a way to obtain syntax information from variable name
* [ ] Store source code in context
* [ ] Remove `on_parse` function from rule
* [ ] Typed objects
* [ ] AST transformations in code.
      * [ ] no captures and single child => return child.
      * [x] captures and no action => return object with captures.
      * [x] captures and action => return action with captures as arguments.
* [x] Errors in code. Thrown in actions
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