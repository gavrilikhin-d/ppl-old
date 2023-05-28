# Todo
* [x] Simplify regex ast
* [x] Char in `''`
* [x] Strings in `""`
* [x] Alternatives
* [x] Sequence
* [ ] Remove parse tree
* [x] Support captures `Rule: <name: RuleName>`
* [ ] Support `["a", "b", "c"]` syntax
* [x] Support `{ name: "abc" }` syntax
* [x] Actions in sequence
* [x] Captures in sequences
* [ ] Print errors only after whole rule failed
* [ ] Add a way to obtain syntax information from variable name
* [ ] Store source code in context
* [ ] AST transformations in code.
      * [ ] no captures and single child => return child.
      * [x] captures and no action => return object with captures.
      * [ ] captures and action => return action with captures as arguments.
* [ ] Errors in code. Thrown in actions
* [ ] Export/Import rules
* [ ] Change the way rules are added to parsing to be more explicit
* [ ] Add syntax highlighting
* [ ] Support left-recursion
* [ ] Packrat parsing
* [ ] Add documentation
* [ ] Make its own library (named *dp*. Reads as *Deep*. Means  **D**ynamic **P**arser)
* [ ] Copy rules on reference
* [ ] Add `&Rule` as a way to reference a rule without copying it.