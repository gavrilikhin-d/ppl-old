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

* [x] Implicit conversions from one type to another (dereference and etc)
* [x] Fix non-instantiated generic functions (`fn<T> foo <:T> => 1` & `let n = foo 1`; `& <:Reference<Bool>>`)
* [x] Rework specialized system and type conversions checks. Add reason, why conversion fails
* [x] Unify `TraitType` in type checking with constraint generic type with random name
* [x] Make `monomorphized` to take `Self`
* [x] Multifile compilation and imports
* [x] Need to search for variables at monomorphization, because the type of the variable can be changed (eg. `let y = reference to x; println y`)
* [x] Replace type references with constructors only after monomorphization
* [x] `lowering_to_hir_within_context` -> `to_hir` and `to_ir`
* [x] Fix `type of <:T>`
* [x] Intern type constructors that are generated in IR
* [x] Support `use module.*` 
* [x] Add tracing to compiler
* [x] Don't define variables right away, when declaring them
---
### Current task
* [ ] Fix definition of predeclared functions
---
* [ ] `<:T> as String` -> `String from <:T>`. `as` should mean force cast (analog of `try_from().unwrap()`)
* [ ] Add all `c` types
* [ ] Intern strings that are generated in IR
* [ ] `VariableReference` and `MemberReference` should have reference types
* [ ] Split builtin module into several files
* [ ] Support `use module.{a, b, submodule.c}`
* [ ] Still return declarations even if they have errors, so there is no `undefined_*` errors later
* [ ] Generic types shouldn't be replaced, but rather constrained (e.g `T: Integer`)
* [ ] Replace calls to trait functions with calls to specialized functions
* [ ] Run monomorphization from the top of the module
* [ ] Reject lowercase names for types
* [ ] Rebinding references
* [ ] Fix `let y = &x; y`
* [ ] Remove need for escaping `type` in `type of <:T>`
* [ ] Fix crash in diagnostics due-to wrong source file
* [ ] Add better errors for inferred generics
* [ ] Unify `Self` and `GenericType`
* [ ] `Any` trait
* [ ] Logic for printing decimals inside ppl
* [ ] Unsafe code marker
* [ ] Check result of the program itself in `test_compiler_result`
* [ ] Format strings
* [ ] Varadic arguments functions
* [ ] Add `Array` type
* [ ] Allow newlines inside parentheses
* [ ] Allow tabs before comments for members
* [ ] Add `assert` and `panic`
* [ ] Fix bus error (caused by llvm 16 [issue](https://github.com/llvm/llvm-project/issues/60432))
* [ ] Remove cached names from functions
* [ ] Remove unnecessary information from errors (like `Error: <Type>`)
* [ ] More functions to stdlib
* [ ] Check compiler errors in repl too
* [ ] Make statements to return `None` type for convenience
* [ ] Make `if` to be an expression?
* [ ] Add `HashMap` type
* [ ] Fix memory leak due to pointers to builtin types
* [ ] Explicit traits implementation
* [ ] Functions as values
* [ ] Add type unions `A | B`, `A & B`

## Important Implementation Details
* Use `IndexMap` instead of `HashMap` to guarantee order of declarations