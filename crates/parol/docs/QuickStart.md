# Quick start

## Installation

To install `parol` use the following command

```shell
cargo install parol
```

This simplifies calling `parol` because the executable is installed in your `~/.cargo/bin` folder.
`parol` is a single executable with all dependencies included and still small enough that you can afford to install it.

## The first run

Now you can just call the `parol` parser generator to get a feeling about how to use it in your own project.

```shell
    >parol -f ./examples/list_auto/list.par -s -v
```

In this first example we don't instruct `parol` to generate any source code but rather we check the given file.
The absence of any error messages is a good sign, everything is ok with that grammar file.

If you want you can modify the list.par file and check if you get error messages from `parol`. After that don't forget to undo your changes.

Now lets run the full example that belongs to this grammar description:

```shell
    >cargo run --example list_auto -- ./examples/list_auto/list_test.txt
...
[1, 2, 3, 4, 5, 6]
```

This generates the example's executable and passes the given text file as input to parse.
The output *[1, 2, 3, 4, 5, 6]* is the internal representation of the parsed input after the successful parse.
Please hae a look at the generated parse tree visualization here: ./examples/list_auto/list_test.svg now.

To have a visualization of a certain parse tree while you're implementing your grammar can be very helpful.

## How to let `parol` generate files

```shell
    >parol help
```

This call will print help for `parol` and its subcommands. You can use ```parol help <subcommand>``` to get help for a specific subcommand.

For full generation of all files you can use `parol` analogously as shown in the following example:

```shell
    >parol -f ./examples/list_auto/list.par -e ./examples/list_auto/list-exp.par -p ./examples/list_auto/list_parser.rs -a ./examples/list_auto/list_grammar_trait.rs -t ListGrammar -m list_grammar -g
```

The `parol` crate also offers a powerful Builder that can be used in build scripts. Please see the [JSON parser auto](https://github.com/jsinger67/json_parser_auto.git) project on how to use this builder in your projects.

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
cargo run --bin my_grammar -- ./test.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.22s
     Running `target\debug\my_grammar.exe ./test.txt`
Parsing took 4 milliseconds.
Success!
MyGrammar { my_grammar: Token { symbol: "Hello world!", token_type: 5, line: 4, column: 5, length: 12, start_pos: 0, pos: 97 } }
```

`parol` has generated a full fledged parser with AST types suitable for your grammar description!

Please check out the [Tutorial](./Tutorial.md) for further information.

## VS Code extension

I provide a VS Code extension [parol-vscode](https://github.com/jsinger67/parol-vscode.git).
Please install this extension from VS Code
[marketplace](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode)

It provides syntax highlighting, folding and language icons and will surely be useful for you.

## Tools

`parol` itself provides several tools with special tasks (see [Supplementary tools](./Tools.md)) as subcommands. As an example let's have a look at the `decidable` subcommand:

```shell
    >parol decidable -f ./examples/list_auto/list.par
Grammar is LL2
```

As you can see it detects the maximum lookahead needed for your grammar. And you see the fact that the simple list example is LL(2). When you look at the generated parser source [list_parser.rs](../examples/list_auto/list_parser.rs) you can see that the non-terminal `ListList` has k: 2. You can find the actual code in the LOOKAHEAD_AUTOMATA struct at the LookaheadDFA of non-terminal `ListList`.

## First glance at the grammar description format

Lets have a look at the used grammar description file `list.par`.

```ebnf
%start List
%title "A possibly empty comma separated list of integers"
%comment "A trailing comma is allowed."

%%

List: [Num {","^ Num}] TrailingComma^;
Num: "0|[1-9][0-9]*";
TrailingComma: [","];
```

It shows us the basic structure of a grammar description file and if you are familiar with yacc/bison grammar files, you will recognize the similarity.

There are basically two sections divided by the %% sign. Above there are declarations of which only the first %start declaration is mandatory. It declares the start symbol of your grammar.
The second section below the %% sign contains the actual grammar description in form of several productions. At least one production must exist.
The complete description of the grammar file's syntax can be found here: [PAR Grammar](./ParGrammar.md)

## What's next

* [PAR Grammar](./ParGrammar.md)
* [The list example](./ListExample.md)
* [Supplementary tools](./Tools.md)
