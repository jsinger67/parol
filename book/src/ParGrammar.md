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

`parol` doesn't simply discard language comments. They are provided during
parse process via a new method `<UserType>GrammarTrait::on_comment` which is called for each
single comment in order of their appearance each time before the parser consumes a normal token from
token stream.
The method is default implemented and the user have to provide an own implementation if she is
interested in language comments.

This is a minimal support but can greatly improve the usability.

## Defining the grammar type

In the global header section you can define the grammar type you want to use in your grammar
description.

The default grammar type is LL(k) and can be omitted.

```parol
%grammar_type 'LL(k)'
```

You have the option to use LALR(1) grammar type this way.

```parol
%grammar_type 'LALR(1)'
```

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

`parol` supports three different styles of terminal representations, all of them being valid and
allowed.

* The **string syntax** (`"..."`). These terminals are treated as if they were **regular expressions.**
* The **single quoted** string literals (`'..'`) are **literals or raw strings**. The user doesn't
need to escape any regex meta character. This is used when you don't want to deal with regexes and
only use plain text. E.g.: `BlockBegin: '{'`
* The **regular expression strings** (`/../`), behaves exactly like the double quoted string, i.e.
they are treated as **regular expressions** but this style better conveys the intent. E.g.:
`Digits: /[\d]+/;`

Internally `parol` creates scanners on the basis of the `scnr2` crate and all terminals are
expressed as regular expressions eventually. You should be aware of this if you get strange errors
from regex generation and want to understand the problem.

Here is an example for a terminal in regular expression form:

```parol
AddOperator: /\+|-/;
```

### Terminal conflicts

* Parol's scanner follows the longest match rule
* Conflicts can only occur, if the matched tokens have the same length and are accepted by more than
one terminal type. In case of such a conflict between different terminals, terminals defined earlier
in the grammar have higher priority than those defined later. This allows you to influence the
priority of tokens with equal length. In all other cases, tokens with the longest match are
preferred.

For example, if you have two terminals "-" and "--", _Minus_ and _Decr_, the scanner will match
based on the longest match basis:

```parol
Decr: /--/
    ;

Minus
    : /-/
    ;

```
An input string `-----` will match the decrement operator twice and then the minus operator once.

As an example for tokens with the same length consider following terminal definitions:

```parol
// ❌
Ident: /[a-zA-Z_][a-zA-Z0-9_]*/
    ;

If: 'if'
    ;
```

In case of same length, the scanner will match based on the order of definition:

On input `if` it will match the `Ident` first. To make this work you have to move the terminal `If`
before the more general `Ident`:

```parol
// ✅
If: 'if'
    ;

Ident: /[a-zA-Z_][a-zA-Z0-9_]*/
    ;
```

Defining _If_ before _Ident_ ensures the correct priority.

#### Conclusion

❗ These two mechanisms, **longest match rule** and **priority by order**, gives you control over
terminal conflicts.

### Terminals that matches an empty string

Please note that terminals should always match non-empty text portions. This means that you have to
avoid terminals like this:

