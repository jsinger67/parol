//! Parser types for the LR parser.
//! The parser types are used during the parsing process.
//! They are duplicated from the `parol` crate which in turn duplicates the `lalr` crate's types.
//! This is suboptimal but necessary to avoid a dependency to the `lalr` crate here.

use core::str;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    parser::parser_types::TreeBuilder, FileSource, NonTerminalIndex, ParolError, ParseTree,
    ParseTreeType, ParserError, Production, ProductionIndex, Result, SyntaxError, TerminalIndex,
    TokenStream, TokenVec, UnexpectedToken, UserActionsTrait,
};

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
    /// The action to take when the end of the input is reached.
    pub eof_action: Option<LRAction>,
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

/// The LR parse stack.
#[derive(Debug)]
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
    pub parse_table: LRParseTable,

    /// Temporary stack that receives recognized grammar symbols before they
    /// are added to the parse tree.
    /// This stack is also used to provide arguments to semantic user actions.
    parse_tree_stack: Vec<ParseTreeType<'t>>,

    /// The stack of the parser.
    parser_stack: LRParseStack,

    ///
    /// The array of generated grammar productions.
    ///
    productions: &'static [Production],

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
    trim_parse_tree: bool,
}

impl<'t> LRParser<'t> {
    ///
    /// Creates a new LR parser.
    ///
    pub fn new(
        start_symbol_index: NonTerminalIndex,
        parse_table: LRParseTable,
        productions: &'static [Production],
        terminal_names: &'static [&'static str],
        non_terminal_names: &'static [&'static str],
    ) -> Self {
        LRParser {
            start_symbol_index,
            parse_table,
            parse_tree_stack: Vec::new(),
            parser_stack: LRParseStack::new(),
            productions,
            terminal_names,
            non_terminal_names,
            trim_parse_tree: false,
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

    fn call_action_and_build_parse_tree<'u>(
        &mut self,
        tree_builder: &mut TreeBuilder<'t>,
        prod_num: ProductionIndex,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<usize> {
        // Calculate the number of symbols in the production
        let n = self.productions[prod_num]
            .production
            .iter()
            .filter(|s| !s.is_switch())
            .count();

        // We remove the last n entries from the parse tree stack and insert them as
        // children under the node laying below on the stack
        let children = self
            .parse_tree_stack
            .split_off(self.parse_tree_stack.len() - n);

        // With the children we can call the user's semantic action
        user_actions.call_semantic_action_for_production_number(prod_num, &children)?;
        self.close_sub_parse_tree(tree_builder, prod_num)?; // Close the production subtree
        Ok(n)
    }

    fn close_sub_parse_tree(
        &mut self,
        tree_builder: &mut syntree::Builder<ParseTreeType<'_>, u32, usize>,
        prod_num: ProductionIndex,
    ) -> Result<()> {
        let checkpoint = match self.parse_tree_stack.pop().unwrap() {
            ParseTreeType::C(c) => c,
            _ => {
                return Err(ParserError::InternalError(
                    "Expected checkpoint on parse tree stack".to_owned(),
                )
                .into())
            }
        };

        if !self.trim_parse_tree {
            // And we close the production subtree
            tree_builder
                .close_at(
                    &checkpoint,
                    ParseTreeType::N(self.non_terminal_names[self.productions[prod_num].lhs]),
                )
                .map(|_| ())
                .map_err(|source| ParserError::TreeError { source }.into())
        } else {
            Ok(())
        }
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

        // Prepare the tree builder
        let mut tree_builder = TreeBuilder::new();

        // Initialize the parse stack and the parse tree stack.
        self.parser_stack = LRParseStack::new();
        self.parse_tree_stack = vec![ParseTreeType::C(
            tree_builder
                .checkpoint()
                .map_err(|source| ParserError::TreeError { source })?,
        )];

        loop {
            self.handle_comments(&stream, user_actions)?;
            let token = stream.borrow_mut().consume()?;
            let terminal_index = token.token_type;
            let current_state = self.parser_stack.current_state();
            let action = match self.parse_table.states[current_state]
                .actions
                .get(&terminal_index)
            {
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
                    self.parser_stack.push(*next_state);
                    self.parse_tree_stack.push(ParseTreeType::T(token.clone()));
                }
                LRAction::Reduce(nt_index, prod_index) => {
                    let production = &self.productions[*prod_index];
                    let nt_index = *nt_index;
                    let n = self.call_action_and_build_parse_tree(
                        &mut tree_builder,
                        *prod_index,
                        user_actions,
                    )?;
                    for _ in 0..n {
                        // Pop n states from the stack
                        self.parser_stack.pop();
                    }
                    self.parse_tree_stack.push(ParseTreeType::C(
                        tree_builder
                            .checkpoint()
                            .map_err(|source| ParserError::TreeError { source })?,
                    ));
                    self.parse_tree_stack
                        .push(ParseTreeType::N(self.non_terminal_names[production.lhs]));
                    // The new state is the one on top of the stack
                    let state = self.parser_stack.current_state();
                    let goto = self
                        .parse_table
                        .states
                        .get(state)
                        .unwrap()
                        .gotos
                        .get(&nt_index)
                        .unwrap();
                    // Push the new state onto the stack
                    self.parser_stack.push(*goto);
                }
                LRAction::Accept => {
                    self.close_sub_parse_tree(
                        &mut tree_builder,
                        self.productions
                            .iter()
                            .position(|p| p.lhs == self.start_symbol_index)
                            .unwrap(),
                    )?;
                    break;
                }
            }
        }
        Ok(tree_builder
            .build()
            .map_err(|source| ParserError::TreeError { source })?)
    }
}
