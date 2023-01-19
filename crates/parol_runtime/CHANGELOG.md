# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

Be aware that this project is still v0.y.z which means that anything can change anytime:

>"4. Major version zero (0.y.z) is for initial development. Anything MAY change at any time. The public API SHOULD NOT be considered stable."
>
>(Semantic Versioning Specification)

## Indicating incompatible changes on major version zero

We defined for this project that while being on major version zero we mark incompatible changes with
new minor version numbers. Please note that this is no version handling covered by `Semver`.

## 0.12.2 - Not released yet

* New benches to measure performance of tokenizer

## v0.12.1 - 2023-01-12

* Removed clippy warning

## v0.12.0 - 2023-01-12

* Removed `miette` as error handling
* General improvements of error handling
* Fixed the problem that regex for white spaces consumed newline characters
* Fixed issue [#54](https://github.com/jsinger67/parol/issues/54)
  * In `TokenStream` the size of the lookahead buffer is always at least 1

## v0.11.2 - 2022-12-22

* Changed repository reference to the [new location](https://github.com/jsinger67/parol/tree/main/crates/parol_runtime)
* Otherwise fully compatible with version 0.11.1

## v0.11.1 - 2022-12-22

* Merged PR [#43](https://github.com/jsinger67/parol/pull/43) from [ryo33](https://github.com/ryo33)
  * Use \s for WHITESPACE_TOKEN
* Supporting Span information for `parol`'s new feature to generate span calculation

## v0.11.0 - 2022-11-29

* Using `derive_builder` in version 0.12.0 now so that we can use re-export decently.

## v0.10.0 - 2022-11-28

* Reexporting once_cell now

## v0.9.0 - 2022-11-16

* Merged PR #2 from ry033. Kudos 👍
  * This introduces a new feature "auto_generation" that should be enabled for crates that use
  `parol`'s auto generation mode. If you don't know exactly what this is, please enable this
  feature! I consider to make it a default feature in future release.

## v0.8.1 - 2022-10-14

* `Token`: Fixed the method `to_owned` and added a method `into_owned`.

## v0.8.0 - 2022-10-12

*This release introduces breaking changes to the public API. To indicate this we increase minor
version number.*

* Removed `OwnedToken` type and used `Cow` to hold the scanned text in `Token`s instead. Anyway this
member is private and can only be accessed via method `text()`. See below for more on this new
method.
* The `Token`'s constructor method `with` had a change in the type of the text parameter which
should be fairly easy to adapt in user code.
* The `Token`'s `to_owned` method returns a `Token` now.
* The parsed text of a token can now be accessed via method `text()` of type `Token` now. Formerly
you used the member `symbol` directly which is not possible anymore.
* Similarly the method to access the token's text via `ParseTree` was renamed from `symbol()` to
`text()` in the implementation of `parser::ParseTreeStackEntry`
* The types `errors::FileSource`, `lexer::Location` and `lexer::TokenIter` now internally use a
`Cow<Path>` for holding the file name instead of a more expensive `Arc<PathBuf>`. This was
originally chosen because of the necessity of `miette::SourceCode` to be `Send + Sync`. But the Cow
will do the same with much less effort.
  * These changes effect user code due to changes in the methods `try_new` of `errors::FileSource`,
`with` of `lexer::Location` and `new` of `lexer::TokenIter`

## v0.7.2 - 2022-08-03

* Better diagnostics to support parol language server
* Changed display format of `Location` to match vscode's format
* Improved traces

## v0.7.1 - 2022-07-09

* Fixed a bug in TokenStream::push_scanner
* Improved debugging support for error `pop from an empty scanner stack`.
* New error type `ParserError::PopOnEmptyScannerStateStack`
* Made `ParseType` a `Copy`

## v0.7.0 - 2022-07-05

* Using miette 0.5.1 now
* Also updated some other crate references

## v0.6.0 - 2022-06-24

This version brings rather breaking changes:

* Provide each token with the file name
* Thus the init method could be removed from `UserActionsTrait`.
* Factored out the location information form the token types into a separate `Location` struct.

## v0.5.9 - 2022-03-31

* Add explicit lifetimes in `UserActionsTrait` to aid the use of Token<'t> in `parol`'s auto-generation feature.

## v0.5.8 - 2022-03-24

* New test for scanner state switching and the consistence of `miette::NamedSource` which is
produced from token stream and token span.
* `TokenStream::ensure_buffer` is called at the end of `TokenStream::consume` to have a more
consistent behavior of `TokenStream::all_input_consumed`

## v0.5.7 - 2022-03-09

* Optimized creation of errors::FileSource using the TokenStream

## v0.5.6 - 2022-02-19

* Referencing `miette ^4.0` now.

## v0.5.5 - 2022-02-03

* Better formatting of file paths
* Revived `OwnedToken` type for auto-generation feature of `parol`

## v0.5.4 - 2022-01-08

* As of this version a detailed changelog is maintained to help people to keep track of changes that
have been made since last version of `parol_runtime`.
* A new (non-default) feature `trim_parse_tree` was added. The feature `trim_parse_tree` is useful
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
