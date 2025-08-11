# Changes in version 4

## Overview

Version 4 of Parol introduces significant changes that may affect existing projects and workflows.
This document summarizes all updates, breaking changes, and migration hints for users upgrading from
previous versions.

---

## Major Changes

### 1. Switch to `scnr2` Scanner Crate

Parol now uses the [`scnr2`](https://crates.io/crates/scnr2) scanner crate for lexical analysis.
This change improves performance and maintainability but may require adjustments in custom scanner
configurations.

**Impact:**  
- All scanner-related features are now based on `scnr2`.
- Custom scanner switching logic must be adapted to the new API.

### 2. Removal of Parser-Based Scanner Switching

Support for parser-based scanner switching has been removed. Scanner switching is now exclusively
handled by the scanner itself.

**Impact:**  
- Any grammar or code relying on parser-driven scanner switching must be refactored.
- See migration notes below.
---

### Scanner Mode Switching in Version 4

Scanner mode switching in Parol version 4 is now exclusively managed by the scanner, using the
`%enter`, `%push`, and `%pop` directives within the scanner specification. These directives allow
you to:

- `%enter`: Switch the scanner to a specific mode, replacing the current mode.
- `%push`: Push the current mode onto a stack and enter a new mode.
- `%pop`: Return to the previous mode by popping the mode stack.

> **Note:** The `%sc`, `%push`, and `%pop` commands are no longer available within grammar
productions. This means that parser-based scanner mode switching is not supported in version 4.

This change was necessary because lookahead in LL(k) grammars with k > 1 can cause the token buffer
to become unsynchronized if tokens are read in the wrong scanner mode. By restricting mode switching
to the scanner, Parol ensures that tokenization remains consistent and reliable, regardless of
parser lookahead.

Note that Parol generates all the data required by the [`scnr2`](https://crates.io/crates/scnr2)
crate to construct valid and efficient scanners. Users should not have to know how `scnr` works and
how it is configured.

#### Token Priority

As in previous versions, terminals defined earlier in the grammar have higher priority than those
defined later. This allows you to influence the priority of tokens with equal length. In all other
cases, tokens with the longest match are preferred.

---

## Migration Notes

- Review your grammar files for any use of parser-based scanner switching and update them to use
scanner-based switching.
- See [Grammar description syntax](./ParGrammar.md#scanner-based-scanner-switching) for details on
scanner-based scanner switching. In addition to `%enter` described there as of version 4 you have
the aforementioned `%push` and `%pop` instructions additionally available
- Test your grammars and integrations thoroughly after upgrading.

---

## Further Information

For a complete list of changes, see the [Changelog](../../crates/parol/CHANGELOG.md).

If you encounter issues, please consult the [Q&A](./QnA.md) or open an issue on the
[Parol GitHub repository](https://github.com/jsinger67/parol).