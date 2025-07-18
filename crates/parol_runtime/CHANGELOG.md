# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 4.0.0 - Not released yet

* Official start of version 4 development.
* Switch to scnr2 scanner crate
* As a result feature `regex-automata` is not supported anymore

## 3.1.0 - 2025-07-11

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

* Add more benchmarks to access throughput.

## 3.0.0 - 2025-03-08

* With this release the transition to version 3.0 is completed
* Switched to edition 2024
* Fixed [#595](https://github.com/jsinger67/parol/issues/595)
  * This fix is the same as on version `2.2.1` on branch `release2.2` but is based on main

## 2.2.0 - 2025-02-13

- Introduced new crate feature `regex_automata` to configure `scnr` crate to use an alternative
regex engine.

  You can enable this feature this way:

  ```toml
  parol_runtime = { version = "2.2.0", default-features = false, features = [ "regex_automata" ] }
  ```

  For more details on the effects on scanner's behavior you can have a look at the
  [CHANGELOG](https://github.com/jsinger67/scnr/blob/main/CHANGELOG.md#080---2025-02-12) and the
  regarding section in the
  [README](https://github.com/jsinger67/scnr/blob/main/README.md#the-feature-regex_automata) of the
  `scnr` crate 

## 2.1.1 - 2025-01-21

- Fix for [#558](https://github.com/jsinger67/parol/issues/558)
  This fix updates the `last_consumed_token_end_pos` in `TokenStream::take_skip_tokens` on skip
  tokens, too

## 2.1.0 - 2025-01-17

- Fixed a subtle bug related to token buffer handling after scanner mode switching near the end of
the input
- Using new `scnr` version 0.7.0 due to increased performance

## 2.0.0 - 2024-12-25

>Please note, that changes made in version 2 are also detailed in an extra
[chapter of the book](https://jsinger67.github.io/ParolVersion2.html).

- Integration of scanner crate `scnr`. `parol_runtime` now uses this crate as scanner crate instead
of regexes created with the help of the `regex-automata` crate. I hope that this way we can better
fulfill the specific needs in context of tokenization. On the other hand I'm aware that we will
surely loose some comfort. I'm curious how things will work out here. Please give feedback on any
problems you encounter.

- The changes coming with this switch to `scnr` lead to a lot of changes internally and in the public
interface as well as in the behavior of generated parsers from the perspective of tokenization.
Thus the bump in the major version. 
Especially regarding these differences in behavior please have a look at the `scnr`'s 
[README](https://github.com/jsinger67/scnr/blob/main/README.md).

- `<UserType>GrammarTrait::on_comment_parsed` is renamed to `<UserType>GrammarTrait::on_comment` for
clarity.

- Support for vanilla mode has been discontinued. The related feature `auto_generation` became
pointless and has therefore been removed.

- The version 1 will be supported and updated regularly on branch `release1.0`, so you aren't forced
to switch to version 2 any time soon.

- Error recovery on generated LL(k) parsers can now be disabled.

## 1.0.0 - 2024-09-02

- Fixed clippy warnings new in Rust 1.80.0

>**The version 1 is maintained on branch `release1.0`. All changes for this version therefore are
 only visible on this branch. This includes this change log too.**

## 0.24.1 - 2024-06-24

- Fix issue [#357](https://github.com/jsinger67/parol/issues/357)

## 0.24.0 - 2024-06-21

- LR parser: Outputting current scanner in error message
- Fixed problem with deterministic and termination of the recovery process in generated LL parsers
  * Add new errors related to recovery, thus minor version bump

## 0.23.0 - 2024-06-06

- Fix default settings for enabling parse tree generation in `LRParser`
- Optimize memory consumption in case parse tree generation is disabled in `LRParser`
- Improved load performance of LRParsers by using static array as data

## 0.22.0 - 2024-05-16

- Provide parse tree generation for LRParser
- Implement scanner-based scanner switching which can be used with LL(k) and LALR(1) parsers
- Public API has changed and fits to `parol` >= 0.29

## 0.21.0 - 2024-04-29

- New parser type to foster LALR(1) grammar support of `parol` 0.28.0

## 0.20.2 - 2024-03-21

- Fixed issue [#310 Access internal data of TokenVec](https://github.com/jsinger67/parol/issues/310)

  I extended the implementation on `TokenVec`. It now provides a `get` method and an `iter` method.
```rust
/// A vector of tokens in a string representation
#[derive(Debug, Default)]
pub struct TokenVec(Vec<String>);

impl TokenVec {
    /// Pushes a token to the vector
    pub fn push(&mut self, token: String) {
        self.0.push(token);
    }

    /// Returns an iterator over the tokens
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.0.iter()
    }

    /// Returns a token at the given index
    pub fn get(&self, index: usize) -> Option<&String> {
        self.0.get(index)
    }
}
```


## 0.20.1 - 2024-01-10

- Refactor `parol_runtime::parser::LLKParser::adjust_token_stream` that is used in error recovery

## 0.20.0 - 2023-10-22

- Improved performance in scanner
  - Imposes some BREAKING CHANGES in types `Location`, `TokenIter`, `TokenStream` and `FileSource`
- Removed warnings in generated sources which were issued by `cargo doc`

## 0.19.0 - 2023-09-18

- Please note that this version is incompatible with previous versions and works only with
  `parol` >= 0.24
- Providing location information on EOI tokens now to support error reporting
- Supports basic error recovery strategies in generated errors
  - Token mismatch and production prediction both handles synchronization of input token stream
    with expected input to enable further parsing

## 0.18.0 - 2023-08-02

- To minimize the size of tokens the types of some members of `Token` have been changed from usize
  to u32.
  - This is a BREAKING CHANGE! Sorry for inconvenience.
- To support the new comment handling feature more generally I added a new member
  `Token::token_number` which is actually an index. So if you use tokens provided by
  `<UserType>GrammarTrait::on_comment_parsed` you can now determine where exactly the comment token
  has been scanned in the input relatively to other normal tokens.

## 0.17.1 - 2023-07-12

- Update crate `regex-automata` to version 0.3.2

## 0.17.0 - 2023-06-09

- New support for handling of user defined comments (`%line_comment`, `%block_comment`)
  - This library works in conjunction wit `parol` >= 0.22.0 to work properly
  - The new method `<UserType>GrammarTrait::on_comment_parsed` is called in order of appearance each
    time before the parser consumes a normal token from token stream.
  - It is default implemented and the user can provide an own implementation if she is interested in
    comments.
  - This is a minimal support but can greatly improve the usability. Feed is appreciated.

## 0.16.0 - 2023-04-02

- More efficient implementation of lookahead DFA
  - This can also lead to smaller generated parser files up to about 5 percents

## 0.15.1 - 2023-03-21

- Add new features to support static disabling of log levels during compile time (see issue
  [#61](https://github.com/jsinger67/parol/issues/61))
  - Thanks to [dalance](https://github.com/dalance) for this proposal

## 0.15.0 - 2023-03-06

- Exchanged `id_tree` by `syntree`
  - This includes major API changes that have impact on user code. Please open discussions for
    migration support

## 0.14.0 - 2023-02-25

- Filled some missing source documentations
- Fixed issue [#58](https://github.com/jsinger67/parol/issues/58)
  - ATTENTION ! Incompatible change !
  - Removed feature `trim_parse_tree`
  - Enable trimming of parse tree in build script by calling `trim_parse_tree` on the builder object

## 0.13.0 - 2023-02-16

- New benches to measure performance of tokenizer
- Using `RegexSet` from `regex-automata` crate as foundation of tokenizing
  - This will result in major performance boost
  - Currently unicode word boundaries are not supported, so one has to use ASCII word boundaries
    instead. Simple change occurrences of `\b` to `(?-u:\b)`.

## v0.12.1 - 2023-01-12

- Removed clippy warning

## v0.12.0 - 2023-01-12

- Removed `miette` as error handling
- General improvements of error handling
- Fixed the problem that regex for white spaces consumed newline characters
- Fixed issue [#54](https://github.com/jsinger67/parol/issues/54)
  - In `TokenStream` the size of the lookahead buffer is always at least 1

## v0.11.2 - 2022-12-22

- Changed repository reference to the [new location](https://github.com/jsinger67/parol/tree/main/crates/parol_runtime)
- Otherwise fully compatible with version 0.11.1

## v0.11.1 - 2022-12-22

- Merged PR [#43](https://github.com/jsinger67/parol/pull/43) from [ryo33](https://github.com/ryo33)
  - Use \s for WHITESPACE_TOKEN
- Supporting Span information for `parol`'s new feature to generate span calculation

## v0.11.0 - 2022-11-29

- Using `derive_builder` in version 0.12.0 now so that we can use re-export decently.

## v0.10.0 - 2022-11-28

- Reexporting once_cell now

## v0.9.0 - 2022-11-16

- Merged PR #2 from ry033. Kudos 👍
  - This introduces a new feature "auto_generation" that should be enabled for crates that use
    `parol`'s auto generation mode. If you don't know exactly what this is, please enable this
    feature! I consider to make it a default feature in future release.

## v0.8.1 - 2022-10-14

- `Token`: Fixed the method `to_owned` and added a method `into_owned`.

## v0.8.0 - 2022-10-12

_This release introduces breaking changes to the public API. To indicate this we increase minor
version number._

- Removed `OwnedToken` type and used `Cow` to hold the scanned text in `Token`s instead. Anyway this
  member is private and can only be accessed via method `text()`. See below for more on this new
  method.
- The `Token`'s constructor method `with` had a change in the type of the text parameter which
  should be fairly easy to adapt in user code.
- The `Token`'s `to_owned` method returns a `Token` now.
- The parsed text of a token can now be accessed via method `text()` of type `Token` now. Formerly
  you used the member `symbol` directly which is not possible anymore.
- Similarly the method to access the token's text via `ParseTree` was renamed from `symbol()` to
  `text()` in the implementation of `parser::ParseTreeStackEntry`
- The types `errors::FileSource`, `lexer::Location` and `lexer::TokenIter` now internally use a
  `Cow<Path>` for holding the file name instead of a more expensive `Arc<PathBuf>`. This was
  originally chosen because of the necessity of `miette::SourceCode` to be `Send + Sync`. But the Cow
  will do the same with much less effort.
  - These changes effect user code due to changes in the methods `try_new` of `errors::FileSource`,
    `with` of `lexer::Location` and `new` of `lexer::TokenIter`

## v0.7.2 - 2022-08-03

- Better diagnostics to support parol language server
- Changed display format of `Location` to match vscode's format
- Improved traces

## v0.7.1 - 2022-07-09

- Fixed a bug in TokenStream::push_scanner
- Improved debugging support for error `pop from an empty scanner stack`.
- New error type `ParserError::PopOnEmptyScannerStateStack`
- Made `ParseType` a `Copy`

## v0.7.0 - 2022-07-05

- Using miette 0.5.1 now
- Also updated some other crate references

## v0.6.0 - 2022-06-24

This version brings rather breaking changes:

- Provide each token with the file name
- Thus the init method could be removed from `UserActionsTrait`.
- Factored out the location information form the token types into a separate `Location` struct.

## v0.5.9 - 2022-03-31

- Add explicit lifetimes in `UserActionsTrait` to aid the use of Token<'t> in `parol`'s auto-generation feature.

## v0.5.8 - 2022-03-24

- New test for scanner state switching and the consistence of `miette::NamedSource` which is
  produced from token stream and token span.
- `TokenStream::ensure_buffer` is called at the end of `TokenStream::consume` to have a more
  consistent behavior of `TokenStream::all_input_consumed`

## v0.5.7 - 2022-03-09

- Optimized creation of errors::FileSource using the TokenStream

## v0.5.6 - 2022-02-19

- Referencing `miette ^4.0` now.

## v0.5.5 - 2022-02-03

- Better formatting of file paths
- Revived `OwnedToken` type for auto-generation feature of `parol`

## v0.5.4 - 2022-01-08

- As of this version a detailed changelog is maintained to help people to keep track of changes that
  have been made since last version of `parol_runtime`.
- A new (non-default) feature `trim_parse_tree` was added. The feature `trim_parse_tree` is useful
  if performance is a goal and the full parse tree is not needed at the end of the parse process.
  You can activate this feature in your dependencies with this entry

      ```toml
      parol_runtime = { version = "0.5.5", default-features = false, features = ["trim_parse_tree"] }
      ```

      The parse tree returned from `LLKParser::parse` contains only the root node and is therefore

  useless if the feature is activated. Also note that you can't access the children of the nodes
  provided as parameters of your semantic actions (each of type `&ParseTreeStackEntry`) because they
  don't have children anymore. Therefore to navigate them will fail.

      This fixes issue (enhancement) #1
