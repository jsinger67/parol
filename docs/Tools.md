# Supplementary tools

As of version v0.5.1. tools are subcommands of the parol binary. Their source code can be found in the `bin/parol/tools` folder. `parol` itself is located in the subfolder `bin/parol`. This sections is devoted to the subcommands only.

Please note, that you do not need to use these tools normally when you want to generate parsers. All of their functionality is completely included in `parol` itself. But when you are about to solve a certain problem they may come handy. So it is useful to know whats in the bag.

Hint: All subcommands give a short help output when called without parameters:

```shell
    >cargo run --bin parol calculate_k
Missing arguments <par-file> <k=5>!
Example:
cargo run --bin parol calculate_k ./src/parser/parol-grammar-exp.par
```

If you installed parol via

```shell
cargo install parol
```

or from local repository

```shell
cargo install --path .
```

you will have another option of calling parol and its subcommands which is even easier because the `parol` executable is installed in your `~/.cargo/bin` folder.

```shell
    >parol calculate_k  ./examples/list/list-exp.par
title: Some("A simple comma separated list of integers")
comment: Some("A trailing comma is allowed.")
start_symbol: list
current_scanner: INITIAL
name: INITIAL;line_comments: [];block_comments: [];auto_newline_off: false;auto_ws_off: false;
list: Alts(Alt());
list: Alts(Alt(N(num), N(list_rest)));
list_rest: Alts(Alt(N(list_item), N(list_rest)));
list_item: Alts(Alt(<0>T(,), N(num)));
list_rest: Alts(Alt());
list_rest: Alts(Alt(<0>T(,)));
num: Alts(Alt(<0>T([0-9]+)));

Ok(
    2,
)
```

## `calculate_k_tuples`

Calculates the lookahead tokens with size k for each non-terminal. Checks the decidability first.

## `calculate_k`

Calculates the maximum lookahead needed for your grammar, similar as `decidable`.

## `decidable`

Can be used to detect the maximum lookahead needed for your grammar.

## `first`

Calculates the FIRST(k) sets for each production and for each non-terminal. The number k defaults to 1 and can be specified as extra parameter after the grammar file.

## `follow`

Calculates the FOLLOW(k) sets for each non-terminal. The number k defaults to 1 and can be specified as extra parameter after the grammar file.

## `generate`

Generates an arbitrary sentence of the given grammar. It can be used to verify your language description.

On complex grammars the generation can get into deeply branching the grammar productions again and again because productions are randomly selected. Therefore generation is aborted with an error if the resulting sentence exceeds a certain limit. This limit currently defaults to a string length of 100 000. This value can be overwritten by giving an additional parameter after the grammar file.
If generation fails with error `parol::generators::language_generator::source_size_exceeded` please give it another try.

## `left_factor`

Applies the left factoring algorithm on the grammar given.

## `left_recursions`

Checks the given grammar for direct and indirect left recursions.

## `productivity`

Checks the given grammar for non-productive non-terminals.

## `serialize`

Serializes a grammar to json format. Seldom to apply.

## `format`

Formats the given grammar with the standard format and prints the result to stdout. Can be used if you messed up the formatting. It always outputs the expanded version of your grammar (i.e. without groups (), optional expressions [] and repetitions {}). This is by design, because the mentioned constructs exist only for convenience and are always eliminated resp. substituted by their expanded equivalents.
