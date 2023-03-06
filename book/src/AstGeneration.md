# AST generation

`parol` can be instructed to generate all types your grammar implies automatically. It therefore
analyzes all productions in your grammar.

## Grammar transformation

The first step is to canonicalize your grammar into a
standard format applying the following transformations.

* All EBNF constructs, i.e. optional elements, repetitions and groupings are substituted by
equivalent representations.
  * A: [B]; => A: BOpt; BOpt: B; BOpt: ;
  * A: {B}; => A: BList; BList: B BList; BList: ;
  * A: (B); => A: BGroup; BGroup: B;
* Alternations are propagated to multiple productions.
  * A: B | C; => A: B; A: C;

These transformations are applied iteratively until all EBNF constructs are replaced.

## Sanity checks

Then `parol` checks this pre-transformed input grammar for several properties that prevent a
successful processing. Those unwanted properties are

* Left-recursions
* Non-productive non-terminals
* Unreachable non-terminals

If the grammar does not have such properties the next step is to left-factor this grammar form. This
step is crucial for decreasing the number of necessary lookahead symbols.

## The Expanded grammar

This finally transformed grammar is the basis for the parser generation and is typically written to
file for later reference. By convention this 'expanded' grammar is stored to files named
\<original-name\>-exp.par.

This expanded grammar is the basis for parser generation.

## Type inference

Having such a transformed grammar all productions have the form
\\[v: s*; \\]
where \\\(v \epsilon V, s \epsilon (V \cup \Sigma)\\\), \\\(V\\\) is the set of non-terminals,
\\\(\Sigma\\\) is the set of terminals.

The relation of the generated productions to their original EBNF constructs is actually lost at this
point.

But because we need the information if a set of productions was originated from, e.g. an optional
construct (`[...]`) `parol`conveys these relationship during the whole transformation process to be
able to infer it into a Rust `Option<T>` eventually.

To explain it using the form of transformation shown above we could write this:

`A: [B]; => A: BOpt; BOpt: B; BOpt: ; => typeof A = Option<typeof B>`

This step leads directly to a solution if non-terminal `A` has only one production.

In this case the the type of `A` is

```rust
struct A {
    b: Option<B>
}
```

We must use a `struct` here because this patterns should work for productions with \\\(n\\\)
elements on its right-hand side. For each such element we then introduce a separate member into the
struct.

If non-terminal `A` has more than one productions the resulting type of `A` will be a Rust `enum`
type with \\\(n\\\) enum variants for \\\(n\\\) productions, e.g.:

`A: B | C; => A: B; A: C; =>`

```rust
struct B {
    // ...
}
struct C {
    // ...
}
// Type of non-terminal A
enum A {
    A0(B),
    A1(C),
}
```

When finally all types for all non-terminals are inferred `parol` generates an overall AST type.
This is also a Rust `enum`. It comprises all non-terminal types of the grammar and provides exactly
one enum variant for each of them. This type is mainly used by the parser itself to be able to
instantiate a typed parse stack. The user rarely have to deal with this AST `enum`.

### Recursive structure of a grammar

A context free grammar is typically defined using recursive constructs. But you can't define types
in Rust that are directly recursive because this would lead to an infinitive type size.

To cope with this limitation `parol` generates boxed types for non-terminals when introducing
elements to `struct`s, e.g.:

```rust
struct A {
    b: Box<B>
}
```

This results in finite type sizes.

## Manage Type generation

### Omission of elements

You can suffix grammar symbols (terminals and non-terminals) with a cut operator (^). This instructs
`parol` to not propagate them to the AST type, e.g.:

```parol
Group: '('^ Alternations ')'^;
```

The AST type for the symbol `Group` will then only contain a member for the non-terminal
`Alternations`. The parentheses are suppressed because they have no special purpose for the grammar
processing itself.

### Assigning user types

