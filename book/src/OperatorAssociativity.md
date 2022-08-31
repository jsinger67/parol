# Operator associativity (WIP!)

If you use `parol` with auto-generation mode (flag -g) all repetitive data structure are generated
as vectors.

A multiplication defined as

```parol
Multiplication
        : Factor { MulOp Factor }
        ;
```

will result in an ATS type like this:

```rust
/// Type derived for non-terminal Multiplication
pub struct Multiplication<'t> {
    pub factor: Box<Factor<'t>>,
    pub multiplication_list: Vec<MultiplicationList<'t>>,
}

/// Type derived for non-terminal MultiplicationList
pub struct MultiplicationList<'t> {
    pub mul_op: Box<MulOp<'t>>,
    pub factor: Box<Factor<'t>>,
}
```
