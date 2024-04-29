//! LALR(1) parse table construction for the given grammar configuration.
//! The LALR(1) parse table is constructed using the `lalr` crate.
//! The LALR(1) parse table is then converted into a representation without a reference to the
//! creating grammar.
//! The reference to the creating grammar is not needed for the actual parsing process. Moreover,
//! it inhibits the use of the parse table in other contexts.
//! This is the first reason why we duplicate the `lalr` types here.
//! The second reason is that we don't handle the eof action in the same way as the `lalr` crate.
//! The `lalr` crate uses a separate field for the eof action, while we include the eof action in
//! the actions field for the terminal EOI.
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use anyhow::{anyhow, Result};
use parol_runtime::{
    lexer::{BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, WHITESPACE},
    log::trace,
    NonTerminalIndex, ProductionIndex, TerminalIndex,
};

use crate::{
    grammar::cfg::{NonTerminalIndexFn, TerminalIndexFn},
    render_par_string, Cfg, GrammarAnalysisError, GrammarConfig, Pr, Terminal,
};

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
        let ti = cfg.get_terminal_index_function();
        let nti = cfg.get_non_terminal_index_function();

        let mut grammar = GrammarLalr {
            rules: BTreeMap::new(),
            start: nti.non_terminal_index(&cfg.st),
        };

        for (i, Pr(s, rhs, _)) in cfg.pr.iter().enumerate() {
            let lhs = nti.non_terminal_index(s.get_n_ref().unwrap());
            let rhs = RhsLalr {
                syms: rhs
                    .iter()
                    .map(|s| match s {
                        crate::Symbol::N(n, _, _) => {
                            lalr::Symbol::Nonterminal(nti.non_terminal_index(n))
                        }
                        crate::Symbol::T(Terminal::Trm(s, k, _, _, _)) => {
                            lalr::Symbol::Terminal(ti.terminal_index(s, *k))
                        }
                        _ => unreachable!(),
                    })
                    .collect(),
                act: i,
            };
            trace!("LALR(1) rule: {} -> {:?}", lhs, rhs);
            grammar.rules.entry(lhs).or_default().push(rhs);
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
    /// The actions to take for each terminal in the state.
    pub actions: BTreeMap<TerminalIndex, LRAction>,
    /// The gotos to take for each non-terminal in the state.
    pub gotos: BTreeMap<NonTerminalIndex, usize>,
}

