# Example application "BASIC Interpreter"

This is an example project of the [`parol`](https://github.com/jsinger67/parol) parser generator. It
is part of the documentation of `parol`.

It uses the auto-generation function of `parol` which generates your AST types automatically solely
from the grammar description. The automatically generated code fills these data structure
automatically during parsing so that at the end of the parsing process the parsed input exits in a
converted form in these data types.

## The BASIC constructs we support

Maybe you remember the old C64 with its BASIC V2?

I decided to re-implement a small part of this BASIC dialect for as an example on how to use `parol`.
You may ask, why to choose a forty years old language? I say, why not? Because we can and because
it's fun. 😉

Since I can't implement the whole BASIC language in this example I selected a useful subset.

We support the following language elements:

* Comments with REM
* Numeric constants (integer and float)
* Floating point variables
* Statements
  * IF THEN
  * IF GOTO
  * GOTO
  * Assignments
* Expressions
  * Arithmetic expressions with addition, subtraction, multiplication and division as well as using
  parentheses
  * Comparison expressions
  * Logical expressions
* BASIC commands
  * PRINT or ?
  * END

## Performance evaluation

For best performance results please use a release build and add the parameter -q (quiet) after the
file name. But keep in mind that this crate is designed for didactical usage and therefore
performance is not the main goal. Especially the parse tree generation and visualization is active
which is usually not the case for a production version.
