# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

---

Be aware that this project is still v0.y.z which means that anything can change anytime:

>"4. Major version zero (0.y.z) is for initial development. Anything MAY change at any time. The public API SHOULD NOT be considered stable."
>
>(Semantic Versioning Specification)

But we try to mark incompatible changes with a new minor version.

---

## v0.11.0 - 2022-10-xx

*This release provides rather breaking changes to the public API. Therefore we increase minor
version number to 11.*

* Reworked recursion detection and fixed it hopefully
  * Replaced proprietary graph based algorithm with a more conventional one
  * Added plenty of tests
* Switched to clap 4
* Removed prettyplease option
  * Opting clearly for rustfmt now

## v0.10.7 - 2022-09-14

* Launching the book as central documentation for `parol`

## v0.10.6 - 2022-08-11

* Fixed a missing newline in between multiple user type definitions
* Better handling of errors from grammar analysis to support parol language server
* Changed decoration format of production attributes

## v0.10.5 - 2022-08-03

* Update reference of `parol_runtime` to v0.7.2
* Fixed display format of non-terminals with attached user types
* Fixed generation of parol grammars (i.e. as expanded grammar) so that user types are correctly
presented

## v0.10.4 - 2022-08-02

* Improved logo. Texts have been converted to curves to ensure equal design on all systems.
* Update [docs\ParGrammar.md](docs\ParGrammar.md) to fit the new features of `parol`'s input grammar.
* Improved auto-generation:
  * `parol` can now handle AST types without lifetime references to the scanned text.
  * See changes in `list_auto` example

## v0.10.3 - 2022-07-09

* `parol new` can now enable parse tree visualization in newly created crates. You can activate it
by adding the new argument `-t`.

> `parol help new`

* Update reference of `parol_runtime` to v0.7.1

## v0.10.2 - 2022-07-08

