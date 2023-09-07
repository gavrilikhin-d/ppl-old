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

* [x] Fix segmentation fault
* [x] Fix loading of pointers to pointers and wrong printing of integers
* [x] Fix infinite loop on wrong input
* [x] Save ir to file (compile file till IR and save it)
* [x] Show errors location when parsing whole module
* [x] Load builtin module from ir file
* [x] Merge `evaluate` and `execute` functions
---
### Current task
* [ ] Generic members
---
* [ ] Make statements to return `None` type for convenience
* [ ] Make `if to be an expression?
* [ ] Add `Array` type
* [ ] Add `HashMap` type
* [ ] Provide runtime as dylib
* [ ] Multiple errors support instead of exiting on first error