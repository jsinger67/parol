# parol-vscode README

<!-- markdownlint-disable Inline HTML -->
<br>
<img src="https://github.com/jsinger67/parol/raw/main/tools/parol-vscode/icons/parol-vscode-300x300.png?raw=true" alt="Logo" height=300 with=300>
<br><br><br>
<!-- markdownlint-enable Inline HTML -->

This is a VS Code extension to support [`parol`](https://github.com/jsinger67/parol.git)'s syntax
of grammar description files (.par files).

## Features

The extension currently supports syntax highlighting, folding and language icons.
When you install the [parol language server](https://github.com/jsinger67/parol/tree/main/crates/parol-ls)
you get the best support. See below for instructions.

I hope these features are helpful for you and improve your workflow.

Further development on this extension is planned.

## Installation

Install this extension from VS Code marketplace
[parol-vscode](https://marketplace.visualstudio.com/items?itemName=jsinger67.parol-vscode)

To be able to use the full Language Server capabilities please install `parol-ls` on your platform
using the following command.

```shell
cargo install --force parol-ls
```

Alternatively you can clone the [repository](https://github.com/jsinger67/parol.git) and build the
language server from sources.

## Development checks

To keep extension configuration definitions and `package.json` contribution settings in sync,
run:

```shell
npm run check-config-sync
```

This check also runs in CI via the dedicated workflow
`Parol VSCode Config Sync`.

Changes regarding the Language Server can be viewed here:
* [CHANGELOG parol-ls](./crates/parol-ls/CHANGELOG.md)

<!-- markdownlint-disable Inline HTML -->
<br>
<img src="https://github.com/jsinger67/parol/raw/main/crates/parol-ls/images/Example.png" alt="Example">
<br><br><br>
<!-- markdownlint-enable Inline HTML -->

Please check regularly for updates.
