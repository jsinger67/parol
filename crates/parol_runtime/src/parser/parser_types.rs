use crate::{
    FileSource, FormatToken, Location, LookaheadDFA, NonTerminalIndex, ParolError, ParseStack,
    ParseTreeStack, ParseTreeType, ParseType, ParserError, ProductionIndex, Result, SyntaxError,
    TerminalIndex, TokenStream, TokenVec, UnexpectedToken, UserActionsTrait,
    lexer::EOI,
    parser::recovery::{EditOp, Recovery},
};
use log::trace;
use std::{cell::RefCell, rc::Rc};
use syntree::{Builder, Tree};

use super::parse_tree_type::{SynTree, TreeConstruct};

///
/// The type that contains all data to process a production within the parser.
///
#[derive(Debug, Clone)]
pub struct Production {
    ///
    /// The non-terminal index of the symbol on the left-hand side of the
    /// production.
    /// It is used as index into the generated LOOKAHEAD_AUTOMATA array.
    ///
    pub lhs: NonTerminalIndex,

    ///
    /// The right-hand side of the production in *reversed order*.
    /// Is pushed onto the parse stack when a production has been chosen for
    /// parsing.
    ///
    pub production: &'static [ParseType],
}

impl Production {
    fn to_string(
        &self,
        terminal_names: &'static [&'static str],
        non_terminal_names: &'static [&'static str],
    ) -> String {
        let rhs = self
            .production
            .iter()
            .rev()
            .map(|s| match s {
                ParseType::N(n) => non_terminal_names[*n].to_owned(),
                ParseType::T(t) => format!(r#""{}""#, terminal_names[*t as usize]),
                _ => "?".to_owned(),
            })
            .collect::<Vec<String>>()
            .join(" ");
        format!("{}: {};", non_terminal_names[self.lhs], rhs)
    }
}

syntree::flavor! {
    /// The flavor of the syntax tree
    pub struct SynTreeFlavor {
        type Index = u32;
        type Width = usize;
    }
}

/// The type used for the parse tree
pub type ParseTree<T = SynTree> = Tree<T, SynTreeFlavor>;

/// The parse tree builder type
pub(crate) type TreeBuilder<T = SynTree> = Builder<T, SynTreeFlavor>;

///
/// The actual LLK parser.
/// It resembles a PDA.
/// All data of the generated parser are provided in the 'new' function.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
#[derive(Debug)]
pub struct LLKParser<'t> {
    ///
    /// The non-terminal index of the start symbol
    ///
    start_symbol_index: NonTerminalIndex,

    ///
    /// Grammar productions stack; is built up in push_production and reduced after
    /// each processed token/variable
    ///
    parser_stack: ParseStack,

    ///
    /// The production depth. Use for logging reasons only.
    ///
    pub production_depth: usize,

    ///
    /// Temporary stack that receives recognized grammar symbols before they
    /// are added to the parse tree.
    /// This stack is also used to provide arguments to semantic user actions.
    ///
    parse_tree_stack: ParseTreeStack<ParseTreeType<'t>>,

    ///
    /// The array of generated lookahead automata.
    ///
    lookahead_automata: &'static [LookaheadDFA],

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

    ///
    /// Enables trimming of the parse tree during parsing.
    /// Thus the parse tree doesn't grow much and runtime overhead is diminished.
    /// Useful when enabling production mode and the whole parse tree is not needed.
    ///
    /// To enable this call the method `trim_parse_tree` on the parser object before parsing.
    ///
    trim_parse_tree: bool,

    /// Enables error recovery
    enable_recovery: bool,

    ///
    /// The parser can generate multiple syntax errors during the course of recovering from an error
    ///
    error_entries: Vec<SyntaxError>,
}

