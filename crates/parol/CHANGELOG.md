# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 4.0.0 - Not released yet

* Official start of version 4 development.
* Switch to scnr2 scanner crate
* Stop support of parser based scanner switching in favor of scanner based scanner switching

## 3.1.0 - 2025-07-11

* Generate untransformed grammar with production index comments
  (i.e. when calling `parol -u  <WRITE_UNTRANSFORMED>` resp.
  `parol --write-untransformed <WRITE_UNTRANSFORMED>`)
* Change handling of token aliases when lookahead expressions are used
* Fix generation of lookahead regex in parser source
* Fixed clippy warnings in tests, benches and production code
* Merged PR [#652](https://github.com/jsinger67/parol/pull/652) from [ryo33](https://github.com/ryo33)

  The new feature allows to additionally generate enum node types for terminals and non-terminals.
  Here is a quote from the PR:
  - Introduce `TreeConstruct` trait and `parse_into` methods
    - `parse` methods now calls `parse_into` with TreeBuilder as `TreeConstruct`.
  - Added `generate_parser_and_export_node_infos` method to the `Builder`
    - Exported information is helpful for users to have their generator code.
  - Added `node_kind_enums` and `node_kind_enums_output_file` to the `Builder`

  For more details please have a look at the conversation at the PR and visit ryo's
  [eure](https://github.com/Hihaheho/eure/tree/main) project and here especially to the crate
  [eure-tree](https://github.com/Hihaheho/eure/tree/main) to find out more about his approach of
  constructing customized trees from parol's parse trees.

  *Thanks a lot, ryo!*


## 3.0.1 - 2025-03-08

* Fix [#595](https://github.com/jsinger67/parol/issues/595)
* Fix [#596](https://github.com/jsinger67/parol/issues/596)
* Fix parol (par) file generation when `%t_type` is used.
* Fix parol (par) file generation when `%nt_type` is used.
* Fix Token alias handling in ParolGrammar

## 3.0.0 - 2025-03-02

The [book](https://jsinger67.github.io) has been updated and received a new chapter on `parol`'s
[version 3](https://jsinger67.github.io/ParolVersion3.html).

### New feature "User defined member names / Grammar labeling"

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


  This feature introduces **breaking changes** in the public API. Therefore we need to bump the
  major version to 3.

####  Detailed list of changes in the public API
  * The number of elements of the tuple struct used in the enum arm `Trm` of the enum
  `parol::grammar::symbol::Terminal` has changed.
  * The number of elements of the tuple struct used in the enum arm `N` of the enum
  `parol::grammar::symbol::Symbol` has changed.
  * In the module `parol::parser::parol_grammar` some public enums have similar changes in some arms.

  I expect that most applications that use `parol` v2 can upgrade to v3 without problems. The
  changes listed above only affect applications that use the `parol` library for very specific tasks.

### New feature "Non-terminal types"

  You can now easily define a user type to which each occurrence of a certain non-terminal should
  be automatically converted to.
  This is done like in the following example:

  ```ebnf
  %nt_type ScannerState = crate::parser::parol_grammar::ScannerConfig
  ```

  It is similar to the already available `%user_type` with what you could define an alias for a
  user defined type which in turn you could apply to single symbols on the right-hand side of
  grammar productions. The `%nt_type` can't be used on terminals but it makes the application to
  non-terminals much easier.
  Here is the old version used in `parol` itself before (only partial)
  ```ebnf
  %user_type ScannerConfig = crate::parser::parol_grammar::ScannerConfig
  // ...
  %%
  // ...
  Prolog
    : StartDeclaration { Declaration } { ScannerState: ScannerConfig }
    ;
  ```
  And here is the new variant in which `%nt_type` is used.
  ```ebnf
  %nt_type ScannerState = crate::parser::parol_grammar::ScannerConfig
  // ...
  %%
  // ...
  Prolog
    : StartDeclaration { Declaration } { ScannerState }
    ;
  ```
  The non-terminal `ScannerState` was automatically defined the be converted to `ScannerConfig`.
  
  It is semantically completely identical to use `%user_type` and the application of it to each
  occurrence of the non-terminal in the grammar explicitly.

### New feature "Terminal type"

  You can now easily define a user type to which each occurrence of a terminal should be
  automatically converted to.
  This is done like in the following example:

  ```ebnf
  %t_type crate::parol_ls_grammar::OwnedToken
  ```

  There can be only one type defined to which all terminals are converted to.

  More precisely, if there are more such directives given the last one will win.

  Here is the old version used in `parol-ls` itself before (only partial)
  ```ebnf
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
  ```ebnf
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
  All terminals are automatically defined the be converted to `crate::parol_ls_grammar::OwnedToken`.

### Fix issue "Find a way to elide lifetimes if possible in generated ToSpan implementations #581"

  We can simply always generate elided lifetimes.

## 2.2.0 - 2025-02-13

Use version 2.2.0 of `parol_runtime` which provides a new crate feature.

## 2.1.2 - 2025-01-21

- Fix for [#558](https://github.com/jsinger67/parol/issues/558)
  This fix is provided by `parol_runtime@2.1.1`

## 2.1.1 - 2025-01-17

- Fix broken `parol new` subcommand

## 2.1.0 - 2025-01-17

- Performance optimization for grammar analysis, especially the calculation of FIRST(k) and
FOLLOW(k) sets, using FxHashSet in `KTuples` type
  * Changed some tuple structs to structs with named fields for clarity
- Fixed `parol::conversions::par::grammar_to_par::render_scanner_config_string` to always output the
transitions of a scanner state in a deterministic alphabetic order
- Merged PR [#537](https://github.com/jsinger67/parol/pull/537) from [ryo33](https://github.com/ryo33)

## 2.0.0 - 2024-12-25

>Please note, that changes made in version 2 are also detailed in an extra
[chapter of the book](https://jsinger67.github.io/ParolVersion2.html)

- Integration of scanner crate `scnr`. `parol_runtime` now uses this crate as scanner crate instead
of regexes created with the help of the `regex-automata` crate. Please have a look at
[parol_runtime's CHANGELOG](../parol_runtime/CHANGELOG.md).

- Refactor some grammar tools to enforce LL grammar type.

- Error recovery on generated LL(k) parsers can now be disabled, via Builder API (`disable_recovery`)
and via command line argument (`--disable-recovery`).

- `parol` creates lossless parse trees in both LL and LR parsers.

- Fix bug Missing negation in LL(k) check for decidable subcommand #489
  * Also fixed similar bug in subcommand `calculate-k` and `calculate-k-tuples`

## 1.0.0 - 2024-09-02

- Merged PR [#393](https://github.com/jsinger67/parol/pull/393) from [ryo33](https://github.com/ryo33)
 This fixes a bug in range generation of enums.

>**The version 1 is maintained on branch `release1.0`. All changes for this version therefore are
 only visible on this branch. This includes this change log too.**

## 0.31.0 - 2024-06-21

- Referencing `parol_runtime` 0.24.0 which introduced braking changes, due to new parser error types.
Please, see [CHANGELOG parol_runtime](../parol_runtime/CHANGELOG.md) for details.

## 0.30.0 - 2024-06-08

- Provide a general strategy to solve conflicts during parse table generation.

  * **Shift-reduce conflicts**
  
    Parol behaves like Bison/YACC now, namely that in general all shift-reduce conflicts are
    resolved by favoring shift.

  * **Reduce-reduce conflicts**

    Parol prefers productions with lower number, i.e. the first seen ones in the grammar
    description.

  All automatically resolved conflicts generate warnings that should not simply be ignored.
  A way to disable warnings on resolved conflicts will be provided later.

- Improved load performance of LRParsers by modified code generation

- Referencing `lalry` instead of `lalr`now


## 0.29.0 - 2024-05-16

- Fixed dependencies generated by the `parol new` command
- Cleanup structure of examples folder. All examples are now listed in crate `parol`'s Cargo.toml
- New example `calc_lr` to showcase LALR(1) parsing with `parol`
- Parse tree generation for LR grammars is now available
- Add the missing command line option `trim_parse_tree` to `parol`'s arguments, see `parol help` for
more details
    
    This configuration was not available until now. If you didn't use a build script and call the
    `parol` binary to generate your source you weren't able to activate this configuration.
    The `trim_parse_tree` option is useful if you don't need a parse tree after finishing the parse
    process. It mostly saves runtime effort, speeds up the parsing performance and also decreases
    the memory consumption.
- Removed clippy warnings new in Rust 1.78.0
- Implement scanner-based scanner switching which can be used in both LL(k) and LALR(1) grammars

    * A new example `scanner_states_lr` is introduced to show the basic functionality. Additionally 
    the example `basic` was migrated to use the scanner-based scanner switching.
- __Public API has changed__

## 0.28.0 - 2024-04-29

- Complete new **LALR(1) grammar support**. Use the new `%grammar_type` directive to switch to LALR(1)
  ```
  %grammar_type 'LALR(1)'
  ```
  The interfacing to the user's grammar trait works just the same as before.
  You can find an example in the examples folder: [list_lr](../../examples/list_lr/list.par).

  Although `parol` is by default a LL(k) parser generator I believe that the support of the popular
  LALR(1) grammars can help users in general.

  A view things are not yet available for LR grammars:
    * No parse tree generation

      This has to be revised since the mechanisms of constructing the parse tree during the parse
      process are different than the ones used in the LL parser.

    * Scanner state switching is not implemented yet

      In fact there is not even a concept to accomplish this in the LR parser. I have to reason
      about this a bit, I think.

    * No general solution of handling conflicts during parse table generation.

      This is a mix of different obstacles that comes from the tooling on the one hand and from the
      philosophy of `parol` on the other hand. Doing it `parol`'s way I would do it in the par file
      (as for the scanner switching).

      Currently any conflict results in an error. I know this might prevent its use for more complex
      grammars but it is the save way for now.

  Please, give feedback of any kind to this new grammar type support.


## 0.27.0 - 2024-04-01

- Heavy performance optimization of the calculation of FIRST and FOLLOW sets and there especially
the k-concatenation.

  Here are the results of the benchmarks produced on my machine:

  ```
  k_tuples_k_concat        time:   [168.17 ns 171.30 ns 174.58 ns]
                          change: [-80.201% -79.680% -79.163%] (p = 0.00 < 0.05)

  k_tuples_k_concat_large: time:   [154.21 ns 156.97 ns 159.79 ns]
                          change: [-65.600% -64.757% -63.912%] (p = 0.00 < 0.05)
  ```

  This involved some changes in the public API that can break your application, so please be aware
  of that.

  _Here are the detailed changes:_

  ### parol::analysis::k_tuple::Terminals

  The Implementation of `parol::analysis::k_tuple::Terminals` has received a huge refactoring. The
  terminals that were former stored in an array of
  `parol::analysis::compiled_terminal::CompiledTerminal`s are now stored in a single u128 value.
  Each terminal takes a fixed number of bits which is determined by the highest index
  of a terminal from the input grammar. This is the reason why some APIs now expect an additional
  parameter `max_terminal_index`.

  The methods `from_slice` and `from_slice_with` of the struct `Terminals` have been removed. You
  should use the `std::iter::Extend` trait that is now implemented for `Terminals` and supports
  types that implement `std::iter::IntoIterator<Item = CompiledTerminal>>` or
  `std::iter::IntoIterator<Item = TerminalIndex>>`. This should make the overall use more idiomatic.

  `Terminals` now also provides an `iter()` method that returns a type which implements
  `std::iter::Iterator<Item = TerminalIndex>`. This should also make the use more idiomatic.

  ### parol::analysis::k_tuple::KTuple

  The `parol::analysis::k_tuple::KTuple` has no method `new()` anymore. Use
  `parol::analysis::k_tuple::KTupleBuilder` instead. This was necessary to ensure that all necessary
  information, such as `k` and `max_terminal_index` have been provided before creating the object.

  `parol::analysis::k_tuple::KTuple` now implements `Copy`.
  
  Its method `from_slice_with` was removed. You can use the `std::iter::Extend` trait that is now
  also implemented for `KTuple` and supports types that implement
  `std::iter::IntoIterator<Item = CompiledTerminal>>` or
  `std::iter::IntoIterator<Item = TerminalIndex>>`.

  ### parol::analysis::k_tuples::KTuples

  The `parol::analysis::k_tuples::KTuples` also has no method `new()` anymore. Use
  `parol::analysis::k_tuples::KTuplesBuilder` instead. This was necessary to ensure that all
  necessary information, such as `k` and `max_terminal_index` have been provided before creating the
  object just like for the `KTuple` detailed above.

  The type `KTuples` now does no more provide an implementation of the `Default` trait because this
  could lead to a creation of invalidly initialized instances.

  ### parol::analysis::terminals_trie::Trie

  The types in this module have been removed from the crate although the file is still under source
  control. The Trie was replaced by a `HashSet` in `KTuples` and has therefore become superfluous.



## 0.26.1 - 2024-02-13

- Fixed issue [#38](https://github.com/jsinger67/parol/issues/38)

  This is a major step towards more efficiency.
  The `parol` tool supports a new command line switch (-b, --min_boxes) that enables the minimization
  of used boxes in generated data structures. The `parol::build::Builder` also provides a new method
  `minimize_boxed_types()` that you can call in your build scripts.

  `parol` then generates first all data types without boxed
  members and afterwards applies an extra step that detects recursions in types and removes them by
  selectively inserting boxes only where they are needed.
  This new feature is somewhat experimental and feedback is appreciated.

- Fixed issue [#236](https://github.com/jsinger67/parol/issues/236)

  The grammar type information now contains a lot more details that could be useful for other
  generators like the one suggested by @ryo33
  [Type-agnostic AST generator](https://github.com/jsinger67/parol/discussions/12#discussioncomment-4153769).

  Also please consider a type browser such as [parsym](https://github.com/jsinger67/parsym) to
  easily browse the grammar type information.

## 0.26.0 - 2024-01-10

- Fixed issue [#225](https://github.com/jsinger67/parol/issues/225)
  - New function `parol::test_support::acceptor_test`
- Refactoring of grammar type generation with the goal to eventually solve issue
  [#236](https://github.com/jsinger67/parol/issues/236)
  - Moved the information about original non-terminals into [parol::generators::GrammarConfig]
  - Removed the member `user_actions` from [parol::generators::user_trait_generator::GrammarTypeInfo].
    This information is retrievable from the member `symbol_table`.
  - Moved the function `add_user_actions` from [parol::generators::grammar_type_generator::UserTraitGenerator]
    to [parol::generators::user_trait_generator::GrammarTypeInfo]
  - Extend module [parol::generators::symbol_table] with information about mutability and reference
    semantics and combine these two with the `used` flag into a struct
    [parol::generators::symbol_table::InstanceEntrails] so the information is accessible on instances
  - Made appropriate changes to the module [parol::generators::symbol_table_facade]
  - Renamed `parol-grammar.par` to `parol.par` to get rid of the somewhat duplicate notation of
    grammar which leads to names like `ParolGrammarGrammarTrait` when external tools, e.g. like
    [parol_symbols](https://github.com/jsinger67/parol_symbols), are applied to it. This tool is
    heavily used during the whole refactoring.
  - Added a function `new` to [parol::generators::grammar_type_generator::UserTraitGenerator] and
    made the `try_new` deprecated.
  - Updated documentation for the module [parol::build] and removed the vector of productions from
    the struct `Builder` because the information is now belonging to the `GrammarTypeInfo`. Also the
    `Builder` now uses `UserTraitGenerator::new` instead of `UserTraitGeneratorBuilder`.
  - Using a CommonReadMe.md file for parol workspace and parol crate

## 0.25.0 - 2023-10-22

- Using `parol_runtime` 0.20.0 which introduced braking changes. Please, see
  [CHANGELOG parol_runtime](../parol_runtime/CHANGELOG.md) for details.
- Using `rayon`'s parallel iterator in FOLLOW(k) calculation to be more scalable
- Removed clippy warnings new in Rust 1.73

## 0.24.0 - 2023-09-18

- Fixed bug in `parol new` when `--tree` option is used
- Reenable subcommand `generate` (see v0.5.3 - 2022-01-02 for original introduction)

  You can now run endless stress tests like in this example using a _powershell_ one-liner again:

  ```powershell
  while ($true) {parol generate -f ./examples/json_parser_auto/json-exp.par | Set-Content "$env:Temp/x.json"; .\target\debug\json_parser_auto "$env:Temp/x.json" -q; if (-not $?) { break } }
  ```

  Also you can use the [parol::LanguageGenerator] in your tests to achieve a similar effect.

- Fixed clippy warnings
- Providing error recovery supported by `parol_runtime` 0.19.0 in all generated parsers now!

  This is a huge improvement because parsers now usually don't stop after encountering the first
  syntax error. They now try hard to sync with the input and continue parsing.

  Related changes introduces some INCOMPATIBILITIES in error handling. Usually generating the
  parsers should help here. In case of problems please open a discussion or file an issue.

## 0.23.1 - 2023-08-12

- Fixed issue [#166](https://github.com/jsinger67/parol/issues/166) reported by
  [nblei](https://github.com/nblei)
  - Removed an aggressive optimization step in the phase of grammar transformation
    (`eliminate_duplicates`)

## 0.23.0 - 2023-08-02

- Cleaner generation of raw strings in parser source. Especially hash characters in raw string
  prefixes and postfixes are now generated only when and as much as needed. This fixes clippy message
  "unnecessary hashes around raw string literal"
- Removed clippy warnings new in Rust 1.71
- Using new version of `parol_runtime`

## 0.20.1 - 2023-07-12

- Ensure deterministic state numbering after minimization of Lookahead DFAs. Before this fix the
  states could be numbered differently after each generation. Although this imposed no problems on the
  functionality of generated parsers nondeterministic output isn't desirable.
- Fixed problem in terminal name generation when

  - regex string representation (/.../) is used
  - the terminal included characters that are automatically escaped (by `regex::escape`) and
  - the production was meant as a "primary non-terminal for a terminal".

  For more details about this special handling please see the
  [book](https://jsinger67.github.io/ParGrammar.html#terminal-name-generation).

- Removed subcommand _generate_ because of dependency conflicts

  Maybe I find a substitute for the crate `rand_regex` which is sadly no more properly maintained.

## 0.22.0 - 2023-06-09

- Fixed a smaller problem with `parol new` that occurs with special module names
- Removed clippy warnings new in Rust 1.70
- Fixed exceeded array bounds when MAX_K is used as lookahead size
- New support for handling of user defined comments (`%line_comment`, `%block_comment`)
  - This fixes issue #107 (Provide better support for language comments)

## 0.21.5 - 2023-05-29

- Fixed panic if parol is executed without arguments
- Fixed issue [#93](https://github.com/jsinger67/parol/issues/93)
  - Fully minimize Lookahead DFAs which decreases size of generated parser source and speeds up
    parsing runtime
  - It is especially important for LL(k) grammars with k > 1. For example a DFA in
    [veryl](https://github.com/dalance/veryl/blob/master/crates/parser/veryl.par)'s LL(3) grammar
    could be reduced from 8592 to 49(!) states. The parser source shortened from nearly 33.000 LOC to
    less than 15.000 LOC.

## 0.21.4 - 2023-05-19

- Provide typescript bindings for `parol`'s symbol table types. Currently these bindings are used by
  a tool called [parol_symbols](https://github.com/jsinger67/parol_symbols.git) that tries to support
  developers by enabling them to comfortably browse `parol` generated symbols for a given grammar.

## 0.21.3 - 2023-04-24

- Partially revoked bug introduced in 4dd1172 (follow_k now uses cached results from step k-1)
  - This means a small drawback in performance but this reversion is inevitable
  - 0.21.2 will therefore be yanked
- Minor performance improvement by using partition_point in `terminals_trie::Node::add_child`

## 0.21.2 - 2023-04-16 - YANKED

- `follow_k` now uses cached results from step k-1 which results in an improvement of performance
  analyzing complex grammars. In the case of [veryl](https://github.com/dalance/veryl/blob/master/crates/parser/veryl.par)
  we achieve another 25% improvement.
- Fixed clippy warnings
- Fixed warnings from `cargo doc`

## 0.21.1 - 2023-04-13

- Using an own trie implementation instead of a `HashSet<KTuple>` in `KTuples`. This considerably
  increases performance of analysis of complex grammars. For instance,
  [veryl](https://github.com/dalance/veryl/blob/master/crates/parser/veryl.par)'s grammar which is a
  k(3) grammar with 810 productions now builds on my machine in about 40 seconds instead of about 75
  seconds before. You should know that `parol` therefore solves an equation system with 1154 equations
  to fully calculate the grammar's FOLLOW(k) sets and an another equation system with 810 equations
  to fully calculate the grammar's FIRST(k) sets. But because of the strict separation of grammar
  definition and the implementation of grammar processing (basically via a generated trait) the full
  grammar analysis is only needed when changes in the grammar definitions occur.
  I currently work hard to improve the performance for such complex grammars.

## 0.21.0 - 2023-04-02

- Changed code generation to facilitate `parol_runtime`'s newer and more efficient lookahead DFA
- Optimized performance in hash implementation of KTuple

## 0.20.0 - 2023-03-21

- Most work in this release was dedicated to better performance of `parol`'s generation step
  - This includes the use of parallel computation of FIST and FOLLOW sets
  - Some minor changes in more 'inner', but public API, thus the bump to version 0.20.0
- Include fix of issue [#61](https://github.com/jsinger67/parol/issues/61) via parol_runtime 0.15.1
  - Thanks to [dalance](https://github.com/dalance) for this proposal
- Improved handling of pre-release mode in test parol_new
- Changed wrapping error type of `GrammarAnalysisError` from `anyhow::Error` to
  `parol_runtime::ParolError::UserError` for better recognition. This improves output of errors
  reported by the `parol-ls`language server.
- Removed clippy warnings new in Rust 1.68

## 0.19.0 - 2023-03-06

- Exchanged `id_tree` by `syntree`
  - This includes major API changes that have impact on user code. Please open discussions for
    migration support

## v0.18.1 - 2023-02-25

- Fixed dependencies generated by `parol new`
- Removed some clippy warnings

## v0.18.0 - 2023-02-25

- Fixed issue [#58](https://github.com/jsinger67/parol/issues/58)
- Abstracting the configuration of parser generation
- Adapted to the removal of feature `trim_parse_tree` at crate `parol_runtime`
- Enable the `trim_parse_tree` behavior in newly created parol crates by default, unless the switch
  -t (Add support for generating visualized parse trees) at parol new subcommand was given.

## v0.17.0 - 2023-02-16

- Fixed [#52](https://github.com/jsinger67/parol/issues/52) (Currently the parser doesn't complain
  about empty groups, repetitions and optionals)
- Fixed [#57](https://github.com/jsinger67/parol/issues/57) (Unreachable non-terminals error at
  parol v0.16.0)
  - The new error reporting approach is implemented for `parol new` to support standard error
    reporting for both build script and the executable itself.
- Some works regarding `parol`'s compile performance (`k_tuples.rs`)
- Improvements of `parol`'s error reporting
  - Sub commands provide the file name that was given to them (if any) to the error reporting method
    for better diagnostics.
  - Tests provide better error reports (`basic_interpreter`, `json_parser`, `json_parser_auto`)
- Extended tests in `run_examples.rs` test
  - `json_parser` and `json_parser_auto` tests are executed
- Removed clippy warnings new in Rust 1.67
- Using `RegexSet` from `regex-automata` crate as foundation of tokenizing, fixing issue
  [#56](https://github.com/jsinger67/parol/issues/56)
  - This will result in major performance boost
  - Currently unicode word boundaries are not supported, so one has to use ASCII word boundaries
    instead. Simple change occurrences of `\b` to `(?-u:\b)`.

## v0.16.0 - 2023-01-12

- Removed `miette` as error reporting crate
- The error reporting for `parol`'s binary is done with the help of `codespan_reporting` (also
  example `basic_interpreter` as PoC)
- General improvements of error handling and reporting

## v0.15.1 - 2022-12-22

- Fixed some problems during publishing process on crates.io like wrong image links and failing
  integration tests.

## v0.15.0 - 2022-12-22

- Merged PR [#34](https://github.com/jsinger67/parol/pull/34) from [ryo33](https://github.com/ryo33)
  which closes issue [#33](https://github.com/jsinger67/parol/issues/33)
  - Opt out for tracking generated parser files by git
- Merged PR [#31](https://github.com/jsinger67/parol/pull/31) from [ryo33](https://github.com/ryo33)
  which closes issue [#20](https://github.com/jsinger67/parol/issues/20)
  - use parol_runtime::once_cell instead of lazy_static
- Merged PR [#27](https://github.com/jsinger67/parol/pull/27) from [ryo33](https://github.com/ryo33)
- Merged PR [#25](https://github.com/jsinger67/parol/pull/25) from [ryo33](https://github.com/ryo33)
  which closes issue [#21](https://github.com/jsinger67/parol/issues/21)
  - This repository is now a Cargo Workspace - great infrastructural changes
  - Many supplementary tools and examples are now included in this repository
    - Still existing repositories will be marked as obsolete soon
- Fixed issue [#28](https://github.com/jsinger67/parol/issues/28)
  - Smarter type generation in auto generation - Part 1 Deduce the variant name from its right-hand
    side.
- Fixed issue [#22](https://github.com/jsinger67/parol/issues/22)
  - The parser detects conflicting token aliases pairwise and issues a dedicated error message
- Extended enhancement from issue [#19](https://github.com/jsinger67/parol/issues/19) to
  `line_comment` and `block_comment` directives
- Fixed [#39](https://github.com/jsinger67/parol/issues/39)
  - Empty scanner states are detected at the end of the parse process
- Fixed issue [#44](https://github.com/jsinger67/parol/issues/44)

  - Add a field for span data in generated types

    - `parol` can now optionally generate the `ToSpan` trait automatically for all generated AST
      types
    - Can be enabled by -r or builder configuration `range()`

      Needs to be evaluated thoroughly!

- Fixed issue [#47](https://github.com/jsinger67/parol/issues/47)
  - `parol` can now optionally insert [inner attribute](https://doc.rust-lang.org/reference/attributes.html)
    `#![allow(clippy::too_many_arguments)]` at the top of the generated trait source.
  - Can be enabled by `--inner-attributes  allow-too-many-arguments` or builder configuration
    `inner_attributes(vec![InnerAttributes::AllowTooManyArguments])`
- Fixed issue [#49](https://github.com/jsinger67/parol/issues/49)
  - Changed `parols`'s Cargo.toml in the suggested way
  - Added a new chapter [Useful tips](https://jsinger67.github.io/UsefulTips.html) that contains a
    appropriate advices
- Working on issue
  [#33 - Automate tasks like generation of derived sources i.e. in examples and run example related tests scripts](https://github.com/jsinger67/parol/issues/33)

  1. Using cargo-make for example generation

     ```shell
     cargo install cargo-make
     cargo make generate_examples
     ```

- Using [ume](https://github.com/ryo33/ume) as substitute for `bart` and `bart_derive`

  - This fixes issues [#46](https://github.com/jsinger67/parol/issues/46) and
    [#50](https://github.com/jsinger67/parol/issues/50)

    `parol` becomes more independent in the fields of code generation.
    A big _Thank you_ goes to [ryo33](https://github.com/ryo33) 👑

## v0.14.0 - 2022-11-18

- Incorporated contributions from [ryo33](https://github.com/ryo33) in both `parol` and
  `parol_runtime` crates
  - Reducing dependencies in user crates by utilizing re-exports in `parol_runtime`
  - Reducing dependencies when user crates don't use aut-generation. This was achieved by
    introducing a new feature `auto_generation` in `parol_runtime`
- Realized enhancement from issue [#19](https://github.com/jsinger67/parol/issues/19)
  - `parol` now understands different styles of terminal representations.
    - The current syntax (`"..."`) stays intact. It will behave like it was defined until now - so
      there is no need to update existing grammars.
    - New single quoted string literals (`'..'`) as literal or raw strings. `parol` will escape any
      regex meta character automatically. This is used when you don't want to deal with regexes and
      only use plain text. E.g.: `BlockBegin: '{'`
    - New regular expression strings (`/../`), behaves exactly like the old double quoted string but
      better conveys the intent. E.g.: `Digits: /[\d]+/`

## v0.13.1 - 2022-11-17

- Looked after new clippy warnings (from Rust 1.65)
- Optimized release profile
- Making the build profile an argument of our test scripts `build_parsers.ps1` and `run_parsers.ps1`
  so you can call it this way:

```powershell
> .\build_parsers.ps1 -Config release
> .\run_parsers.ps1 -Config debug
```

## v0.13.0 - 2022-10-28

- Consolidated cargo docs
- Using macros in generated adapter grammars, i.e. in auto-generation mode
  - This results in changed generated code which should be completely compatible but considerably
    smaller (in the order of 10 percent)
  - It also forces the user to have `parol-macros` in his dependencies. `parol new` subcommand is
    modified accordingly.

## v0.12.1 - 2022-10-14

- Using `parol_runtime` in version 0.8.1 now.

## v0.12.0 - 2022-10-08

_This release provides rather breaking changes to the public API. Therefore we increase minor
version number._

- Using `parol_runtime` in version 0.8.0 now. This implies some changes in token handling.
  - Access the parsed text of a token with method `text()` of the `Token` type now. Formerly you
    could access the member `symbol` directly which is not possible anymore.
  - Similarly the method to access the token's text via `ParseTree` was renamed from `symbol()` to
    `text()` in the implementation of `ParseTreeStackEntry`

## v0.11.0 - 2022-10-06

_This release provides rather breaking changes to the public API. Therefore we increase minor
version number to 11._

- Reworked recursion detection and fixed it hopefully
  - Replaced proprietary graph based algorithm with a more conventional one
  - Added plenty of tests
- Switched to clap 4
- Removed prettyplease option
  - Opting clearly for rustfmt now

## v0.10.7 - 2022-09-14

- Launching the book as central documentation for `parol`

## v0.10.6 - 2022-08-11

- Fixed a missing newline in between multiple user type definitions
- Better handling of errors from grammar analysis to support parol language server
- Changed decoration format of production attributes

## v0.10.5 - 2022-08-03

- Update reference of `parol_runtime` to v0.7.2
- Fixed display format of non-terminals with attached user types
- Fixed generation of parol grammars (i.e. as expanded grammar) so that user types are correctly
  presented

## v0.10.4 - 2022-08-02

- Improved logo. Texts have been converted to curves to ensure equal design on all systems.
- Update [docs\ParGrammar.md](docs\ParGrammar.md) to fit the new features of `parol`'s input grammar.
- Improved auto-generation:
  - `parol` can now handle AST types without lifetime references to the scanned text.
  - See changes in `list_auto` example

## v0.10.3 - 2022-07-09

- `parol new` can now enable parse tree visualization in newly created crates. You can activate it
  by adding the new argument `-t`.

> `parol help new`

- Update reference of `parol_runtime` to v0.7.1

## v0.10.2 - 2022-07-08

- New artwork - fixing issue [#15](https://github.com/jsinger67/parol/issues/15)
- Supporting user defined types by a dedicated `%user_type` directive which allows you to define
  aliases for possibly complex user defined types:

  > %user_type Number = crate::list_grammar::Number

  allows you to refer via the short name to the complex user type:

<!-- markdownlint-disable Reference links and images should use a label that is defined -->

> Num: "0|[1-9][0-9]\*": Number;

<!-- markdownlint-enable Reference links and images should use a label that is defined -->

Please see example `list_auto` for an use case.

## v0.10.1 - 2022-07-01

- Feature 'User defined symbol types' completed
  - You can now define User defined types on non-terminal symbols too. Please, see example
    `list_auto` for a first impression.
- Remove `init` function from user's GrammarTrait in `parol new` to fit `parol_runtime` v0.6.0
  - The file name is now available at each token and thus we don't need to convey it in an `init`
    function.
- Repair `parol new` when it's supposed to generates library crates.
- `parol` is now the default binary run when using `cargo run`.

  You can use

  > cargo run -- ...

  instead of

  > cargo run --bin parol -- ...

  now.

## v0.10.0 - 2022-06-24

A lot of breaking changes.

- Use `parol_runtime` v0.6.0
- Refactoring of symbol table
- Start with new feature 'User defined symbol types'

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

- Added possibility to clip grammar symbols in the grammar description by suffixing them with an
  optional cut operator (`^`). This instructs `parol` in auto-generation mode to not propagate this
  symbol to the AST types. This can simplify and shorten the generated code dramatically.

  > The symbol `^` for the cut operator is chosen in the style of [oak](https://github.com/ptal/oak)'s
  > [invisible type](http://hyc.io/oak/typing-expression.html).

- I applied this ability in the example grammars that uses auto-gen and in `parol`'s grammar itself.
- Adapt documentation

## v0.9.3 - 2022-06-05

- Fixed allow(unused_imports) directive
- Added some test files to git which are missing yet which causes `run_parsers.ps1` to fail
- `parol` is now implemented using the auto-generation approach
  - This is the basis for further improvements like user defined symbol types or clipping of AST
    content because grammar changes are likely. Then such changes won't have much influence on the
    grammar processing code.

## v0.9.2 - 2022-06-01

- Worked on tutorial
- Fixed case in crate name generation in subcommand `parol new`
- Merged fix for #16 - Thanks a lot to [mobotsar](https://github.com/mobotsar)

## v0.9.1 - 2022-05-28

- The auto-generation is now able to generate true option types. This improves this feature a lot
  and only now one can say it is quite complete.

## v0.9.0 - 2022-05-27

- Worked on tutorial
- Changes in `parol new`:

  - The referenced version of `parol` is taken from CARGO_PKG_VERSION environment variable. If the
    current version is not released yet on [crates.io](https://crates.io/crates/parol) you can fallback to github:

    ```toml
    [build-dependencies]
    parol = { git = "https://github.com/jsinger67/parol.git" }
    ```

  - The parsed data is now printed to standard output automatically.
  - The `init` function is implemented with default handling.
  - A file with test input data (`test.txt`) is also created automatically.

- Removed serialization support - no use case anymore
- Removed some useless derives
- Took over some improvements from branch `optionals`
- Function `left_factor` now correctly transfers ProductionAttributes. This is a small part of the
  fix of the bug described next.
- New bug in auto-generation detected and fixed:
  - Using an optional expression within a repetition confused the type generation.
    So constructs like `{ [A] B }` didn't work correctly.
  - The fix includes major changes in grammar transformation, especially the way optional
    expressions are handled. I therefore _increment the minor version to nine_ to indicate a rather
    breaking change.

## v0.8.3 - 2022-05-14

- Fixed comments on generated user actions
- Avoid possible name clashes on user action names with the `init` function
- Worked on tutorial

## v0.8.2 - 2022-05-11

- Using updated version of `function-name` crate to fix the raw identifier problem occurred at
  context generation

## v0.8.1 - 2022-05-08

- Minor cleanups
- Fixes and updates in documentation
- Fixed `parol left-factor` subcommand. The result is now printed as expected.
- Fixed compile error in crates generated by `parol new` subcommand, when module name contains
  invalid characters.
- Using `named` marco from the crate `function-name` for the `context` variable in generated
  semantic actions. This automatically keeps the context name in sync with the function name.

## v0.8.0 - 2022-05-06

- Removed some cases where type name collisions occurred
  - This involved considerable refactoring of grammar type generation
  - Another effect of these changes is that the generated source contains names of types and
    arguments that are more catchy and don't always contain suffixes like "\_0" etc. Also the resulting
    code should be more robust against changes in your grammar. The downside is that all user code has
    to be adapted to the new generated names.
    We therefore increment the minor version to eight to indicate a rather breaking change.
- Improved change detection of builder to only trigger build script on changed grammar description
- If you used the auto-generation functionality of `parol` it is strongly recommended to switch over
  to this new ^0.8 version.

## v0.7.0 - 2022-04-17

- Changed generated semantic action names

  To be more invariant when changing a grammar description the names don't include the production
  number anymore. Instead I generate a relative index which only changes potentially within a
  certain non-terminal.

  Note that this change needs a manual readjustment of already used
  code. Sorry for this inconvenience. But this change generally results in better maintainability.

  We therefore increment the minor version to seven to indicate a rather breaking change.

- Added a new tutorial which is still under construction

  It describes the new approach available since auto-generation is implemented.

  The old tutorial is moved to [TutorialOld.md](./docs/TutorialOld.md). It is still useful and
  explains the approaches that are now superseded by the new auto-generation related ones.

## v0.6.2 - 2022-04-03

- Add new subcommand `new`
  - Use this to create new crates that use `parol` as parser generator
  - It can generate both binary and library crates
  - It needs `cargo` as well as `cargo-edit` to be installed

## v0.6.1 - 2022-03-31

- Changes regarding the new auto-generation feature
  - Added new examples `list_auto` and `calc_auto`, that uses this new feature
    - You can compare them with the examples `list` and `calc` which use the traditional method.
  - Modified code generation for auto-generation (clippy)
  - More efficient call method of user actions (by reference)
  - Fixed a name clash between a popped AST value and the built result value in auto-generated actions
  - Using Token<'t> instead of OwnedToken in generated code now for performance reasons
    - This requires `parol_runtime` crate with version v0.5.9 now
- Partly reworked documentation

## v0.6.0 - 2022-03-20

- Added new experimental auto-generation feature is available now
  - Documentation still has to be added.
  - There exists a new example that uses this feature here: [JSON parser auto](https://github.com/jsinger67/json_parser_auto.git).
  - The old behavior is still intact and should be usable without restrictions.

## v0.5.10-pre - Not separately released, but included in 0.6.0

- Refactoring of module user_trait_generator
  - Changed from a bunch of functions to a struct `UserTraitGenerator` with `impl`.

## v0.5.9 - 2022-02-19

- Updated some dependencies and referenced some crates with caret requirements in semver.
  - Most prominent change was to reference `miette ^4.0` now.
  - Also `parol_runtime` is referenced with a new version (0.5.6).
- Using derive_builder to handle `bart` template data
  - The use of builder pattern shall be extended in the future
- More robust name generation with check against Rust keywords
- Enable use of `prettyplease` instead of `rustfmt` for code formatting.
  - This is enabled by non-default feature "pretty".
  - Also note that this is still experimental and the result of code formatting by `prettyplease` is
    currently not optimal. Mostly because of suppressed comments. Therefore I don't encourage to use
    this feature yet.

## v0.5.8 - 2022-02-03

- Included PR #13: _Clap 3.0 (derive + builder styles)_. **_Thanks a lot to oaleaf._**
  This closes Issue #10

## v0.5.7 - 2022-01-22

- New examples [Keywords](./examples/keywords) and [Keywords2](./examples/keywords2) to demonstrate
  handling of keywords in `parol`'s scanner
- Compiling more test grammars in `run_parsers.ps1`. Also negative cases are checked.
- Factored out grammar transformation from the parser to the module transformation

## v0.5.6 - 2022-01-10

- Even better integration of tools, i.e. subcommands with `clap`. Preparation for planned switch
  over to `clap v3`.
- Fixed issue #4: _It appears the --only-lookahead option (-c) doesn't work_. This option is useless
  and was removed.
- Builder: Write out a preliminary version of the expanded grammar after parsing to support grammars
  that fail later checks.
- Added CONTRIBUTING.md
- Consolidated Public API (fixes #11)
- Updated documentation
  - Using `parol` like an installed tool in example invocations instead of
    `cargo run --bin parol -- ...` now
  - Fixed links in cargo's doc output
- Improved termination behavior of the language generation feature (`parol generate`) introduced in
  v0.5.3
- Improved error report (Undeclared variable) in example `calc`

## v0.5.5 - 2022-01-05

- Included PR #8: _Rename default actions file from grammar.rs -> grammar_trait.rs_. **_Thanks a lot
  to Techcable_**

## v0.5.4 - 2022-01-05

- Fixed a serious bug in parsing groups, repetitions and optionals introduced in commit [6476e75].
- Started issuing more detailed miette-like errors from parol itself.
- Fixed an invalid generation of the %pop() instruction from '%pop' to '%pop()'.
- More tests to check the parol parser's internal representation.
- Fixed some problems related to platform specific newline characters.
- Fixed issues #5: _Bizarre error running scanner_states_. **_Thanks a lot to Techcable_**
- Included PR #6: _Add API to invoke parol from build scripts_. **_Thanks a lot to Techcable_**

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

Also you can run endless stress tests like in this example using a _powershell_ one-liner:

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
