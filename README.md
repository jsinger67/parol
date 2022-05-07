# Example application "BASIC Interpreter"

This is an example project of the [`parol`](https://github.com/jsinger67/parol) parser generator. It is part of the documentation of `parol`.

It uses the auto-generation function of `parol` which generates your AST types automatically solely from the grammar description. The automatically generated code fills these data structure automatically during parsing so that at the end of the parsing process the parsed input exits in a converted form in these data types.

## Performance evaluation

For best performance results please use a release build and add the parameter -q (quiet) after the file name.
But keep in mind that this crate is designed for didactical usage and therefore performance is not the main goal. Especially the parse tree generation and visualization is active which is usually not the case for a production version.
