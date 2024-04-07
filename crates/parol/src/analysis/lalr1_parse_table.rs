//! LALR(1) parse table construction for the given grammar configuration.
//! The LALR(1) parse table is constructed using the `lalr` crate.
//! The LALR(1) parse table is then converted into a representation without a reference to the
//! creating grammar.
//! The reference to the creating grammar is not needed for the actual parsing process. Moreover,
//! it inhibits the use of the parse table in other contexts.
//! This is the reason why we duplicate the `lalr` types here.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use parol_runtime::{NonTerminalIndex, ProductionIndex, TerminalIndex};

use crate::{Cfg, GrammarAnalysisError, GrammarConfig, Pr, Terminal};

/// Type aliases for the LALR(1) parse table construction.
/// The generic parameters are defined to be terminal, non-terminal, and production indices.

type LR1ParseTableLalr<'a> =
    lalr::LR1ParseTable<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type LR1StateLalr<'a> = lalr::LR1State<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type LRActionLalr<'a> = lalr::LRAction<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type LR1ConflictLalr<'a> = lalr::LR1Conflict<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type ItemLalr<'a> = lalr::Item<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type ItemSetLalr<'a> = lalr::ItemSet<'a, TerminalIndex, NonTerminalIndex, ProductionIndex>;
type RhsLalr = lalr::Rhs<TerminalIndex, NonTerminalIndex, ProductionIndex>;
type GrammarLalr = lalr::Grammar<TerminalIndex, NonTerminalIndex, ProductionIndex>;

/// Convert the given grammar configuration into a LALR(1) grammar that can be used to construct
/// the LALR(1) parse table.
impl From<&Cfg> for GrammarLalr {
    fn from(cfg: &Cfg) -> Self {
        let terminal_index = cfg.get_terminal_index_function();
        let non_terminal_index = cfg.get_non_terminal_index_function();

        let mut grammar = GrammarLalr {
            rules: BTreeMap::new(),
            start: non_terminal_index(&cfg.st),
        };

        for (i, Pr(s, rhs, _)) in cfg.pr.iter().enumerate() {
            let lhs = non_terminal_index(s.get_n_ref().unwrap());
            let rhs = RhsLalr {
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

/// An item in the LR(0) state machine.
/// Duplicate of the `lalr` crate's `Item` type without the reference to the creating grammar.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Item {
    /// The production of the item.
    pub prod: ProductionIndex,
    /// The position in the production.
    pub pos: usize,
}

impl From<ItemLalr<'_>> for Item {
    fn from(item: ItemLalr) -> Self {
        Item {
            prod: item.rhs.act,
            pos: item.pos,
        }
    }
}

/// An item set in the LR(0) state machine.
/// Duplicate of the `lalr` crate's `ItemSet` type without the reference to the creating grammar.
#[derive(Debug)]
pub struct ItemSet {
    /// The items in the set.
    pub items: BTreeSet<Item>,
}

impl From<ItemSetLalr<'_>> for ItemSet {
    fn from(item_set: ItemSetLalr) -> Self {
        let items = item_set.items.into_iter().map(|item| item.into()).collect();
        ItemSet { items }
    }
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

impl From<LRActionLalr<'_>> for LRAction {
    fn from(action: LRActionLalr<'_>) -> Self {
        match action {
            lalr::LRAction::Shift(s) => LRAction::Shift(s),
            lalr::LRAction::Reduce(p, r) => LRAction::Reduce(*p, r.act),
            lalr::LRAction::Accept => LRAction::Accept,
        }
    }
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

impl From<LR1StateLalr<'_>> for LR1State {
    fn from(state: LR1StateLalr) -> Self {
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

        LR1State {
            eof_action: state.eof.map(|action| match action {
                lalr::LRAction::Shift(s) => LRAction::Shift(s),
                lalr::LRAction::Reduce(p, r) => LRAction::Reduce(*p, r.act),
                lalr::LRAction::Accept => LRAction::Accept,
            }),
            actions,
            gotos,
        }
    }
}

/// A LALR(1) parse table conflict.
/// Duplicate of the `lalr` crate's `LR1Conflict` type without the reference to the creating grammar.
#[derive(Debug)]
pub enum LRConflict {
    /// A reduce-reduce conflict.
    ReduceReduce {
        /// The LR(0) state in which the conflict occurs.
        state: ItemSet,
        /// The token leading to the conflict, or `None` if the token is EOF.
        token: Option<TerminalIndex>,
        /// The first conflicting rule.
        r1: ProductionIndex,
        /// The second conflicting rule.
        r2: ProductionIndex,
    },
    /// A shift-reduce conflict.
    ShiftReduce {
        /// The LR(0) state in which the conflict appears.
        state: ItemSet,
        /// The token leading to the conflict, or `None` if the token is EOF.
        token: Option<TerminalIndex>,
        /// The reduce rule involved in the conflict.
        rule: ProductionIndex,
    },
}

impl From<LR1ConflictLalr<'_>> for LRConflict {
    fn from(conflict: LR1ConflictLalr) -> Self {
        match conflict {
            LR1ConflictLalr::ReduceReduce {
                state,
                token,
                r1,
                r2,
            } => LRConflict::ReduceReduce {
                state: state.into(),
                token: token.copied(),
                r1: r1.1.act,
                r2: r2.1.act,
            },
            LR1ConflictLalr::ShiftReduce { state, token, rule } => LRConflict::ShiftReduce {
                state: state.into(),
                token: token.copied(),
                rule: rule.1.act,
            },
        }
    }
}
/// The LALR(1) parse table.
#[derive(Debug)]
pub struct LRParseTable {
    /// The states in the parse table.
    pub states: Vec<LR1State>,
}

impl From<LR1ParseTableLalr<'_>> for LRParseTable {
    fn from(parse_table: LR1ParseTableLalr) -> Self {
        let mut states = Vec::new();
        for state in parse_table.states {
            let mut actions = BTreeMap::new();
            for (terminal, action) in state.lookahead {
                actions.insert(*terminal, action.into());
            }

            let gotos = state.goto.into_iter().map(|(n, s)| (*n, s)).collect();

            states.push(LR1State {
                eof_action: state.eof.map(|action| action.into()),
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
    let grammar = GrammarLalr::from(cfg);
    let reduce_on = |_rhs: &RhsLalr, _lookahead: Option<&TerminalIndex>| false;
    let priority_of = |_rhs: &RhsLalr, _lookahead: Option<&TerminalIndex>| 0;
    let parse_table = grammar.lalr1(reduce_on, priority_of).map_err(|e| {
        anyhow!(GrammarAnalysisError::LALR1ParseTableConstructionFailed { conflict: e.into() })
    })?;
    Ok(parse_table.into())
}
