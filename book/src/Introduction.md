<!-- markdownlint-disable first-line-h1 -->
<!-- markdownlint-disable Inline HTML -->

<br>
<img src="./Parol.svg" alt="Logo" height="150" width="150">
<br><br><br>

<!-- markdownlint-enable Inline HTML -->
<!-- markdownlint-enable first-line-h1 -->

# The `parol` Parser Generator

[`parol`](https://github.com/jsinger67/parol) is a parser generator with unique features.

It is available as a command-line tool that generates complete parsers from a single grammar
description file. `parol` is also a library you can use in your own crates.

With its builder API, you can easily integrate code generation into your crate's build process via a
Cargo build script (`build.rs`).

`parol` automatically infers and generates all AST data types by analyzing your language's grammar
description.

You can control AST type generation in several ways:
- Mark elements to omit from your AST.
- Specify custom types for language elements, which are inserted at the correct position in the
resulting AST type.
- Define how each symbol on the right-hand side of a production is named in the generated structure.

Language description and implementation are strictly separated in `parol`. You can design your
grammar without processing anything, as generated parsers function by default as acceptors. This
enables **rapid prototyping** of your grammar.

`parol` generates a trait that serves as the interface between the generated parser and your
language processing. The trait contains functions for each non-terminal in your grammar, which you
can implement as needed. In the simplest case, you implement the trait function for the start symbol,
which is called after the entire input string is parsed. This function receives a parameter
representing the complete structure of the parsed document.

The parser automatically calls the interface trait's functions via a separately generated adapter
during parsing.

`parol` provides an ecosystem of tools, including a
[Visual Studio Code Extension](https://github.com/jsinger67/parol/tree/main/tools/parol-vscode) and
a [Language Server](https://github.com/jsinger67/parol/tree/main/crates/parol-ls).

Generated parsers can recover from syntax errors automatically. This means the parser does not stop
parsing after the first syntax error, but instead tries to synchronize with the input and continue
analysis.