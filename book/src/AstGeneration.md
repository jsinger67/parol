# AST generation (WIP!)

`parol` can be instructed to generate all types your grammar implies automatically. It therefore
analyzes all productions in your grammar. The first step is to canonicalize your grammar into a
standard format with the following properties.

* All EBNF constructs, i.e. optional elements, repetitions and groupings are substituted by
equivalent representations.
  * A: [B]; => A: BOpt; BOpt: B; BOpt: ;
  * A: {B}; => A: BList; BList: B BList; BList: ;
  * A: (B); => A: BGroup; BGroup: B;
* Alternations are propagated to multiple productions.
  * A: B | C; => A: B; A: C;

These transformations are applied iteratively until all EBNF constructs are replaced.

Then `parol` checks this pre-transformed input grammar for several properties that prevent a
successful processing. Those unwanted properties are

* Left-recursions
* Non-productive non-terminals
* Unreachable non-terminals

If the grammar does not have such properties the next step is to left-factor this grammar form. This
step is crucial for decreasing the number of necessary lookahead symbols.

This finally transformed grammar is the basis for the parser generation and is typically written to
file for later reference. By convention this 'expanded' grammar is stored to files named
\<original-name\>-exp.par.

This expanded grammar is checked and left-factored. It is the basis for parser generation.

## Type inference

Having such a transformed grammar all productions have the form
\\[v: s*; \\]
where \\\(v \epsilon V, s \epsilon (V \cup \Sigma)\\\), \\\(V\\\) is the set of non-terminals,
\\\(\Sigma\\\) is the set of terminals.

The relation of the generated productions to their original EBNF constructs is actually lost at this
point.

But because we need the information if a set of productions was originated from, e.g. an optional
construct (`[...]`) `parol`conveys these relationship during the whole transformation process to be
able to infer it into a Rust `Option<T>` eventually.

To explain it using the form of transformation shown above we could write this:

`A: [B]; => A: BOpt; BOpt: B; BOpt: ; => typeof A = Option<typeof B>`

This step leads directly to a solution if non-terminal `A` has only one production.

In this case the the type of `A` is

```rust
struct A {
    b: Option<B>
}
```

We must use a `struct` here because this patterns should work for productions with n elements on its
right-hand side. For each such element we then introduce a separate member into the struct.

If non-terminal `A` has more than one productions the resulting type of `A` will be an `enum` type
with n enum variants for n productions:

`A: B | C; => A: B; A: C;`

```rust
struct B {
    // ...
}
struct C {
    // ...
}
// Type of non-terminal A
enum A {
    A0(B),
    A1(C),
}
```

The finally generated AST type is also an `enum` in the end. It comprises all non-terminal types of
the grammar and provides exactly one enum variant for each of them. This type is mainly used by the
parser itself to be able to instantiate a typed parse stack. The user rarely have to deal with this
AST `enum`.

### Recursive structure of a grammar

A context free grammar is typically defined using recursive constructs. But you can't define types
in Rust that are directly recursive because this would lead to an infinitive type size.

To cope with this limitation `parol` generates boxed types for non-terminals when introducing
elements to `structs`, e.g.:

```rust
struct A {
    b: Box<B>
}
```

This results in finite type sizes.

## Manage Type generation

### Omission of elements

### Assigning user types
