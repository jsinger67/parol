# Supplementary tools

Tools are separate binaries. Their source code can be found in the `bin` folder. The most prominent one is parol itself which is located in a dedicated subfolder.
But this sections is devoted to the other tools around.

Please note, that you do not need to use these tools normally when you want to generate parsers. All of their functionality is completely included in `parol` itself. But when you are about to solve a certain problem they may come handy. So it is useful to know whats in the bag.

Hint: All tools give a short help output when called without parameters:

```shell
    >cargo run --bin calculate_k
Missing arguments <par-file> <k=5>!
Example:
cargo run --bin calculate_k ./src/parser/parol-grammar-exp.par
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
