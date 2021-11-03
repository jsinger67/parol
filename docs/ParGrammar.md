# The syntax of PAR Grammar definition

Here I provide the definition of the PAR grammar in EBNF. It is actually written in PAR syntax and can be found here [parol_grammar.par](../src/parser/parol-grammar.par).

```ebnf
(* PAR Grammar defined in EBNF *)
Grammar             = Prolog GrammarDefinition.         (* The start symbol of the PAR grammar *)
Prolog              = StartDeclaration {Declaration}.
StartDeclaration    = '%start' Identifier.
Declaration         = '%title' String
                    | '%comment' String
                    | '%line_comment' String
                    | '%block_comment' String String
                    | '%auto_newline_off'
                    | '%auto_ws_off'.
GrammarDefinition   = '%%' Production {Production}.     (* There must be at least one production - with the start symbol *)
Production          = Identifier ':' Alternations ';'.
Alternations        = Alternation {'|' Alternation}.
Alternation         = {Factor}.
Factor              = Group
                    | Repeat
                    | Optional
                    | Symbol.
Symbol              = Identifier                        (* EBNF: Meta-identifier *)
                    | String.                           (* EBNF: Terminal-string, always treated as a regular expression! *)
Group               = '(' Alternations ')'.             (* A grouping *)
Optional            = '[' Alternations ']'.             (* An optional expression *)
Repeat              = '{' Alternations '}'.             (* A repetition *)
Identifier          = '[a-zA-Z_]\w*'.
String              = '\u{0022}([^\\]|\\.)*?\u{0022}'.
```

This grammar is very concise and most programmers should be familiar with. But there are several specialties which will be described here. First please notice the built-in support for language comments.

Using the `%line_comment` and `%block_comment` constructs you can easily define your language's comments. For example you can define comments like it's done in the calc example [calc.par](../examples/calc/calc.par):

```ebnf
%line_comment "//"
%block_comment  "/\*" "\*/"
```

You can supply more than one of these two comment declarations. They will all be considered as valid comments.

As opposed to EBNF you use C-like line comments starting with two slashes (//) and bock comments (/\* ... \*/) in PAR files. This is a result of the close relationship between PAR grammar and bison's grammar.

## Case sensitivity

Non-terminals are treated case sensitive, i. e. "list" and "List" are different symbols.

## Sections

`parols`'s input language consists of two sections divided by the %% token. Above there are declarations of which only the first %start declaration is mandatory. It declares the start symbol of your grammar.
The second section below the %% token contains the actual grammar description in form of several productions. At least one production must exist.

## The start symbol

It is important to note that the start symbol of the grammar must always be declared with the `%start` declaration. It is the very first declaration in the PAR file.

```ebnf
%start Grammar
```

## Scanner control

A scanner (aka lexer) is automatically created from all used terminal symbols.

### New line handling

The scanner per default skips newlines automatically. To suppress this use the `%auto_newline_off` directive.
You have to handle newline tokens on your own in your grammar.

### Whitespace handling

The scanner also per default skips whitespace automatically. To suppress this use the `%auto_ws_off` directive.
You have to handle whitespace tokens on your own in your grammar.

### Terminal name generation

The names of the terminals are deduced from the content of the terminal itself. For instance, for a terminal ":=" it creates the terminal name "ColonEqu", see generated parser for Oberon-0. If you want this name to be more expressive, you can dedicate a separate production to the terminal, lets say:

```ebnf
Assign: ":=";
```

With this trick you define a so called "primary non-terminal for a terminal" (I coined it this way) that instructs the name generation to name the terminal "Assign".

### Terminal conflicts

Since `parol` creates a scanner on the basis of the rust regex crate all terminals are treated as if they were regular expressions.
Thus you have to consider the following caveats.

* If you want to use a character that is a regex meta-character you have to escape it, like the '+' in the following example:

```ebnf
AddOperator: "\+|-|OR";
```

* In case of conflicts between different terminals _the first seen will win_

The last point needs a more detailed explanation.
It's best to show an example for such a situation.
Say you have two terminals "-" and "--", _minus_ and _decrement_. The generated scanner is then based on the following regular expression:

```regex
    "-|--"
```

The rust regex will now match two times _minus_ when actually a _decrement_ operator should be detected.
It behaves here differently than a classic scanner/lexer like Lex that obeys the _longest match_ strategy.

Fortunately there is a simple way to achieve what we want. We just need a resulting regular expression with a different order:

```regex
    "--|-"
```

This will perfectly do the job.

To get such an order the _decrement_ terminal has to be defined ***before*** the _minus_ terminal as in the following snippet.

```ebnf
decrement: "--"
;
...
minus: "-"
;
```

Thats all.

With this simple but effective means you have the control over terminal conflicts.

## Semantic actions

Semantic actions are strictly separated from your grammar description.
You will use a generated trait with default implementations for each production of your grammar. You can implement this trait in your grammar processing item and provide concrete implementations for those productions you are interested in.

More on implementing semantic actions see

* [Tutorial](Tutorial.md)
