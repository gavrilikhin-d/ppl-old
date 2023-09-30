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

## To-do
* Rationals by default
* Algebraic effects
* Types arithmetics
* Pattern matching
* Metaprogramming
* Documentation

* [X] Generate libraries and executables
* [X] Provide runtime as dylib
* [X] Default implementations inside traits
* [x] Make `print <:String>` print without newline
---
### Current task
* [ ] More functions to stdlib
---
* [ ] Fix formatter error when `candidate is not viable`
* [ ] Multiple output types
* [ ] Generate temporary files in tmp dir
* [ ] Allow generics types usage
* [ ] Check errors in repl too
* [ ] Add more checks for compiler
* [ ] Add constrains to generics
* [ ] Make statements to return `None` type for convenience
* [ ] Make `if` to be an expression?
* [ ] Add `Array` type
* [ ] Add `HashMap` type
* [ ] Multiple errors support instead of exiting on first error
* [ ] Fix memory leak due to pointers to builtin types
* [ ] Add testing CI
* [ ] Explicit traits implementation
* [ ] Functions as values
* [ ] Add type unions `A | B`, `A & B`
* [ ] Multifile compilation and imports