# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

Be aware that this project is still v0.y.z which means that anything can change anytime:

>"4. Major version zero (0.y.z) is for initial development. Anything MAY change at any time. The public API SHOULD NOT be considered stable."
>
>(Semantic Versioning Specification)

But we try to mark incompatible changes with a new minor version.

## v0.8.0 - 2022-10-08

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
`text()` in the implementation of `ParseTreeStackEntry`

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
