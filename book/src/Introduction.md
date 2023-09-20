<!-- markdownlint-disable first-line-h1 -->
<!-- markdownlint-disable Inline HTML -->
<br>
<img src="./Parol.svg" alt="Logo" height=150 with=150>
<br><br><br>
<!-- markdownlint-enable Inline HTML -->
<!-- markdownlint-enable first-line-h1 -->

# The `parol` Parser Generator

[`parol`](https://github.com/jsinger67/parol) is a parser generator with some unique characteristics.

It is an installable command line tool that can generate complete parsers from a single grammar
description file. `parol` is also a library that you can use in your own crates.

Using a builder API it is easy to integrate the code generation process into your crate's build
process via a cargo build script (`build.rs`).

`parol` can be instructed to infer and generate all AST data types that you would otherwise have to
design yourself. `parol` can do this simply by analyzing your language's grammar description.

You can control the process of AST type generation in two ways. Firstly, you can mark elements for
omission in your AST. Secondly, you can specify your own types for language elements, which are then
inserted at the right position into the resulting AST type.

Language description and language implementation is strictly separated in `parol`. Thus, you can
design your language's grammar without any need to process anything because generated parsers
function by default as acceptors. This allows you to do **real rapid prototyping** of your grammar.

`parol` generates a trait as interface between your language processing and the generated parser.
The trait contains functions for each non-terminal of your grammar which you can implement for
non-terminals you need to process. In the simplest case you only implement the trait function for
the start symbol of your grammar which is called after the whole input string is parsed. This
function then is called with a parameter that comprises the complete structure of the parsed
document.

The parser calls the interface trait's functions via a separately generated adapter automatically
during the process of parsing.

`parol` now provides a whole ecosystem of tools including an
[Extension](https://github.com/jsinger67/parol/tree/main/tools/parol-vscode) for Visual Studio Code
and a [Language Server](https://github.com/jsinger67/parol/tree/main/crates/parol-ls).

As of version 0.24.0 generated parsers can recover from syntax errors automatically. This means that
the parser usually doesn't not stop parsing the input after the first syntax error occurs, and
instead tries to synchronize with the input in order to continue the analysis accordingly.