# Useful Tips

## Build Performance

To optimize build performance when using `parol`'s [Builder](https://github.com/jsinger67/parol/blob/main/crates/parol/src/build.rs) API in your `build.rs` script, add the following overrides to your `Cargo.toml` file:

```toml
# Optimized build performance
[profile.dev.build-override]
opt-level = 3

[profile.release.build-override]
opt-level = 3
```

#### Credits

Thanks to [dalance](https://github.com/dalance) for reporting [issue #49 (build.rs performance)](https://github.com/jsinger67/parol/issues/49).

## Parser Generation Performance

The need to frequently regenerate the parser from a grammar is greatly reduced in `parol` due to its design. `parol` generates data structures for your grammar, an interface, and the plumbing to call its methods. This separates parser generation from any code you write to process your grammar, such as the interface's implementation.

> This feature enables ad hoc generation of acceptors for any valid grammar, which can be considered *rapid prototyping* for your grammar.

You only need to regenerate the parser when you change your grammar description (i.e., your `.par` file). If parser generation is expensive for your grammar, consider placing the generated parser and user trait under source control.

It is beneficial to design your grammar to be LL(k) with the smallest possible k. Although this can be challenging, it is worthwhile.

Also, optimize your grammar for a minimal number of productions. Consider these guidelines:

- Avoid productions that only rename a non-terminal, such as:
  ```parol
  A: B;
  ```
- Disambiguate your productions and avoid duplications like:
  ```parol
  A: X Y Z;
  B: X Y Z;
  ```
  Determine why you need productions with identical right-hand sides. If they are actually the same, consider unifying them.

If you have a historical grammar definition that is left-recursive (common in Yacc/Bison grammar descriptions), allow extra time and effort to convert it to a right-recursive form.

Alternatively, you can use LALR(1) grammars without sacrificing the convenience of `parol`. See the [grammar type specification](https://jsinger67.github.io/ParGrammar.html#defining-the-grammar-type).

`parol` currently does not provide special support for this phase, except for detecting left recursions in your grammar.

Support for removing direct left recursions may be provided in the future.