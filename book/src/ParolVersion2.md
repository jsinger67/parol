# Changes in version 2

## New scanner/lexer crate

As of version 2.0 `parol` uses the brand new and self written scanner crate
[scrn](https://github.com/jsinger67/scnr).

### Motivation

The motivation for switching from `regex-automata` to `scnr` was the performance of the creation of
tokenizers out of regexes during the startup of generated parsers. For small input texts this share
of the parser runtime exceeded the share of actual parsing times by a factor of >30. This meant that
for the advantage of fast parsing you had to sacrifice a lot of costs for building this efficient
scanner.

This doesn't mean that `regex-automata` is in any way poor, on the contrary, it is a very versatile
and valuable crate for a multitude of uses cases. It only means that the use in a lexical scanner,
where you don't need the majority of features like named and unnamed capture groups is not the
optimal decision.

Also `scnr` is not perfect and can't compete with `regex-automata` regarding the supported regex
features. But being able and willing to forgo some comfort opens up the opportunity to gain a lot of
speed in the phase of tokenizer creation and even during the actual scanning.

I can give some figures to support the claims made:

First for the speed of building the scanner resp. the tokenizer:

| 118 terminals | Version 1 | Version 2 |
|-------------- |----------:|----------:|
|build scanner  | 33ms      | 1ms       |

In both cases the same conditions are applied. The regex comprised 118 terminals.

Then some measurements of parsing speed.

| Input (Bytes; Tokens) | Version 1 | Version 2 |
|---|--:|--:|
|549; 175 | 0.01ms + 33ms | 1.2ms + 1ms
|58525; 3111 | 0.76ms + 33ms | 2.34ms + 1 ms |
|5873100; 1159200 | 122ms + 33ms |  80ms + 1ms |

The added value in cells with times is the constant time to create the tokenizer, which is required
once for each call to the generated parser.

As you can see, `scnr` does its job well.

### Impact on lexical analysis

Since `scnr` doesn't support some regex features which are either complicated, costly or
unnecessary, some things can't be resolved simply by the means regex crates like `regex-automata`
provide.
One major feature worth mentioning is non-greediness. Lets dive in this topic with the help of an
example.

Let's use the commonly known block comment as our example.

In version 1 you would have simply used a non-greedy repetition to skip over all characters in
between the start of the comment and the end of the comment.

```regexp
/\*[.\r\n]*?\*/
```
or even using `s` flag, which allows `.` to match `\n` too:

```regexp
/\*(?s).*?\*/
```

In version 2 with `scnr` you can not use either of these two versions. `scnr` does not support
non-greediness and does also not support flags currently. In this regard it is in good company with
other lexer/scanner generators like Lex.

You can although simulate non-greediness by carefully creating your regex or by introducing new
dedicated scanner modes. Both variants are explained now.

#### Special regex design

You can consider the following solution

```regexp
/\*([^*]|\*[^/])*\*/
```

In this working example you explicitly restrict the portion that has to be repeated non-greedily by
suppressing the acceptance of the following regex (the end of comment part) within the repetition.

The above solution can be phrased like this:
>Match all characters except `*` OR a `*` NOT followed by the character `/`, where `*` is the start
of the end comment part and `/` is the next character after the start of the end comment part.

I know that this is cumbersome and maybe sometimes not even feasible when the following part is too
complex. Therefore there exists a second approach to cope with missing non-greediness.

#### Scanner modes

You can make the repetition also non-greedy by creating a second scanner mode, here named `COMMENT`.

This mode is entered on the **comment start** `/\*`, then handles all tokens inside a comment and
enters `INITIAL` mode on the **comment end** `\*/` again.

```parol
%start NgBlockComment
%comment "Non-greedy block comment"

%on CommentStart %enter COMMENT

%scanner COMMENT {
    %auto_newline_off
    %auto_ws_off
    %on CommentEnd %enter INITIAL
}

%%

NgBlockComment: Comments;
Comments: { Comment };
Comment: CommentStart { CommentContent } CommentEnd;
CommentStart: '/*';                 // Valid in mode INITIAL
CommentEnd: <COMMENT>'*/';          // Valid in mode COMMENT
CommentContent: <COMMENT>/[.\r\n]/; // Valid in mode COMMENT
```

The `CommentEnd` terminal has precedence over `CommentContent` simply by preceding it in the
grammar description. This way it can't be 'eaten up' by the `CommentContent` terminal.

## Lookahead for terminals

Having an own scanner implementation enables us to support more features common for lexical analysis.
Since version 2 parol's scanner generation supports positive and negative lookahead.

The syntax can be seen in the following examples:

```parol
FunctionName: /[a-zA-Z_][0-9a-zA-Z_]*/ ?= /\s*\(/;
Operator: /<:|>:/ ?! ':';
```

The semantic can be described as follows:
* Match the terminal `FunctionName` only if it is followed by a left parenthesis.
* Match the terminal `Operator` only if it is **NOT** followed by a colon.

The right sides of the lookahead operators `?=` and `?!` are not considered part of the read token.

Note that the syntax is explicitly defined as a `TokenLiteral` follow by an optional `LookAhead`
that in turn is defined as positive or negative lookahead operator followed by a `TokenLiteral`.

This means that a terminal
```parol
XTerm1: "x" ?= "y";
```
is different from the terminal
```parol
XTerm2: "x";
```
in the sense that parol generates two different terminals for the scanner generator, `"x" ?= "y"`
and `"x"` that have different terminal types.

Be sure to define a "primary non-terminal for a terminal"
(see [Grammar description syntax](./ParGrammar.md#terminal-name-generation)) as in the examples
above to let `parol` generate different terminal names (here `XTerm1` and `XTerm2`). Using
terminals with the same `TokenLiteral` and differing lookahead expressions directly in productions,
i.e. without defining separate primary non-terminals for each, can lead to unexpected behavior.

## No vanilla mode

The so called [Vanilla mode](./VanillaMode.md) is not supported anymore.

It turned out that this mode, although potentially a little more efficient, is not really used by
anyone noticeable. The effort of maintaining two different modes is therefore no longer justified.

The vanilla mode has in summary the following disadvantages:
* Maintenance is hard, especially if you often change your grammar
* Only for power users because it requires deeper insight into the inner structure and the inner
mode of operation
* It tends to distract or confuse new users of `parol`

### Consequences

The auto generation mode is now the only mode `parol` provides. No distinction between two different
modes is necessary anymore. In version 2 you can forget about modes entirely.

All configurations that explicitly enabled the auto generation mode are removed and should be also
removed from your code when switching from version 1 to version 2.

Things you need to change:

* Remove the call of `enable_auto_generation()` on the `Builder` in your `build.rs`.
* Remove the switch `--auto-generate` resp. `-g` from the command line arguments of the `parol` tool.

If you want to keep your implementation in the vanilla mode your only option is to stay on
version 1. This should although be no problem, since this version is kept stable and will of course
receive fixes in the future as well.

## Lossless parse trees in both LL and LR parsers

From the beginning of it's development `parol` generates parse trees and returns them from the
`parse` function, as far as the parse tree generation is not suppressed.

These parse trees are now lossless. This means that tokens which are normally skipped like
whitespaces, newlines and comments are now preserved and inserted in the parse tree as well.
This should foster a whole category of applications, including language servers that need
precise information about the parsed input available in the parse result.

The parse tree itself is an advanced kind of syntax tree similar to
[rowan](https://github.com/rust-analyzer/rowan), the one that
[rust-analyzer](https://github.com/rust-lang/rust-analyzer) uses. More precisely it is a tree from
the [syntree](https://github.com/udoprog/syntree) crate.

## General refactoring and changes in the public API

- `<UserType>GrammarTrait::on_comment_parsed` has been renamed to
`<UserType>GrammarTrait::on_comment` for clarity