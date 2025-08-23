# Operator Associativity

Operator associativity defines the direction in which **operators of the same precedence** are
evaluated.

## Left Associativity

Left associativity means operators are grouped from the left. For example, `x * y * z` is evaluated
as `(x * y) * z`.

Consider this example grammar, which supports left-associative multiplication:

```parol
%start LeftAssociativity
%title "Operator associativity"
%comment "Shows the handling of operator associativity in `parol`"

%%

Literal : /[0-9]+/ ;

// ---------------------------------------------------------
// OPERATOR SYMBOLS
MulOp   : '*' ;

// ---------------------------------------------------------
// EXPRESSIONS
LeftAssociativity : Multiplication ;

Multiplication : Literal { MulOp Literal } ;
```

To try this grammar:

```shell
parol new --bin --path .\left_associativity --tree
```

Replace the generated dummy grammar with the example above. Set `test.txt` to:

```text
5 * 6 * 2
```

Parse the text by running:

```shell
cargo run ./test.txt
```

from the root of the generated crate.

Parsing `5 * 6 * 2` produces this parse tree:

![Parse Tree](./left_associativity/test.svg)

At first glance, the parse tree may appear to impose right associativity (evaluated right to left).
However, in `parol`, **all repetitive grammar constructs are represented as vectors in AST types**.

Example from the generated types in `src/left_associativity_grammar_trait.rs`:

```rust
/// Type derived for non-terminal Multiplication
pub struct Multiplication<'t> {
    pub literal: Box<Literal<'t>>,
    pub multiplication_list: Vec<MultiplicationList<'t>>,
}

/// Type derived for non-terminal MultiplicationList
pub struct MultiplicationList<'t> {
    pub mul_op: Box<MulOp<'t>>,
    pub literal: Box<Literal<'t>>,
}
```

Items in repetitions (`{...}`) are stored in vectors and can be processed in the desired direction.
This behavior applies to all grammar repetitions.

It is up to your grammar processing to choose the evaluation direction. To implement left
associativity, apply these changes to `src/left_associativity_grammar.rs`:

Replace the use statements at the top of the file:

```rust
use crate::left_associativity_grammar_trait::{
    LeftAssociativity, LeftAssociativityGrammarTrait, Literal,
};
use parol_runtime::Result;
#[allow(unused_imports)]
use parol_runtime::parol_macros::{bail, parol};
use std::fmt::{Debug, Display, Error, Formatter};
```

Add a `result` member to the struct:

```rust
pub struct LeftAssociativityGrammar<'t> {
    pub left_associativity: Option<LeftAssociativity<'t>>,
    pub result: u32,
}
```

Add these functions to the `impl LeftAssociativityGrammar<'_>` block:

```rust
fn number(literal: &Literal) -> Result<u32> {
    literal
        .literal
        .text()
        .parse::<u32>()
        .map_err(|e| parol!("'{}': {e}", literal.literal.text()))
}

fn process_operation(&mut self) -> Result<()> {
    if let Some(grammar) = &self.left_associativity {
        let init = Self::number(&grammar.multiplication.literal)?;
        self.result = grammar
            .multiplication
            .multiplication_list
            .iter()
            .try_fold(init, |acc, mul| -> Result<u32> {
                Ok(acc * Self::number(&mul.literal)?)
            })?;
        Ok(())
    } else {
        bail!("No valid parse result!")
    }
}
```

Update the `Display` implementation:

```rust
impl Display for LeftAssociativityGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.left_associativity {
            Some(_) => writeln!(f, "{}", self.result),
            None => write!(f, "No parse result"),
        }
    }
}
```

Change the last line of the `left_associativity` function from:

```rust
    Ok(())
```

to:

```rust
    self.process_operation()
```

Run the parser again:

```shell
cargo run ./test.txt
```

Sample output:

```
Parsing took 0 milliseconds.
Success!
60
```

The parser correctly calculates the result: **60**.

The key part is the `process_operation` function, which folds the multiplication results into
`result`. The initial value is the first element (`literal`) of the `Multiplication` struct,
matching the grammar structure:

```parol
Multiplication : Literal { MulOp Literal } ;
```

## Right Associativity

Right associativity means operators are grouped from the right. For example, `x ^ y ^ z` is
evaluated as `x ^ (y ^ z)`.

Here is a grammar for right-associative potentiation:

```parol
%start RightAssociativity
%title "Operator associativity"
%comment "Shows the handling of operator associativity in `parol`"

%%

Literal : /[0-9]+/ ;

// ---------------------------------------------------------
// OPERATOR SYMBOLS
PowOp   : '^' ;

// ---------------------------------------------------------
// EXPRESSIONS
RightAssociativity : Potentiation ;

Potentiation : Literal { PowOp Literal } ;
```

To try this grammar:

```shell
parol new --bin --path .\right_associativity --tree
```

Replace the generated dummy grammar with the example above. Set `test.txt` to:

```text
4 ^ 3 ^ 2
```

Parse the text by running:

```shell
cargo run ./test.txt
```

from the root of the generated crate.

Parsing `4 ^ 3 ^ 2` produces this parse tree:

![Parse Tree](./right_associativity/test.svg)

The parse tree structure is identical to the left-associative example. `parol` handles all
repetitive constructs as vectors.

To implement right associativity, modify `src/right_associativity_grammar.rs` as in the
left-associativity example, changing prefixes from `Left`/`left_` to `Right`/`right_`.

Replace the `process_operation` function with:

```rust
fn process_operation(&mut self) -> Result<()> {
    if let Some(grammar) = &self.right_associativity {
        self.result = grammar
            .potentiation
            .potentiation_list
            .iter()
            .rev()
            .try_fold(1, |acc, mul| -> Result<u32> {
                Ok(Self::number(&mul.literal)?.pow(acc))
            })?;
        let last = Self::number(&grammar.potentiation.literal)?;
        self.result = last.pow(self.result);
        Ok(())
    } else {
        bail!("No valid parse result!")
    }
}
```

Here, the fold is performed in reverse order (`.rev()`), starting with `1`, and the last operand is
the single literal in the `Potentiation` struct.

Run the parser again:

```shell
cargo run ./test.txt
```

Sample output:

```
Parsing took 0 milliseconds.
Success!
262144
```

The parser correctly calculates the result: **262144**.
