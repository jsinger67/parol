// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Scope } from "./Scope";
import type { Symbol } from "./Symbol";

/**
 *
 * Collection of symbols
 *
 * Mimics rust's rules of uniqueness of symbol names within a certain scope.
 * This struct models the scopes and symbols within them only to the extend needed to auto-generate
 * flawless type and instance names.
 * Especially the deduction of the existence of lifetime parameter on generated types is modelled
 * as simple as possible.
 *
 */
export type SymbolTable = { symbols: Array<Symbol>, scopes: Array<Scope>, };