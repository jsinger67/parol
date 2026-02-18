# Questions and Answers

## Q: I get stack overflow in compiler-generated traits like `Drop`, `Clone`, and `Debug`
A: The reason is most likely a deeply nested structure generated during parsing. There are two
recommendations that can lead to a working solution:

> The `Drop`/`Clone`/`Debug` naming is Rust-specific, but the grammar advice below also helps in C#
> if deeply recursive grammar shapes cause stack overflows at runtime.

1. Avoid 'plain' recursions in your grammar like this (for LL(k) grammars):
```parol
List: ListItem List;
List: ;
ListItem: Number;
Number: /0|[1-9][0-9]*/;
```
This will generate recursive data structures like this:
```rust
pub enum List<'t> {
    ListItemList(ListListItemList<'t>),
    ListEmpty(ListListEmpty),
}
pub struct ListListItemList<'t> {
    pub list_item: Box<ListItem<'t>>,
    pub list: Box<List<'t>>,
}
pub struct Number<'t> {
    pub number: Token<'t>, /* 0|[1-9][0-9]* */
}
```
The recursion occurs here by containing `List` in `ListListItemList`.


Use instead `parol`'s own repetition construct (`{...}`), which will result in the generation of a data type
containing a vector.
```parol
List: { ListItem };
ListItem: Number;
Number: /0|[1-9][0-9]*/: Number;
```
This will generate iterative data structures like this:
```rust
pub struct List<'t> {
    pub list_list: Vec<ListList<'t>>,
}
pub struct ListList {
    pub list_item: Box<ListItem>,
}
pub struct ListItem<'t> {
    pub number: Box<Number<'t>>,
}
```
2. Implement the problematic traits yourself and avoid recursion by using a loop instead.
I cannot give general advice here, but there are plenty of examples that cover this topic
thoroughly.

## Q: I get strange errors while developing my new grammar and cannot figure out the problem
A: Consider the following recommendations:

### Break down the problem with as little input as possible

This will limit the possible error location and also minimize the amount of traces to scrutinize.

### Disable error recovery for LL(k) parsers

The process of error recovery will surely shroud the original error location.
Therefore it is advisable to temporarily disable it.

Use the Builder API (`disable_recovery()`) or the command-line argument (`--disable-recovery`).

### Enable traces

In all projects generated with `parol new`, `env_logger` is built in. First activate all traces.
I will show the principle in PowerShell because this works on Windows as well as on Linux.

> The `RUST_LOG` examples below are for Rust projects. In C#, use your application's logging setup
> and inspect generated parser/action code similarly when tracing parse behavior.

```powershell
$env:RUST_LOG="trace"
```
Then run your scenario and examine the output. If necessary, restrict the traces further by tweaking
the RUST_LOG variable, e.g. for parser and scanner internals use this:
```powershell
$env:RUST_LOG="parol_runtime=trace"
# or
$env:RUST_LOG="parol_runtime::lexer=trace"
# or
$env:RUST_LOG="parol_runtime::parser=trace"
# or
$env:RUST_LOG="parol_runtime::lr_parser=trace"
```
* Examine the traces from the beginning to pin down the first occurrence of the problem
* Often problems are related to incorrect terminal definitions, terminal conflicts, or even
scanner-state-related issues. Therefore:
    * Check for token types attached to the tokens provided during parse, the numbers can be found
    in the generated parser
    * Check the current scanner state and if the tokens are valid there

## Q: I get warnings in generated code 'This function has too many arguments'
A: Configure the builder in your `build.rs` to let `parol` generate a
```rust
#![allow(clippy::too_many_arguments)]
```
line at the beginning of your generated file.

Add this line in the builder configuration somewhere before the call to `.generate_parser()`:

```rust
        .inner_attributes(vec![InnerAttributes::AllowTooManyArguments])
```
Don't forget to import the `InnerAttributes` into your `build.rs`:

```rust
use parol::{build::Builder, InnerAttributes, ParolErrorReporter};
```

> This warning and the `clippy` suppression are Rust-specific. For C#, prefer reducing very long
> RHS productions by factoring grammar rules, as described below.

Another way to avoid this warning is to modify your grammar such that the lengths of the right-hand
sides of your productions are decreased. Therefore examine the productions that correlate to the
functions where the warnings occur. Then consider to factor out parts of the RHS into separate
productions.
