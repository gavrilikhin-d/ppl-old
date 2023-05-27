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
* [ ] AST transformations in code.
      * [ ] no captures and single child => return child.
      * [x] captures and no action => return object with captures.
      * [ ] captures and action => return action with captures as arguments.
* [ ] Errors in code. Thrown in actions
* [ ] Export/Import rules
* [ ] Add syntax highlighting