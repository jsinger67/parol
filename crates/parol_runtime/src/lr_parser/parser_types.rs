//! Parser types for the LR parser.
//! The parser types are used during the parsing process.
//! They are nearly duplicates of the ones in the `parol` crate which in turn duplicates the `lalr`
//! crate's types.
//! This is suboptimal but necessary to avoid a dependency to the `lalr` crate here.

use core::str;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    iter::FromIterator,
    rc::Rc,
};

use log::trace;

use crate::{
    parser::parser_types::TreeBuilder, FileSource, NonTerminalIndex, ParolError, ParseTree,
    ParseTreeStack, ParseTreeType, ParserError, ProductionIndex, Result, SyntaxError,
    TerminalIndex, TokenStream, TokenVec, UnexpectedToken, UserActionsTrait,
};

///
/// The type that contains all data to process a production within the lr-parser.
///
#[derive(Debug, Clone)]
pub struct LRProduction {
    ///
    /// The non-terminal index of the symbol on the left-hand side of the
    /// production.
    /// It is used as index into the generated LOOKAHEAD_AUTOMATA array.
    ///
    pub lhs: NonTerminalIndex,

    ///
    /// The length of the right-hand side of the production.
    ///
    pub len: usize,
}

/// An item in the LR(0) state machine.
/// Duplicate of the `lalr` crate's `Item` type without the reference to the creating grammar.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Item {
    /// The production of the item.
    pub prod: ProductionIndex,
    /// The position in the production.
    pub pos: usize,
}

/// An item set in the LR(0) state machine.
/// Duplicate of the `lalr` crate's `ItemSet` type without the reference to the creating grammar.
#[derive(Debug)]
pub struct ItemSet {
    /// The items in the set.
    pub items: BTreeSet<Item>,
}

/// A LALR(1) parse table action.
/// The action can be either a shift, reduce, or accept action.
/// Duplicate of the `lalr` crate's `LRAction` type without the reference to the creating grammar.
#[derive(Debug)]
pub enum LRAction {
    /// Shift the current token and go to the next state.
    Shift(usize),
    /// Reduce to the given production.
    Reduce(NonTerminalIndex, ProductionIndex),
    /// Accept the input.
    Accept,
}

/// A state in the LALR(1) parse table.
/// Duplicate of the `lalr` crate's `LR1State` type without the reference to the creating grammar.
#[derive(Debug)]
pub struct LR1State {
    /// The actions to take for each terminal in the state.
    pub actions: BTreeMap<TerminalIndex, LRAction>,
    /// The gotos to take for each non-terminal in the state.
    pub gotos: BTreeMap<NonTerminalIndex, usize>,
}

/// The LALR(1) parse table.
#[derive(Debug)]
pub struct LRParseTable {
    /// The states in the parse table.
    pub states: Vec<LR1State>,
}

impl LRParseTable {
    /// Creates a new instance.
    pub fn new(states: Vec<LR1State>) -> Self {
        Self { states }
    }
}

impl FromIterator<LR1State> for LRParseTable {
    fn from_iter<T: IntoIterator<Item = LR1State>>(iter: T) -> Self {
        Self {
            states: iter.into_iter().collect(),
        }
    }
}

/// The LR parse stack.
#[derive(Debug, Default)]
pub struct LRParseStack {
    /// The state indices from in the parse table.
    pub stack: Vec<usize>,
}

impl LRParseStack {
    /// Creates a new instance.
    /// The stack is initialized with the start state.
    pub fn new() -> Self {
        Self { stack: vec![0] }
    }

    /// Returns the current state.
    /// The current state is the top of the stack.
    /// The stack is never empty.
    pub fn current_state(&self) -> usize {
        *self.stack.last().unwrap()
    }

    /// Pushes a new state onto the stack.
    pub fn push(&mut self, state: usize) {
        self.stack.push(state);
    }

    /// Pops the top state from the stack.
    /// The stack is never empty.
    /// The start state is never popped.
    pub fn pop(&mut self) {
        self.stack.pop();
    }
}

