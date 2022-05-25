# Tutorial

> This tutorial is still under construction!
>
> If you find any inconsistencies, typos or bugs, if you have questions or suggestions feel free to
create an issue against it or contribute to our [discussions](https://github.com/jsinger67/parol/discussions).
>
> The old tutorial can be found here [Old Tutorial](./TutorialOld.md). It is still useful and
explains the approaches that are now superseded by the new auto-generation related ones, but which
are still applicable. Also the old tutorial explains the grammar transformation `parol` applies.

This tutorial will help new users to get quickly familiar with the tool `parol`.
To get something useful we need a goal that is not too complicated but covers the most steps to be
able to use `parol` in real-world projects.

How about a BASIC interpreter? Maybe you remember the old C64 with its BASIC V2.0?

I decided to re-implement a small part of this BASIC dialect for this tutorial.
You may ask, why to choose a forty years old language? I say, why not? Because we can and because
it's fun. 😉

You can find the **complete example project** here [`parol_basic_interpreter`](https://github.com/jsinger67/parol_basic_interpreter.git).

## Prerequisites

First we need to install `parol`:

```shell
cargo install parol
```

Having this completed we can take advantage of the `parol new` subcommand and let `parol` create our
new test project for us.

Change your working directory to where the new project should be created in a subfolder.

```shell
parol new --bin --path ./basic
```

Then change into the new project folder and start the initial build. Here `parol` is already
generating two files from the initial grammar definition.

```shell
cd ./basic
cargo build
```

If this is completed open your favorite editor.

```shell
code .
```

>One side note about your favorite editor. If it happens that you use Visual Studio Code you can
install my VS Code extension [parol-vscode](https://github.com/jsinger67/parol-vscode.git).
Download the vsix package from the latest release and install it with
>
>> ```code --install-extension ./parol-vscode-0.1.2.vsix```
>
>This extension provides syntax highlighting, folding and language icons and will surely be useful
for you.

## Initial commit

Before we change anything in the new project please do the initial commit in git now.

## Project structure

You should see the following project structure:

```text
─ src ─┐
       ├─ basic_grammar_trait.rs
       ├─ basic_grammar.rs
       ├─ basic_parser.rs
       └─ main.rs
─ .gitignore
─ basic-exp.par
─ basic.par
─ build.rs
─ Cargo.lock
─ Cargo.toml
```

Some of them are the usual constituent parts of a rust project, such as Cargo.toml, Cargo.lock etc.

We are more interested in the parts that are specific for a `parol` project.

### The grammar description file `basic.par`

This file is the initial grammar description file that `parol` has created for us.
Our grammar will be developed here later on.

### The expanded grammar description file `basic-exp.par`

This file was derived from the `basic.par` by `parol`. It is actually an equivalent transformation of
our original grammar definition. This transformation is optimized for LL parsing. Normally we seldom
need to look at it.

### The build script `build.rs`

This file contains the build instructions for `parol` to generate the necessary output files from the
file `basic.par` during cargo build.

### The main module `src/main.rs`

Here we have our usual main function where we call the generated parser and feed it with a text file
that was given as command line argument. We will provide our basic files here later.

### The parser module `src/basic_parser.rs`

This is the generated parser module we never change manually. It contains data the LL(k) parser from
the `parol_runtime` crate is initialized with. We actually don't need to understand the internals of
it.

### The module with the grammar trait `src/basic_grammar_trait.rs`

This is also a generated file that receives a special-made trait `BasicGrammarTrait` with default
implementations of our semantic actions. We will later look into it in more detail.

### The grammar implementation module `src/basic_grammar.rs`

Here we will actually do our coding and will develop our Basic interpreter in the course of this
tutorial.

## The BASIC grammar we want to support

We will of course not be able to implement the whole BASIC language in this tutorial so we need to
select a useful subset.

We will support the following language elements:

* Comments with REM
* Numeric constants (integer and float)
* Floating point variables
* Statements
  * IF THEN
  * IF GOTO
  * GOTO
  * Assignments
* Expressions
  * Arithmetic expressions with addition, subtraction, multiplication and division as well as using
  parenthesis
  * Comparison expressions
  * Logical expressions
* BASIC commands
  * PRINT or ?
  * END

### The structure of a BASIC program

We will first have a look at the basic structure of any BASIC program.

It is a list of lines:

```ebnf
Basic  : { Line }
       ;
```

In turn a line is a list of statements separated by colons:

```ebnf
Line   : LineNumber Statement { ":" Statement } EndOfLine
       ;
```

And we start with the simplest statement: the REM statement:

```ebnf
Statement
       : "REM" Comment EndOfLine
       ;
```

### Our grammar so far

I will give here the complete content of the `basic.par` at this stage of development:

```ebnf
%start Basic
%title "Basic grammar"
%comment "Subset of C64 Basic used in tutorial for parser generator `parol`"
%auto_newline_off

%%

Basic   : [EndOfLine] Line { EndOfLine Line } [EndOfLine]
        ;
Line    : LineNumber Statement { ":" Statement }
        ;
Statement
        : Remark
        ;
Remark  : "REM" [Comment]
        ;
LineNumber
        : "[0 ]*[1-9] *([0-9] *){1,4}|[0 ]+"
        ;
EndOfLine
        : "(\r?\n|\r)+"
        ;
Comment : "[^\r\n]+"
        ;
```

Please substitute the content of your `basic.par` file with the lines given above.

Some details like the handling of new lines we will explain later (because we are in the 21st
century now we additionally allow empty lines 😉).

Just let us come quickly up and running.

Now we can build our new BASIC interpreter with

```shell
cargo build
```

And we write a little BASIC program that we should be able to parse now.

`test.bas`:

```basic
10 REM
20 REM Hello
30 REM World!
```

```shell
cargo run --release  -- ./test.bas
   Compiling basic v0.1.0 ...
Parsing took 1 milliseconds.
Success!
No parse result
```

Wow, very impressive! We can parse BASIC just out of the box only by defining some lines in the
`basic.par` grammar description. No code had to be written by us until now.

Here you can see one of the principles in `parol`. Grammar definition and grammar processing are kept
separate to be able to develop both sides independently of each other.

Also `parol` works by default as acceptor, i.e. if you don't do any language processing, we can
still evaluate the correctness of the grammar description.

We can build a little error in our `test.bas` to test the error detection:

```basic
10 REM
20 
30 REM World!
```

```shell
cargo run  -- .\test.bas 
```

You should see errors reported by `parol` now.

```shell
Error: parol_runtime::parser::syntax_error

  × Failed parsing file ./test.bas
  ╰─▶ Found "'\r\n'(EndOfLine) at ./test.bas:2:4"
      Current scanner is INITIAL
      Current production is:
      /* 12 */ Remark: "REM" RemarkSuffix;
      Expecting one of REM
  help: Syntax error in input prevents prediction of next production

Error: parol_runtime::unexpected_token

  × Unexpected token: LA(1) (EndOfLine)
   ╭─[./test.bas:1:1]
 1 │ 10 REM
 2 │ 20
   ·    ─┬
   ·     ╰── Unexpected token
 3 │ 30 REM World!
   ╰────
  help: Unexpected token

error: process didn't exit successfully: `target\release\basic.exe ./test.bas` (exit code: 1)
```

Wow, great!

With this in mind we can from now on easily develop our BASIC grammar. If the grammar works
sufficient for us we can then step over to the next stage, the actual grammar processing. In our
case the grammar processing encompasses the interpreter's functionality.

### Minor details explained

But before we go on we should have a look at some details I previously didn't explain thoroughly.

In contrast to the initial grammar proposal I gave first, I changed a detail in the grammar
description.

First I changed the `Basic` so that it must contain at least one line. An empty basic program is no
valid program now.

Then I moved the `EndOfLine` symbol from the `Line` production to the `Basic` production.
This decision I made to have the new line handling in one single place. A new line in the wrong
place of the grammar can screw it up literally.

Such decisions are not easy to explain and everybody has his own preferences in writing grammars.
Additionally these preferences can change over time when one has written more grammars.

So it's up to you to gain a lot of experiences in writing grammars 😉.

Next I want to explain the token literals I presented above without any comment.

>One general word to terminal symbols in `parol`. All terminal symbols (entities in productions
that are enclosed by a pair of double quotes) are treated as regular expressions. There is no
exception from this rule. This means that if you want to build in a character that is a meta symbol
of the regular expression language you have to escape it properly. On the other hand you can fully
benefit from the rich possibilities that Rust `regex` provides.

I explain now the more complex terminals, i.e. regular expressions.

```ebnf
LineNumber: "[0 ]*[1-9] *([0-9] *){1,4}|[0 ]+";
```

Line Numbers in the C64 BASIC can encompass numbers from 0 to 63999. This means that it can consist
of one up to five digits. Also trailing zeros are allowed by the C64 so you could possibly get valid
line numbers with more then 5 digits (for instance `00012000`). Also, to be close to the original we
support spaces in between the digits. The regular expression here boils down to this: **Match any
number of trailing zeros followed by one non-zero digit ant then match up to four digits from `0` to
`9` and accept spaces in between OR match one or more zeros with spaces in between**.

```ebnf
EndOfLine: "(\r?\n|\r)+";
```

This means at least one of (the `+` at the end) an optional carriage return character followed by a
new line character or single carriage return character. They are all necessary because different
line ending stiles exist on different platforms.

```ebnf
Comment : "[^\r\n]+";
```

This regular expression defines valid characters within a comment. The `+` at the end again means
**at least one of the items before** which are defined by a character set (embedded in brackets).
The circumflex as first symbol makes the character set a negated one meaning it matches all
characters except the listed ones. So when we put it all together it means **Match at least one
character that is neither a carriage return nor a new line character**.

With this definition this regex will match all characters until the end of the line. Then the
`EndOfLine` symbol will be matched as demanded by the grammar definition.

We have to match at least one character (implied by the `+` at the end). This is a general rule:

> Terminals should always match non-empty text portions. This means that you have to avoid terminals
like this:
>
>```regex
>"a?", "a*", "\b"
>```
>
>Internally the tokenizer will enter a loop and match the empty string over and over again without
making progress in the input. Currently there is no check for this scenario in `parol_runtime`.

To support empty comments after `REM` I made it optional:

```ebnf
Remark  : "REM" [Comment]
        ;
```

With these details out of the way we can continue safely.

## More statements

Next we will extend the set of statements we want to support.
We again take the easiest one

### The infamous `GOTO`

We extend the Statement rules this way. Note that the pipe symbol `|` separates alternative rules.
> In `parol` all alternatives of a single non-terminal have the same priority, regardless of their
order! Their selection is solely made by looking at k lookahead tokens in the input.

```ebnf
Statement
        : Remark
        | GotoStatement
        ;
GotoStatement
        : "GOTO" LineNumber
        ;
```

Well, easy. Lets build and test it:

```basic
10 REM Hello World!
20 GOTO 30
30 REM The End
```

```shell
cargo run  -- .\test.bas
...
Parsing took 3 milliseconds.
Success!
No parse result
```

Great!

Before we can go on with more statements we now have to implement the expressions first, simply
because we need expressions in the remaining statements.

## Expressions

Expressions are mostly calculations in our case. They obtain their operands either form fixed values
(i.e. literals like 1.0E-6) or from variables like `A`.

So the first part is to define literals and variables as syntactic items. Please insert these lines
behind the production of `LineNumber`.

```ebnf
Literal : Number
        ;
Number  : Float
        | Integer
        ;
Float   : Float1
        | Float2
        ;
// [Integer] DecimalDot [Integer] [Exponent]
Float1  : "(([0-9] *)+)?\. *(([0-9] *)+)? *(E *[-+]? *([0-9] *)+)?"
        ;
// Integer Exponent
Float2  : "([0-9] *)+E *[-+]? *([0-9] *)+"
        ;
Integer : "([0-9] *)+"
        ;
```

We introduce a category for literals named `Literal` to be able to easily expand with other literals
like string literals. But for now there is only one kind, the `Number`. The underlying regex's  for
integer and float literals are a bit quirky because the C64 accepts spaces anywhere within a numeric
literal. Later we have to post-process the matched tokens to be parsable by Rust.

To be able to test this regex we also introduce the assign statement `Assignment`:

```ebnf
Statement
        : "REM" [Comment]
        | GotoStatement
        | Assignment
        ;
Assignment
        : ["LET"] Variable AssignOp Literal
        ;
```

Further we define the missing assign operator and the variable names.

```ebnf
AssignOp:
        "="
        ;
Variable:
        "[A-Z][0-9A-Z]*"
        ;
```

The whole `basic.par` should now look like this:

```text
%start Basic
%title "Basic grammar"
%comment "Subset of C64 Basic used in tutorial for parser generator `parol`"
%auto_newline_off

%%

Basic   : [EndOfLine] Line { EndOfLine Line } [EndOfLine]
        ;
Line    : LineNumber Statement { ":" Statement }
        ;
Statement
        : Remark
        | GotoStatement
        | Assignment
        ;
Assignment
        : ["LET"] Variable AssignOp Literal
        ;
Remark  : "REM" [Comment]
        ;
GotoStatement
        : "GOTO" LineNumber
        ;
Literal : Number
        ;
LineNumber
        : "[0 ]*[1-9] *([0-9] *){1,4}|[0 ]+"
        ;
Number  : Float
        | Integer
        ;
Float   : Float1
        | Float2
        ;
// [Integer] DecimalDot [Integer] [Exponent]
Float1  : "(([0-9] *)+)?\. *(([0-9] *)+)? *(E *[-+]? *([0-9] *)+)?"
        ;
// Integer Exponent
Float2  : "([0-9] *)+E *[-+]? *([0-9] *)+"
        ;
Integer : "([0-9] *)+"
        ;
EndOfLine
        : "(\r?\n|\r)+"
        ;
AssignOp:
        "="
        ;
Variable:
        "[A-Z][0-9A-Z]*"
        ;
Comment : "[^\r\n]+"
        ;
```

Let's test the literal parsing now.

```basic
10 LET A = 1 2 3
```

```shell
cargo run  -- .\test.bas
...

Error: parol_runtime::parser::syntax_error

  × Failed parsing file ./test.bas
  ├─▶ Production prediction failed at state 0
  ╰─▶ LA(1): '1 2 3'(LineNumber) at ./test.bas:1:12.
      at non-terminal "Number"
      Current scanner is INITIAL
      Current production is:
      /* 20 */ Literal: Number;
      Expecting one of "Float1", "Float2", "Integer"
  help: Syntax error in input prevents prediction of next production

Error: parol_runtime::unexpected_token

  × Unexpected token: LA(1) (LineNumber)
   ╭─[./test.bas:1:1]
 1 │ 10 LET A = 1 2 3
   ·            ──┬──
   ·              ╰── Unexpected token
   ╰────
  help: Unexpected token

error: process didn't exit successfully: `target\debug\basic.exe ./test.bas` (exit code: 1)
```

Oh, an error! What went wrong? We can actually see here what the problem is. The `Number` token
isn't recognized as expected. Instead the terminal type `LineNumber` is associated with the text
portion '1 2 3'. We have conflicting terminals (line number and integer) that are mixed up. How we
can solve this?

>**Logging**
>
>To be able to debug the entrails of `parol` we can switch on logging, a method supported by `parol`
intrinsically.
>
>We have to activate logging by setting the `RUST_LOG` environment variable. The following command
line will activate the logging for the scanner/lexer module of `parol_runtime`.
>In my tutorial I use Powershell, but it should be easy to transfer it to your shell's syntax:
>
>```powershell
>$env:RUST_LOG="parol_runtime::lexer::token_iter=trace"  
>```
>
>```shell
>cargo run -- .\test.bas
>...
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] '10 ', Ty:9, Loc:1,1-4, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] 'LET', Ty:6, Loc:1,4-7, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] ' ', Ty:2, Loc:1,7-8, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] 'A', Ty:15, Loc:1,8-9, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] ' ', Ty:2, Loc:1,9-10, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] '=', Ty:14, Loc:1,10-11, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] ' ', Ty:2, Loc:1,11-12, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] '1 2 3', Ty:9, Loc:1,12-17, newline count: 0
>[2022-05-07T10:27:42Z TRACE parol_runtime::lexer::token_iter] '
>    ', Ty:13, Loc:1,17-19, newline count: 1
>...
>```
>
>With a little practice we can see here that the line number `10` has terminal type 9 (Ty:9). It can
>be looked up in the generated `basic_parser.rs`:
>
>```rust
>pub const TERMINAL_NAMES: &[&str; 18] = &[
>        ...
>    /*  9 */ "LineNumber",
>        ...
>];
>```
>
>Use logging in any case you get stuck. It has proven to be very useful in different scenarios. 

### Terminal conflicts

We have here our first terminal conflict. This is actually quite common so `parol` provides several
ways to handle such conflicts.

1. Avoid conflicts by reusing the terminal

    This means that we actually should use the same terminal for both line number and integer.
    Actually this doesn't help us here because we need different terminals due to different scan
    requirements. Although I must admit this is deliberately exaggerated for the sake of this
    tutorial.

2. The order of appearance rule

    This rule states that terminals that appear earlier in the grammar description match with higher
    priority.

    >Note that this is different from the priority of alternatives of a non-terminal.
    Their priorities are independent from their order.

    But this will also not help us here because the result would be the other way round: if we want
    to match a `LineNumber` `parol`'s scanner will match an `Integer`.

3. Scanner states

    The third one is the most versatile solution. We can put conflicting terminals in different
    groups that are called scanner states. And we then switch the current scanner state in our
    grammar.

So we have to introduce a special scanner state `Expr` (for expression) here. Please, add this line
right before the `%%` mark in `basic.par`.

```ebnf
%scanner Expr { %auto_newline_off }
```

This introduces a new scanner state or terminal group named `Expr` to which we can associate our
numeric literals:

```ebnf
// [Integer] DecimalDot [Integer] [Exponent]
Float1  : <Expr>"(([0-9] *)+)?\. *(([0-9] *)+)? *(E *[-+]? *([0-9] *)+)?"
        ;
// Integer Exponent
Float2  : <Expr>"([0-9] *)+E *[-+]? *([0-9] *)+"
        ;
Integer : <Expr>"([0-9] *)+"
        ;
```

Also we need a new scanner state for the comment terminal because this would otherwise match pretty
much everything. Add this line after the scanner state `Expr` and before the `%%` mark.

```ebnf
%scanner Cmnt { %auto_newline_off }
```

Then we attach the terminal to this scanner state:

```ebnf
Comment : <Cmnt>"[^\r\n]+"
        ;
```

And do the state switching in the production for remarks:

```ebnf
Remark  : "REM" %push(Cmnt) [Comment] %pop()
        ;
```

And we add the state switch to `Expr` in the production for assignment:

```ebnf
Assignment
        : ["LET"] Variable AssignOp %push(Expr) Literal %pop()
        ;
```

Viola! Scanner state switching in our grammar description!

In contrast to other parser generators that switch their scanner states in the semantic actions (one
prominent example is the lex/yacc pair of scanners and parsers) again `parol` advocates the
principle of strict separation of grammar description and grammar processing via semantic actions.

This means, you can write your grammar until it works. Than you start with the sematic actions, i.e.
the actual language processing. No intermingling development is necessary.

Back to our grammar and the application of scanner states. We have to change the belonging to
scanner states also for the `EndOfLine` terminal:

```ebnf
EndOfLine
        : <INITIAL, Expr>"(\r?\n|\r)+"
        ;
```

It should belong to both the `INITIAL` (the default scanner state) and the `Expr` scanner state.
Can you figure out, why?

The `EndOfLine` is used at the beginning of production `Basic` in the `INITIAL` scanner state. Then
later in this production it can be acquired in state `Expr` as a lookahead token. Therefore we need
to add it to both states.

>You may need to associate terminals with multiple scanner states because the provision of lookahead
tokens will be made with the current active scanner and may fail if a token is not known by it. Thus
some terminals need to belong to the state from where the switch comes from. These are typically
special terminals that trigger the scanner switch.
>
>I admit, that this is hard to understand because it has to do with the way scanner states and the
acquisition of lookahead tokens work in `parol_runtime`.
>
>Currently we don't need to understand that in depth.

And still another clarification:

>**Terminals vs. tokens**
>
> Terminals are syntactical entities that describe your grammar. You define them in the grammar
description file usually by means of regular expressions.
>
> Tokens are scanned portions of text that belong to a certain terminal type. They emerge during the
process of parsing.

Now we hopefully understand the nature of terminal conflicts and the ways to handle them. Let's
continue with the definition of expressions.

Our complete grammar description should now look like this:

```text
%start Basic
%title "Basic grammar"
%comment "Subset of C64 Basic used in tutorial for parser generator `parol`"
%auto_newline_off

%scanner Cmnt { %auto_newline_off }
%scanner Expr { %auto_newline_off }

%%

Basic   : [EndOfLine] Line { EndOfLine Line } [EndOfLine]
        ;
Line    : LineNumber Statement { ":" Statement }
        ;
Statement
        : Remark
        | GotoStatement
        | Assignment
        ;
Remark  : "REM" %push(Cmnt) [Comment] %pop()
        ;
LineNumber
        : "[0 ]*[1-9] *([0-9] *){1,4}|[0 ]+"
        ;
GotoStatement
        : "GOTO" LineNumber
        ;
Assignment
        : ["LET"] Variable AssignOp %push(Expr) Literal %pop()
        ;
EndOfLine
        : <INITIAL, Expr>"(\r?\n|\r)+"
        ;
Literal : Number
        ;
Number  : Float
        | Integer
        ;
Float   : Float1
        | Float2
        ;
// [Integer] DecimalDot [Integer] [Exponent]
Float1  : <Expr>"(([0-9] *)+)?\. *(([0-9] *)+)? *(E *[-+]? *([0-9] *)+)?"
        ;
// Integer Exponent
Float2  : <Expr>"([0-9] *)+E *[-+]? *([0-9] *)+"
        ;
Integer : <Expr>"([0-9] *)+"
        ;

// -------------------------------------------------------------------------------------------------
// OPERATOR SYMBOLS
AssignOp
        : "="
        ;

// -------------------------------------------------------------------------------------------------
// COMMENT
Comment : <Cmnt>"[^\r\n]+"
        ;

// -------------------------------------------------------------------------------------------------
// VARIABLE
Variable: "[A-Z][0-9A-Z]*"
        ;
```

### Variables

We silently introduces variables in the previous paragraph. Here we will explain them in more detail.
A variable's name can only start with an alphabetic character (A to Z). All following characters can
be alphanumeric if any exist. Actually only the first two characters contribute to the name of a
variable, i.e. all additional ones are ignored.

We only want to support floating point variables in out interpreter, so we don't support suffixes
like `%` or `$` for integer and string variables.

So we define the terminal for variable names as already seen above:

```ebnf
Variable:
        "[A-Z][0-9A-Z]*"
        ;
```

Let's test the variable parsing together with literals now.

```basic
10 A1 = 1 2 3
20 BB = . 1 E 6
30 LET CCC = 1 22 E -1 2
```

```shell
cargo run  -- .\test.bas
```

### Add parse tree visualization

To see if our grammar works right before we start implementing the actual language processing we can
use the parse tree visualization of `parol`. Therefore we need to add an additional dependency.

```shell
cargo add id_tree_layout
```

Then we add to `main.rs` the following use instructions:

```rust
use id_tree::Tree;
use id_tree_layout::Layouter;
use parol_runtime::parser::ParseTreeType;
```

and at the end we add this function:

```rust
fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .into_diagnostic()
        .wrap_err("Failed writing layout")
}
```

Then we need to assign the returned parse tree to a value:

```rust
let syntax_tree = parse(&input, &file_name, &mut basic_grammar)
```

And call the parse tree generation in case of success:

```rust
        println!("Success!\n{}", basic_grammar);
        generate_tree_layout(&syntax_tree, &file_name)
```

Delete the `Ok(())` in the success branch.

Finally `main.rs` should look like this:

```rust
#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod basic_grammar;
// The output is version controlled
mod basic_grammar_trait;
mod basic_parser;

use crate::basic_grammar::BasicGrammar;
use crate::basic_parser::parse;
use id_tree::Tree;
use id_tree_layout::Layouter;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use parol_runtime::parser::ParseTreeType;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./basic.par -e ./basic-exp.par -p ./src/basic_parser.rs -a ./src/basic_grammar_trait.rs -t BasicGrammar -m basic_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut basic_grammar = BasicGrammar::new();
        let now = Instant::now();
        let syntax_tree = parse(&input, &file_name, &mut basic_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", basic_grammar);
            generate_tree_layout(&syntax_tree, &file_name)
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}

fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .into_diagnostic()
        .wrap_err("Failed writing layout")
}
```

When we now parse our basic file we will find a parse tree beside the `test.bas` as `test.svg`.
You can open this file in a browser and see if the structure of the tree fits the needs.

It is helpful to minimize the parse tree by only parsing the parts that are interesting for us at
the moment.

```basic
10 A1 = 1 2 3
```

In this tree we should be able to identify the assignment as sub-category of the statement. And
under the assignment you should find the variable and the other parts of it.

> Eventually, when your grammar is flawless you can remove the parse tree visualization and even
suppress the generation of a parse tree by using the `trim_parse_tree` feature of the `parol_runtime`
create. Doing so will increase the performance of the parsing process.
>
>```toml
>[dependencies]
>parol_runtime = { version = "0.5.9", features = ["trim_parse_tree"] }
>```

## Complete Basic grammar

To speed up our tutorial I will present here the final grammar description. It is recommended that
you try to understand it thoroughly.

```text
%start Basic
%title "Basic grammar"
%comment "Subset of C64 Basic used in tutorial for parser generator `parol`"
%auto_newline_off

%scanner Cmnt { %auto_newline_off }
%scanner Expr { %auto_newline_off }

%%

Basic   : [EndOfLine] Line { EndOfLine Line } [EndOfLine]
        ;
Line    : LineNumber Statement { <INITIAL, Expr>":" Statement }
        ;
LineNumber
        : "[0 ]*[1-9] *(?:[0-9] *){1,4}|[0 ]+"
        ;
Statement
        : Remark
        | GotoStatement
        | IfStatement
        | Assignment
        | PrintStatement
        | EndStatement
        ;
Remark  : "REM" %push(Cmnt) [Comment] %pop()
        ;
GotoStatement
        : Goto LineNumber
        ;
IfStatement
        : If %push(Expr) Expression %pop() IfBody
        ;
Assignment
        : [Let] Variable AssignOp %push(Expr) Expression %pop()
        ;
IfBody  : Then Statement
        | Goto LineNumber
        ;
PrintStatement
        : Print %push(Expr) Expression  {<INITIAL, Expr>"," Expression } %pop()
        ;
EndStatement
        : End
        ;
EndOfLine
        : <INITIAL, Expr>"(?:\r?\n|\r)+"
        ;
Literal : Number
        ;
Number  : Float
        | Integer
        ;
Float   : Float1
        | Float2
        ;
// [Integer] DecimalDot [Integer] [Exponent]
Float1  : <Expr>"(?:(?:[0-9] *)+)?\. *(?:(?:[0-9] *)+)? *(?:E *[-+]? *(?:[0-9] *)+)?"
        ;
// Integer Exponent
Float2  : <Expr>"(?:[0-9] *)+E *[-+]? *(?:[0-9] *)+"
        ;
Integer : <Expr>"(?:[0-9] *)+"
        ;

// -------------------------------------------------------------------------------------------------
// KEYWORDS
If      : "IF"
        ;
Then    : <INITIAL, Expr>"THEN"
        ;
Goto    : <INITIAL, Expr>"GOTO"
        ;
Let     : "LET"
        ;
Print   : "PRINT|\?"
        ;
End     : "END"
        ;

// -------------------------------------------------------------------------------------------------
// OPERATOR SYMBOLS
AssignOp
        : "="
        ;
LogicalOrOp
        : <Expr>"N?OR"
        ;
LogicalAndOp
        : <Expr>"AND"
        ;
LogicalNotOp
        : <Expr>"NOT"
        ;
RelationalOp
        : <Expr>"<\s*>|<\s*=|<|>\s*=|>|="
        ;
Plus    : <Expr>"\+"
        ;
Minus   : <Expr>"-"
        ;
MulOp   : <Expr>"\*|/"
        ;

// -------------------------------------------------------------------------------------------------
// PARENTHESIS
LParen  : <Expr>"\("
        ;
RParen  : <Expr>"\)"
        ;

// -------------------------------------------------------------------------------------------------
// COMMENT
Comment : <Cmnt>"[^\r\n]+"
        ;

// -------------------------------------------------------------------------------------------------
// VARIABLE
Variable: <INITIAL, Expr>"[A-Z][0-9A-Z]*"
        ;

// -------------------------------------------------------------------------------------------------
// EXPRESSIONS

Expression
        : LogicalOr
        ;
LogicalOr
        : LogicalAnd { LogicalOrOp LogicalAnd }
        ;
LogicalAnd
        : LogicalNot { LogicalAndOp LogicalNot }
        ;
LogicalNot
        : [LogicalNotOp] Relational
        ;
Relational
        : Summation { RelationalOp Summation }
        ;
Summation
        : Multiplication { (Plus | Minus) Multiplication }
        ;
Multiplication
        : Factor { MulOp Factor }
        ;
Factor  : Literal
        | Variable
        | Minus Factor
        | LParen Expression RParen
        ;
```

Operator precedence is realized by sub-categorizing higher prioritized elements. By this approach
you force the parser to branch into those first which leads to earlier evaluation in the end.

Please, try to comprehend this by looking at the parse tree of this program:

```basic
10 A1 = 1 + 2 * -3
```

You should now test the grammar with own basic programs to verify the grammar we wrote until now.
Maybe something like this:

```basic
0010IFAF=AF THENIFAF GOTO10
```

This line is accepted by the C64. By the way I used the [VICE emulator](https://vice-emu.sourceforge.io/>)
to evaluate some parsing properties but not to check for complete compatibility.

## Implementing the interpreter

We were able to postpone the language implementation until now. I appraise this as one of the most
unique and useful properties of `parol`. One can actually do rapid prototyping of a language!

Anyhow, eventually we need to do some grammar processing to have more than an acceptor for a
language.

### Where our implementation starts

All the traits and types `parol` generated for us and which we need further on can be found in the
`src/basic_grammar_traits.rs`.

First we start at the beginning of this file and find the trait `BasicGrammarTrait`. It contains at
the top an `init` function. This function is called by the parser before parsing starts and it
conveys the file name of the input to us. We typically use this for error messages.
We implement this function for our `BasicGrammar` item, more precisely in out implementation of the
`BasicGrammarTrait` for it. Copy it into the

```rust
impl<'t> BasicGrammarTrait<'t> for BasicGrammar<'t> {}
```

block at the end of the `src/basic_grammar.rs`. We will capture the file name the init function is
called with in a member of `BasicGrammar`. Please, add the member `file_name` now:

```rust
#[derive(Debug, Default)]
pub struct BasicGrammar<'t> {
    // ...
    file_name: PathBuf,
    // ...
    phantom: PhantomData<&'t str>, // Just to hold the lifetime generated by parol
}
```

Then we change the init function like this:

```rust
    fn init(&mut self, file_name: &Path) {
        self.file_name = file_name.into();
    }
```

In this impl block we will add our Basic interpreter functionality.

No please switch back to the file `src/basic_grammar_traits.rs`. Further on in the
`BasicGrammarTrait` and after the `init` function follow functions for each non-terminal of our
language. All these functions have default implementations to enable us to skip them in our
implementation.

The parser or better a special adapter layer will call them any time a non-terminal was parsed
completely.

This means we can chose those non-terminals we are interested in to build appropriate actions on
them.
Because we are lazy we chose only one non-terminal, the start symbol, for our implementation. Is
this sufficient? Yes, because the function for the start non-terminal is called, like any
non-terminal function, when the non-terminal is completely parsed. The start symbol is completely
parsed exactly then when the complete input is parsed. So effectively we implement a single semantic
action on the completely parsed input.

The start symbol of our Basic grammar is the symbol `Basic`. See `basic.par` for this detail.

So please copy the default implementation of the `fn basic` our of the `src/basic_grammar_traits.rs`
into the `impl<'t> BasicGrammarTrait<'t> for BasicGrammar<'t>` block right after the `init` function:

```rust
impl<'t> BasicGrammarTrait<'t> for BasicGrammar<'t> {
    fn init(&mut self, file_name: &Path) {
        self.file_name = file_name.into();
    }

    /// Semantic action for non-terminal 'Basic'
    fn basic(&mut self, basic: &Basic<'t>) -> Result<()> {
        Ok(())
    }
}
```

### Understand the data structures generated for our grammar

As I mentioned above the semantic action `basic` is called at the end of the parse process and its
argument `basic` contains the complete basic program in some kind of data structure.

The interesting thing is how this structure is formed. And indeed `parol` has analyzed our Basic
grammar and created all data structures accordingly.
So all `parol` needs is a grammar description to be able to create fitting data types for it.

Let's have a look at the type `Basic<'t>`. You can find it in the file `src/basic_grammar_traits.rs`.

```rust
///
/// Type derived for non-terminal Basic
///
pub enum Basic<'t> {
    Basic0(Basic0<'t>),
    Basic1(Basic1<'t>),
}
```

It is an enum with two variants. Why two variants? To understand this let's look at the inner types
of both variants:

```rust
///
/// Type derived for production 0
///
/// Basic: Line BasicList /* Vec */ BasicSuffix0;
///
pub struct Basic0<'t> {
    pub line: Box<Line<'t>>,
    pub basic_list: Vec<BasicList<'t>>,
    pub basic_suffix0: Box<BasicSuffix0<'t>>,
}

///
/// Type derived for production 1
///
/// Basic: EndOfLine Line BasicList /* Vec */ BasicSuffix;
///
pub struct Basic1<'t> {
    pub end_of_line: Box<EndOfLine<'t>>,
    pub line: Box<Line<'t>>,
    pub basic_list: Vec<BasicList<'t>>,
    pub basic_suffix: Box<BasicSuffix<'t>>,
}
```

Also we need to look at the expanded grammar `basic-exp.par` the first time. Especially at the
productions 0 and 1.

```ebnf
Basic: Line BasicList /* Vec */ BasicSuffix0;
Basic: EndOfLine Line BasicList /* Vec */ BasicSuffix;
```

The reason why we have two enum variants here is that the grammar transformation of our original
production

```ebnf
Basic   : [EndOfLine] Line { EndOfLine Line } [EndOfLine];
```

has created two alternations for then non-terminal `Basic`.

Both structures `Basic0` and `Basic1` can be regarded as variations of our original production and
they only differ in the presence or absence of the `EndOfLine` non-terminal at the beginning. The
rest is structural equivalent.

>One thing is noticeable in the structures `Basic0` and `Basic1`. Members that constitute other
non-terminals are wrapped in `Box`es. This is necessary due to the recursive nature of each grammar.
If we used members that are non-terminals directly in our structures we'll soon get error messages
from the Rust compiler complaining about infinitive type sizes. If you see `Box`es in generated
structures you can easily recognize them as data types which are generated for non-terminals.
>
>The same holds for `Vec`s. But they have another additional semantic and, you guess it, they
represent repetitions or collections of other grammar items.

*To be continued.*
