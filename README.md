<!-- markdownlint-disable first-line-h1 -->
[![Rust](https://github.com/jsinger67/parol/actions/workflows/rust.yml/badge.svg)](https://github.com/jsinger67/parol/actions/workflows/rust.yml)
[![Docs.rs](https://docs.rs/parol/badge.svg)](https://docs.rs/parol)
[![Crates.io](https://img.shields.io/crates/v/parol.svg)](https://crates.io/crates/parol)
<!-- markdownlint-enable first-line-h1 -->

# About `parol`

<!-- markdownlint-disable Inline HTML -->
<br>
<img src="./crates/parol/logo/Parol.svg" alt="Logo" height=300 with=300>
<br><br><br>
<!-- markdownlint-enable Inline HTML -->

## ATTENTION - The main branch is subject to constant changes, so the experience can be bumpy

Therefore, please use an officially released version from
[crates.io](https://crates.io/crates/parol) or refer to one of the latest
[tags](https://github.com/jsinger67/parol/tags) applied to main branch.

---

This workspace contains four essential crates that are all separately released on crates.io.

* [parol](https://crates.io/crates/parol)
* [parol_runtime](https://crates.io/crates/parol_runtime)
* [parol-macros](https://crates.io/crates/parol-macros)
* [parol-ls](https://crates.io/crates/parol-ls)

New changes can be viewed in the change logs of the respective projects.

* [CHANGELOG parol](./crates/parol/CHANGELOG.md)
* [CHANGELOG parol_runtime](./crates/parol_runtime/CHANGELOG.md)
* [CHANGELOG parol-macros](./crates/parol-macros/CHANGELOG.md)
* [CHANGELOG parol-ls](./crates/parol-ls/CHANGELOG.md)

It also contains the vs-code extension `parol-vscode` which is released on VS Code marketplace
[parol-vscode](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode)

* [CHANGELOG parol-vscode](./tools/parol-vscode/CHANGELOG.md)

---

[parol](https://github.com/jsinger67/parol/tree/main/crates/parol) is a LL(k) parser generator
**for Rust**.

It's an installable command line tool that can generate complete parsers from a single grammar
description file including all AST data types you would otherwise had to design by yourself. `parol`
does this solely by analyzing your language's grammar. `parol` is also a library that you can use in
your own crates.

You can control the process of AST type generation. First you can mark elements for omission in your
AST. Also you can specify your own types for language elements.

Language description and language implementation is strictly separated in `parol`. Thus you can
design your language's grammar without any need to process anything because generated parsers
function by default as acceptors. This empowers you to do a real *rapid prototyping* of your grammar.

`parol` generates a trait as interface between your language processing and the generated parser.
The trait contains functions for each non-terminal of your grammar which you can implement for
non-terminals you need to process. In the simplest case you only implement the trait function for
the start symbol of your grammar which is called after the whole input string is parsed. This
function then is called with a parameter that comprises the complete structure of the parsed
document.

The parser calls the interface trait's functions via a separately generated adapter automatically
during the process of parsing.

With such a generated interface trait you theoretically never have to let `parol` generate new code
for you anymore and you can concentrate on the development of your language processing. Although,
often a more iterative approach is taken.

## Generated parsers

* are true LL(k) parsers implemented by push down automata (PDAs).
* are predictive, i.e. they implement a **non-backtracking** parsing technique. This often results
in much faster parsers.
* are clean and easy to read.
* use only as much lookahead as needed for a certain non-terminal (from 0 to k)
* are generated from **a single grammar description** file.
* can generate types that resemble the AST of your grammar automatically. Semantic actions are then
called with these types. This greatly improves the development process and makes it less error-prone.

## Other properties of `parol`

* Selection of production is done by a deterministic finite **lookahead automaton** for each
non-terminal.
* **Semantic actions** with empty default implementations are generated as a trait. You can
implement this trait for your grammar processing item and implement only needed actions. This
provides a loose coupling between your language definition and the language processing.
* As a result semantic actions are strictly separated from the grammar definition in contrast to
Bison. No parser generation step is needed when you merely change the implementation of a semantic
action.
* The grammar description is provided in a **Yacc/Bison-like style** with additional features known
from EBNF such as grouping, optional elements and repetitions.
* You can define multiple scanner states (aka start conditions) and define switches between them
directly in the productions of your grammar.
* You can opt out the default handling of whitespace and newlines for each scanner state separately.
* The grammar description supports definition of language comments via **%line_comment** and
**%block_comment** declarations for each scanner state.
* The crate provides several tools for **grammar analysis**, **transformation** and **parse tree visualization**
to support your grammar implementation.
* The parser generator **detects direct and indirect left recursions** in your grammar description.
* `parol`'s parser is generated by `parol` itself.
* Use `parol new` to create your own crate that uses `parol`.

## Why should you use LL(k) parsers in your language implementation?

LL parsing technique is a top-down parsing strategy that always starts from the start symbol of your
grammar. This symbol becomes the root node of the parse tree. Then it tries to derive the left-most
symbol first. All such symbols are then processed in a pre-order traversal. During this process the
parse tree is created from the root downwards.

Both, processing the input and producing the parse tree in 'natural' direction ensures that at every
point during parsing you can see where you came from and what you want to derive next. `parol`'s
parse stack contains 'End of Production' markers which reflect the 'call hierarchy' of productions.

This tremendously helps to put your language processing into operation. In contrast, anyone who has
ever debugged a LR parser will remember the effect of 'coming out of nowhere'.

Although LL grammars are known to be less powerful than LR grammars many use cases exist where LL
grammars are sufficient. By supporting more than one lookahead token the abilities of traditional
LR(1) grammars and LL(k) grammars become more and more indistinct.

## Why should you use `parol`?

`parol` is simple. You can actually understand all parts of it without broader knowledge in parsing
theory.

`parol` is fast. The use of deterministic automata ensures a minimal overhead during parsing, no
backtracking needed.

`parol` is a true LL(k) parser. You won't find much working LL(k) parsers out there.

`parol` generates beautiful code that is easy to read which fosters debugging.

`parol` is young. Although this might be a problem some times, especially regarding the stability of
the API, the best is yet to come.

`parol` is actively developed. Thus new features are likely to be added as the need arises.

## Documentation

### [Examples](https://github.com/jsinger67/parol/tree/main/examples)

This project contains some introductory grammar examples from entry level up to a more complex
[C-like expression language](https://github.com/jsinger67/parol/tree/main/examples/calc_auto)
and an acceptor for [Oberon-0](https://github.com/jsinger67/parol/tree/main/examples/oberon_0) grammar.

A complete Oberon-2 acceptor generated by `parol` can be found in the examples of this
[repository](https://github.com/jsinger67/parol/tree/main/examples/oberon2).

A rudimentary [Basic interpreter](https://github.com/jsinger67/parol/tree/main/examples/basic_interpreter)
strives to mimic a small part of C64 Basic.

A TOML parser can be found [here](https://github.com/jsinger67/parol/tree/main/examples/toml).

I also provide a [JSON Parser](https://github.com/jsinger67/parol/tree/main/examples/json_parser_auto).

`parol`'s input language processing is an additional and very practical example.

### [The book](https://jsinger67.github.io/)

A book explains some internals and the practical use of `parol` in detail. It is still a work in
progress but should be considered as the central documentation.

## State of the project

`parol` has proved its ability in many examples and tests during its development. Early adopters can
quite safely use it.

But `parol` is not ready for production yet. Features are still in development and the crate's
interface can change at any time. There is still a lot of work to be done and any help is
appreciated.

## Dependencies

Please note that any necessary dependencies are automatically added to your new `parol` project if
you use the `parol new` subcommand to create your new crate. The following sections are therefore
for information only.

### Runtime library

Parsers generated by `parol` have to add a dependency to the
[parol_runtime](https://crates.io/crates/parol_runtime) crate. It provides the scanner and parser
implementations needed. The parol_runtime crate is very lightweight.

### Macros

As of version 0.13.0 you have to add the [parol-macros](https://github.com/jsinger67/parol-macros)
crate to your dependencies if you use  `parol`'s *auto-generation mode*.

## License

`parol` and its accompanied tools included in this workspace are free, open source and permissively
licensed! Except where noted (below and/or in individual files), all code in this repository is
dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or
[http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

### Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Further readings

* [CHANGELOG](crates/parol/CHANGELOG.md)
* [The book](book/README.md)

## Contributors

Thanks to all the contributors for improving this project!

* [Techcable](https://github.com/Techcable)
* [oaleaf](https://github.com/oaleaf)
* [mobotsar](https://github.com/mobotsar)
* [ryo33](https://github.com/ryo33)
* [dalance](https://github.com/dalance)
* [udoprog](https://github.com/udoprog)
* [AumyF](https://github.com/AumyF)
