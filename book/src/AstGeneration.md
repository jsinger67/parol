# AST Generation

`parol` can automatically generate all types implied by your grammar. It analyzes every production
in your grammar.

## Grammar Transformation

The first step is to canonicalize your grammar by applying the following transformations:

- All EBNF constructs—optional elements, repetitions, and groupings—are replaced with equivalent
representations:
  - `A: [B];` → `A: BOpt; BOpt: B; BOpt: ;`
  - `A: {B};` → `A: BList; BList: B BList; BList: ;`
  - `A: (B);` → `A: BGroup; BGroup: B;`
- Alternations are expanded into multiple productions:
  - `A: B | C;` → `A: B; A: C;`

These transformations are applied iteratively until all EBNF constructs are eliminated.

Note: Transformations for LR grammars differ slightly, but the principle remains the same.

## Sanity Checks

Next, `parol` checks the transformed grammar for properties that would prevent successful processing:

- Left recursion
- Non-productive non-terminals
- Unreachable non-terminals

If none of these issues are present, the grammar is left-factored to reduce the number of required
lookahead symbols.

## The Expanded Grammar

The fully transformed grammar serves as the basis for parser generation and is typically saved for
reference. By convention, this "expanded" grammar is stored in files named `<original-name>-exp.par`.

## Type Inference

With the transformed grammar, all productions take the form:

```
[v: s*;]
```
where \\\(v \epsilon V, s \epsilon (V \cup \Sigma)\\\), \\\(V\\\) is the set of non-terminals,
\\\(\Sigma\\\) is the set of terminals.

At this stage, the relationship between generated productions and their original EBNF constructs is
lost.

However, since it is necessary to know if a set of productions originated from an optional construct
(`[...]`), `parol` maintains this relationship throughout the transformation process. This allows it
to infer types such as Rust's `Option<T>`.

For example, using the transformation above:

`A: [B];` → `A: BOpt; BOpt: B; BOpt: ;` → `typeof A = Option<typeof B>`

If non-terminal `A` has only one production, its type is:

```rust
struct A {
    b: Option<B>
}
```

A `struct` is used because productions may have multiple elements on the right-hand side, each
becoming a separate member.

If non-terminal `A` has multiple productions, its type becomes a Rust `enum` with one variant per
production:

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

Once all types for non-terminals are inferred, `parol` generates an overall AST type as a Rust `enum`.
This enum contains one variant for each non-terminal type and is mainly used by the parser to
instantiate a typed parse stack. Users rarely need to interact with this AST enum directly.

### Recursive Structure of a Grammar

Context-free grammars are typically defined using recursive constructs. However, Rust does not allow
directly recursive types, as this would result in infinite type sizes.

To address this, `parol` generates boxed types for non-terminals when adding elements to structs:

```rust
struct A {
    b: Box<B>
}
```

This ensures finite type sizes.

`parol` can minimize the use of boxed types in the generated parser. The tool supports a
command-line switch (`-b`, `--min_boxes`) to enable box minimization. The `parol::build::Builder`
also provides a `minimize_boxed_types()` method for use in build scripts.

`parol` determines where recursion cannot occur by analyzing the grammar structure.

## Managing AST Generation

### Omission of Elements

You can suffix grammar symbols (terminals and non-terminals) with a cut operator (`^`) to prevent
them from being included in the AST type. For example:

```parol
Group: '('^ Alternations ')'^;
```

The AST type for `Group` will only contain a member for the non-terminal `Alternations`. The
parentheses are omitted since they are not needed for grammar processing.

### Assigning User Types

You can specify a user type to be inserted into the AST structure in place of the automatically
generated type. Add a colon followed by a user type name after a grammar symbol to instruct `parol`
to use this type. In your implementation, provide fallible conversions from references of the
original generated types (`&T`) to your types (`U`) by implementing the trait `TryFrom<&T> for U`.
See the `list` example for details.

You can also define aliases for user type names using `%user_type` directives. Use these aliases
after the colons.

See the [list example](https://github.com/jsinger67/parol/blob/main/examples/list/list.par) for user
type handling:

```parol
%start List
%title "A possibly empty comma separated list of integers"
%comment "A trailing comma is allowed."
%user_type Number = crate::list_grammar::Number
%user_type Numbers = crate::list_grammar::Numbers
%line_comment "//"

%%

List: [Items: Numbers] TrailingComma^;
Items: Num {","^ Num};
Num: "0|[1-9][0-9]*": Number;
TrailingComma: [","^];
```

In this grammar, the terminal in the `Num` production is assigned to the user type `Number`, which
is an alias for `crate::list_grammar::Number`. The non-terminal `Items` is assigned to the user type
`Numbers`, an alias for `crate::list_grammar::Numbers`.

The parser generator replaces the automatically inferred type with the user-provided type and calls
the conversion from the original type to the user type during parsing.

The original type is the type of the source item in the grammar—terminal or non-terminal. See the
generated semantic action for production 1 of the expanded grammar `list-exp.par` in the traits file
`examples\list\list_grammar_trait.rs`:

```rust
/// Semantic action for production 1:
///
/// `ListOpt /* Option<T>::Some */: Items : Numbers;`
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
    self.push(ASTType::ListOpt(Some(list_opt_0_built)), context);
    Ok(())
}
```

After tracing, the original item is popped from the parse stack. Its Rust type is `Items`:

```rust
/// Type derived for non-terminal Items
pub struct Items {
    pub num: Box<Num>,
    pub items_list: Vec<ItemsList>,
}
```

When constructing the `ListOpt` structure, the conversion to the user type is called:
`(&items).try_into()`.

The `TryFrom` trait is provided by the user. See `examples\list\list_grammar.rs`:

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

This demonstrates how non-terminal types are converted into user types.

For terminals (tokens), conversion is simpler. See `examples\list\list_grammar.rs`:

```rust
impl<'t> TryFrom<&Token<'t>> for Number {
    type Error = anyhow::Error;

    fn try_from(number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        Ok(Self(number.text().parse::<u32>()?))
    }
}
```

Here, the scanned text of the token is accessed using the `text` method of the `Token` type from
`parol_runtime`. The text is parsed into a `u32` and wrapped in a `Number` newtype.

By implementing `TryFrom` traits for your user types, you can easily integrate them into the parse
process.

Several examples are available to help you become familiar with this concept. You may also review
the [basic interpreter example](https://github.com/jsinger67/parol/tree/main/examples/basic_interpreter).

> For a complete list of ways to control AST type generation, see:
[Controlling the AST generation](./ParGrammar.md#controlling-the-ast-generation)