```parol
/a?/, /a*/
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


### Scanner switching

Scanner-based scanner switching in Parol is managed by the scanner using the `%enter`, `%push`, and
`%pop` directives within the scanner specification:

- `%enter`: Switches the scanner to a specific mode, replacing the current mode.
- `%push`: Pushes the current mode onto a stack and enters a new mode.
- `%pop`: Returns to the previous mode by popping the mode stack.

These directives ensure that scanner mode switching is handled consistently and reliably, preventing
token buffer desynchronization in LL(k) grammars with k > 1. All scanner-related features are based
on the [`scnr2`](https://crates.io/crates/scnr2) crate.

Example usage:

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

After the `%on` directive, specify a list of primary non-terminals. After the `%enter` directive,
specify the target scanner state. `%push` and `%pop` provide stack-based mode management.

Mixing parser-based and scanner-based switching in one grammar file is not allowed and will result
in errors.

Parol generates all data required by `scnr2` to construct valid and efficient scanners. Users do not
need to understand the internal configuration of `scnr2`.

## Controlling the AST generation

### Omitting grammar symbols from the AST

You can suffix grammar symbols (terminals and non-terminals) with a cut operator (`^`). This
instructs `parol` to not propagate them to the AST.

```parol
Group: '('^ Alternations ')'^;
```

The AST type for the symbol `Group` will then only contain a member for the non-terminal
`Alternations`. The parentheses are left out.

### Assigning user types to grammar symbols

You can specify a user type to be inserted into the AST structure at the place where the symbol
would otherwise had the originally generated type.
Add after a grammar symbol a colon followed by a user type name to instruct `parol` to use this type
instead. In your language implementation you have to provide fallible conversions from references of
the original generated types (`&T`) to your types (`U`) by implementing the trait
`TryFrom<&T> for U`.

An examples can be found in the `list` example.

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

### Define user types for non-terminals

As of version 3.0 you can easily define a user type to which each occurrence of a certain
non-terminal should be automatically converted to.
This is done like in the following example:

```parol
%nt_type ScannerState = crate::parser::parol_grammar::ScannerConfig
```

It is similar to the already available `%user_type` with what you could define an alias for a
user defined type which in turn you could apply to single symbols on the right-hand side of
grammar productions. The `%nt_type` can't be used on terminals but it makes the application to
non-terminals much easier.
Here is the old version used in `parol` itself before (only partial)
```parol
%user_type ScannerConfig = crate::parser::parol_grammar::ScannerConfig
// ...
%%
// ...
Prolog
: StartDeclaration { Declaration } { ScannerState: ScannerConfig }
;
```
And here is the new variant in which `%nt_type` is used.
```parol
%nt_type ScannerState = crate::parser::parol_grammar::ScannerConfig
// ...
%%
// ...
Prolog
: StartDeclaration { Declaration } { ScannerState }
;
```
The non-terminal `ScannerState` was automatically defined the be converted to `ScannerConfig`.

It is semantically completely identical to use `%user_type` and the application of it to each
occurrence of the non-terminal in the grammar explicitly.

### User defined terminal type

As of version 3.0 you can easily define a user type to which each occurrence of a terminal should be
automatically converted to.
This is done like in the following example:

```parol
%t_type crate::parol_ls_grammar::OwnedToken
```

There can be only one type defined to which all terminals are converted to.

More specifically, if several such instructions are given, the last one wins.

Here is the old version used in `parol-ls` itself before (only partial)
```parol
%user_type OwnedToken = crate::parol_ls_grammar::OwnedToken
// ...
%%
// ...
ScannerSwitch
    : "%sc": OwnedToken '(': OwnedToken [ Identifier ] ')': OwnedToken
    | "%push": OwnedToken '(': OwnedToken Identifier ')': OwnedToken
    | "%pop": OwnedToken '(': OwnedToken ')': OwnedToken
    ;
```
And here is the new variant in which `%t_type` is used.
```parol
%t_type crate::parol_ls_grammar::OwnedToken
// ...
%%
// ...
ScannerSwitch
    : "%sc" '(' [ Identifier ] ')'
    | "%push" '(' Identifier ')'
    | "%pop" '(' ')'
    ;
```
All terminals are automatically defined the be converted to `crate::parol_ls_grammar::OwnedToken`.

### Define user defined member names

As of version 3.0 you can specify for each symbol on the right-hand side of a production how its
corresponding member in the generated struct should be named.

To achieve this you can use the newly introduced `@` operator.

```parol
Declaration :
    ...
    | "%nt_type" Identifier@nt_name "="^ UserTypeName@nt_type
    ...
```

In this example the member for Identifier in the production will be named `nt_name` and the member
for UserTypeName will receive the name `nt_type` in the generated struct type for this production.

## Semantic actions

Semantic actions are strictly separated from your grammar description.
You will use a generated trait with default implementations for each non-terminal of your grammar.
You can implement this trait in your grammar processing item and provide concrete implementations
for those non-terminals you are interested in.

In the chapter [Operator Precedence](./OperatorPrecedence.md) there are some examples on how to
implement simple semantic actions.

A separate chapter [Semantic Actions](./SemanticActions.md) deals more deeply with this topic.