# List - A more detailed example

If you want you can build the parser for the *List* example with the following command lines:

```shell
cargo run --bin parol -- -f ./examples/list/list.par -e ./examples/list/list-exp.par -p ./examples/list/list_parser.rs -a ./examples/list/list_grammar_trait.rs -t ListGrammar -m list_grammar
```

This step is not necessary because the generated sources are already under source control.

Then you can start and test the *List* examples executable by executing the following command:

```shell
cargo run --example list -- ./examples/list/list_test.txt
```

You should get the following output
> [1, 2, 3, 4, 5, 6]

This single output line shows the result the parser has returned from the parsing of the content '1, 2, 3, 4, 5, 6,'. A list of numbers just as expected.

Here we will only have a closer look at the implementation of the semantic action for production 6 (`num`).

But first we will have a look at the grammar description [list.par](../examples/list/list.par):

```ebnf
%start list
%title "A simple comma separated list of integers"
%comment "A trailing comma is allowed."

%%

/* 0 */ list: ;
/* 1 */ list: num list_rest;
/* 2 */ list_rest: list_item list_rest;
/* 3 */ list_item: "," num;
/* 4 */ list_rest: ;
/* 5 */ list_rest: ",";
/* 6 */ num: "\d+";
```

What we need to know is that the parser will call the semantic actions for a certain production after it has recognized all symbols that are on the right hand side of it.
The generated function has arguments that correspond to the symbols on the right hand side. The name of the function is derived from the left hand side of the production (the non-terminal) plus the production number to ensure uniqueness of function names. This results in the name "num_6" in our case here.

The current number token ("\d+") in production 6 corresponds with the `num_0` parameter. Its type is `&ParseTreeStackEntry`. This type is predetermined by the `parol` parser's runtime, and can therefor be found in the `parol_runtime` crate. It can be thought of as being either a token from the input string that matched a terminal or a non-terminal that is actually a root node of a sub-tree of the parse tree.

Form the production we know that "\d+" is a terminal

We extract the token's text from this `num_0` parameter with the helper function `symbol` of the `ParseTreeStackEntry`. Then we convert it to `usize`, the type defined by `DefinitionRange`. If this succeeds we push the new `ListGrammarItem::Num` on our item stack.  

```rust
/// Semantic action for production 6:
///
/// num: "\d+";
///
fn num_6(
    &mut self,
    num_0: &ParseTreeStackEntry,
    parse_tree: &Tree<ParseTreeType>,
) -> Result<()> {
    let context = "num_6";
    let symbol = num_0.symbol(parse_tree)?;
    let number = symbol.parse::<DefinitionRange>().into_diagnostic().wrap_err( {
        format!(
            "{}: Error accessing token from ParseTreeStackEntry",
            context
        )
    })?;
    self.push(ListGrammarItem::Num(number), context);
    Ok(())
}
```

At the end of the parsing our item stack will contain all 'pushed in' `ListGrammarItem::Num` items, but in reversed order.
In the `list_1` semantic action of the *List* example we take these number from the user stack, reverse them and pushing it as `ListGrammarItem::List`:

```rust
    list.reverse();
    self.push(ListGrammarItem::List(list.to_vec()), context);
```

## Further readings

* [Tutorial](Tutorial.md)
