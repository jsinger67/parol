//! Parser types for the LR parser.
//! The parser types are used during the parsing process.
//! Some are nearly duplicates of the ones in the `parol` crate which in turn duplicates the `lalr`
//! crate's types.
//! This is suboptimal but necessary to avoid a dependency to the `lalr` crate here.

use core::str;
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use log::trace;

use crate::{
    FileSource, LRParseTree, NonTerminalIndex, ParolError, ParseTree, ParseTreeStack,
    ParseTreeType, ParserError, ProductionIndex, Result, SyntaxError, TerminalIndex, TokenStream,
    TokenVec, UnexpectedToken, UserActionsTrait,
    lr_parser::parse_tree::build_tree,
    parser::{parse_tree_type::TreeConstruct, parser_types::TreeBuilder},
};

/// The type of the index of a LR action in the parse table's actions array.
pub type LRActionIndex = usize;

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
#[derive(Debug, PartialEq, Eq)]
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
    pub actions: &'static [(TerminalIndex, LRActionIndex)],
    /// The gotos to take for each non-terminal in the state.
    pub gotos: &'static [(NonTerminalIndex, usize)],
}

impl LR1State {
    /// Returns the action index for the given terminal index.
    /// If the terminal index is not found in the state, `None` is returned.
    pub fn action_index(&self, terminal_index: TerminalIndex) -> Option<LRActionIndex> {
        self.actions
            .iter()
            .find(|(t, _)| *t == terminal_index)
            .map(|(_, a)| *a)
    }

    /// Returns the goto state for the given non-terminal index.
    /// If the non-terminal index is not found in the state, `None` is returned.
    pub fn goto_state(&self, non_terminal_index: NonTerminalIndex) -> Option<usize> {
        self.gotos
            .iter()
            .find(|(nt, _)| *nt == non_terminal_index)
            .map(|(_, g)| *g)
    }

    /// Returns a list of all terminal indices in the state.
    /// The list is displayed in case of an error.
    pub fn viable_terminal_indices(&self) -> Vec<TerminalIndex> {
        self.actions.iter().map(|(t, _)| *t).collect()
    }
}

/// The LALR(1) parse table.
#[derive(Debug)]
pub struct LRParseTable {
    /// The actions used in the parse table.
    pub actions: &'static [LRAction],

    /// The states in the parse table.
    pub states: &'static [LR1State],
}

impl LRParseTable {
    /// Returns the action for the given state and terminal index.
    /// If the terminal index is not found in the state, `None` is returned.
    pub fn action(&self, state: usize, terminal_index: TerminalIndex) -> Option<&LRAction> {
        let state = &self.states[state];
        state.action_index(terminal_index).map(|a| &self.actions[a])
    }

    /// Returns the goto state for the given state and non-terminal index.
    /// If the non-terminal index is not found in the state, `None` is returned.
    pub fn goto(&self, state: usize, non_terminal_index: NonTerminalIndex) -> Option<usize> {
        self.states[state].goto_state(non_terminal_index)
    }

    /// Returns a list of all terminal indices in the state.
    /// The list is displayed in case of an error.
    pub fn viable_terminal_indices(&self, state: usize) -> Vec<TerminalIndex> {
        self.states[state].viable_terminal_indices()
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
    /// This stack is also used to provide arguments to semantic user actions.
    parse_tree_stack: ParseTreeStack<LRParseTree<'t>>,

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
    /// Default is `false`.
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

    fn call_action<'u>(
        &mut self,
        prod_num: ProductionIndex,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<usize> {
        // Calculate the number of symbols in the production
        let n = self.productions[prod_num].len;

        // We remove the last n entries from the parse tree stack including skip tokens which are
        // not part of the LALR(1) automaton.
        let children: Vec<LRParseTree<'_>> = self.parse_tree_stack.pop_n(n, |pt| match pt {
            LRParseTree::Terminal(t) => !t.is_skip_token(),
            LRParseTree::NonTerminal(_, _) => true,
        });

        // Prepare the arguments for the user's semantic action
        let arguments = children
            .iter()
            .filter(|pt| !pt.is_skip_token())
            .map(|pt| pt.into())
            .collect::<Vec<ParseTreeType<'t>>>();
        debug_assert_eq!(
            n,
            arguments.len(),
            "Number of arguments does not match number of symbols in production"
        );

        // Insert children under the new non-terminal node of the production being reduced
        let non_terminal = LRParseTree::NonTerminal(
            self.non_terminal_names[self.productions[prod_num].lhs],
            if self.trim_parse_tree {
                None
            } else {
                Some(children)
            },
        );
        // Push the new non-terminal node onto the parse tree stack
        self.parse_tree_stack.push(non_terminal);

