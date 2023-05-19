# Typescript bindings for `parol`'s symbol table types

This folder contains generated typescript bindings.
These bindings are generated with the help of the awesome
[ts-r](https://github.com/Aleph-Alpha/ts-rs.git) crate.

The update of these bindings is triggered by calling

```shell
cargo test
```

in the root folder of crate `parol`.

Currently these bindings are used by a tool called
[parol_symbols](https://github.com/jsinger67/parol_symbols.git) that tries to support developers by
enabling them to comfortably browse `parol` generated symbols for a given grammar.
