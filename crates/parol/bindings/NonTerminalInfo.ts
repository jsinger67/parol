// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Child } from "./Child";
import type { ChildrenType } from "./ChildrenType";

/**
 * Information about the non-terminals
 */
export type NonTerminalInfo = { 
/**
 * The name of the non-terminal
 */
name: string, 
/**
 * The enum variant name for this non-terminal in the generated NonTerminalKind enum
 */
variant: string, 
/**
 * The children of the non-terminal
 */
children: Array<Child>, 
/**
 * The kind of the non-terminal
 */
kind: ChildrenType, };
