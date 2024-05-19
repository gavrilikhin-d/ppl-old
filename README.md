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

* [x] Split builtin module into several files
* [x] Fix crash in diagnostics due-to wrong source file
* [x] Check result of the program itself in `test_compiler_result`
* [x] Add `Array` type
* [x] Fix memory leak due to pointers to builtin types
* [x] Destructors for parameters
* [x] No tmps for literals inside of constructors
* [x] Fix recursive trait (AsString with prints)
* [x] Fix references in traits test
* [x] Fix self mapping
---
### Current task
* [ ] Printable trait should take references
---
* [ ] migrate to pass-by-ref (branch `arc`)
* [ ] Prefer candidates with mutable references, when possible
* [ ] Fix problems with to_ir and loading references (especially globals). This causes issues in iterator
* [ ] Benchmark for linear algebra
* [ ] Sum of series benchmark
* [ ] Use traits to check for `clone` and `destructoy` functions
* [ ] Forbid recursion without `@recursive` annotation
* [ ] Generate clone for types with clonable members
* [ ] Generate destructors for types with destructible members
* [ ] Add type aliases
* [ ] Add dependency analysis for modules and declarations
* [ ] Add all `c` types
* [ ] Intern strings that are generated in IR
* [ ] `VariableReference` and `MemberReference` should have reference types
* [ ] Support `use module.{a, b, submodule.c}`
* [ ] Still return declarations even if they have errors, so there is no `undefined_*` errors later
* [ ] Generic types shouldn't be replaced, but rather constrained (e.g `T: Integer`)
* [ ] Replace calls to trait functions with calls to specialized functions
* [ ] Run monomorphization from the top of the module
* [ ] Reject lowercase names for types
* [ ] Rebinding references
* [ ] Remove need for escaping `type` in `type of <:T>`
* [ ] Add better errors for inferred generics
* [ ] Unify `Self` and `GenericType`
* [ ] `Any` trait
* [ ] Logic for printing decimals inside ppl
* [ ] Unsafe code marker
* [ ] Format strings
* [ ] Varadic arguments functions
* [ ] Allow newlines inside parentheses
* [ ] Allow tabs before comments for members
* [ ] Add `assert` and `panic`
* [ ] Fix bus error (caused by llvm 16 [issue](https://github.com/llvm/llvm-project/issues/60432))
* [ ] Remove cached names from functions
* [ ] Remove unnecessary information from errors (like `Error: <Type>`)
* [ ] Check compiler errors in repl too
* [ ] Make statements to return `None` type for convenience
* [ ] Make `if` to be an expression?
* [ ] Add `HashMap` type
* [ ] Explicit traits implementation
* [ ] Functions as values
* [ ] Add values as types (e.g `1 | 2 | "lol"`)
* [ ] Add type unions `A | B`, `A & B`

## Important Implementation Details
* Use `IndexMap` instead of `HashMap` to guarantee order of declarations