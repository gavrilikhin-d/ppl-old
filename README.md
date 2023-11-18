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
* [x] Add `@builtin`
* [x] Types as values
* [x] Fix `Type<None> as String`
* [x] Multiple errors support instead of exiting on first error
* [x] Link ppl.dylib to executable
* [x] Fix IR generation for complex while loops (benchmarks/factorial/main.ppl)
* [x] Run initializer for global variables (`global_ctor` or add to main)
* [x] Fix undefined variable when returning `=> global`
* [x] Fix `<:Self> > <:Self>`
* [x] Print exactly decimals in decimal form
* [x] Multiple output types
* [x] Add constrains to generics
---
### Current task
* [ ] Mapping of generic types while checking candidates
---
* [ ] Logic for printing decimals inside ppl
* [ ] Implicit conversions from one type to another (dereference and etc)
* [ ] Unsafe code marker
* [ ] Check result of the program itself in `test_compiler_result`
* [ ] Format strings
* [ ] Varadic arguments functions
* [ ] Add `Array` type
* [ ] Allow newlines inside parentheses
* [ ] Allow tabs before comments for members
* [ ] Add `assert` and `panic`
* [ ] Rework specialized system and type conversions checks. Add reason, why conversion fails
* [ ] Fix bus error (caused by llvm 16 [issue](https://github.com/llvm/llvm-project/issues/60432))
* [ ] Remove cached names from functions
* [ ] Remove unnecessary information from errors (like `Error: <Type>`)
* [ ] More functions to stdlib
* [ ] Check compiler errors in repl too
* [ ] Add more checks for compiler
* [ ] Make statements to return `None` type for convenience
* [ ] Make `if` to be an expression?
* [ ] Add `HashMap` type
* [ ] Fix memory leak due to pointers to builtin types
* [ ] Explicit traits implementation
* [ ] Functions as values
* [ ] Add type unions `A | B`, `A & B`
* [ ] Multifile compilation and imports
* [ ] Add all `c` types

## Important Implementation Details
* Use `BTreeMap` instead of `HashMap` to guarantee order of errors