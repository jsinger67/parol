# Change Log

All notable changes to the "parol-ls" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this
file.

## v0.3.0

* Using `parol_runtime` in version 0.8.1 and `parol`in version 0.12.1 now to take advantage of
latest fixes.
* Rethought the concept of `OwnedToken`
* Format document: Improved formatting of line comments

## v0.2.0

* Switched to clap 4
* Using parol 0.11.0 now
  * Implied some changes in error handling

## v0.1.12

* Support for "Format document"
  * Currently no configuration available, i.e the formatting result is a standard format. That may
  change in the future.

## v0.1.11

* Fixed handling of prepare rename request which sometimes rejected the rename action

## v0.1.10

* Add support for renaming non-terminal symbols
* Fixed location_to_range

## v0.1.9

* Refactored the request handling
* Made the maximum lookahead configurable via server arguments
* Added Acknowledgements to README
* Fixed flaw in grammar analysis
* Add support for document symbols

## v0.1.8

* Fighting the position to offset problem again
* Clearing document state now before each analysis
* Allowing comments after alternation separator

## v0.1.7

* Hopefully now finally fixed crash of language server on Linux machines

## v0.1.6

* Fixed crash of language server on Linux machines

## v0.1.5

* Fixed range calculation to be more OS aware, hopefully
* Added tests and ran them on Windows and Linux (on WSL2)

## v0.1.4

* Add basic Hover support

## v0.1.3

* GotoDefinition can be applied to the Start symbol too
* First attempt to handle comments by the grammar and preserve them
  * This is not complete yet and may be a little bumpy - please report any misbehavior

## v0.1.2

* Include more grammar checks

## v0.1.1

* Fixed utils::source_code_span_to_range

## v0.1.0

* Initial release
