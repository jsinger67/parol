use crate::{
    parser::recovery::Recovery, FileSource, FormatToken, Location, LookaheadDFA, NonTerminalIndex,
    ParseStack, ParseTreeStack, ParseTreeType, ParseType, ParserError, ProductionIndex, Result,
    SyntaxError, TerminalIndex, TokenStream, TokenVec, UnexpectedToken, UserActionsTrait,
};
use log::trace;
use std::{cell::RefCell, cmp::Ord, rc::Rc};
use syntree::{Builder, Tree};

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
                ParseType::S(s) => format!("%sc({})", s),
                ParseType::Push(s) => format!("%push({})", s),
                ParseType::Pop => "%pop".to_string(),
                _ => "?".to_owned(),
            })
            .collect::<Vec<String>>()
            .join(" ");
        format!("{}: {};", non_terminal_names[self.lhs], rhs)
    }
}

/// The type used for the parse tree
pub type ParseTree<'t> = Tree<ParseTreeType<'t>, u32, usize>;

/// The parse tree builder type
pub(crate) type TreeBuilder<'t> = Builder<ParseTreeType<'t>, u32, usize>;

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

    fn push_production(
        &mut self,
        tree_builder: &mut TreeBuilder<'t>,
        prod_num: ProductionIndex,
    ) -> Result<()> {
        self.parser_stack.stack.push(ParseType::E(prod_num));
        for s in self.productions[prod_num].production {
            self.parser_stack.stack.push(*s);
        }

        // Open a 'production entry' node in the tree
        if !self.trim_parse_tree {
            tree_builder
                .open(ParseTreeType::N(
                    self.non_terminal_names[self.productions[prod_num].lhs],
                ))
                .map_err(|source| ParserError::TreeError { source })?;
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

    fn process_item_stack<'u>(
        &mut self,
        tree_builder: &mut TreeBuilder<'t>,
        prod_num: ProductionIndex,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()> {
        let l = self.productions[prod_num]
            .production
            .iter()
            .filter(|s| !s.is_switch())
            .count();
        // We remove the last n entries from the parse tree stack and insert them as
        // children under the node laying below on the stack
        let children = self
            .parse_tree_stack
            .split_off(self.parse_tree_stack.len() - l);

        // With the children we can call the user's semantic action
        user_actions.call_semantic_action_for_production_number(prod_num, &children)?;

        if !self.trim_parse_tree {
            // And we close the production subtree
            tree_builder
                .close()
                .map_err(|source| ParserError::TreeError { source }.into())
        } else {
            Ok(())
        }
    }

    fn predict_production(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: Rc<RefCell<TokenStream<'t>>>,
    ) -> Result<ProductionIndex> {
        let lookahead_dfa = &self.lookahead_automata[non_terminal];
        lookahead_dfa.eval(&mut stream.borrow_mut(), non_terminal)
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

    fn diagnostic_message(&self, msg: &str, stream: Rc<RefCell<TokenStream<'t>>>) -> String {
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
            format!("{}\n", msg,)
        }
    }

    ///
    /// The actual parsing function.
    /// It is normally not called directly.
    /// The generated parser sources contain all appropriate initialization and
    /// the actual execution of this parse function.
    ///
    pub fn parse<'u>(
        &mut self,
        stream: TokenStream<'t>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<ParseTree<'t>> {
        let stream = Rc::new(RefCell::new(stream));
        let prod_num = match self.predict_production(self.start_symbol_index, stream.clone()) {
            Ok(prod_num) => prod_num,
            Err(source) => {
                match self.handle_prediction_error(self.start_symbol_index, stream.clone(), source)
                {
                    Ok(p) => p,
                    Err(e) => return Err(e),
                }
            }
        };

        let mut tree_builder = TreeBuilder::new();

        self.push_production(&mut tree_builder, prod_num)?;

        'WHILE: while !self.input_accepted() {
            if let Some(entry) = self.parser_stack.stack.last().cloned() {
                match entry {
                    ParseType::T(t) => {
                        let token = stream.borrow_mut().lookahead(0)?;
                        if token.token_type == t {
                            trace!("Consuming token {}", token);
                            self.handle_comments(&stream, user_actions)?;
                            stream.borrow_mut().consume()?;
                            self.parser_stack.stack.pop();
                            if !self.trim_parse_tree {
                                tree_builder
                                    .token(ParseTreeType::T(token.clone()), 1)
                                    .map_err(|source| ParserError::TreeError { source })?;
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
                            self.push_production(&mut tree_builder, prod_num)?;
                        }
                        Err(source) => {
                            match self.handle_prediction_error(n, stream.clone(), source) {
                                Err(_) => break 'WHILE,
                                Ok(prod_num) => {
                                    self.parser_stack.stack.pop();
                                    self.push_production(&mut tree_builder, prod_num)?;
                                }
                            }
                        }
                    },
                    ParseType::S(s) => {
                        stream.borrow_mut().switch_scanner(s)?;
                        self.parser_stack.stack.pop();
                    }
                    ParseType::Push(s) => {
                        trace!("%push({}) at production {:?}", s, self.current_production());
                        stream.borrow_mut().push_scanner(s)?;
                        self.parser_stack.stack.pop();
                    }
                    ParseType::Pop => {
                        trace!("%pop() at production {:?}", self.current_production());
                        let result = stream.borrow_mut().pop_scanner();
                        if let Err(source) = result {
                            return Err(ParserError::PopOnEmptyScannerStateStack {
                                context: self.diagnostic_message(
                                    format!(
                                        "Current scanner is {}",
                                        &stream.borrow().current_scanner(),
                                    )
                                    .as_str(),
                                    stream.clone(),
                                ),
                                input: FileSource::from_stream(&stream.borrow()),
                                source,
                            }
                            .into());
                        }
                        self.parser_stack.stack.pop();
                    }
                    ParseType::E(p) => {
                        self.production_depth -= 1;
                        trace!("Popped production {} -> depth {}", p, self.production_depth);
                        self.parser_stack.stack.pop(); // Pop the End of production marker
                        self.process_item_stack(&mut tree_builder, p, user_actions)?;
                    }
                }
            }
        }

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
            Ok(tree_builder
                .build()
                .map_err(|source| ParserError::TreeError { source })?)
        }
    }

    fn handle_token_mismatch(
        &mut self,
        t: u16,
        token: crate::Token<'_>,
        stream: Rc<RefCell<TokenStream<'t>>>,
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

    fn handle_prediction_error(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: Rc<RefCell<TokenStream<'t>>>,
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

    fn recover_from_prediction_error(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: Rc<RefCell<TokenStream<'t>>>,
    ) -> Result<ProductionIndex> {
        if !self.enable_recovery {
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
            trace!("Sync with {:?}", expected_token_types);
            self.adjust_token_stream(scanned_token_types, expected_token_types, stream.clone())?;
            let result = self.predict_production(non_terminal, stream);
            match result {
                Ok(prod_num) => {
                    trace!("recovering with production {}", prod_num);
                    return Ok(prod_num);
                }
                Err(source) => {
                    trace!("predict_production failed {:?}", source);
                    return Err(source);
                }
            }
        }

        if !possible_terminal_strings.is_empty() {
            // Steamroller tactics: sync with the first possible token string
            let first_token_string = possible_terminal_strings.iter().next().unwrap();
            trace!("Force sync with {:?}", first_token_string);
            self.sync_token_stream(
                scanned_token_types,
                first_token_string.clone(),
                stream.clone(),
            )?;
            let result = self.predict_production(non_terminal, stream);
            match result {
                Ok(prod_num) => {
                    trace!("recovering with production {}", prod_num);
                    return Ok(prod_num);
                }
                Err(source) => {
                    trace!("predict_production failed {:?}", source);
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
    fn recover_from_token_mismatch(&mut self, stream: Rc<RefCell<TokenStream<'t>>>) -> Result<()> {
        if !self.enable_recovery {
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

    fn adjust_token_stream(
        &mut self,
        scanned_token_types: Vec<TerminalIndex>,
        expected_token_types: Vec<TerminalIndex>,
        stream: Rc<RefCell<TokenStream<'t>>>,
    ) -> Result<()> {
        if let Some((act, exp)) =
            Recovery::calculate_match_ranges(&scanned_token_types, &expected_token_types)
        {
            trace!("Match ranges are {act:?}, {exp:?}");
            match act.start.cmp(&exp.start) {
                std::cmp::Ordering::Less => {
                    (act.start..exp.start).try_for_each(|i| -> Result<()> {
                        Ok(stream
                            .borrow_mut()
                            .insert_token_at(i, expected_token_types[i])?)
                    })?;
                    trace!("{}", stream.borrow().diagnostic_message());
                }
                std::cmp::Ordering::Equal => {
                    (0..act.start).try_for_each(|i| -> Result<()> {
                        Ok(stream
                            .borrow_mut()
                            .replace_token_type_at(i, expected_token_types[i])?)
                    })?;
                    trace!("{}", stream.borrow().diagnostic_message());
                }
                std::cmp::Ordering::Greater => {
                    (exp.start..act.start).try_for_each(|_| -> Result<()> {
                        trace!("Consuming superfluous token");
                        Ok(stream.borrow_mut().consume().map(|_| ())?)
                    })?;
                }
            }
            return Ok(());
        }

        // Steamroller tactics: sync with the expected token string
        trace!("Force sync with {:?}", expected_token_types);
        self.sync_token_stream(scanned_token_types, expected_token_types, stream.clone())
    }

    fn sync_token_stream(
        &mut self,
        scanned_token_types: Vec<TerminalIndex>,
        expected_token_types: Vec<TerminalIndex>,
        stream: Rc<RefCell<TokenStream<'t>>>,
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
}
