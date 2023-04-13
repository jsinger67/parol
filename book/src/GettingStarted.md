# Getting started

## Installation

Before you can use `parol` you have to install it.

Since `parol` generates Rust code it is assumed that you have a Rust toolchain installed. Otherwise
head over to [Rustup](https://rustup.rs/) or [Install Rust](https://www.rust-lang.org/tools/install)
first.

`parol` only needs stable Rust.

Now you should be able to install `parol` on your platform:

```shell
cargo install parol
```

To confirm a correct installation invoke this command:

```shell
$ parol -V
parol 0.10.6
```

If you see an error saying the tool couldn't be found please check your PATH variable. It should
include ~/.cargo/bin.

## Let `parol` generate a crate for you

We can use the `parol new` subcommand and let `parol` create our new project for us.

```shell
parol new --bin --path ./my_grammar
```

Then change into the new project's folder and start the initial build. Here `parol` is generating
two files from the initial grammar definition.

```shell
cd ./my_grammar
cargo build
```

And run the test with the generated parser:

```shell
$ cargo run -- ./test.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.20s
     Running `target\debug\my_grammar.exe ./test.txt`
Parsing took 4 milliseconds.
Success!
MyGrammar { my_grammar: Token { symbol: "Hello world!", token_type: 5, location: Location { line: 4, column: 5, length: 12, start_pos: 0, pos: 97, file_name: "./test.txt" } } }
```

`parol` has generated a full fledged parser with AST types suitable for your grammar description!

Now you can open your favorite editor

```shell
code .
```

and adapt the grammar description in the file `my_grammar.par` to fit your requirements. Any
subsequent invocations of `cargo build` will trigger `parol` to generate the derived sources
automatically if the grammar description file `my_grammar.par` has been changed.

**This is all you need to set up a working development environment.**

> ## VS Code extension and Language Server
>
> I provide a VS Code extension [parol-vscode](https://github.com/jsinger67/parol/tree/main/tools/parol-vscode).
>
> Please install this extension from VS Code
> [marketplace](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode).
> It provides syntax highlighting, folding and language icons and will surely be useful for you.
>
> The extension utilizes a [Language Server](https://github.com/jsinger67/parol/tree/main/crates/parol-ls) that you have
> to install separately.
>
> ```shell
> cargo install --force parol-ls
> ```
>
