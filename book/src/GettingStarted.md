# Getting Started

## Installation

Before you can use `parol`, you need to install it.

Since `parol` generates Rust code, a Rust toolchain must be installed. If you do not have Rust,
visit [Rustup](https://rustup.rs/) or [Install Rust](https://www.rust-lang.org/tools/install).

`parol` requires only stable Rust.

To install `parol` on your platform, run:

```shell
cargo install parol
```

To verify the installation, execute:

```shell
parol -V
parol 4.0.1
```

If you see an error indicating the tool could not be found, check your PATH variable. It should
include `~/.cargo/bin`.

### Video Introduction

For a visual introduction, watch the [introductory video](https://youtu.be/TJMwMqD4XSo) on YouTube.

## Generate a Crate with `parol`

Use the `parol new` subcommand to create a new project:

```shell
parol new --bin --path ./my_grammar
```

Change into the new project folder and start the initial build. `parol` will generate two files from
the initial grammar definition.

```shell
cd ./my_grammar
cargo build
```

> You can safely ignore the `#[warn(unused_imports)]` warning for now. It will disappear as the
grammar receives more content.

Run the test with the generated parser:

```shell
cargo run ./test.txt
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s
     Running `target\debug\my_grammar.exe ./test.txt`
Parsing took 0 milliseconds.
Success!
MyGrammar { my_grammar: Token { text: "Hello world!", token_type: 5, location: Location {
start_line: 4, start_column: 5, end_line: 4, end_column: 17, start: 62, end: 74, file_name:
"./test.txt" }, token_number: 2 } }
```

`parol` has generated a complete parser with AST types suitable for your grammar description.

Now, open your preferred editor:

```shell
code .
```

Edit the grammar description in `my_grammar.par` to fit your requirements. Any subsequent invocation
of `cargo build` will trigger `parol` to regenerate the derived sources automatically if
`my_grammar.par` has changed.

**This is all you need to set up a working development environment.**

> ## VS Code Extension and Language Server
> 
> A VS Code extension, [parol-vscode](https://github.com/jsinger67/parol/tree/main/tools/parol-vscode),
is available.
> 
> Install this extension from the VS Code
> [Marketplace](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode).
> It provides syntax highlighting, folding, and language icons, which will be useful for you.
> 
> The extension utilizes a
[Language Server](https://github.com/jsinger67/parol/tree/main/crates/parol-ls) that must be
installed separately.
> 
> ```shell
> cargo install --force parol-ls
