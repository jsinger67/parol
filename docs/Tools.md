# Subcommands of `parol`

As of version v0.5.1. all supplementary tools are subcommands of the parol binary. Their source code
can be found in the `bin/parol/tools` folder. `parol` itself is located in the subfolder `bin/parol`.
This sections is devoted to the subcommands only.

Please note, that all tools are designed to help in specific situations that can show up when you
want to generate parsers. So it is useful to know what's available.

>Hint: Calling `parol help` will list all available subcommands. You get a short help output for
each subcommand when called after parameter `help`:

```shell
    >parol help decidable
parol.exe-decidable 
Detects the maximum lookahead needed for your grammar and shows possible conflicts in k-tuples

USAGE:
    parol.exe decidable [OPTIONS] --grammar-file <GRAMMAR_FILE>

OPTIONS:
    -f, --grammar-file <GRAMMAR_FILE>
            The grammar file to use

    -h, --help
            Print help information

    -k, --lookahead <LOOKAHEAD>
            The maximum number of lookahead tokens to be used [default: 5]
```

Or call `parol` with the subcommand as only parameter:

```shell
    >parol calculate-k
error: The following required arguments were not provided:
    --grammar-file <GRAMMAR_FILE>

USAGE:
    parol.exe calculate-k [OPTIONS] --grammar-file <GRAMMAR_FILE>

For more information try --help
```

Here some examples:

```shell
    >parol calculate-k  -f ./examples/list/list-exp.par
title: Some("A simple comma separated list of integers")
comment: Some("A trailing comma is allowed.")
start_symbol: List
current_scanner: INITIAL
name: INITIAL;line_comments: [];block_comments: [];auto_newline_off: false;auto_ws_off: false;
List: Alts(Alt(N(Num), N(ListRest), N(ListSuffix)));
ListSuffix: Alts(Alt(<0>T(,)));
ListSuffix: Alts(Alt());
List: Alts(Alt());
ListRest: Alts(Alt(<0>T(,), N(Num), N(ListRest)));
ListRest: Alts(Alt());
Num: Alts(Alt(<0>T(0|[1-9][0-9]*)));

Ok(
    2,
)
```

```shell
    >parol decidable  -f ./examples/list/list.par
Conflicts for non-terminal 'List':
  Conflict in productions 0 and 1:
    0: {[Num, Comma, $], [Num, Comma, Num, Comma, $], [Num, Comma, Num, Comma, Num]}(k=5)
    1: {[Num, $], [Num, Comma, Num, $], [Num, Comma, Num, Comma, Num]}(k=5)
    ∩: {[Num, Comma, Num, Comma, Num]}(k=5)

1 undecidable non-terminal(s)
```

```shell
    >parol decidable  -f ./examples/list/list-exp.par
Grammar is LL2
```

## `calculate-k`

Calculates the maximum lookahead needed for your grammar, similar as `decidable`.

## `calculate-k-tuples`

Calculates the tuples of lookahead tokens with size k for each non-terminal. Checks the decidability
first.

## `decidable`

Detects the maximum lookahead needed for your grammar and shows possible conflicts in k-tuples.

## `deduce-types`

 Calculates the type structure of the generated expanded grammar (to verify AST types generated with
 the auto-generation functionality)

## `first`

Calculates the FIRST(k) sets for each production and for each non-terminal. The number k defaults to
1 and can be specified as extra parameter.

## `follow`

Calculates the FOLLOW(k) sets for each non-terminal. The number k defaults to 1 and can be specified
as extra parameter.

## `format`

Formats the given grammar with the standard format and prints the result to stdout. Can be used if
you messed up the formatting.

It always outputs the expanded version of your grammar (i.e. without groups (), optional expressions
[] and repetitions {}) although no left-factoring is applied on it.

This is by design, because the mentioned constructs exist only for convenience and are always
eliminated resp. substituted by their expanded equivalents.

## `generate`

Generates a random sentence of the given grammar. It can be used to verify your language description.

On complex grammars the generation can get into deeply branching the grammar productions again and
again because productions are randomly selected. Therefore generation is aborted with an error if
the resulting sentence exceeds a certain limit. This limit currently defaults to a string length of
100 000. This value can be overwritten by giving an additional parameter after the grammar file.

If generation fails with error `parol::generators::language_generator::source_size_exceeded` please
give it another try.

With the help of this command you can run endless stress tests like in this example using a
*powershell* one-liner:

```powershell
for (;;) { parol generate -f ./examples/json/json-exp.par | Set-Content "$env:Temp/x.json"; json_parser "$env:Temp/x.json"; if (-not $?) { break } }
```

or if examples are generated

```powershell
for(;;) { parol generate -f ./examples/list_auto/list-exp.par | Set-Content "$env:Temp/x.txt"; ./target/debug/examples/list_auto.exe "$env:Temp/x.txt"; if (-not $?) { break } }
```

Note that you have to use the expanded grammar here, because the tool currently can't (and perhaps
never will be able to) use enhanced features like optional expressions, repetitions and grouping.

## `left-factor`

Applies the left factoring algorithm on the grammar given.

Example:

```shell
    >parol left-factor -f .\examples\list\list.par
%start List
%title "A simple comma separated list of integers"
%comment "A trailing comma is allowed."

%%

/* 0 */ List: Num ListRest ListSuffix;
/* 1 */ ListSuffix: ",";
/* 2 */ ListSuffix: ;
/* 3 */ List: ;
/* 4 */ ListRest: "," Num ListRest;
/* 5 */ ListRest: ;
/* 6 */ Num: "0|[1-9][0-9]*";
```

## `left-recursions`

Checks the given grammar for direct and indirect left recursions.

## `new`

Creates a new crate that uses `parol`. This way you can easily setup your new crate. You can
generate binary crates as well as library crates with this subcommand.

It needs `cargo-edit`to be installed. Use the following command line to install it.

```shell
cargo install cargo-edit
```

## `productivity`

Checks the given grammar for non-productive non-terminals.
