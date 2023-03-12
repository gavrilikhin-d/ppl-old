Special `syntax` keyword allows to add new syntax to the language.

```ppl
/// Declare syntax
syntax NTimes = <n: Integer> times: <code: CodeBlock>

/// Define action, that compiler must take on it
fn <{ n, code }: NTimes>:
	for i in range(n):
		execute code

/// Use new syntax
5 times:
	print "Hello"
```

Syntax is valid, if it's imported from the module.

It's planned to implement all of the ppl syntax in this way.

`token` statements are used to define tokens, which don't have spaces between them.

Syntax for `syntax` statements:
```ppl
token Identifier = [a-zA-Z_][a-zA-Z0-9_]*
syntax Pattern = Identifier
```