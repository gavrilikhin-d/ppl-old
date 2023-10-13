# PPL

**PPL** - **P**seudo-**P**rogramming **L**anguage

## Goals

* Convenience
* Simplicity
* Readability
* Safety

## Done
* Mixfix operators
* Big integers
* Generics (traits)
* Rationals by default

## To-do
* Algebraic effects
* Types arithmetics
* Pattern matching
* Metaprogramming
* Documentation

* [X] Generate libraries and executables
* [X] Provide runtime as dylib
* [X] Default implementations inside traits
* [X] Make `print <:String>` print without newline
* [X] Add testing CI
* [x] Fix formatter error when `candidate is not viable`
* [x] Generic functions
* [x] Specialize types of expressions inside generic functions
* [x] Generate temporary files in tmp dir
* [x] Allow generics types usage
* [x] Fix UB in errors printing order due to hash maps
* [x] Add `SourceFile` as source code for errors
* [x] Simplify builtin module compilation
---
### Current task
* [ ] Print exactly decimals in decimal form
---
* [ ] Add `@builtin`
* [ ] Remove cached names from functions
* [ ] Remove unnecessary information from errors
* [ ] More functions to stdlib
* [ ] Multiple output types
* [ ] Check errors in repl too
* [ ] Add more checks for compiler
* [ ] Add constrains to generics
* [ ] Make statements to return `None` type for convenience
* [ ] Make `if` to be an expression?
* [ ] Add `Array` type
* [ ] Add `HashMap` type
* [ ] Multiple errors support instead of exiting on first error
* [ ] Fix memory leak due to pointers to builtin types
* [ ] Explicit traits implementation
* [ ] Functions as values
* [ ] Add type unions `A | B`, `A & B`
* [ ] Multifile compilation and imports
* [ ] Add all `c` types

## Important Implementation Details
* Use `BTreeMap` instead of `HashMap` to guarantee order of errors