* New artwork - fixing issue [#15](https://github.com/jsinger67/parol/issues/15)
* Supporting user defined types by a dedicated `%user_type` directive which allows you to define
aliases for possibly complex user defined types:
  > %user_type Number = crate::list_grammar::Number

  allows you to refer via the short name to the complex user type:

  >Num: "0|[1-9][0-9]*": Number;

  Please see example `list_auto` for an use case.

## v0.10.1 - 2022-07-01

* Feature 'User defined symbol types' completed
  * You can now define User defined types on non-terminal symbols too. Please, see example
  `list_auto` for a first impression.
* Remove `init` function from user's GrammarTrait in `parol new` to fit `parol_runtime` v0.6.0
  * The file name is now available at each token and thus we don't need to convey it in an `init`
  function.
* Repair `parol new` when it's supposed to generates library crates.
* `parol` is now the default binary run when using `cargo run`.

  You can use
  > cargo run -- ...

  instead of
  > cargo run --bin parol -- ...

  now.

## v0.10.0 - 2022-06-24

A lot of breaking changes.

* Use `parol_runtime` v0.6.0
* Refactoring of symbol table
* Start with new feature 'User defined symbol types'

  Since documentation is not updated yet, please see examples `list_auto` and `calc_auto`.

  Basically you can define an onw type for terminals in your grammar description like this:

  ```ebnf
  number: "0|[1-9][0-9]*": crate::calc_grammar::Number;
  ```

    Then you have to implement

  ```rust
  impl<'t> TryFrom<&Token<'t>> for Number;
  ```

    in the given module (here `crate::calc_grammar`). This way the generated structures get
    "magically" filled with your own types.

## v0.9.4 - 2022-06-10

* Added possibility to clip grammar symbols in the grammar description by suffixing them with an
optional cut operator (`^`). This instructs `parol` in auto-generation mode to not propagate this
symbol to the AST types. This can simplify and shorten the generated code dramatically.

  > The symbol `^` for the cut operator is chosen in the style of [oak](https://github.com/ptal/oak)'s
  [invisible type](http://hyc.io/oak/typing-expression.html).

* I applied this ability in the example grammars that uses auto-gen and in `parol`'s grammar itself.
* Adapt documentation

## v0.9.3 - 2022-06-05

* Fixed allow(unused_imports) directive
* Added some test files to git which are missing yet which causes `run_parsers.ps1` to fail
* `parol` is now implemented using the auto-generation approach
  * This is the basis for further improvements like user defined symbol types or clipping of AST
  content because grammar changes are likely. Then such changes won't have much influence on the
  grammar processing code.

## v0.9.2 - 2022-06-01

* Worked on tutorial
* Fixed case in crate name generation in subcommand `parol new`
* Merged fix for #16 - Thanks a lot to [mobotsar](https://github.com/mobotsar)

## v0.9.1 - 2022-05-28

* The auto-generation is now able to generate true option types. This improves this feature a lot
and only now one can say it is quite complete.

## v0.9.0 - 2022-05-27

* Worked on tutorial
* Changes in `parol new`:
  * The referenced version of `parol` is taken from CARGO_PKG_VERSION environment variable. If the
  current version is not released yet on [crates.io](https://crates.io/crates/parol) you can fallback to github:
  
    ```toml
    [build-dependencies]
    parol = { git = "https://github.com/jsinger67/parol.git" }
    ```

  * The parsed data is now printed to standard output automatically.
  * The `init` function is implemented with default handling.
  * A file with test input data (`test.txt`) is also created automatically.
* Removed serialization support - no use case anymore
* Removed some useless derives
* Took over some improvements from branch `optionals`
* Function `left_factor` now correctly transfers ProductionAttributes. This is a small part of the
fix of the bug described next.
* New bug in auto-generation detected and fixed:
  * Using an optional expression within a repetition confused the type generation.
  So constructs like ```{ [A] B }``` didn't work correctly.
  * The fix includes major changes in grammar transformation, especially the way optional
  expressions are handled. I therefore *increment the minor version to nine* to indicate a rather
  breaking change.

## v0.8.3 - 2022-05-14

* Fixed comments on generated user actions
* Avoid possible name clashes on user action names with the `init` function
* Worked on tutorial

## v0.8.2 - 2022-05-11

* Using updated version of `function-name` crate to fix the raw identifier problem occurred at
context generation

## v0.8.1 - 2022-05-08

* Minor cleanups
* Fixes and updates in documentation
* Fixed `parol left-factor` subcommand. The result is now printed as expected.
* Fixed compile error in crates generated by `parol new` subcommand, when module name contains
invalid characters.
* Using `named` marco from the crate `function-name` for the `context` variable in generated
semantic actions. This automatically keeps the context name in sync with the function name.

## v0.8.0 - 2022-05-06

* Removed some cases where type name collisions occurred
  * This involved considerable refactoring of grammar type generation
  * Another effect of these changes is that the generated source contains names of types and
  arguments that are more catchy and don't always contain suffixes like "_0" etc. Also the resulting
  code should be more robust against changes in your grammar. The downside is that all user code has
  to be adapted to the new generated names.
  We therefore increment the minor version to eight to indicate a rather breaking change.
* Improved change detection of builder to only trigger build script on changed grammar description
* If you used the auto-generation functionality of `parol` it is strongly recommended to switch over
to this new ^0.8 version.

## v0.7.0 - 2022-04-17

* Changed generated semantic action names

  To be more invariant when changing a grammar description the names don't include the production
  number anymore. Instead I generate a relative index which only changes potentially within a
  certain non-terminal.
  
  Note that this change needs a manual readjustment of already used
  code. Sorry for this inconvenience. But this change generally results in better maintainability.

  We therefore increment the minor version to seven to indicate a rather breaking change.

* Added a new tutorial which is still under construction

  It describes the new approach available since auto-generation is implemented.
  
  The old tutorial is moved to [TutorialOld.md](./docs/TutorialOld.md). It is still useful and
  explains the approaches that are now superseded by the new auto-generation related ones.

## v0.6.2 - 2022-04-03

* Add new subcommand `new`
  * Use this to create new crates that use `parol` as parser generator
  * It can generate both binary and library crates
  * It needs `cargo` as well as `cargo-edit` to be installed

## v0.6.1 - 2022-03-31

* Changes regarding the new auto-generation feature
  * Added new examples `list_auto` and `calc_auto`, that uses this new feature
    * You can compare them with the examples `list` and `calc` which use the traditional method.
  * Modified code generation for auto-generation (clippy)
  * More efficient call method of user actions (by reference)
  * Fixed a name clash between a popped AST value and the built result value in auto-generated actions
  * Using Token<'t> instead of OwnedToken in generated code now for performance reasons
    * This requires `parol_runtime` crate with version v0.5.9 now
* Partly reworked documentation

## v0.6.0 - 2022-03-20

* Added new experimental auto-generation feature is available now
  * Documentation still has to be added.
  * There exists a new example that uses this feature here: [JSON parser auto](https://github.com/jsinger67/json_parser_auto.git).
  * The old behavior is still intact and should be usable without restrictions.

## v0.5.10-pre - Not separately released, but included in 0.6.0

* Refactoring of module user_trait_generator
  * Changed from a bunch of functions to a struct `UserTraitGenerator` with `impl`.

## v0.5.9 - 2022-02-19

* Updated some dependencies and referenced some crates with caret requirements in semver.
  * Most prominent change was to reference `miette ^4.0` now.
  * Also `parol_runtime` is referenced with a new version (0.5.6).
* Using derive_builder to handle `bart` template data
  * The use of builder pattern shall be extended in the future
* More robust name generation with check against Rust keywords
* Enable use of `prettyplease` instead of `rustfmt` for code formatting.
  * This is enabled by non-default feature "pretty".
  * Also note that this is still experimental and the result of code formatting by `prettyplease` is
  currently not optimal. Mostly because of suppressed comments. Therefore I don't encourage to use
  this feature yet.

## v0.5.8 - 2022-02-03

* Included PR #13: *Clap 3.0 (derive + builder styles)*. ***Thanks a lot to oaleaf.***
  This closes Issue #10

## v0.5.7 - 2022-01-22

* New examples [Keywords](./examples/keywords) and [Keywords2](./examples/keywords2) to demonstrate
handling of keywords in `parol`'s scanner
* Compiling more test grammars in `run_parsers.ps1`. Also negative cases are checked.
* Factored out grammar transformation from the parser to the module transformation

## v0.5.6 - 2022-01-10

* Even better integration of tools, i.e. subcommands with `clap`. Preparation for planned switch
over to `clap v3`.
* Fixed issue #4: *It appears the --only-lookahead option (-c) doesn't work*. This option is useless
and was removed.
* Builder: Write out a preliminary version of the expanded grammar after parsing to support grammars
that fail later checks.
* Added CONTRIBUTING.md
* Consolidated Public API (fixes #11)
* Updated documentation
  * Using `parol` like an installed tool in example invocations instead of
`cargo run --bin parol -- ...` now
  * Fixed links in cargo's doc output
* Improved termination behavior of the language generation feature (`parol generate`) introduced in
v0.5.3
* Improved error report (Undeclared variable) in example `calc`

## v0.5.5 - 2022-01-05

* Included PR #8: *Rename default actions file from grammar.rs -> grammar_trait.rs*. ***Thanks a lot
 to Techcable***

## v0.5.4 - 2022-01-05

* Fixed a serious bug in parsing groups, repetitions and optionals introduced in commit [6476e75].
* Started issuing more detailed miette-like errors from parol itself.
* Fixed an invalid generation of the %pop() instruction from '%pop' to '%pop()'.
* More tests to check the parol parser's internal representation.
* Fixed some problems related to platform specific newline characters.
* Fixed issues #5: *Bizarre error running scanner_states*. ***Thanks a lot to Techcable***
* Included PR #6: *Add API to invoke parol from build scripts*. ***Thanks a lot to Techcable***

## v0.5.3 - 2022-01-02

As of this version a detailed changelog is maintained to help people to keep track of changes that
have been made since last version of `parol`.

### Generation of sentences

An new tool (subcommand) `generate` was added to `parol` to generate a random sentence of a given
grammar.
You can use it this way:

```shell
    >parol generate -f ./examples/json/json-exp.par
{ "\r" : "uA7Fcu8a4A񚥚\r" , "\b\f\nuD1C0u5daf\b" : null , "\n\/\f𘃈򘱵" : true , "\\󸽿\\\\uCfC4𚍑𞱁uD852" : "\b\buEA01\\" } 
```

I already found some quirks in a few regular expressions 😉.

Also you can run endless stress tests like in this example using a *powershell* one-liner:

```powershell
for (;;) {parol generate -f ./examples/json/json-exp.par | Set-Content "$env:Temp/x.json"; json_parser "$env:Temp/x.json"; if (-not $?) { break } }
```

#### Acknowledge

This was possible with the help of the awesome
[rand_regex](https://github.com/kennytm/rand_regex.git) crate.

#### Disclaimer

On complex grammars the generation can get into deeply branching the grammar productions again and
again because productions are randomly selected. Therefore generation is aborted with an error if
the resulting sentence exceeds a certain limit. This limit currently defaults to a string length of
100 000. This value can be overwritten by giving an additional parameter after the grammar file.
If generation fails with error `parol::generators::language_generator::source_size_exceeded` please
give it another try.