impl<'t> LLKParser<'t> {
    ///
    /// Creates a new instance with the given parameters.
    ///
    pub fn new(
        start_symbol_index: NonTerminalIndex,
        lookahead_automata: &'static [LookaheadDFA],
        productions: &'static [Production],
        terminal_names: &'static [&'static str],
        non_terminal_names: &'static [&'static str],
    ) -> Self {
        Self {
            start_symbol_index,
            parser_stack: ParseStack::new(terminal_names, non_terminal_names),
            production_depth: 0,
            parse_tree_stack: ParseTreeStack::new(),
            lookahead_automata,
            productions,
            terminal_names,
            non_terminal_names,
            trim_parse_tree: false,
            enable_recovery: true,
            error_entries: Vec::new(),
        }
    }

    ///
    /// Call this method to enable trimming of the parse tree during parsing before parsing.
    ///
    /// Doing so the parse tree doesn't grow much and runtime overhead is diminished.
    /// Useful when enabling production mode and the whole parse tree is not needed.
    ///
    pub fn trim_parse_tree(&mut self) {
        self.trim_parse_tree = true;
    }

    /// Returns true if the parser is currently in error recovery mode
    #[inline]
    pub fn is_in_recovery_mode(&self) -> bool {
        !self.error_entries.is_empty()
    }

    /// Returns true if error recovery is enabled for this parser
    #[inline]
    pub fn is_recovery_enabled(&self) -> bool {
        self.enable_recovery
    }

    /// Disables error recovery
    /// The recovery is enabled by default
    pub fn disable_recovery(&mut self) {
        self.enable_recovery = false;
    }

    fn input_accepted(&self) -> bool {
        matches!(self.parser_stack.stack[..], [] | [ParseType::T(0)])
    }

    fn current_production(&self) -> Option<ProductionIndex> {
        for e in self.parser_stack.stack.iter().rev() {
            if let ParseType::E(p) = e {
                return Some(*p);
            }
        }
        None
    }

    #[inline]
    fn add_error(&mut self, error: SyntaxError) -> Result<()> {
        if self
            .error_entries
            .iter()
            .any(|e| e.error_location == error.error_location)
        {
            return Err(ParserError::RecoveryFailed.into());
        }
        self.error_entries.push(error);
        if self.error_entries.len() > 100 {
            return Err(ParserError::TooManyErrors {
                count: self.error_entries.len(),
            }
            .into());
        }
        Ok(())
    }

    fn push_production<T: TreeConstruct<'t>>(
        &mut self,
        tree_builder: &mut T,
        prod_num: ProductionIndex,
    ) -> Result<()>
    where
        ParolError: From<T::Error>,
    {
        self.parser_stack.stack.push(ParseType::E(prod_num));
        for s in self.productions[prod_num].production {
            self.parser_stack.stack.push(*s);
        }

        // Open a 'production entry' node in the tree
        if !self.trim_parse_tree {
            tree_builder.open_non_terminal(
                self.non_terminal_names[self.productions[prod_num].lhs],
                None,
            )?;
        }

        // Now push a 'production entry' onto the parse stack
        self.parse_tree_stack.push(ParseTreeType::N(
            self.non_terminal_names[self.productions[prod_num].lhs],
        ));

        self.production_depth += 1;
        trace!(
            "Pushed production {}({}) -> depth {}",
            prod_num,
            self.productions[prod_num].production.len(),
            self.production_depth
        );

        Ok(())
    }

    fn process_item_stack<'u, T: TreeConstruct<'t>>(
        &mut self,
        tree_builder: &mut T,
        prod_num: ProductionIndex,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()>
    where
        ParolError: From<T::Error>,
    {
        let l = self.productions[prod_num].production.len();
        // We remove the last n entries from the parse tree stack and insert them as
        // children under the node laying below on the stack
        let children = self
            .parse_tree_stack
            .split_off(self.parse_tree_stack.len() - l);

        // With the children we can call the user's semantic action
        match user_actions.call_semantic_action_for_production_number(prod_num, &children) {
            Ok(()) => {}
            Err(e) => {
                if self.is_in_recovery_mode() {
                    trace!("Ignoring semantic action error during recovery: {:?}", e);
                    // Add the error to the error entries

                    // Try to get a location from the error if it implements LocationProvider
                    if let ParolError::UserError(anyhow_error) = e.as_ref() {
                        for cause in anyhow_error.chain() {
                            if let Some(syntax_error) = cause.downcast_ref::<SyntaxError>() {
                                trace!(
                                    "Found location provider for semantic action error during recovery"
                                );
                                let _ = self.add_error(
                                    SyntaxError::default()
                                        .with_cause("Semantic action error during recovery")
                                        .with_location(syntax_error.error_location.as_ref().clone())
                                        .with_source(Box::new(e)),
                                );
                                return Ok(());
                            }
                        }
                    } else {
                        trace!(
                            "Ignoring semantic action error during recovery without location provider\n{:?}",
                            e
                        );
                    }
                } else {
                    return Err(e);
                }
            }
        }

        if !self.trim_parse_tree {
            // And we close the production subtree
            tree_builder.close_non_terminal()?;
        }

        Ok(())
    }

    fn predict_production<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> Result<ProductionIndex> {
        let lookahead_dfa = &self.lookahead_automata[non_terminal];
        lookahead_dfa.eval(&mut stream.borrow_mut(), non_terminal)
    }

    fn diagnostic_message<F: Fn(char) -> Option<usize> + Clone>(
        &self,
        msg: &str,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> String {
        trace!(
            "\nParser stack:\n{}\n{}",
            self.parser_stack,
            stream.borrow().diagnostic_message()
        );
        if let Some(prod_num) = self.current_production() {
            format!(
                "{}\n\
                Current production is:\n\
                /* {} */ {}\n",
                msg,
                prod_num,
                self.productions[prod_num].to_string(self.terminal_names, self.non_terminal_names),
            )
        } else {
            format!("{msg}\n",)
        }
    }

    ///
    /// The actual parsing function for the default tree builder.
    /// It is normally not called directly.
    /// The generated parser sources contain all appropriate initialization and
    /// the actual execution of this parse function.
    ///
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
    /// The actual parsing function for a custom tree builder.
    /// It is normally not called directly.
    /// The generated parser sources contain all appropriate initialization and
    /// the actual execution of this parse function.
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
        // Add a root node to the tree that can receive besides the root symbol all other symbols
        // of the parse tree, e.g. comments, whitespace, etc.
        tree_builder.open_non_terminal("", None)?;

        let stream = Rc::new(RefCell::new(stream));
        let prod_num = match self.predict_production(self.start_symbol_index, stream.clone()) {
            Ok(prod_num) => prod_num,
            Err(source) => {
                self.handle_prediction_error(self.start_symbol_index, stream.clone(), source)?
            }
        };

        self.push_production(tree_builder, prod_num)?;

        'WHILE: while !self.input_accepted() {
            if let Some(entry) = self.parser_stack.stack.last().cloned() {
                match entry {
                    ParseType::T(t) => {
                        let token = stream.borrow_mut().lookahead(0)?;
                        if token.token_type == t {
                            trace!("Consuming token {token}");
                            self.handle_additional_tokens(
                                tree_builder,
                                stream.clone(),
                                user_actions,
                            )?;
                            stream.borrow_mut().consume()?;
                            self.parser_stack.stack.pop();
                            if !self.trim_parse_tree {
                                tree_builder.add_token(&token)?;
                            }
                            self.parse_tree_stack.push(ParseTreeType::T(token));
                        } else if self
                            .handle_token_mismatch(t, token, stream.clone())
                            .is_err()
                        {
                            break 'WHILE;
                        }
                    }
                    ParseType::N(n) => match self.predict_production(n, stream.clone()) {
                        Ok(prod_num) => {
                            self.parser_stack.stack.pop();
                            self.push_production(tree_builder, prod_num)?;
                        }
                        Err(source) => {
                            match self.handle_prediction_error(n, stream.clone(), source) {
                                Err(_) => break 'WHILE,
                                Ok(prod_num) => {
                                    self.parser_stack.stack.pop();
                                    self.push_production(tree_builder, prod_num)?;
                                }
                            }
                        }
                    },
                    ParseType::E(p) => {
                        self.production_depth -= 1;
                        trace!("Popped production {} -> depth {}", p, self.production_depth);
                        self.parser_stack.stack.pop(); // Pop the End of production marker
                        self.process_item_stack(tree_builder, p, user_actions)?;
                    }
                }
            }
        }
        // Handle additional tokens after the last token relevant for the grammar
        self.handle_additional_tokens(tree_builder, stream.clone(), user_actions)?;
        if !self.error_entries.is_empty() {
            return Err(ParserError::SyntaxErrors {
                entries: self.error_entries.drain(..).collect(),
            }
            .into());
        }
        if !stream.borrow().all_input_consumed() {
            Err((ParserError::UnprocessedInput {
                input: Box::new(FileSource::from_stream(&stream.borrow())),
                last_token: Box::new(stream.borrow().last_token()?.into()),
            })
            .into())
        } else {
            // We close the global root node,
            tree_builder.close_non_terminal()?;
            // build the tree and return it
            Ok(())
        }
    }

    fn handle_token_mismatch<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        t: u16,
        token: crate::Token<'_>,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> Result<()> {
        let mut expected_tokens = TokenVec::default();
        expected_tokens.push(self.terminal_names[t as usize].to_string());
        self.add_error(SyntaxError {
            cause: self.diagnostic_message(
                format!(
                    "Syntax error: found {} \n\
                                            Current scanner is {}",
                    token.format(self.terminal_names),
                    stream.borrow().current_scanner(),
                )
                .as_str(),
                stream.clone(),
            ),
            input: Some(Box::new(FileSource::from_stream(&stream.borrow()))),
            error_location: Box::new((&token).into()),
            unexpected_tokens: vec![UnexpectedToken::new(
                "LA(1)".to_owned(),
                self.terminal_names[token.token_type as usize].to_owned(),
                &token,
            )],
            expected_tokens,
            source: None,
        })?;
        self.recover_from_token_mismatch(stream.clone())
    }

    fn handle_prediction_error<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
        source: crate::ParolError,
    ) -> Result<ProductionIndex> {
        let nt_name = self.non_terminal_names[non_terminal];
        let (message, unexpected_tokens, expected_tokens) =
            self.lookahead_automata[non_terminal]
                .build_error(self.terminal_names, &stream.borrow())?;
        self.add_error(SyntaxError {
            cause: self.diagnostic_message(
                format!(
                    "{}\nat non-terminal \"{}\" \n\
                                            Current scanner is {}",
                    message,
                    nt_name,
                    stream.borrow().current_scanner(),
                )
                .as_str(),
                stream.clone(),
            ),
            input: Some(Box::new(FileSource::from_stream(&stream.borrow()))),
            error_location: unexpected_tokens
                .first()
                .map_or(Box::<Location>::default(), |t| Box::new(t.token.clone())),
            unexpected_tokens,
            expected_tokens,
            source: Some(Box::new(source)),
        })?;
        self.recover_from_prediction_error(non_terminal, stream.clone())
    }

    fn recover_from_prediction_error<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> Result<ProductionIndex> {
        if !self.is_recovery_enabled() {
            return Err(ParserError::RecoveryFailed.into());
        }
        stream.borrow_mut().enter_recovery_mode();
        let scanned_token_types = stream.borrow().token_types();
        let la_dfa = &self.lookahead_automata[non_terminal];
        let mut possible_terminal_strings =
            Recovery::restore_terminal_strings(la_dfa.transitions, la_dfa.prod0);

        if let Some(expected_token_types) =
            Recovery::minimal_token_difference(&scanned_token_types, &mut possible_terminal_strings)
        {
            trace!("Sync with {expected_token_types:?}");
            self.adjust_token_stream(scanned_token_types, expected_token_types, stream.clone())?;
            let result = self.predict_production(non_terminal, stream);
            match result {
                Ok(prod_num) => {
                    trace!("recovering with production {prod_num}");
                    return Ok(prod_num);
                }
                Err(source) => {
                    trace!("predict_production failed {source:?}");
                    return Err(source);
                }
            }
        }

        if !possible_terminal_strings.is_empty() {
            // Steamroller tactics: sync with the first possible token string
            // Prefer the ones that don't contain the EOI token
            let forced_token_string = possible_terminal_strings
                .iter()
                .find(|ts| !ts.contains(&EOI))
                .unwrap_or_else(|| possible_terminal_strings.iter().next().unwrap());
            trace!("Force sync with {forced_token_string:?}");
            self.sync_token_stream(
                scanned_token_types,
                forced_token_string.clone(),
                stream.clone(),
            )?;
            let result = self.predict_production(non_terminal, stream);
            match result {
                Ok(prod_num) => {
                    trace!("recovering with production {prod_num}");
                    return Ok(prod_num);
                }
                Err(source) => {
                    trace!("predict_production failed {source:?}");
                    return Err(source);
                }
            }
        }

        trace!(
            "{}",
            self.diagnostic_message("Can't recover prediction error", stream.clone())
        );
        let current_token = stream.borrow_mut().lookahead(0).unwrap_or_default();
        let _ = self.add_error(
            SyntaxError::default()
                .with_cause("Can't recover")
                .with_location(current_token.location.clone()),
        );
        Err(ParserError::SyntaxErrors {
            entries: self.error_entries.drain(..).collect(),
        }
        .into())
    }

    // Sync input tokens with expected tokens if possible
    fn recover_from_token_mismatch<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> Result<()> {
        if !self.is_recovery_enabled() {
            return Err(ParserError::RecoveryFailed.into());
        }
        stream.borrow_mut().enter_recovery_mode();
        stream.borrow_mut().ensure_buffer()?;
        let scanned_token_types = stream.borrow().token_types();
        let expected_token_types = self.parser_stack.expected_token_types();
        trace!("LA: [{scanned_token_types:?}]");
        trace!("PS: [{expected_token_types:?}]");

        self.adjust_token_stream(scanned_token_types, expected_token_types, stream.clone())
    }

    fn adjust_token_stream<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        scanned_token_types: Vec<TerminalIndex>,
        expected_token_types: Vec<TerminalIndex>,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> Result<()> {
        let (_, ops) = Recovery::levenshtein_distance(&scanned_token_types, &expected_token_types);
        trace!("Levenshtein ops: {ops:?}");

        let mut stream_idx = 0;
        let mut exp_idx = 0;

        for op in ops {
            match op {
                EditOp::Keep => {
                    stream_idx += 1;
                    exp_idx += 1;
                }
                EditOp::Replace => {
                    stream
                        .borrow_mut()
                        .replace_token_type_at(stream_idx, expected_token_types[exp_idx])?;
                    stream_idx += 1;
                    exp_idx += 1;
                }
                EditOp::Insert => {
                    stream
                        .borrow_mut()
                        .insert_token_at(stream_idx, expected_token_types[exp_idx])?;
                    stream_idx += 1;
                    exp_idx += 1;
                }
                EditOp::Delete => {
                    stream.borrow_mut().remove_token_at(stream_idx)?;
                }
            }
        }
        Ok(())
    }

    fn sync_token_stream<F: Fn(char) -> Option<usize> + Clone>(
        &mut self,
        scanned_token_types: Vec<TerminalIndex>,
        expected_token_types: Vec<TerminalIndex>,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
    ) -> Result<()> {
        let mut replaced = false;
        scanned_token_types
            .iter()
            .zip(expected_token_types.iter())
            .enumerate()
            .try_for_each(|(i, (_, exp_t))| -> Result<()> {
                replaced = true;
                Ok(stream.borrow_mut().replace_token_type_at(i, *exp_t)?)
            })?;
        if replaced {
            Ok(())
        } else {
            let _ = self.add_error(SyntaxError::default().with_cause("Can't sync"));
            Err(ParserError::SyntaxErrors {
                entries: self.error_entries.drain(..).collect(),
            }
            .into())
        }
    }

    fn handle_additional_tokens<'u, T: TreeConstruct<'t>, F: Fn(char) -> Option<usize> + Clone>(
        &self,
        tree_builder: &mut T,
        stream: Rc<RefCell<TokenStream<'t, F>>>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()>
    where
        ParolError: From<T::Error>,
    {
        stream
            .borrow_mut()
            .take_skip_tokens()
            .into_iter()
            .try_for_each(|t| {
                if !self.trim_parse_tree {
                    tree_builder.add_token(&t)?;
                }
                if t.is_comment_token() {
                    user_actions.on_comment(t);
                }
                Ok::<(), ParolError>(())
            })
    }
}
