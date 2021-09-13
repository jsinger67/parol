use super::errors::*;
use crate::lexer::TokenStream;
use crate::parser::{
    AstStackEntry, AstType, LookaheadDFA, NonTerminalIndex, ParseStack, ParseType, ProductionIndex,
    UserActionsTrait,
};
use id_tree::{InsertBehavior, MoveBehavior, Node, Tree};
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
    /// The left-hand side of the production in reversed order.
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
pub struct LLKParser {
    ///
    /// The non-terminal index of the start symbol
    ///
    start_symbol_index: NonTerminalIndex,

    ///
    /// Grammar rules stack; is built up in push_production and reduced after
    /// each processed token/variable
    ///
    parser_stack: ParseStack,

    ///
    /// The rule depth. Use for logging reasons only.
    ///
    pub rule_depth: usize,

    ///
    /// AST - the abstract syntax tree the parser creates
    ///
    pub parse_tree: Tree<AstType>,

    ///
    /// Temporary stack that receives recognized grammar symbols before they
    /// are added to the parse tree.
    /// This stack is also used to provide arguments to semantic user actions.
    ///  
    ast_stack: Vec<AstStackEntry>,

    ///
    /// The array of generated lookahead automata.
    ///
    lookahead_automata: &'static [LookaheadDFA],

