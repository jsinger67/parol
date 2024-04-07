use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use lalr::LR1ParseTable;
use parol_runtime::{NonTerminalIndex, ProductionIndex, TerminalIndex};

use crate::{Cfg, GrammarConfig, Pr, Terminal};

type ParseTable<'a> = LR1ParseTable<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type Rhs = lalr::Rhs<TerminalIndex, NonTerminalIndex, ProductionIndex>;
type Grammar = lalr::Grammar<TerminalIndex, NonTerminalIndex, ProductionIndex>;

impl From<&Cfg> for Grammar {
    fn from(cfg: &Cfg) -> Self {
        let terminal_index = cfg.get_terminal_index_function();
        let non_terminal_index = cfg.get_non_terminal_index_function();

        let mut grammar = Grammar {
            rules: BTreeMap::new(),
            start: non_terminal_index(&cfg.st),
        };

        for (i, Pr(s, rhs, _)) in cfg.pr.iter().enumerate() {
            let lhs = non_terminal_index(s.get_n_ref().unwrap());
            let rhs = Rhs {
                syms: rhs
                    .iter()
                    .map(|s| match s {
                        crate::Symbol::N(n, _, _) => {
                            lalr::Symbol::Nonterminal(non_terminal_index(n))
                        }
                        crate::Symbol::T(Terminal::Trm(s, k, _, _, _)) => {
                            lalr::Symbol::Terminal(terminal_index(s, *k))
                        }
                        _ => unreachable!(),
                    })
                    .collect(),
                act: i,
            };
            grammar.rules.insert(lhs, vec![rhs]);
        }

        grammar
    }
}

/// A LALR(1) parse table action.
/// The action can be either a shift, reduce, or accept action.
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
#[derive(Debug)]
pub struct LRParseTableState {
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
    pub states: Vec<LRParseTableState>,
}

impl From<ParseTable<'_>> for LRParseTable {
    fn from(parse_table: ParseTable) -> Self {
        let mut states = Vec::new();
        for state in parse_table.states {
            let mut actions = BTreeMap::new();
            for (terminal, action) in state.lookahead {
                let action = match action {
                    lalr::LRAction::Shift(s) => LRAction::Shift(s),
                    lalr::LRAction::Reduce(p, r) => LRAction::Reduce(*p, r.act),
                    lalr::LRAction::Accept => LRAction::Accept,
                };
                actions.insert(*terminal, action);
            }

            let gotos = state.goto.into_iter().map(|(n, s)| (*n, s)).collect();

            states.push(LRParseTableState {
                eof_action: state.eof.map(|action| match action {
                    lalr::LRAction::Shift(s) => LRAction::Shift(s),
                    lalr::LRAction::Reduce(p, r) => LRAction::Reduce(*p, r.act),
                    lalr::LRAction::Accept => LRAction::Accept,
                }),
                actions,
                gotos,
            });
        }

        LRParseTable { states }
    }
}

/// Calculate the LALR(1) parse table for the given grammar configuration.
pub fn calculate_lalr1_parse_table(grammar_config: &GrammarConfig) -> Result<LRParseTable> {
    let cfg = &grammar_config.cfg;
    let grammar = Grammar::from(cfg);
    let reduce_on = |_rhs: &Rhs, _lookahead: Option<&TerminalIndex>| false;
    let priority_of = |_rhs: &Rhs, _lookahead: Option<&TerminalIndex>| 0;
    let parse_table = grammar
        .lalr1(reduce_on, priority_of)
        // TODO: Add separate error type for LALR(1) parse table construction
        .map_err(|e| anyhow!("Error during LALR(1) parse table construction: {:?}", e))?;
    Ok(parse_table.into())
}
