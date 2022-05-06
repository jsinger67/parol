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