///
/// The LR parser.
/// It implements a LALR(1) parsing strategy.
/// All data of the generated parser are provided in the 'new' function.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
#[derive(Debug)]
pub struct LRParser<'t> {
    ///
    /// The non-terminal index of the start symbol
    ///
    start_symbol_index: NonTerminalIndex,

    /// The parse table.
    pub parse_table: &'static LRParseTable,

    /// Temporary stack that receives recognized grammar symbols before they
    /// are added to the parse tree.
    ///
    /// !!!!!
    /// Parse tree generation is not implemented yet because of the syntree's implementation.
    /// TODO: Rethink the parse tree generation.
    /// !!!!!
    ///
    /// This stack is also used to provide arguments to semantic user actions.
    parse_tree_stack: ParseTreeStack<ParseTreeType<'t>>,

    /// The stack of the parser.
    parser_stack: LRParseStack,

    ///
    /// The array of generated grammar productions.
    ///
    productions: &'static [LRProduction],

    ///
    /// Array of generated terminal names.
    ///
    terminal_names: &'static [&'static str],

    ///
    /// Array of generated non-terminal names.
    ///
    non_terminal_names: &'static [&'static str],

    /// Enables trimming of the parse tree during parsing.
    /// Thus the parse tree doesn't grow much and runtime overhead is diminished.
    /// Useful when enabling production mode and the whole parse tree is not needed.
    ///
    /// To enable this call the method `trim_parse_tree` on the parser object before parsing.
    ///
    /// Default is `true` as longs a parse tree generation is not implemented.
    trim_parse_tree: bool,
}