    ///
    /// The array of generated grammar productions.
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

impl<'t> LLKParser {
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
            rule_depth: 0,
            parse_tree: Tree::new(),
            ast_stack: Vec::new(),
            lookahead_automata,
            productions,
            terminal_names,
            non_terminal_names,
        }
    }

    #[allow(dead_code)]
    fn log_tree(&self) {
        let mut s = String::new();
        self.parse_tree.write_formatted(&mut s).unwrap();
        debug!("\n{}", s);
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
            self.parser_stack.stack.push(s.clone());
        }
        // Now push a 'rule entry' onto the AST stack
        let root_node_id = self.parse_tree.root_node_id().cloned();

        let node_id = if let Some(root_node_id) = root_node_id {
            // We create a new non-terminal node and temporarily insert it under the root node
            self.parse_tree.insert(
                Node::new(AstType::N(
                    self.non_terminal_names[self.productions[prod_num].lhs],
                )),
                InsertBehavior::UnderNode(&root_node_id),
            )
        } else {
            // We create a new non-terminal node and insert it as the root node
            self.parse_tree.insert(
                Node::new(AstType::N(
                    self.non_terminal_names[self.productions[prod_num].lhs],
                )),
                InsertBehavior::AsRoot,
            )
        };

        // The node's id is pushed on the AST stack
        self.ast_stack.push(AstStackEntry::Id(node_id.unwrap()));

        self.rule_depth += 1;
        debug!(
            "Pushed production {} -> depth {}",
            prod_num, self.rule_depth
        );
    }

    fn process_ast_stack(
        &mut self,
        prod_num: ProductionIndex,
        user_actions: &mut dyn UserActionsTrait,
    ) -> Result<()> {
        let l = self.productions[prod_num].production.len();
        // We remove the last n entries from the ast stack and insert them as
        // children under the node laying below on the stack
        let children = self.ast_stack.split_off(self.ast_stack.len() - l);
        user_actions.call_semantic_action_for_production_number(
            prod_num,
            &children,
            &self.parse_tree,
        )?;
        let tos = self.ast_stack.pop();
        if let Some(AstStackEntry::Id(node_id)) = tos {
            children.into_iter().for_each(|c| match c {
                AstStackEntry::Id(child_node_id) => {
                    self.parse_tree
                        .move_node(&child_node_id, MoveBehavior::ToParent(&node_id))
                        .expect("Node should be moved.");
                }
                AstStackEntry::Nd(node) => {
                    let _ = self
                        .parse_tree
                        .insert(node, InsertBehavior::UnderNode(&node_id))
                        .expect("Node should be inserted.");
                }
            });

            // The node's id is pushed on the AST stack
            self.ast_stack.push(AstStackEntry::Id(node_id));
        } else {
            panic!("Expected node id on ast stack, found {:?}", tos);
        }

        Ok(())
    }

    fn predict_production(
        &mut self,
        non_terminal: NonTerminalIndex,
        stream: &RefCell<TokenStream<'t>>,
    ) -> Result<ProductionIndex> {
        let lookahead_dfa = &self.lookahead_automata[non_terminal];
        lookahead_dfa.eval(&mut stream.borrow_mut())
    }

    fn diagnostic_message(&self, msg: &str, stream: &RefCell<TokenStream<'t>>) -> String {
        let file_name = stream.borrow().file_name.clone();
        let (token, token_type, location) =
            if let Ok(token) = stream.borrow_mut().owned_lookahead(0) {
                (
                    format!("{}", token),
                    token.token_type,
                    format!("{}:({},{})", file_name, token.line, token.column),
                )
            } else {
                ("<<Error>>".to_string(), 0, format!("{}(?)", file_name))
            };

        trace!(
            r"
Parser stack:
{}
",
            self.parser_stack
        );

        let prod_num = self.current_production().unwrap();

        let msg = format!(
            r#"{} at {}
Current production is:
/* {} */ {}
Lookahead token: {} ({})
"#,
            msg,
            location,
            prod_num,
            self.productions[prod_num].to_string(self.terminal_names, self.non_terminal_names),
            token,
            self.terminal_names[token_type],
        );
        msg
    }

    ///
    /// The actual parsing function.
    /// It is normally not called directly.
    /// The generated parser sources contain all appropriate initialization and
    /// the actual execution of this parse function.
    ///
    pub fn parse(
        &mut self,
        stream: &RefCell<TokenStream<'t>>,
        user_actions: &mut dyn UserActionsTrait,
    ) -> Result<()> {
        let prod_num = self
            .predict_production(self.start_symbol_index, stream)
            .chain_err(|| {
                format!(
                    "Can't predict production while trying to start parsing with {}!",
                    self.non_terminal_names[self.start_symbol_index]
                )
            })?;

        self.push_production(prod_num);

        while !self.input_accepted() {
            if let Some(entry) = self.parser_stack.stack.last().cloned() {
                match entry {
                    ParseType::T(t) => {
                        let token = stream
                            .borrow_mut()
                            .owned_lookahead(0)
                            .chain_err(|| "Failed accessing lookahead token!")?;
                        if token.token_type == t {
                            trace!("Consuming token {}", token);
                            stream
                                .borrow_mut()
                                .consume(1)
                                .chain_err(|| "Failed consuming the next token!")?;
                            self.parser_stack.stack.pop();
                            self.ast_stack
                                .push(AstStackEntry::Nd(Node::new(AstType::T(token))));
                        } else {
                            let msg = self.diagnostic_message(
                                format!(
                                    "Expecting token {}, but found {}",
                                    self.terminal_names[t], token
                                )
                                .as_str(),
                                stream,
                            );
                            return Err(msg.into());
                        }
                    }
                    ParseType::N(n) => {
                        let prod_num = self.predict_production(n, stream).chain_err(|| {
                            let nt_name = self.non_terminal_names[n];
                            self.diagnostic_message(
                                format!(
                                    r#"Expecting one of {} at non-terminal "{}""#,
                                    self.lookahead_automata[n]
                                        .expected_terminals(self.terminal_names),
                                    nt_name,
                                )
                                .as_str(),
                                stream,
                            )
                        })?;
                        self.parser_stack.stack.pop();
                        self.push_production(prod_num);
                    }
                    ParseType::E(p) => {
                        self.rule_depth -= 1;
                        debug!("Popped production {} -> depth {}", p, self.rule_depth);
                        self.parser_stack.stack.pop(); // Pop the End of production marker
                        self.process_ast_stack(p, user_actions)?;
                    }
                }
            }
        }

        if !stream.borrow().all_input_consumed() {
            Err(self.diagnostic_message("Unprocessed input", stream).into())
        } else {
            Ok(())
        }
    }
}
