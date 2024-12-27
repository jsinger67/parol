# Change Log

All notable changes to the "parol-vscode" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## v0.3.2 - 2024-12-27

* Included lookahead operators into the syntax highlighting

## v0.3.1 - 2024-12-25

* Fixed regexes for Regex and String terminals 

## v0.3.0 - 2024-05-16

* Add new keywords "%on" and "%enter" to syntax.

## v0.2.0 - 2024-04-01

* Support grammar type specification in par files with the `%grammar_type` directive
* Little fix in tm-grammar specification regarding regex-like quoted strings

## v0.1.15

* Support of several new formatting options
  * formatting.empty_line_after_prod
    * Add an empty line after each production
  * formatting.prod_semicolon_on_nl
    * Place the semicolon after each production on a new line
  * formatting.max_line_length
    * Maximum number of characters per line

  This requires `Parol Language Server` (`parol-ls`) of version >= 	0.13.0.

## v0.1.13

* Added missing license file

## v0.1.12

* Infrastructural changes
  * Moved repository into parol workspace
  * Changed repository reference in package.json
* Update license to dual-license either MIT License or Apache License, Version 2.0

## v0.1.11

* Support for new parol features from 0.14.0 (new terminal representation forms)

## v0.1.10

* Improved support for configuration properties.

## v0.1.9

* Add basic support for configuration properties and the client informs the language server about
changed configuration properties.

## v0.1.8

* Notify if a newer language server is available

## v0.1.7

* Improved detection of the language server's version

## v0.1.6

* Minor improvements

## v0.1.5

* Add Support for parol-ls Language Server

## v0.1.4

* New artwork
* Support of new language features of parol v0.10.2
* Availability in VS Code marketplace [parol-vscode](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode)

## v0.1.3

* Add support for cut operator of parol v0.9.4

## v0.1.1

* Language icons are available now
* Providing vsix package

## v0.1.0

* Initial release
  * Support for syntax highlighting and folding is available
