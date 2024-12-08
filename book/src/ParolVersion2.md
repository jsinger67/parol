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

This doesn't mean that `regex-automata` is in any way bad, on the contrary, it is a very versatile
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

| Input (Bytes, Tokens) | Version 1 | Version 2 |
|---|--:|--:|
|549, 175 | 0.01ms + 33ms | 1.2ms + 1ms
|58525, 3111 | 0.76ms + 33ms | 2.34ms + 1 ms |
|5873100, 1159200 | 122ms + 33ms |  80ms + 1ms |

The added value in cells with times is the constant time to create the tokenizer, which is required
once for each call to the generated parser.

As you can see, `scnr` does its job well.

### Impact on lexical analysis

Since `scnr` doesn't support some regex feature which are either complicated, costly or unnecessary,
some things can't be resolved simply by the means regex crates like `regex-automata` provide.
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
complex. Therefore there exist a second approach to cope with missing non-greediness.

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