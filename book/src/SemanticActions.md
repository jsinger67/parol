# Semantic actions

The `parol` parser generator creates traits with functions that represent semantic actions.
The generated parser then calls these functions at parse time at the appropriate points with correct
arguments.

The generated trait for user actions (i.e. semantic actions) will be named after the following
scheme:

```rust
pub trait <NameOfYourGrammar>GrammarTrait<'t> {
    // ...
}
```

The lifetime parameter `<'t>` can be left out if the types used don't hold references to the scanned
text. This is automatically deduced.

Eventually your grammar processing item implements this trait and can overwrite those functions of
the trait in which it is interested in.

It doesn't need to implement all trait functions because the trait is created in a way where all of
its functions have default implementations.

All semantic actions are generated for non-terminals of your input grammar, and are typed accordingly.

The `parol` parser generator creates a trait with functions that represent semantic actions. Here,
the semantic actions are typed and they are generated for the *non-terminals of your input grammar*
instead of for *productions of the [expanded grammar](AstGeneration.html#the-expanded-grammar)*.

You therefore don't have to mess around with `ParseTreeType` although you still encounter items of
type `Token`. Also the expanded version of your grammar is much less of interest for you.

`parol`'s great merit is that it can generate an adapter layer automatically that provides the
conversion to typed grammar items. Indeed I carved out some simple rules that can be applied
universally to provide this layer of abstraction by generating the production bound semantic
actions accordingly.

This and the automatic AST type inference are the most outstanding properties of `parol`.

We will use the example
[calc](https://github.com/jsinger67/parol/tree/main/examples/calc) for detailed explanations.

The file
[calc_grammar_trait.rs](https://github.com/jsinger67/parol/blob/main/examples/calc/calc_grammar_trait.rs)
contains the generated traits and types we are interested in.

First we will have a look at the `CalcGrammarTrait` at the top of this file. For each non-terminal
of the input grammar
[calc.par](https://github.com/jsinger67/parol/blob/main/examples/calc/calc.par) it contains exactly
one semantic action.

```rust
/// Semantic actions trait generated for the user grammar
/// All functions have default implementations.
pub trait CalcGrammarTrait<'t> {
    /// Semantic action for non-terminal 'Calc'
    fn calc(&mut self, _arg: &Calc<'t>) -> Result<()> {
        Ok(())
    }
    // ...
}
```

The approach taken in this example is quite interesting. We only implement the semantic action for
the start symbol of our grammar: *Calc*.

The implementation can be found in
[calc_grammar.rs](https://github.com/jsinger67/parol/blob/main/examples/calc/calc_grammar.rs).

Near the end you can find the one and only semantic action we implement here and thereby creating
the functionality of a calculator language.

```rust
impl<'t> CalcGrammarTrait<'t> for CalcGrammar<'t> {
    /// Semantic action for non-terminal 'Calc'
    fn calc(&mut self, arg: &Calc<'t>) -> Result<()> {
        self.process_calc(arg)?;
        Ok(())
    }
}
```

But what is the advantage of implementing only the start symbols's semantic action? Well, since the
start symbol is the root node of each and every concrete parse tree, we know that the generated type
for it should comprise the complete input as the result of the parsing.

The key to this is the structure of the generated type `Calc`. It resembles the structure of all
productions belonging to the non-terminal `Calc`. There is actually only one production for `Calc`:

```parol
Calc: { Instruction ";"^ };
```

```rust
///
/// Type derived for non-terminal Calc
///
pub struct Calc<'t> {
    pub calc_list: Vec<CalcList<'t>>,
}
```

The type `Calc` is basically a vector, which can be deduced from the repetition construct at the
right-hand side of the production (`{ Instruction ";"^ }`).

The elements of the vector are of type `CalcList` that is defined this way:

```rust
///
/// Type derived for non-terminal calcList
///
pub struct CalcList<'t> {
    pub instruction: Instruction<'t>,
}
```

And in turn the type `Instruction` looks like this:

```rust
///
/// Type derived for non-terminal instruction
///
pub enum Instruction<'t> {
    Assignment(InstructionAssignment<'t>),
    LogicalOr(InstructionLogicalOr<'t>),
}
```

The latter one is an enum with two variants because the non-terminal `Instruction` has two
productions:

```parol
// ---------------------------------------------------------
// INSTRUCTION
Instruction: Assignment;
Instruction: LogicalOr;
```

This concept is applied for all non-terminals of your grammar. Actually your grammar became
*typified*.

This means eventually that any variable of type `Calc` can represent a validly parsed input sentence
that belongs to the grammar defined by
[calc.par](https://github.com/jsinger67/parol/blob/main/examples/calc/calc.par).

You then only have to evaluate the content of this value as done in this calculator example.
I recommend to study this example more deeply and the approach will become obvious to you.

As mentioned earlier the implementation can be found here:
[calc_grammar.rs](https://github.com/jsinger67/parol/blob/main/examples/calc/calc_grammar.rs).
