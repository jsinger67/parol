# History of This Project

The development of the `parol` parser generator started as a personal journey to master LL(k)
parsing with deterministic finite automata.

Two parser generators with contrasting approaches greatly influenced its design:

- The classic Unix tool Yacc and [Bison](https://www.gnu.org/software/bison/)
- [ANTLR](https://www.antlr.org/)

Each has its own quirks and idiosyncrasies.

Bison often produces shift/reduce or reduce/reduce conflicts, which can be difficult to understand.
ANTLR generates recursive descent parsers that are prone to stack overflows. It is easy to write or
generate a program that crashes a parser produced by ANTLR. For example, a deeply nested expression
with 6000 parentheses can cause such an issue.[^1]

Despite these differences, Bison generates deterministic parsers using finite automata, and ANTLR
also uses deterministic finite automata to select the next production for a non-terminal.

This raised the question: Why not combine the best of both worlds?

With this goal in mind, I began my first attempts using F# ([Lelek](https://github.com/jsinger67/Lelek)). Eventually, I discontinued this project because it no longer felt suitable.

Lelek was a necessary step to understand what was feasible and what was not.

After several attempts, I transitioned to Rust, which felt more vibrant and compelling.

Thus, `parol` was born—initially as a rewrite of Lelek. I was willing to discard some parts of Lelek
and introduce new approaches.

## What I Retained

- The basic approach of using regular expressions to generate scanners
- Using DFAs to solve the [Rule Decision Problem](https://github.com/jsinger67/Lelek/blob/master/docs/Approach.md), though I changed the method for obtaining k-sets for productions
- The foundational ideas behind the grammar description language and its similarity to Bison's input
format
- The separation between language description and implementation
- The strategy of checking a grammar for preconditions before generating parser data, ensuring
termination of certain algorithms
- The algorithm for [visualizing parse trees](https://github.com/jsinger67/id-tree-layout)

## What I Changed

- Recursion detection
- Generation of k-sets for productions, including algorithms for FIRST(k) and FOLLOW(k)
- Terminology: I now prefer 'Production' over 'Rule'
- The [parser runtime](https://github.com/jsinger67/parol_runtime) was separated into a small crate

## What I Added

- Automatic inference and generation of all types for the grammar's AST, making the grammar
description sufficient for `parol` to build a fully functional acceptor with no extra
effort—enabling **real rapid prototyping** for your language!
- Built-in tools for:
  - Generating new crates
  - Checking a grammar for properties (left-recursion, reachability, productivity)
  - Left-factoring a grammar
  - Calculating FIRST(k) and FOLLOW(k) sets
  - Generating random sentences from a grammar description
- Scanner states, also known as [start conditions](https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex_toc.html#TOC11)
- Build script integration to invoke `parol` automatically during your crate's build process
- A [Visual Studio Code extension](https://github.com/jsinger67/parol/tree/main/tools/parol-vscode)
and a [Language Server](https://github.com/jsinger67/parol/tree/main/crates/parol-ls)
- Optional support for LALR(1) grammars in addition to LL(k)
- Features that Lelek never received

[^1]: To be fair, parol is not immune to stack overflows. In deeply nested expressions, the
resulting data structures also become deeply nested. Some compiler-generated trait implementations
like `Debug`, `Clone`, or `Drop` can then cause stack overflows. This can be avoided by carefully
implementing such traits yourself.