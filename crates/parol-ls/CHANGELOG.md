# Change Log

All notable changes to the "parol-ls" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this
file.

## 2.0.0 - Not released yet

* Switched to versions `parol_runtime@2.0.0` and `parol@2.0.0`
* No functional changes.

## 1.0.0 - 2024-09-02

* No changes, just a re-release with new version

## 0.21.0 - 2024-06-21

* Fixed formatting problem with comments before the `%start` directive.
* Using new `parol_runtime@0.24` and `parol@0.31` which are providing fixes for recovery of LL
parsers

## 0.20.0 - 2024-06-08

* Diagnostic warnings on automatically resolved conflicts

## 0.19.1 - 2024-05-19

* Better presentation of multiple errors
* Enhanced support for scanner states and especially the %on/%enter directives in document outline.
  * Goto primary non-terminals
  * Rename primary non-terminals
  * Hover of primary non-terminals
  * Goto scanner states
  * Rename scanner states (except INITIAL)
* Goto scanner states from token with state
* Renaming of Scanner states from token with state and scanner definitions itself
* Hover of Scanner states from token with state
* Some inner refactoring
* Bump to `lsp-types@0.96.0`

## 0.19.0 - 2024-05-16

* Support of new scanner switching of `parol_runtime@0.22` and `parol@0.29`

## 0.18.0 - 2024-04-29

* Support grammar type specification in par files with the `%grammar_type` directive
* Support of new `parol` errors

## 0.17.1 - 2024-01-10

- Update references to `parol_runtime@0.20.1` and `parol@0.26`. Please, see
  [CHANGELOG parol_runtime](../parol_runtime/CHANGELOG.md) and
  [CHANGELOG parol](../parol/CHANGELOG.md) for details.

## 0.17.0 - 2023-10-22

- Update references to `parol_runtime@0.20` and `parol@0.25`. Please, see
  [CHANGELOG parol_runtime](../parol_runtime/CHANGELOG.md) and
  [CHANGELOG parol](../parol/CHANGELOG.md) for details.

## 0.16.0 - 2023-09-18

- This release is dedicated to the new error recovery feature in generated parsers.
  The language server itself benefits from this feature and can continue parsing of par files beyond
  the first syntax error. This should improve the overall user experience.

## 0.15.0 - 2023-08-02

- Refactoring of formatting algorithm using the new comment handling feature provided by
  `parol_runtime`. If you experience strange shifts of comments please file an issue.

## 0.14.0 - 2023-07-12

- This release is dedicated to code formatting. A lot of tests have been added to ensure a more
  consistent formatting over different configuration settings.

Please file an issue against formatting if you encounter any problem or have suggestions.

In later releases I plan to refactor the whole formatting algorithm and incorporate the comment
handling provided by `parol_runtime@0.17`.

## 0.13.0 - 2023-06-25

- The `Parol Language Server` now supports a single end comment.
  - This is no restriction of `parol` itself but of the special grammar the language server uses.
    This grammar captures as much comments as possible from the input grammar.
    Feedback is appreciated.
- Support of several new formatting options

  - formatting.empty_line_after_prod
    - Add an empty line after each production
  - formatting.prod_semicolon_on_nl
    - Place the semicolon after each production on a new line
  - formatting.max_line_length
    - Maximum number of characters per line

  This requires vs-code extension `parol-vscode` of version >= 0.1.15.

- Fix of minor formatting behavior problems, like handling of nested EBNF constructs
  - This fixes [#114](https://github.com/jsinger67/parol/issues/114)

## 0.12.0 - 2023-06-09

- Using newer `parol 0.22.0` + `parol_runtime 0.17.0`

## 0.11.0 - 2023-04-24

- Merged PR [#78](https://github.com/jsinger67/parol/pull/78) provided by [AumyF](https://github.com/AumyF).
- Using bug fixed version `parol 0.21.3`

## 0.10.0 - 2023-04-13

- Using newer and faster `parol 0.21.1`

## 0.9.0 - 2023-04-02

- Decoupled the expensive LL(k) analysis to speed up the language server
- Using newer and faster `parol 0.21.0` + `parol_runtime 0.16.0`

## 0.8.0 - 2023-03-21

- Using newer and faster `parol 0.20.0` + `parol_runtime 0.15.1`

## v0.7.0 - 2023-03-06

- Using newer and faster `parol 0.19.0` + `parol_runtime 0.15.0`

## v0.6.0 - 2023-02-16

- Using newer and faster `parol 0.18.0` + `parol_runtime 0.14.0`

## v0.5.0 - 2023-02-16

- Removed clippy warnings new in Rust 1.67
- Using newer and faster `parol 0.17.0` + `parol_runtime 0.13.0`

## v0.4.0

- Removed `miette` as opaque error handling crate and substituted it by `thiserror` + `anyhow`
- General improvements of error handling and reporting

## v0.3.4

- Infrastructural changes: Moved repository into parol workspace

## v0.3.3

- Support for new parol features from 0.14.0 (new terminal representation forms)
- Improved document formatting

## v0.3.2

- Configuration is now possible via settings in VSCode. Changes are updated in the language server
  now.
  - The first effective property is the maximum number of lookahead tokens that is calculated
    during grammar analysis.

## v0.3.1

- Using `parol` in version 0.13.0 now to take advantage of latest fixes. Also add `parol-macros` as
  dependency. This reduced the code size of generated parser.

## v0.3.0

- Using `parol_runtime` in version 0.8.1 and `parol`in version 0.12.1 now to take advantage of
  latest fixes.
- Rethought the concept of `OwnedToken`
- Format document: Improved formatting of line comments

## v0.2.0

- Switched to clap 4
- Using parol 0.11.0 now
  - Implied some changes in error handling

## v0.1.12

- Support for "Format document"
  - Currently no configuration available, i.e the formatting result is a standard format. That may
    change in the future.

## v0.1.11

- Fixed handling of prepare rename request which sometimes rejected the rename action

## v0.1.10

- Add support for renaming non-terminal symbols
- Fixed location_to_range

## v0.1.9

- Refactored the request handling
- Made the maximum lookahead configurable via server arguments
- Added Acknowledgements to README
- Fixed flaw in grammar analysis
- Add support for document symbols

## v0.1.8

- Fighting the position to offset problem again
- Clearing document state now before each analysis
- Allowing comments after alternation separator

## v0.1.7

- Hopefully now finally fixed crash of language server on Linux machines

## v0.1.6

- Fixed crash of language server on Linux machines

## v0.1.5

- Fixed range calculation to be more OS aware, hopefully
- Added tests and ran them on Windows and Linux (on WSL2)

## v0.1.4

- Add basic Hover support

## v0.1.3

- GotoDefinition can be applied to the Start symbol too
- First attempt to handle comments by the grammar and preserve them
  - This is not complete yet and may be a little bumpy - please report any misbehavior

## v0.1.2

- Include more grammar checks

## v0.1.1

- Fixed utils::source_code_span_to_range

## v0.1.0

- Initial release
