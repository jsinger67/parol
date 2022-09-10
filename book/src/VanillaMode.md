# Vanilla mode

Although the auto-generation mode (switch `-g`, `--auto_generate`) is the recommended way to use
`parol` you can alternatively work in *vanilla mode*.

That means that `parol` skips generating AST types for you and it generates only a trait with
semantic actions for each production of the expanded grammar instead of semantic actions for each
non-terminal.

This means that you gain more control although you may loose some comfort.

Basically it is a matter of taste what mode you use. But keep in mind that growing complexity can
have an impact on the maintainability of your software.

So although you may loose full speed and give up some control you obtain maintainability when using
the auto-generation mode.

Actually `parol` itself was build in the simple mode at the first stages of its development (before
version 0.9.3). But the implementation of new features required more and more changes in the grammar
and showed the vulnerability of the existing implementation to changes in the input grammar.

Anyway, this chapter is dedicated to the way `parol` functions without auto-generation.

You may have a look at example
[list](https://github.com/jsinger67/parol/blob/8bd3a5f3f0341cc95a6eb2cee548c123209e0ccc/examples/list/list.par)
that uses the *vanilla mode* and actually shows how easy it is to work this way.

We will elaborate this by implementing a list example in an alternative way.

```parol
%start List
%title "A possibly empty comma separated list of integers"
%comment "A trailing comma is allowed."

%%

List: Items TrailingComma^;
Items: Num {","^ Num} | ;
Num: "0|[1-9][0-9]*";
TrailingComma: [","^];
```

Let's generate a new binary crate:

You can try this grammar by calling

```shell
parol new --bin --path ./vanilla_list --tree
```

Open the generated crate and substitute the generated dummy grammar by the one above.
Open the build.rs and delete the line 11:

```rust
        .enable_auto_generation()
```

For the sake of completeness delete the `-g` from the CLI equivalent in the comment at the
beginning of `main`.

Also change the `test.txt` to the content

```text
1, 2, 3, 4, 5, 6,
```

Now you can parse this text by calling

```shell
cargo run ./test.txt
```

This will actually result in a bunch of errors because `parol new` generated the source for the new
crate in the spirit of auto-generation mode.

But fortunately it is easy to correct the errors and create the basis for our *vanilla mode* crate.

Replace the content of `vanilla_list_grammar.rs` with the following lines

```rust
use id_tree::Tree;
use miette::{IntoDiagnostic, Result, WrapErr};
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};

use crate::vanilla_list_grammar_trait::VanillaListGrammarTrait;

///
/// The value range for the supported list elements
///
pub type DefinitionRange = usize;

///
/// Data structure that implements the semantic actions for our list grammar
///
#[derive(Debug, Default)]
pub struct VanillaListGrammar {
    pub numbers: Vec<DefinitionRange>,
}

impl VanillaListGrammar {
    pub fn new() -> Self {
        VanillaListGrammar::default()
    }

    fn push(&mut self, item: DefinitionRange) {
        self.numbers.push(item)
    }
}

impl Display for VanillaListGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(
            f,
            "[{}]",
            self.numbers
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl VanillaListGrammarTrait for VanillaListGrammar {}
```

Now you should be able to run the parser

```shell
$ cargo run ./test.txt
Finished dev [unoptimized + debuginfo] target(s) in 0.23s
     Running `target\debug\vanilla_list.exe .\test.txt`
Parsing took 3 milliseconds.
Success!
[]
```

Also some warnings should occur. But we resolve them soon.

What we see here is that the parser accepts the input but doesn't collect the list items for us
immediately (there are no list items in between `[` and `]`). The parser functions as an acceptor
but without any processing.

We need to do this on our own.

To be able to 'hook' into the right production we need to examine the expanded grammar more closely
than we had to in the auto-generation mode.

So open the generated file `vanilla_list-exp-par` and look for the production where a `Num` token
is accepted:

```parol
/* 5 */ Num: "0|[1-9][0-9]*";
```

Then we need to implement the semantic action for exactly this production number 5. We find the
trait function to implement in the file `src\vanilla_list_grammar_trait.rs` and copy it into the
impl block at the end of the file `src\vanilla_list_grammar.rs`:

```rust
impl VanillaListGrammarTrait for VanillaListGrammar {
    /// Semantic action for production 5:
    ///
    /// Num: "0|[1-9][0-9]*";
    ///
    fn num(&mut self, _num: &ParseTreeStackEntry, _parse_tree: &Tree<ParseTreeType>)
      -> Result<()> {
        Ok(())
    }
}
```

Here we can implement our handling:

```rust
    /// Semantic action for production 5:
    ///
    /// Num: "0|[1-9][0-9]*";
    ///
    fn num(&mut self, num: &ParseTreeStackEntry, parse_tree: &Tree<ParseTreeType>)
      -> Result<()> {
        let symbol = num.symbol(parse_tree)?;
        let number = symbol
            .parse::<DefinitionRange>()
            .into_diagnostic()
            .wrap_err("num: Error accessing token from ParseTreeStackEntry")?;
        self.push(number);
        Ok(())
    }
```

Now run the parser again

```shell
$ cargo run ./test.txt
    Finished dev [unoptimized + debuginfo] target(s) in 1.54s
     Running `target\debug\vanilla_list.exe .\test.txt`
Parsing took 4 milliseconds.
Success!
[1, 2, 3, 4, 5, 6]
```

Yep! This worked fine.

The remaining warnings can be removed be deleting the following lines in `main.rs`:

```rust
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate function_name;
```

Finally you can remove the references to crates `derive_builder` and `function_name` from
`cargo.toml`.

Note that you can`t use user defined types for your ATS types in vanilla mode because no AST types
are generated at all. But actually you opted in to build the AST types on your own when you disable
auto-generation mode.
