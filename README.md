# parol-ls README

<!-- markdownlint-disable Inline HTML -->
<br>
<img src="./images/ParolLS_300x300.png" alt="Logo" height=300 with=300>
<br><br><br>
<!-- markdownlint-enable Inline HTML -->

This project provides a Language Server to support
[`parol`](https://github.com/jsinger67/parol.git)'s syntax of grammar description files (.par files).

It is used for instance by
[Parol's VSCode extension](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode)

This Language Server is developed with the help of `parol` itself.

## Features

The Language Server currently supports

* GotoDefinition
* Hover
* and shows syntax errors as you are used to.

Also problems in your grammar are detected and reported.

This tool is still in early development phase. But it can be used in conjunction with Parol's VSCode
extension.

## Installation

Please install this language server on your platform using the following command.

```shell
cargo install parol-ls
```

Also check regularly for updates and issue the command above again on demand.

## Acknowledgements

I took some snippets and inspirations from the language server for
[Lelwel](https://github.com/0x2a-42/lelwel.git).
It is licensed under MIT and Apache-2.0. Thanks a lot and kudos!

## Further readings

* [CHANGELOG](./CHANGELOG.md)
