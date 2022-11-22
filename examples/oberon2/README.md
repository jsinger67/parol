# Example application "Oberon2 parser"

This is an example project of the [`parol`](https://github.com/jsinger67/parol) parser generator.

It uses the auto-generation function of `parol` which generates your AST types automatically solely
from the grammar description. When this function is enabled the automatically generated code fills
these data structures automatically during parsing so that at the end of the parsing process the
parsed input exits in a converted form in these data types.

## Performance evaluation

For best performance results please use a release build and add the parameter -q (quiet) after the
file name.
