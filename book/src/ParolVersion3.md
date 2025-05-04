# Changes in version 3

## Changes in public API

Detailed list of changes in the public API
* The number of elements of the tuple struct used in the enum variant `Trm` of the enum
`parol::grammar::symbol::Terminal` has changed.
* The number of elements of the tuple struct used in the enum variant `N` of the enum
`parol::grammar::symbol::Symbol` has changed.
* In the module `parol::parser::parol_grammar` some public enums have similar changes in some
variants.

I expect that most applications that use `parol` v2 can upgrade to v3 without problems. The
changes listed above only affect applications that use the `parol` library for very specific tasks.

## New feature "User defined member names"

You can now specify for each symbol on the right-hand side of a production how its corresponding
member in the generated struct should be named.

To achieve this you can use the newly introduced `@` operator.

```parol
Declaration :
    ...
    | "%nt_type" Identifier@nt_name "="^ UserTypeName@nt_type
    ...
```

In this example the Identifier in the production will be named `nt_name` and the UserTypeName will
receive the name `nt_type` in the generated struct data type for this production.

## New feature "Non-terminal types"

You can now easily define a user type to which each occurrence of a certain non-terminal should
be automatically converted to.
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
The non-terminal `ScannerState` was defined to be automatically converted to `ScannerConfig`.

It is semantically completely identical to use `%user_type` for a certain non-terminal and then
apply the defined `%user_type` alias to each occurrence of the concerned non-terminal in the grammar
definition explicitly.

## New feature "Terminal type"

You can now easily define a user type to which each occurrence of a terminal should be
automatically converted to.
This is done like in the following example:

```parol
%t_type crate::parol_ls_grammar::OwnedToken
```

There can be only one type defined to which all terminals are converted to.

More precisely, if there are more such directives given the last one will win.

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
All terminals are automatically defined to be converted to `crate::parol_ls_grammar::OwnedToken`.

## parol re-exports parol_runtime

This simplifies the use and avoids version conflicts between both libraries.

The initial `Cargo.toml` will then simple be similar like this:

```toml
[package]
name = "re_export"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
env_logger = "0.11"
parol_runtime = "2.2"
thiserror = "2.0"

[build-dependencies]
parol = "3.0.0"
```

This makes a lot more sense, since you would expect to reference `parol_runtime` in your parser
crate and have `parol` as build dependency.

Anyway, as before, it is advisable to use `parol_runtime` in the version that `parol` in the build
dependencies uses. These two crates are precisely coordinated.

To check this you can use the following command:

```shell
cargo tree -i parol_runtime
```
