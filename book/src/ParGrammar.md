# The syntax of `parol`'s Grammar description

I provide the definition of the PAR grammar in PAR grammar [itself](https://github.com/jsinger67/parol/blob/main/crates/parol/src/parser/parol.par).

This grammar is quite concise and most programmers should be familiar with it. But there are several
specialties which will be described here. First please notice the built-in support for language
comments.

Using the `%line_comment` and `%block_comment` constructs you can easily define your language's
comments. For example you can define comments like it's done in the calc example
`calc.par`:

```parol
%line_comment "//"
%block_comment  "/\*" "\*/"
```

You can supply more than one of these two comment declarations. They will all be considered as valid
comments.

As opposed to EBNF you use C-like line comments starting with two slashes (//) and bock comments
(/\* ... \*/) in PAR files. This is a result of the close relationship between PAR grammar and
bison's grammar.

>As of version 0.22.0 `parol` doesn't simply discard language comments. They are provided during
parse process via a new method `<UserType>GrammarTrait::on_comment_parsed` which is called for each
single comment in order of their appearance each time before the parser consumes a normal token from
token stream.
>
> The method is default implemented and the user have to provide an own implementation if she is
interested in language comments.
>
>This is a minimal support but can greatly improve the usability. Also note that this comment
handling is currently only supported in `parols`'s auto-generation mode.
>
>Any feedback is appreciated.

## Defining the grammar type

In the global header section you can define the grammar type you want to use in your grammar
description.

The default grammar type is LL(k) and can be omitted.

```parol
%grammar_type 'LL(k)'
```

You have to option to use LALR(1) grammar type this way.

```parol
%grammar_type 'LALR(1)'
```

The support of the new grammar type is still in a phase of improvement. If there are any obstacles
here, you can be sure that they will be soon got out of the way.

## Case sensitivity

Non-terminals are treated case sensitive, i. e. "list" and "List" are different symbols. But it is
not encouraged to rely on this in your grammar definition. It is much better to keep a consistent
style on casing in your description.

## Sections

`parols`'s input language consists of two sections divided by the `%%` token. Above there are
declarations of which only the first `%start` declaration is mandatory. It declares the start symbol
of your grammar.
The second section below the `%%` token contains the actual grammar description in form of several
productions. At least one production must exist.

## The start symbol

It is important to note that the start symbol of the grammar must always be declared with the
`%start` declaration. It is the very first declaration in the PAR file.

```parol
%start Grammar
```

## Scanner control

<!-- markdownlint-disable no-inline-html -->
A scanner (aka lexer) is automatically created from all used terminal symbols. Terminal symbols can
also be associated with different scanner states. See section
<a href="#scanner-states">Scanner states</a> below for more details.
<!-- markdownlint-enable no-inline-html -->

### Newline handling

The scanner per default skips newlines automatically. To suppress this use the `%auto_newline_off`
directive.
With this you have to handle newline tokens on your own in your grammar.

### Whitespace handling

The scanner also per default skips whitespace automatically. To suppress this use the `%auto_ws_off`
directive.
With this you have to handle whitespace tokens on your own in your grammar.

### Terminal name generation

The names of the terminals are deduced from the content of the terminal itself. For instance, for a
terminal ":=" it creates the terminal name "ColonEqu", see generated parser for Oberon-0. If you
want this name to be more expressive, you can dedicate a separate production to the terminal, lets
say:

```parol
Assign: ":=";
```

With this trick you define a so called "primary non-terminal for a terminal" (I coined it this way)
that instructs the name generation to name the terminal "Assign".

### Terminal representation

As of version 0.14.0 `parol` supports three different styles of terminal representations, all of
them being valid and allowed.

* The legacy syntax (`"..."`). These terminals are treated as if they were regular expressions.
* New single quoted string literals (`'..'`) are literal or raw strings. The user doesn't need to
escape any regex meta character. This is used when you don't want to deal with regexes and only use
plain text. E.g.: `BlockBegin: '{'`
* New regular expression strings (`/../`), behaves exactly like the old double quoted string but
better conveys the intent. E.g.: `Digits: /[\d]+/`

Internally `parol` creates scanners on the basis of the Rust regex crate and all terminals are
embedded in a regular expression eventually. You should be aware of this if you get strange errors
from regex generation and want to understand the problem.

Here is an example for a terminal in regular expression form:

```parol
AddOperator: /\+|-/;
```

### Terminal conflicts

* In case of conflicts between different terminals _the first seen will win_

The last point needs a more detailed explanation.
It's best to show an example for such a situation.
Say you have two terminals "-" and "--", _minus_ and _decrement_. The generated scanner is then
based on the following regular expression:

```regexp
/-|--/
```

The Rust regex will now match two times _minus_ when actually a _decrement_ operator should be
detected.
It behaves here differently than a classic scanner/lexer like Lex that obeys the _longest match_
strategy.

Fortunately there is a simple way to achieve what we want. We just need a resulting regular
expression with a different order:

```regexp
/--|-/
```

This will perfectly do the job.

To get such an order the _decrement_ terminal has to be defined __before__ the _minus_ terminal as
in the following snippet.

```parol
decrement: /--/
;
...
minus: /-/
;
```

Thats all.

With this simple but effective means you have the control over terminal conflicts.

### Terminals that matches an empty string

Please note that terminals should always match non-empty text portions. This means that you have to
avoid terminals like this:

```parol
/a?/, /a*/, /\b/
```

Internally the tokenizer will enter a loop and match the empty string over and over again without
making progress in the input. Currently there is no check for this scenario in `parol_runtime`.

There is a **workaround** when you simply need possibly empty tokens, at least for the `?` and `*`
ones.
Make the token `+` and put their uses in optional expressions `[]`. This makes them non-empty and
also their possible emptiness explicit for the grammar:

```parol
RuleWithWhiteSpaces: WhiteSpaces;
WhiteSpaces: /[ \t]*/;

// =>

RuleWithWhiteSpaces: [ WhiteSpaces ];
WhiteSpaces: /[ \t]+/;
```


<!-- markdownlint-disable no-inline-html -->
<h2 id=scanner-states>Scanner states</h2>
<!-- markdownlint-enable no-inline-html -->

`Parol` supports __multiple scanner states__. This feature is known from Flex as
[Start conditions](https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex_toc.html#TOC11)
and provides more flexibility in defining several scanners for several parts of your grammar.

`Parol` provides two different ways to control scanner states directly within your grammar
description thereby holding the principle of strict separation of grammar description and grammar
processing in semantic actions. This means no scanner switching in your code, but in the grammar
description. Only because of this rapid prototyping is possible.

### The Default scanner state INITIAL

INITIAL is the name of the default scanner state 0. Its behavior is defined with `ScannerDirectives`
in the global `Declaration` section, such as:

```parol
%line_comment "//"
%block_comment "/\*" "\*/"
```

### Introduce new scanner states with the %scanner directive

Use the `%scanner Name {...}` construct after the global `Declaration` section and before the `%%`
sign to introduce arbitrary scanner states. The identifier following the %scanner token defines the
name of the state which is used to refer to it from scanner state lists at terminals.

```parol
%scanner String {
    %auto_newline_off
    %auto_ws_off
}

%scanner Pragma {
    %block_comment "\{" "\}"
}
```

You can place any of the `ScannerDirectives` within the block that defines the scanner state.

By default each scanner handles (and skips) whitespace and newlines. Use `%auto_newline_off` and
`%auto_ws_off` to modify each scanner state appropriately.

Associate terminals with scanner states by prefixing them with a list of comma separated state names
in angle brackets. Like this:

```parol
StringDelimiter
    : <String, INITIAL>/"/
    ;
```

Scanner state references in different occurrences of the same terminal are accumulated. I.e.,

```parol
<State1>"term"
...
<State2>"term"
```

will result in

```parol
<State1, State2>"term"
```

Terminals without explicitly associated scanner state are implicitly associated with scanner state
INITIAL.

### Parser-bases scanner switching

The first way to control scanner states is to define switching directives within your productions.
This way can only be used for LL(k) grammars because the parser has full knowledge about which
production to handle next when certain input has been encountered from left to right.

Parser-bases scanner state switching is initiated within your productions like in the following two
examples:

```parol
String: StringDelimiter %sc(String) StringContent StringDelimiter %sc();

```

or

```parol
String: StringDelimiter %push(String) StringContent StringDelimiter %pop();

```

The `%sc` instruction is used to switch directly to the state named in the parentheses. The INITIAL
state can be omitted as seen in the second occurrence of the first example, i.e. `%sc()` and
`%sc(INITIAL)` are equivalent.

The `%push` instruction is used to push the index of the current scanner on the internal scanner
stack and to switch to a scanner configuration with the given index in parentheses.

The `%pop` instruction is used to pop the index of the scanner pushed before and to switch to the
scanner configuration with that index.

> Note that `%push` and `%pop` instructions should be balanced. This means that in one context use
**only one** of the combinations `%push(S1)`/`%pop` and `%sc(<S1>)`/`%sc(<S2>)`. `%push`/`%pop`
provides a (call) stack semantics over scanner states whereas `%sc`/`%sc` can be used to represent
scanner state graphs semantics. Mixing both semantics should be avoided or at should least be
carefully considered.

> Currently the scanner parser-based state switching only works if the lookahead
__at the point where the switch is made__ is only of size 1 because the lookahead mechanism is
directly influenced by the current scanner state. This means the provision of lookahead tokens will
be made with the current active scanner and may fail if a token is not known by it. In most cases
this can be circumvented by an appropriate grammar formulation. If this is not possible consider to
use `Scanner-bases scanner switching` instead.

You may have look at example `scanner_states` that demonstrates the handling of scanner states.

### Scanner-bases scanner switching

LR grammars reduce the parser stack from the right side and thus you can't decide the scanner state
switching from the perspective of the parser. The tokens are already read and pushed on the parse
stack before it can be decided what production to reduce on them. This means scanner state switching
must work different here.
When incorporating the scanner state switching into the scanner itself the state can be chosen as
early as possible solely from the current state the scanner is in and the token read next.
The good new is that this kind of scanner switching works for LL parsers too and most LL(k) grammars
can be adopted to use scanner-based scanner switching.

Scanner-based scanner switching is defined solely in the header of the grammar file right where the
scanners are defined.

You use the `%on` and `%enter` directives to control it (snippets taken from the `basic` example):

```parol
%on Rem %enter Cmnt
%on If, AssignOp, Print %enter Expr

%scanner Cmnt {
    %auto_newline_off
    %on EndOfLine %enter INITIAL
}
%scanner Expr {
    %auto_newline_off
    %on Then, Goto, EndOfLine %enter INITIAL
}
```

After the `%on` directive you can name a list of primary non-terminals which only contain the
terminal like this:

```parol
Rem : 'REM'^;
```

After the `%enter` directive you name the target scanner state.

You also may have look at examples `scanner_states_lr` for a simple demonstration and at example
`basic` for a more advanced one.

>_Be aware that mixing of both parser-bases and scanner-based scanner state switching in one grammar
file is not allowed and will result in errors._


## Omitting grammar symbols from the AST in auto-gen modus

You can suffix grammar symbols (terminals and non-terminals) with a cut operator (^). This instructs
`parol` to not propagate them to the AST in auto-gen modus.

```parol
Group: '('^ Alternations ')'^;
```

The AST type for the symbol `Group` will then only contain a member for the non-terminal
`Alternations`. The parentheses are left out.

## Assigning user types to grammar symbols

You can specify a user type to be inserted into the AST structure at the place where the symbol
would otherwise had the originally generated type.
Add after a grammar symbol a colon followed by a user type name to instruct `parol` to use this type
instead. In your language implementation you have to provide fallible conversions from references of
the original generated types (`&T`) to your types (`U`) by implementing the trait
`TryFrom<&T> for U`.

An examples can be found in the `list_auto` example.

```rust
impl<'t> TryFrom<&Token<'t>> for Number {
    type Error = anyhow::Error;

    fn try_from(number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        Ok(Self(number.text().parse::<u32>()?))
    }
}
```

You can also define aliases for the user type names by inserting as many `%user_type` directives as
you want.

```parol
%user_type Number = crate::list_grammar::Number
```
Then use these aliases behind the colons.

```parol
Num: "0|[1-9][0-9]*": Number;
```

## Semantic actions

Semantic actions are strictly separated from your grammar description.
You will use a generated trait with default implementations for each production of your grammar. You
can implement this trait in your grammar processing item and provide concrete implementations for
those productions you are interested in.
