// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

use parol_runtime::once_cell::sync::Lazy;
use parol_runtime::parser::parse_tree_type::TreeConstruct;
#[allow(unused_imports)]
use parol_runtime::parser::{LLKParser, LookaheadDFA, ParseType, Production, Trans};
use parol_runtime::{ParolError, ParseTree, TerminalIndex};
use parol_runtime::{ScannerConfig, TokenStream, Tokenizer};
use std::path::Path;

use crate::list_grammar::ListGrammar;
use crate::list_grammar_trait::ListGrammarAuto;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};

pub const TERMINALS: &[(&str, Option<(bool, &str)>); 8] = &[
    /* 0 */ (UNMATCHABLE_TOKEN, None),
    /* 1 */ (UNMATCHABLE_TOKEN, None),
    /* 2 */ (UNMATCHABLE_TOKEN, None),
    /* 3 */ (UNMATCHABLE_TOKEN, None),
    /* 4 */ (UNMATCHABLE_TOKEN, None),
    /* 5 */ (r",", None),
    /* 6 */ (r"0|[1-9][0-9]*", None),
    /* 7 */ (ERROR_TOKEN, None),
];

pub const TERMINAL_NAMES: &[&str; 8] = &[
    /* 0 */ "EndOfInput",
    /* 1 */ "Newline",
    /* 2 */ "Whitespace",
    /* 3 */ "LineComment",
    /* 4 */ "BlockComment",
    /* 5 */ "Comma",
    /* 6 */ "Num",
    /* 7 */ "Error",
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 2]) = (
    &[
        /* 0 */ UNMATCHABLE_TOKEN,
        /* 1 */ NEW_LINE_TOKEN,
        /* 2 */ WHITESPACE_TOKEN,
        /* 3 */ r"//.*(\r\n|\r|\n)?",
        /* 4 */ UNMATCHABLE_TOKEN,
    ],
    &[5 /* Comma */, 6 /* Num */],
);

const MAX_K: usize = 2;

pub const NON_TERMINALS: &[&str; 7] = &[
    /* 0 */ "Items",
    /* 1 */ "ItemsList",
    /* 2 */ "List",
    /* 3 */ "ListOpt",
    /* 4 */ "Num",
    /* 5 */ "TrailingComma",
    /* 6 */ "TrailingCommaOpt",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 7] = &[
    /* 0 - "Items" */
    LookaheadDFA {
        prod0: 3,
        transitions: &[],
        k: 0,
    },
    /* 1 - "ItemsList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 3, 5),
            Trans(0, 5, 1, -1),
            Trans(1, 0, 3, 5),
            Trans(1, 6, 2, 4),
        ],
        k: 2,
    },
    /* 2 - "List" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 3 - "ListOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 2), Trans(0, 5, 2, 2), Trans(0, 6, 1, 1)],
        k: 1,
    },
    /* 4 - "Num" */
    LookaheadDFA {
        prod0: 6,
        transitions: &[],
        k: 0,
    },
    /* 5 - "TrailingComma" */
    LookaheadDFA {
        prod0: 7,
        transitions: &[],
        k: 0,
    },
    /* 6 - "TrailingCommaOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 9), Trans(0, 5, 1, 8)],
        k: 1,
    },
];

pub const PRODUCTIONS: &[Production; 10] = &[
    // 0 - List: ListOpt /* Option */ TrailingComma^ /* Clipped */;
    Production {
        lhs: 2,
        production: &[ParseType::N(5), ParseType::N(3)],
    },
    // 1 - ListOpt: Items : crate::list_grammar::Numbers ;
    Production {
        lhs: 3,
        production: &[ParseType::N(0)],
    },
    // 2 - ListOpt: ;
    Production {
        lhs: 3,
        production: &[],
    },
    // 3 - Items: Num ItemsList /* Vec */;
    Production {
        lhs: 0,
        production: &[ParseType::N(1), ParseType::N(4)],
    },
    // 4 - ItemsList: ","^ /* Clipped */ Num ItemsList;
    Production {
        lhs: 1,
        production: &[ParseType::N(1), ParseType::N(4), ParseType::T(5)],
    },
    // 5 - ItemsList: ;
    Production {
        lhs: 1,
        production: &[],
    },
    // 6 - Num: "0|[1-9][0-9]*";
    Production {
        lhs: 4,
        production: &[ParseType::T(6)],
    },
    // 7 - TrailingComma: TrailingCommaOpt /* Option */;
    Production {
        lhs: 5,
        production: &[ParseType::N(6)],
    },
    // 8 - TrailingCommaOpt: ","^ /* Clipped */;
    Production {
        lhs: 6,
        production: &[ParseType::T(5)],
    },
    // 9 - TrailingCommaOpt: ;
    Production {
        lhs: 6,
        production: &[],
    },
];

static SCANNERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| {
    vec![ScannerConfig::new(
        "INITIAL",
        Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap(),
        &[],
    )]
});

pub fn parse<T>(
    input: &str,
    file_name: T,
    user_actions: &mut ListGrammar,
) -> Result<ParseTree, ParolError>
where
    T: AsRef<Path>,
{
    use parol_runtime::parser::parse_tree_type::SynTree;
    use parol_runtime::parser::parser_types::SynTreeFlavor;
    use parol_runtime::syntree::Builder;
    let mut builder = Builder::<SynTree, SynTreeFlavor>::new_with();
    parse_into(input, &mut builder, file_name, user_actions)?;
    Ok(builder.build()?)
}
#[allow(dead_code)]
pub fn parse_into<'t, T: TreeConstruct<'t>>(
    input: &'t str,
    tree_builder: &mut T,
    file_name: impl AsRef<Path>,
    user_actions: &mut ListGrammar,
) -> Result<(), ParolError>
where
    ParolError: From<T::Error>,
{
    let mut llk_parser = LLKParser::new(
        2,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    // Initialize wrapper
    let mut user_actions = ListGrammarAuto::new(user_actions);
    llk_parser.parse_into::<T>(
        tree_builder,
        TokenStream::new(input, file_name, &SCANNERS, MAX_K).unwrap(),
        &mut user_actions,
    )
}
