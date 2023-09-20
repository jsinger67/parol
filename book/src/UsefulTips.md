# Useful tips

## Build performance

To get an optimized build performance when using `parol`'s
[Builder](https://github.com/jsinger67/parol/blob/main/crates/parol/src/build.rs) API in your
`build.rs` script you can insert the following overrides into your `Cargo.toml` file:

```toml
# Optimized build performance
[profile.dev.build-override]
opt-level = 3

[profile.release.build-override]
opt-level = 3
```

### Credits

Thanks to [dalance](https://github.com/dalance) for reporting
[#49 (build.rs performance)](https://github.com/jsinger67/parol/issues/49)

## Performance of parser generation

First you need to understand that the necessity to frequently generate the parser from a given
grammar is drastically diminished in `parol` because of its design.
That means `parol` generates besides the data structures for your grammar only an interface and the
plumping to call its methods.
This cuts the dependencies for parser generation from any code you write to process your grammar,
i.e. the interface's implementation.

>By the way, this property enables ad hoc generation of acceptors for any valid grammar, which I like
to call *rapid prototyping* for your grammar.

So, you only need to generate the parser if you change anything in your grammar description, i.e.
in your *.par file.
If parser generation is expensive for your grammar, what indeed can be the case, I advice you to put
the generated parser and the user trait under source control.

The next thing you should understand is that you should design your grammar to be LL(k) with k as
minimal as possible. I know, this can be hard but will pay out in the end.

Also try to optimize your grammar for the goal "Minimal number of productions". This can be often
broken down to these constraints:
* Avoid productions that only rename a non-terminal, i.e. the ones in the form
    ```parol
    A: B;
    ```
* Try to disambiguate your productions, i.e. avoid duplications that have the following form
    ```parol
    A: X Y Z;
    B: X Y Z;
    ```
    Pin down why you need productions with identical right-hand-sides. Aren't they actually the same
    and shouldn't they rather be unified?

If you have a historical grammar definition that is left recursive, which in deed is possible
for instance because of the ubiquity of Yacc/Bison grammar descriptions, you should allow for extra
time and effort to convert it to a working right recursive one.

`parol` currently provides no special support for this phase except that it is able to detect left
recursions in your grammar.

I may provide support for parts of this problem in the future, for instance to remove direct left
recursions somehow.