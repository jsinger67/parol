use crate::{
    FileSource, FormatToken, Location, LookaheadDFA, NonTerminalIndex, ParseStack,
    ParseTreeStackEntry, ParseTreeType, ParseType, ParserError, ProductionIndex, Result,
    TokenStream, TokenVec, UnexpectedToken, UserActionsTrait,
};
use id_tree::{InsertBehavior, MoveBehavior, Node, RemoveBehavior, Tree};
use log::{debug, trace};
use std::cell::RefCell;

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
                ParseType::T(t) => format!(r#""{}""#, terminal_names[*t]),
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

///
/// The actual LLK parser.
/// It resembles a PDA.
/// All data of the generated parser are provided in the 'new' function.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
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
    /// The parse tree the parser creates
    ///
    pub parse_tree: Tree<ParseTreeType<'t>>,

    ///
    /// Temporary stack that receives recognized grammar symbols before they
    /// are added to the parse tree.
    /// This stack is also used to provide arguments to semantic user actions.
    ///
    parse_tree_stack: Vec<ParseTreeStackEntry<'t>>,

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
            parse_tree: Tree::new(),
            parse_tree_stack: Vec::new(),
            lookahead_automata,
            productions,
            terminal_names,
            non_terminal_names,
        }
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

    fn push_production(&mut self, prod_num: ProductionIndex) {
        self.parser_stack.stack.push(ParseType::E(prod_num));
        for s in self.productions[prod_num].production {
            self.parser_stack.stack.push(*s);
        }
        // Now push a 'production entry' onto the parse stack
        let root_node_id = self.parse_tree.root_node_id().cloned();

        let node_id = if let Some(root_node_id) = root_node_id {
            // We create a new non-terminal node and temporarily insert it under the root node
            self.parse_tree.insert(
                Node::new(ParseTreeType::N(
                    self.non_terminal_names[self.productions[prod_num].lhs],
                )),
                InsertBehavior::UnderNode(&root_node_id),
            )
        } else {
            // We create a new non-terminal node and insert it as the root node
            self.parse_tree.insert(
                Node::new(ParseTreeType::N(
                    self.non_terminal_names[self.productions[prod_num].lhs],
                )),
                InsertBehavior::AsRoot,
            )
        };

        // The node's id is pushed on the parse stack
        self.parse_tree_stack
            .push(ParseTreeStackEntry::Id(node_id.unwrap()));

        self.production_depth += 1;
        trace!(
            "Pushed production {}({}) -> depth {}",
            prod_num,
            self.productions[prod_num].production.len(),
            self.production_depth
        );
    }

    fn process_item_stack<'u>(
        &mut self,
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
        user_actions.call_semantic_action_for_production_number(
            prod_num,
            &children,
            &self.parse_tree,
        )?;

        // At the top of the parse tree stack we find the node id of the left-hand side of the
        // current processed production.
        let tos = self.parse_tree_stack.pop();

        if let Some(ParseTreeStackEntry::Id(non_terminal_node_id)) = tos {
            if cfg!(feature = "trim_parse_tree") {
                // Remove the node from the tree unless it is the root node.
                if Some(&non_terminal_node_id) == self.parse_tree.root_node_id() {
                    self.parse_tree_stack
                        .push(ParseTreeStackEntry::Id(non_terminal_node_id));
                    Ok(())
                } else {
                    self.parse_tree
                        .remove_node(non_terminal_node_id, RemoveBehavior::DropChildren)
                        .map(|_| {
                            self.parse_tree_stack.push(ParseTreeStackEntry::Id(
                                self.parse_tree.root_node_id().unwrap().clone(),
                            ))
                        })
                        .map_err(|e| crate::ParserError::IdTreeError { source: e }.into())
                }
            } else {
                // Insert the children under the non-terminal node
                children
                    .into_iter()
                    .fold(Ok(()), |mut acc, c| match c {
                        ParseTreeStackEntry::Id(child_node_id) => {
                            if acc.is_ok() {
                                acc = self.parse_tree.move_node(
                                    &child_node_id,
                                    MoveBehavior::ToParent(&non_terminal_node_id),
                                );
                            }
                            acc
                        }
                        ParseTreeStackEntry::Nd(node) => {
                            if acc.is_ok() {
                                acc = self
                                    .parse_tree
                                    .insert(node, InsertBehavior::UnderNode(&non_terminal_node_id))
                                    .map(|_| ());
                            }
                            acc
                        }
                    })
                    .map(|_| {
                        self.parse_tree_stack
                            .push(ParseTreeStackEntry::Id(non_terminal_node_id))
                    })
                    .map_err(|e| ParserError::IdTreeError { source: e }.into())
            }
        } else {
            Err(ParserError::InternalError(format!(
                "Expected node id on parse tree stack, found {:?}",
                tos
            ))
            .into())
        }
    }

    fn predict_production(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: &RefCell<TokenStream<'t>>,
    ) -> Result<ProductionIndex> {
        let lookahead_dfa = &self.lookahead_automata[non_terminal];
        Ok(lookahead_dfa.eval(&mut stream.borrow_mut())?)
    }

    fn diagnostic_message(&self, msg: &str) -> String {
        trace!("\nParser stack:\n{}\n", self.parser_stack);
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
        stream: RefCell<TokenStream<'t>>,
        user_actions: &'u mut dyn UserActionsTrait<'t>,
    ) -> Result<()> {
        let prod_num = match self.predict_production(self.start_symbol_index, &stream) {
            Ok(prod_num) => prod_num,
            Err(source) => {
                let nt_name = self.non_terminal_names[self.start_symbol_index];
                let (message, unexpected_tokens, expected_tokens) = self.lookahead_automata
                    [self.start_symbol_index]
                    .build_error(self.terminal_names, &stream.borrow())?;
                return Err(ParserError::PredictionErrorWithExpectations {
                    cause: self.diagnostic_message(
                        format!(
                            "{}\nat non-terminal \"{}\" \n\
                                Current scanner is {}",
                            message,
                            nt_name,
                            &stream.borrow().current_scanner(),
                        )
                        .as_str(),
                    ),
                    input: FileSource::from_stream(&stream.borrow()),
                    error_location: unexpected_tokens
                        .get(0)
                        .map_or(Location::default(), |t| t.token.clone()),
                    unexpected_tokens,
                    expected_tokens,
                    source: Some(Box::new(source)),
                }
                .into());
            }
        };

        self.push_production(prod_num);

        while !self.input_accepted() {
            if let Some(entry) = self.parser_stack.stack.last().cloned() {
                match entry {
                    ParseType::T(t) => {
                        let token = stream.borrow_mut().lookahead(0)?;
                        if token.token_type == t {
                            trace!("Consuming token {}", token);
                            stream.borrow_mut().consume()?;
                            self.parser_stack.stack.pop();
                            self.parse_tree_stack
                                .push(ParseTreeStackEntry::Nd(Node::new(ParseTreeType::T(token))));
                        } else {
                            let mut expected_tokens = TokenVec::default();
                            expected_tokens.push(self.terminal_names[t].to_string());
                            return Err(ParserError::PredictionErrorWithExpectations {
                                cause: self.diagnostic_message(
                                    format!(
                                        "Found \"{}\" \n\
                                        Current scanner is {}",
                                        token.format(self.terminal_names),
                                        stream.borrow().current_scanner(),
                                    )
                                    .as_str(),
                                ),
                                input: FileSource::from_stream(&stream.borrow()),
                                error_location: (&token).into(),
                                unexpected_tokens: vec![UnexpectedToken::new(
                                    "LA(1)".to_owned(),
                                    self.terminal_names[token.token_type].to_owned(),
                                    &token,
                                )],
                                expected_tokens,
                                source: None,
                            }
                            .into());
                        }
                    }
                    ParseType::N(n) => match self.predict_production(n, &stream) {
                        Ok(prod_num) => {
                            self.parser_stack.stack.pop();
                            self.push_production(prod_num);
                        }
                        Err(source) => {
                            let nt_name = self.non_terminal_names[n];
                            let (message, unexpected_tokens, expected_tokens) = self
                                .lookahead_automata[n]
                                .build_error(self.terminal_names, &stream.borrow())?;
                            return Err(ParserError::PredictionErrorWithExpectations {
                                cause: self.diagnostic_message(
                                    format!(
                                        "{}\nat non-terminal \"{}\" \n\
                                            Current scanner is {}",
                                        message,
                                        nt_name,
                                        &stream.borrow().current_scanner(),
                                    )
                                    .as_str(),
                                ),
                                input: FileSource::from_stream(&stream.borrow()),
                                error_location: unexpected_tokens
                                    .get(0)
                                    .map_or(Location::default(), |t| t.token.clone()),
                                unexpected_tokens,
                                expected_tokens,
                                source: Some(Box::new(source)),
                            }
                            .into());
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
                        debug!("Popped production {} -> depth {}", p, self.production_depth);
                        self.parser_stack.stack.pop(); // Pop the End of production marker
                        self.process_item_stack(p, user_actions)?;
                    }
                }
            }
        }

        if !stream.borrow().all_input_consumed() {
            Err((ParserError::UnprocessedInput {
                input: FileSource::from_stream(&stream.borrow()),
                last_token: stream.borrow().last_token()?.into(),
            })
            .into())
        } else {
            Ok(())
        }
    }
}
