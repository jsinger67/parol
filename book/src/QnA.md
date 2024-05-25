# Questions and answers

## Q: I get stack overflow in compiler generated trait's like `Drop`, `Clone` and `Debug`
A: The reason is most likely a deeply nested structure generated during parsing. There are two
advices which could lead to working solution:

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
2. Implement the problematic traits by yourself and avoid the recursion by using a loop instead.
I can't give a general advice here, but there are plenty examples out there that covers this topic
thoroughly.

## Q: I get strange errors while commissioning my new grammar and can't figure out what the problem is
A: Consider the following advices

* Break down the problem with a least input as possible
* Enable traces:

In all projects that were generated with `parol new` the env_logger is built in. First activate all
traces. I'll show the principle in powershell because this will work on Windows as well as on Linux

```powershell
$env:RUST_LOG="trace"
```
Then run your scenario and examine the output. If necessary restrict the traces further by tweaking
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
* Often the problems are related to wrong terminal definitions or terminal conflicts or evens
scanner state related problems, therefore
    * Check for token types attached to the tokens provided during parse, the numbers can be found
    in the generated parser
    * Check the current scanner state and if the tokens are valid there
