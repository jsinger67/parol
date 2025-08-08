# Changes in version 4

## Overview

Version 4 of Parol introduces significant changes that may affect existing projects and workflows. This document summarizes all updates, breaking changes, and migration hints for users upgrading from previous versions.

---

## Major Changes

### 1. Switch to `scnr2` Scanner Crate

Parol now uses the [`scnr2`](https://crates.io/crates/scnr2) scanner crate for lexical analysis. This change improves performance and maintainability but may require adjustments in custom scanner configurations.

**Impact:**  
- All scanner-related features are now based on `scnr2`.
- Custom scanner switching logic must be adapted to the new API.

### 2. Removal of Parser-Based Scanner Switching

Support for parser-based scanner switching has been removed. Scanner switching is now exclusively handled by the scanner itself.

**Impact:**  
- Any grammar or code relying on parser-driven scanner switching must be refactored.
- See migration notes below.

---

## Migration Notes

- Review your grammar files and custom code for any use of parser-based scanner switching and update them to use scanner-based switching.
- Refer to the [scnr2 documentation](https://crates.io/crates/scnr2) for details on the new scanner API.
- Test your grammars and integrations thoroughly after upgrading.

---

## Further Information

For a complete list of changes, see the [Changelog](../../crates/parol/CHANGELOG.md).

If you encounter issues, please consult the [Q&A](./QnA.md) or open an issue on the [Parol GitHub repository](https://github.com/jsinger67/parol).