impl<'t> LRParser<'t> {
    ///
    /// Creates a new LR parser.
    ///
    pub fn new(
        start_symbol_index: NonTerminalIndex,
        parse_table: &'static LRParseTable,
        productions: &'static [LRProduction],
        terminal_names: &'static [&'static str],
        non_terminal_names: &'static [&'static str],
    ) -> Self {
        LRParser {
            start_symbol_index,
            parse_table,
            parse_tree_stack: ParseTreeStack::new(),
            parser_stack: LRParseStack::new(),
            productions,
            terminal_names,
            non_terminal_names,
            trim_parse_tree: true,
        }
    }

    ///
    /// Trims the parse tree during parsing.
    /// Thus the parse tree doesn't grow much and runtime overhead is diminished.
    /// Useful when enabling production mode and the whole parse tree is not needed.
    ///
    pub fn trim_parse_tree(&mut self) {
        self.trim_parse_tree = true;
    }

    fn call_action<'u>(
        &mut self,
        prod_num: ProductionIndex,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<usize> {
        // Calculate the number of symbols in the production
        let n = self.productions[prod_num].len;

        // We remove the last n entries from the parse tree stack and insert them as
        // children under the node laying below on the stack
        let children: Vec<ParseTreeType<'_>> = self
            .parse_tree_stack
            .split_off(self.parse_tree_stack.len() - n);

        // With the children we can call the user's semantic action
        trace!("Call semantic action for production {}", prod_num);
        user_actions.call_semantic_action_for_production_number(prod_num, &children)?;
        Ok(n)
    }

    fn handle_comments<'u>(
        &mut self,
        stream: &RefCell<TokenStream<'t>>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()> {
        stream
            .borrow_mut()
            .drain_comments()
            .into_iter()
            .for_each(|c| user_actions.on_comment_parsed(c));
        Ok(())
    }

    ///
    /// Parses the input text.
    ///
    pub fn parse<'u>(
        &mut self,
        stream: TokenStream<'t>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<ParseTree<'t>> {
        let stream = Rc::new(RefCell::new(stream));

        // Initialize the parse stack and the parse tree stack.
        self.parser_stack = LRParseStack::new();
        self.parse_tree_stack.push(ParseTreeType::N(
            self.non_terminal_names[self.start_symbol_index],
        ));

        loop {
            self.handle_comments(&stream, user_actions)?;
            let token = stream.borrow_mut().lookahead(0)?;
            let terminal_index = token.token_type;
            let current_state = self.parser_stack.current_state();
            trace!(
                "Current state: {}, token type: {} ({})",
                current_state,
                terminal_index,
                self.terminal_names[terminal_index as usize]
            );
            // Get the action for the current state and the current terminal
            // In case the terminal index is 0 (EOI) we use the eof_action (if present)
            let action = self.parse_table.states[current_state]
                .actions
                .get(&terminal_index);
            let action = match action {
                Some(action) => action,
                None => {
                    let entries = vec![SyntaxError {
                        cause: format!(
                            "No action for terminal '{}' in state {}",
                            self.terminal_names[terminal_index as usize], current_state
                        ),
                        input: Some(Box::new(FileSource::from_stream(&stream.borrow()))),
                        error_location: Box::new((&token).into()),
                        unexpected_tokens: vec![UnexpectedToken::new(
                            "LA(1)".to_owned(),
                            self.terminal_names[token.token_type as usize].to_owned(),
                            &token,
                        )],
                        expected_tokens: TokenVec::new(),
                        source: None,
                    }];
                    return Err(ParolError::ParserError(ParserError::SyntaxErrors {
                        entries,
                    }));
                }
            };
            match action {
                LRAction::Shift(next_state) => {
                    trace!("Shift to state {}", next_state);
                    self.parser_stack.push(*next_state);
                    trace!(
                        "Push token {} ({})",
                        token.text,
                        self.terminal_names[token.token_type as usize]
                    );
                    let token = ParseTreeType::T(token.clone());
                    self.parse_tree_stack.push(token.clone());
                    // Consume the token
                    stream.borrow_mut().consume()?;
                }
                LRAction::Reduce(nt_index, prod_index) => {
                    trace!("Reduce by production {}", prod_index);
                    let nt_index = *nt_index;
                    let n = self.call_action(*prod_index, user_actions)?;
                    for _ in 0..n {
                        // Pop n states from the stack
                        if self.parser_stack.stack.is_empty() {
                            return Err(ParserError::InternalError(
                                "Attempted to pop from an empty stack".to_owned(),
                            )
                            .into());
                        }
                        self.parser_stack.pop();
                    }
                    let non_terminal = ParseTreeType::N(self.non_terminal_names[nt_index]);
                    self.parse_tree_stack.push(non_terminal.clone());
                    // The new state is the one on top of the stack
                    let state = self.parser_stack.current_state();
                    trace!("Current state after removing {} states is {}", n, state);
                    let goto = match self
                        .parse_table
                        .states
                        .get(state)
                        .unwrap()
                        .gotos
                        .get(&nt_index)
                    {
                        Some(goto) => goto,
                        None => {
                            return Err(ParserError::InternalError(format!(
                                "No goto for non-terminal '{}' in state {}",
                                nt_index, state
                            ))
                            .into());
                        }
                    };
                    // Push the new state onto the stack
                    trace!("Push goto state {}", goto);
                    self.parser_stack.push(*goto);
                }
                LRAction::Accept => {
                    trace!("Accept");
                    // The non-terminal of the start symbol lies on top of the stack here
                    trace!("Final parse stack: {:?}", self.parser_stack.stack);
                    trace!("Final parse tree stack:\n{}", self.parse_tree_stack);
                    // Find the production number of the start symbol
                    let prod_index = if let Some(index) = self
                        .productions
                        .iter()
                        .position(|p| p.lhs == self.start_symbol_index && p.len == 1)
                    {
                        index
                    } else {
                        return Err(ParserError::InternalError(format!(
                            "No production found for start symbol '{}'",
                            self.non_terminal_names[self.start_symbol_index]
                        ))
                        .into());
                    };
                    // Call the action for the start symbol
                    let _n = self.call_action(prod_index, user_actions)?;
                    break;
                }
            }
        }
        Ok(TreeBuilder::new()
            .build()
            .map_err(|source| ParserError::TreeError { source })?)
    }
}