You can specify a user type to be inserted into the AST structure at the place where the symbol
would otherwise had the originally generated type.
Add after a grammar symbol a colon followed by a user type name to instruct `parol` to use this type
instead. In your language implementation you have to provide fallible or infallible conversions
from the original generated types to your types by implementing one of the traits `From` or `TryFrom`.
An examples can be found in the `list_auto` example.
You can also define aliases for the user type names by inserting as many `%user_type` directives as
you want. Then use these aliases behind the colons.

You may have look at example [list_auto](https://github.com/jsinger67/parol/blob/6800c3060bad0df033e55cf113cfd16e860a5373/examples/list_auto/list.par)
that demonstrates the handling of user types.

```parol
%start List
%title "A possibly empty comma separated list of integers"
%comment "A trailing comma is allowed."
%user_type Number = crate::list_grammar::Number
%user_type Numbers = crate::list_grammar::Numbers

%%

List: [Items: Numbers] TrailingComma^;
Items: Num {","^ Num};
Num: "0|[1-9][0-9]*": Number;
TrailingComma: [","^];
```

In this example grammar the terminal in the production `Num` is assigned to the user type `Number`
which in turn is a shorthand for `crate::list_grammar::Number`. Also the non-terminal `Items` is
assigned to the user type `Numbers` which in turn is a shorthand for `crate::list_grammar::Numbers`.

The parser generator substitutes the automatically inferred type in the type of the production by
the user provided one and the parser calls the conversion form the original type to the user type at
parse time.

The original type is the one of the source item in the grammar - terminal or non-terminal. Please
have a look at the generated semantic action of the internal wrapper for production 1 of the
expanded grammar `list-exp.par` which can be found in also generated traits file
`examples\list_auto\list_grammar_trait.rs`:

```rust
    /// Semantic action for production 1:
    ///
    /// ListOpt /* Option<T>::Some */: Items : Numbers;
    ///
    #[parol_runtime::function_name::named]
    fn list_opt_0(&mut self, _items: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let items = pop_item!(self, items, Items, context);
        let list_opt_0_built = ListOpt {
            items: (&items)
                .try_into()
                .map_err(parol_runtime::ParolError::UserError)?,
        };
        self.push(ASTType::ListOpt(Some(Box::new(list_opt_0_built))), context);
        Ok(())
    }
```

At the line after the trace the original item is popped from the parse stack. It has the Rust type
`Items`:

```rust
/// Type derived for non-terminal Items
pub struct Items {
    pub num: Box<Num>,
    pub items_list: Vec<ItemsList>,
}
```

Then later at the construction of the `ListOpt` structure the conversion to the user's type is
called: `.items((&items).try_into()`.

The `TryFrom` trait is provided by the user. Please see `examples\list_auto\list_grammar.rs` for
that:

```rust
impl TryFrom<&Items> for Numbers {
    type Error = anyhow::Error;

    fn try_from(items: &Items) -> std::result::Result<Self, Self::Error> {
        Ok(Self(items.items_list.iter().fold(
            vec![items.num.num.0],
            |mut acc, e| {
                acc.push(e.num.num.0);
                acc
            },
        )))
    }
}
```

This is an example how non-terminal types are converted into user types.

The easier variant is the conversion of a terminal type (i.e. a `Token`) into a user type. You can
find an example also in `examples\list_auto\list_grammar.rs`:

```rust
impl<'t> TryFrom<&Token<'t>> for Number {
    type Error = anyhow::Error;

    fn try_from(number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        Ok(Self(number.text().parse::<u32>()?))
    }
}
```

Here the scanned text of the token is accessed using the method `text` of the `Token` type that was
imported from the `parol_runtime`crate. This text is then parsed into an `u32` type and finally
wrapped into a `Number`type which is a *newtype* for `u32`.

By implementing some `From` or `TryFrom` traits for your user type you can integrate them easily
into the parse process.

There exist some examples that can help to become familiar with this concept. Maybe you would like
to have a look at my rudimentary
[basic interpreter example](https://github.com/jsinger67/parol/tree/main/examples/basic_interpreter).