impl From<LR1StateLalr<'_>> for LR1State {
    fn from(state: LR1StateLalr) -> Self {
        let mut actions = BTreeMap::new();

        // Add EOF action if present
        if let Some(action) = state.eof {
            actions.insert(EOI, action.into());
        };

        // Add actions for all other terminals
        for (terminal, action) in state.lookahead {
            actions.insert(*terminal, action.into());
        }

        let gotos = state.goto.into_iter().map(|(n, s)| (*n, s)).collect();

        LR1State { actions, gotos }
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

/// An error that occurs when a LALR(1) parse table conflict is detected.
/// It supports better diagnostics than the plain `LRConflict`.
#[derive(Debug)]
pub struct LRConflictError {
    /// The conflict that occurred.
    pub conflict: LRConflict,
    cfg: Option<Cfg>,
}

impl LRConflictError {
    /// Create a new `LRConflictError` with the given conflict and optional grammar configuration.
    pub fn new(conflict: LRConflict, cfg: Option<Cfg>) -> Self {
        LRConflictError { conflict, cfg }
    }

    /// Set the grammar configuration for the error.
    pub fn set_cfg(&mut self, cfg: Cfg) {
        self.cfg = Some(cfg);
    }
}

impl From<LRConflict> for LRConflictError {
    fn from(conflict: LRConflict) -> Self {
        LRConflictError::new(conflict, None)
    }
}

impl Display for LRConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Provide a terminal resolver and non-terminal resolver implementation in the Cfg

        // Terminal index to string function (terminal resolver)
        let tr: Box<dyn Fn(TerminalIndex) -> String> = if let Some(cfg) = &self.cfg {
            let terminals = cfg
                .get_ordered_terminals()
                .iter()
                .map(|(t, _, _)| t.to_string())
                .collect::<Vec<_>>();
            Box::new(move |ti: TerminalIndex| {
                if ti >= FIRST_USER_TOKEN {
                    terminals[(ti - FIRST_USER_TOKEN) as usize].clone()
                } else {
                    match ti {
                        EOI => "<$>".to_owned(),
                        NEW_LINE => "<NL>".to_owned(),
                        WHITESPACE => "<WS>".to_owned(),
                        LINE_COMMENT => "<LC>".to_owned(),
                        BLOCK_COMMENT => "<BC>".to_owned(),
                        _ => unreachable!(),
                    }
                }
            }) as Box<dyn Fn(TerminalIndex) -> String>
        } else {
            // Default resolver that just returns the index as string
            Box::new(|i: TerminalIndex| i.to_string()) as Box<dyn Fn(TerminalIndex) -> String>
        };

        match &self.conflict {
            LRConflict::ReduceReduce {
                state,
                token,
                r1,
                r2,
            } => {
                writeln!(
                    f,
                    "Reduce-reduce conflict in state {:?} on token {}",
                    state,
                    token.map_or("<$>".to_owned(), tr)
                )?;
                if let Some(cfg) = &self.cfg {
                    writeln!(
                        f,
                        "Can't decide which of the following two productions to reduce with:",
                    )?;
                    writeln!(f, "  Production {}: {}", r1, cfg.pr[*r1])?;
                    writeln!(f, "  Production {}: {}", r2, cfg.pr[*r2])?;
                }
                Ok(())
            }
            LRConflict::ShiftReduce { state, token, rule } => {
                if let Some(cfg) = &self.cfg {
                    writeln!(f, "Shift-reduce conflict in state")?;
                    state.items.iter().for_each(|item| {
                        let Pr(lhs, rhs, _) = &cfg.pr[item.prod];
                        let mut r = rhs
                            .iter()
                            .enumerate()
                            .map(|(i, s)| {
                                if i == item.pos {
                                    format!("•{}", s)
                                } else {
                                    format!("{}", s)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        if item.pos == rhs.len() {
                            r.push('•');
                        }
                        writeln!(f, "  {},{}: {} -> {}", item.prod, item.pos, lhs, r).unwrap();
                    });
                    writeln!(
                        f,
                        "Can't decide between shifting the token or reducing with the production:",
                    )?;
                    writeln!(
                        f,
                        "  Token      {}: {}",
                        token.map_or("<$>".to_owned(), |t| t.to_string()),
                        token.map_or("<$>".to_owned(), tr)
                    )?;
                    writeln!(f, "  Production {}: {}", rule, cfg.pr[*rule])?;
                } else {
                    writeln!(
                        f,
                        "Shift-reduce conflict in state {:?} on token {:?}",
                        state, token
                    )?;
                }
                Ok(())
            }
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
        for state in parse_table.states.into_iter() {
            let state = state.into();
            states.push(state);
        }

        LRParseTable { states }
    }
}

/// Calculate the LALR(1) parse table for the given grammar configuration.
pub fn calculate_lalr1_parse_table(grammar_config: &GrammarConfig) -> Result<LRParseTable> {
    trace!("CFG: \n{}", render_par_string(grammar_config, true)?);
    let cfg = &grammar_config.cfg;
    let grammar = GrammarLalr::from(cfg);
    trace!("{:#?}", grammar);
    let reduce_on = |_rhs: &RhsLalr, _lookahead: Option<&TerminalIndex>| true;
    let priority_of = |rhs: &RhsLalr, _lookahead: Option<&TerminalIndex>| {
        // Use negative production index as priority:
        // The production which comes earlier in the grammar description has the higher priority.
        -(rhs.act as i32)
    };
    let parse_table = grammar.lalr1(reduce_on, priority_of).map_err(|e| {
        let conflict: LRConflict = e.into();
        let mut conflict: LRConflictError = conflict.into();
        conflict.set_cfg(cfg.clone());
        anyhow!(GrammarAnalysisError::LALR1ParseTableConstructionFailed { conflict })
    })?;
    trace!("LALR(1) parse table: {:#?}", parse_table);
    let parse_table = LRParseTable::from(parse_table);
    trace!("Converted LALR(1) parse table: {:#?}", parse_table);
    Ok(parse_table)
}