        // With the argument built from children we can call the user's semantic action
        trace!("Call semantic action for production {}", prod_num);
        user_actions.call_semantic_action_for_production_number(prod_num, &arguments)?;
        Ok(n)
    }

    fn handle_additional_tokens<'u, F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()> {
        stream
            .borrow_mut()
            .take_skip_tokens()
            .drain(..)
            .try_for_each(|t| {
                if !self.trim_parse_tree {
                    self.parse_tree_stack.push(LRParseTree::Terminal(t.clone()));
                }
                if t.is_comment_token() {
                    user_actions.on_comment(t);
                }
                Ok::<(), ParolError>(())
            })
    }

    /// Parses the input text to a parse tree using the default tree builder.
    pub fn parse<'u, F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        stream: TokenStream<'t, F>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<ParseTree> {
        let mut builder = TreeBuilder::new_with();
        self.parse_into(&mut builder, stream, user_actions)?;
        Ok(builder.build()?)
    }

    ///
    /// Parses the input text.
    ///
    pub fn parse_into<'u, T: TreeConstruct<'t>, F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        tree_builder: &mut T,
        stream: TokenStream<'t, F>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()>
    where
        ParolError: From<T::Error>,
    {
        let stream = Rc::new(RefCell::new(stream));

        // Initialize the parse stack and the parse tree stack.
        self.parser_stack = LRParseStack::new();
        self.parse_tree_stack = ParseTreeStack::new();

        loop {
            self.handle_additional_tokens(stream.clone(), user_actions)?;
            let terminal_index = stream.borrow_mut().lookahead_token_type(0)?;
            let current_state = self.parser_stack.current_state();
            trace!(
                "Current state: {}, token type: {} ({})",
                current_state, terminal_index, self.terminal_names[terminal_index as usize]
            );
            // Get the action for the current state and the current terminal
            let action = self.parse_table.action(current_state, terminal_index);

            match action {
                Some(action) => {
                    match action {
                        LRAction::Shift(next_state) => {
                            // Consume the token
                            let token = stream.borrow_mut().consume()?;
                            trace!("Shift to state {}", next_state);
                            self.parser_stack.push(*next_state);
                            trace!(
                                "Push token {} ({})",
                                token.text, self.terminal_names[token.token_type as usize]
                            );
                            let token = LRParseTree::Terminal(token.clone());
                            self.parse_tree_stack.push(token);
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
                            // The new state is the one on top of the stack
                            let state = self.parser_stack.current_state();
                            trace!("Current state after removing {} states is {}", n, state);
                            let goto = match self.parse_table.goto(state, nt_index) {
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
                            self.parser_stack.push(goto);
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
                                .position(|p| p.lhs == self.start_symbol_index)
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
                None => {
                    self.handle_parse_error(&stream, current_state, terminal_index)?;
                }
            }
        }
        if !self.trim_parse_tree {
            // The parse tree stack should contain only one element at this point
            // Handle additional tokens after the last token relevant for the grammar
            debug_assert!(!self.parse_tree_stack.is_empty());
            self.handle_additional_tokens(stream.clone(), user_actions)?;
            // Add a root node to the tree that can receive besides the root symbol all other symbols
            // of the parse tree, e.g. comments, whitespace, etc.
            let parse_tree = LRParseTree::NonTerminal("", Some(self.parse_tree_stack.pop_all()));
            build_tree::<T>(tree_builder, parse_tree)?;
        }
        Ok(())
    }

    fn handle_parse_error<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        stream: &Rc<RefCell<TokenStream<'t, F>>>,
        current_state: usize,
        terminal_index: u16,
    ) -> Result<()> {
        let token = stream.borrow_mut().lookahead(0)?;
        trace!("No action for token '{}' in state {}", token, current_state);
        trace!("Current scanner is '{}'", stream.borrow().current_scanner());
        trace!("Parse stack: {:?}", self.parser_stack.stack);
        trace!("Parse tree stack:\n{}", self.parse_tree_stack);
        let entries = vec![SyntaxError {
            cause: format!(
                "No action for token '{}' in state {}\nCurrent scanner is '{}'",
                self.terminal_names[terminal_index as usize],
                current_state,
                stream.borrow().current_scanner()
            ),
            input: Some(Box::new(FileSource::from_stream(&stream.borrow()))),
            error_location: Box::new((&token).into()),
            unexpected_tokens: vec![UnexpectedToken::new(
                "LA(1)".to_owned(),
                self.terminal_names[terminal_index as usize].to_owned(),
                &token,
            )],
            expected_tokens: self
                .parse_table
                .viable_terminal_indices(current_state)
                .iter()
                .fold(TokenVec::new(), |mut acc, t| {
                    acc.push(self.terminal_names[*t as usize].to_owned());
                    acc
                }),
            source: None,
        }];
        Err(ParolError::ParserError(ParserError::SyntaxErrors {
            entries,
        }))
    }
}

impl From<syntree::Error> for ParolError {
    fn from(source: syntree::Error) -> Self {
        ParolError::ParserError(ParserError::TreeError { source })
    }
}
