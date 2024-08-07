// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

use parol_runtime::once_cell::sync::Lazy;
#[allow(unused_imports)]
use parol_runtime::parser::{LLKParser, LookaheadDFA, ParseTreeType, ParseType, Production, Trans};
use parol_runtime::{ParolError, ParseTree, TerminalIndex};
use parol_runtime::{ScannerConfig, TokenStream, Tokenizer};
use std::path::Path;

use crate::parol_ls_grammar::ParolLsGrammar;
use crate::parol_ls_grammar_trait::ParolLsGrammarAuto;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};

pub const TERMINALS: &[&str; 41] = &[
    /*  0 */ UNMATCHABLE_TOKEN,
    /*  1 */ UNMATCHABLE_TOKEN,
    /*  2 */ UNMATCHABLE_TOKEN,
    /*  3 */ UNMATCHABLE_TOKEN,
    /*  4 */ UNMATCHABLE_TOKEN,
    /*  5 */ r"%start",
    /*  6 */ r"%title",
    /*  7 */ r"%comment",
    /*  8 */ r"%user_type",
    /*  9 */ r"=",
    /* 10 */ r"%grammar_type",
    /* 11 */ r"%line_comment",
    /* 12 */ r"%block_comment",
    /* 13 */ r"%auto_newline_off",
    /* 14 */ r"%auto_ws_off",
    /* 15 */ r"%on",
    /* 16 */ r"%enter",
    /* 17 */ r"%%",
    /* 18 */ r"::",
    /* 19 */ r":",
    /* 20 */ r";",
    /* 21 */ r"\|",
    /* 22 */ r"<",
    /* 23 */ r">",
    /* 24 */ r"\(",
    /* 25 */ r"\)",
    /* 26 */ r"\[",
    /* 27 */ r"\]",
    /* 28 */ r"\{",
    /* 29 */ r"\}",
    /* 30 */ r"[a-zA-Z_][a-zA-Z0-9_]*",
    /* 31 */ r#""(\\.|[^\\])*?""#,
    /* 32 */ r"'(\\'|[^'])*?'",
    /* 33 */ r"%scanner",
    /* 34 */ r",",
    /* 35 */ r"%sc",
    /* 36 */ r"%push",
    /* 37 */ r"%pop",
    /* 38 */ r"\^",
    /* 39 */ r"\u{2f}(\\.|[^\\])*?\u{2f}",
    /* 40 */ ERROR_TOKEN,
];

pub const TERMINAL_NAMES: &[&str; 41] = &[
    /*  0 */ "EndOfInput",
    /*  1 */ "Newline",
    /*  2 */ "Whitespace",
    /*  3 */ "LineComment",
    /*  4 */ "BlockComment",
    /*  5 */ "PercentStart",
    /*  6 */ "PercentTitle",
    /*  7 */ "PercentComment",
    /*  8 */ "PercentUserUnderscoreType",
    /*  9 */ "Equ",
    /* 10 */ "PercentGrammarUnderscoreType",
    /* 11 */ "PercentLineUnderscoreComment",
    /* 12 */ "PercentBlockUnderscoreComment",
    /* 13 */ "PercentAutoUnderscoreNewlineUnderscoreOff",
    /* 14 */ "PercentAutoUnderscoreWsUnderscoreOff",
    /* 15 */ "PercentOn",
    /* 16 */ "PercentEnter",
    /* 17 */ "PercentPercent",
    /* 18 */ "DoubleColon",
    /* 19 */ "Colon",
    /* 20 */ "Semicolon",
    /* 21 */ "Or",
    /* 22 */ "LT",
    /* 23 */ "GT",
    /* 24 */ "LParen",
    /* 25 */ "RParen",
    /* 26 */ "LBracket",
    /* 27 */ "RBracket",
    /* 28 */ "LBrace",
    /* 29 */ "RBrace",
    /* 30 */ "Identifier",
    /* 31 */ "String",
    /* 32 */ "LiteralString",
    /* 33 */ "PercentScanner",
    /* 34 */ "Comma",
    /* 35 */ "PercentSc",
    /* 36 */ "PercentPush",
    /* 37 */ "PercentPop",
    /* 38 */ "CutOperator",
    /* 39 */ "Regex",
    /* 40 */ "Error",
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 35]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ NEW_LINE_TOKEN,
        /*  2 */ WHITESPACE_TOKEN,
        /*  3 */ r"(//.*(\r\n|\r|\n|$))",
        /*  4 */ r"((?ms)/\*.*?\*/)",
    ],
    &[
        5,  /* PercentStart */
        6,  /* PercentTitle */
        7,  /* PercentComment */
        8,  /* PercentUserUnderscoreType */
        9,  /* Equ */
        10, /* PercentGrammarUnderscoreType */
        11, /* PercentLineUnderscoreComment */
        12, /* PercentBlockUnderscoreComment */
        13, /* PercentAutoUnderscoreNewlineUnderscoreOff */
        14, /* PercentAutoUnderscoreWsUnderscoreOff */
        15, /* PercentOn */
        16, /* PercentEnter */
        17, /* PercentPercent */
        18, /* DoubleColon */
        19, /* Colon */
        20, /* Semicolon */
        21, /* Or */
        22, /* LT */
        23, /* GT */
        24, /* LParen */
        25, /* RParen */
        26, /* LBracket */
        27, /* RBracket */
        28, /* LBrace */
        29, /* RBrace */
        30, /* Identifier */
        31, /* String */
        32, /* LiteralString */
        33, /* PercentScanner */
        34, /* Comma */
        35, /* PercentSc */
        36, /* PercentPush */
        37, /* PercentPop */
        38, /* CutOperator */
        39, /* Regex */
    ],
);

const MAX_K: usize = 1;

pub const NON_TERMINALS: &[&str; 43] = &[
    /*  0 */ "ASTControl",
    /*  1 */ "Alternation",
    /*  2 */ "AlternationList",
    /*  3 */ "Alternations",
    /*  4 */ "AlternationsList",
    /*  5 */ "CutOperator",
    /*  6 */ "Declaration",
    /*  7 */ "DoubleColon",
    /*  8 */ "Factor",
    /*  9 */ "GrammarDefinition",
    /* 10 */ "GrammarDefinitionList",
    /* 11 */ "Group",
    /* 12 */ "Identifier",
    /* 13 */ "IdentifierList",
    /* 14 */ "IdentifierListList",
    /* 15 */ "LiteralString",
    /* 16 */ "NonTerminal",
    /* 17 */ "NonTerminalOpt",
    /* 18 */ "Optional",
    /* 19 */ "ParolLs",
    /* 20 */ "Production",
    /* 21 */ "ProductionLHS",
    /* 22 */ "Prolog",
    /* 23 */ "PrologList",
    /* 24 */ "PrologList0",
    /* 25 */ "Regex",
    /* 26 */ "Repeat",
    /* 27 */ "ScannerDirectives",
    /* 28 */ "ScannerState",
    /* 29 */ "ScannerStateList",
    /* 30 */ "ScannerSwitch",
    /* 31 */ "ScannerSwitchOpt",
    /* 32 */ "SimpleToken",
    /* 33 */ "SimpleTokenOpt",
    /* 34 */ "StartDeclaration",
    /* 35 */ "String",
    /* 36 */ "Symbol",
    /* 37 */ "TokenLiteral",
    /* 38 */ "TokenWithStates",
    /* 39 */ "TokenWithStatesOpt",
    /* 40 */ "UserTypeDeclaration",
    /* 41 */ "UserTypeName",
    /* 42 */ "UserTypeNameList",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 43] = &[
    /* 0 - "ASTControl" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 19, 2, 67), Trans(0, 38, 1, 66)],
        k: 1,
    },
    /* 1 - "Alternation" */
    LookaheadDFA {
        prod0: 26,
        transitions: &[],
        k: 0,
    },
    /* 2 - "AlternationList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 2, 28),
            Trans(0, 21, 2, 28),
            Trans(0, 22, 1, 27),
            Trans(0, 24, 1, 27),
            Trans(0, 25, 2, 28),
            Trans(0, 26, 1, 27),
            Trans(0, 27, 2, 28),
            Trans(0, 28, 1, 27),
            Trans(0, 29, 2, 28),
            Trans(0, 30, 1, 27),
            Trans(0, 31, 1, 27),
            Trans(0, 32, 1, 27),
            Trans(0, 35, 1, 27),
            Trans(0, 36, 1, 27),
            Trans(0, 37, 1, 27),
            Trans(0, 39, 1, 27),
        ],
        k: 1,
    },
    /* 3 - "Alternations" */
    LookaheadDFA {
        prod0: 23,
        transitions: &[],
        k: 0,
    },
    /* 4 - "AlternationsList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 2, 25),
            Trans(0, 21, 1, 24),
            Trans(0, 25, 2, 25),
            Trans(0, 27, 2, 25),
            Trans(0, 29, 2, 25),
        ],
        k: 1,
    },
    /* 5 - "CutOperator" */
    LookaheadDFA {
        prod0: 68,
        transitions: &[],
        k: 0,
    },
    /* 6 - "Declaration" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 6, 1, 7),
            Trans(0, 7, 2, 8),
            Trans(0, 8, 3, 9),
            Trans(0, 10, 4, 10),
            Trans(0, 11, 5, 11),
            Trans(0, 12, 5, 11),
            Trans(0, 13, 5, 11),
            Trans(0, 14, 5, 11),
            Trans(0, 15, 5, 11),
        ],
        k: 1,
    },
    /* 7 - "DoubleColon" */
    LookaheadDFA {
        prod0: 20,
        transitions: &[],
        k: 0,
    },
    /* 8 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 22, 4, 32),
            Trans(0, 24, 1, 29),
            Trans(0, 26, 3, 31),
            Trans(0, 28, 2, 30),
            Trans(0, 30, 4, 32),
            Trans(0, 31, 4, 32),
            Trans(0, 32, 4, 32),
            Trans(0, 35, 4, 32),
            Trans(0, 36, 4, 32),
            Trans(0, 37, 4, 32),
            Trans(0, 39, 4, 32),
        ],
        k: 1,
    },
    /* 9 - "GrammarDefinition" */
    LookaheadDFA {
        prod0: 17,
        transitions: &[],
        k: 0,
    },
    /* 10 - "GrammarDefinitionList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 19), Trans(0, 30, 1, 18)],
        k: 1,
    },
    /* 11 - "Group" */
    LookaheadDFA {
        prod0: 46,
        transitions: &[],
        k: 0,
    },
    /* 12 - "Identifier" */
    LookaheadDFA {
        prod0: 52,
        transitions: &[],
        k: 0,
    },
    /* 13 - "IdentifierList" */
    LookaheadDFA {
        prod0: 58,
        transitions: &[],
        k: 0,
    },
    /* 14 - "IdentifierListList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 60),
            Trans(0, 23, 2, 60),
            Trans(0, 34, 1, 59),
        ],
        k: 1,
    },
    /* 15 - "LiteralString" */
    LookaheadDFA {
        prod0: 54,
        transitions: &[],
        k: 0,
    },
    /* 16 - "NonTerminal" */
    LookaheadDFA {
        prod0: 49,
        transitions: &[],
        k: 0,
    },
    /* 17 - "NonTerminalOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 50),
            Trans(0, 20, 2, 51),
            Trans(0, 21, 2, 51),
            Trans(0, 22, 2, 51),
            Trans(0, 24, 2, 51),
            Trans(0, 25, 2, 51),
            Trans(0, 26, 2, 51),
            Trans(0, 27, 2, 51),
            Trans(0, 28, 2, 51),
            Trans(0, 29, 2, 51),
            Trans(0, 30, 2, 51),
            Trans(0, 31, 2, 51),
            Trans(0, 32, 2, 51),
            Trans(0, 35, 2, 51),
            Trans(0, 36, 2, 51),
            Trans(0, 37, 2, 51),
            Trans(0, 38, 1, 50),
            Trans(0, 39, 2, 51),
        ],
        k: 1,
    },
    /* 18 - "Optional" */
    LookaheadDFA {
        prod0: 47,
        transitions: &[],
        k: 0,
    },
    /* 19 - "ParolLs" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 20 - "Production" */
    LookaheadDFA {
        prod0: 22,
        transitions: &[],
        k: 0,
    },
    /* 21 - "ProductionLHS" */
    LookaheadDFA {
        prod0: 21,
        transitions: &[],
        k: 0,
    },
    /* 22 - "Prolog" */
    LookaheadDFA {
        prod0: 1,
        transitions: &[],
        k: 0,
    },
    /* 23 - "PrologList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 6, 1, 4),
            Trans(0, 7, 1, 4),
            Trans(0, 8, 1, 4),
            Trans(0, 10, 1, 4),
            Trans(0, 11, 1, 4),
            Trans(0, 12, 1, 4),
            Trans(0, 13, 1, 4),
            Trans(0, 14, 1, 4),
            Trans(0, 15, 1, 4),
            Trans(0, 17, 2, 5),
            Trans(0, 33, 2, 5),
        ],
        k: 1,
    },
    /* 24 - "PrologList0" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 17, 2, 3), Trans(0, 33, 1, 2)],
        k: 1,
    },
    /* 25 - "Regex" */
    LookaheadDFA {
        prod0: 73,
        transitions: &[],
        k: 0,
    },
    /* 26 - "Repeat" */
    LookaheadDFA {
        prod0: 48,
        transitions: &[],
        k: 0,
    },
    /* 27 - "ScannerDirectives" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 11, 1, 12),
            Trans(0, 12, 2, 13),
            Trans(0, 13, 3, 14),
            Trans(0, 14, 4, 15),
            Trans(0, 15, 5, 16),
        ],
        k: 1,
    },
    /* 28 - "ScannerState" */
    LookaheadDFA {
        prod0: 55,
        transitions: &[],
        k: 0,
    },
    /* 29 - "ScannerStateList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 11, 1, 56),
            Trans(0, 12, 1, 56),
            Trans(0, 13, 1, 56),
            Trans(0, 14, 1, 56),
            Trans(0, 15, 1, 56),
            Trans(0, 29, 2, 57),
        ],
        k: 1,
    },
    /* 30 - "ScannerSwitch" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 35, 1, 61),
            Trans(0, 36, 2, 62),
            Trans(0, 37, 3, 63),
        ],
        k: 1,
    },
    /* 31 - "ScannerSwitchOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 25, 2, 65), Trans(0, 30, 1, 64)],
        k: 1,
    },
    /* 32 - "SimpleToken" */
    LookaheadDFA {
        prod0: 40,
        transitions: &[],
        k: 0,
    },
    /* 33 - "SimpleTokenOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 41),
            Trans(0, 20, 2, 42),
            Trans(0, 21, 2, 42),
            Trans(0, 22, 2, 42),
            Trans(0, 24, 2, 42),
            Trans(0, 25, 2, 42),
            Trans(0, 26, 2, 42),
            Trans(0, 27, 2, 42),
            Trans(0, 28, 2, 42),
            Trans(0, 29, 2, 42),
            Trans(0, 30, 2, 42),
            Trans(0, 31, 2, 42),
            Trans(0, 32, 2, 42),
            Trans(0, 35, 2, 42),
            Trans(0, 36, 2, 42),
            Trans(0, 37, 2, 42),
            Trans(0, 38, 1, 41),
            Trans(0, 39, 2, 42),
        ],
        k: 1,
    },
    /* 34 - "StartDeclaration" */
    LookaheadDFA {
        prod0: 6,
        transitions: &[],
        k: 0,
    },
    /* 35 - "String" */
    LookaheadDFA {
        prod0: 53,
        transitions: &[],
        k: 0,
    },
    /* 36 - "Symbol" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 22, 3, 35),
            Trans(0, 30, 1, 33),
            Trans(0, 31, 2, 34),
            Trans(0, 32, 2, 34),
            Trans(0, 35, 4, 36),
            Trans(0, 36, 4, 36),
            Trans(0, 37, 4, 36),
            Trans(0, 39, 2, 34),
        ],
        k: 1,
    },
    /* 37 - "TokenLiteral" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 31, 1, 37),
            Trans(0, 32, 2, 38),
            Trans(0, 39, 3, 39),
        ],
        k: 1,
    },
    /* 38 - "TokenWithStates" */
    LookaheadDFA {
        prod0: 43,
        transitions: &[],
        k: 0,
    },
    /* 39 - "TokenWithStatesOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 44),
            Trans(0, 20, 2, 45),
            Trans(0, 21, 2, 45),
            Trans(0, 22, 2, 45),
            Trans(0, 24, 2, 45),
            Trans(0, 25, 2, 45),
            Trans(0, 26, 2, 45),
            Trans(0, 27, 2, 45),
            Trans(0, 28, 2, 45),
            Trans(0, 29, 2, 45),
            Trans(0, 30, 2, 45),
            Trans(0, 31, 2, 45),
            Trans(0, 32, 2, 45),
            Trans(0, 35, 2, 45),
            Trans(0, 36, 2, 45),
            Trans(0, 37, 2, 45),
            Trans(0, 38, 1, 44),
            Trans(0, 39, 2, 45),
        ],
        k: 1,
    },
    /* 40 - "UserTypeDeclaration" */
    LookaheadDFA {
        prod0: 69,
        transitions: &[],
        k: 0,
    },
    /* 41 - "UserTypeName" */
    LookaheadDFA {
        prod0: 70,
        transitions: &[],
        k: 0,
    },
    /* 42 - "UserTypeNameList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 6, 2, 72),
            Trans(0, 7, 2, 72),
            Trans(0, 8, 2, 72),
            Trans(0, 10, 2, 72),
            Trans(0, 11, 2, 72),
            Trans(0, 12, 2, 72),
            Trans(0, 13, 2, 72),
            Trans(0, 14, 2, 72),
            Trans(0, 15, 2, 72),
            Trans(0, 17, 2, 72),
            Trans(0, 18, 1, 71),
            Trans(0, 20, 2, 72),
            Trans(0, 21, 2, 72),
            Trans(0, 22, 2, 72),
            Trans(0, 24, 2, 72),
            Trans(0, 25, 2, 72),
            Trans(0, 26, 2, 72),
            Trans(0, 27, 2, 72),
            Trans(0, 28, 2, 72),
            Trans(0, 29, 2, 72),
            Trans(0, 30, 2, 72),
            Trans(0, 31, 2, 72),
            Trans(0, 32, 2, 72),
            Trans(0, 33, 2, 72),
            Trans(0, 35, 2, 72),
            Trans(0, 36, 2, 72),
            Trans(0, 37, 2, 72),
            Trans(0, 39, 2, 72),
        ],
        k: 1,
    },
];

pub const PRODUCTIONS: &[Production; 74] = &[
    // 0 - ParolLs: Prolog GrammarDefinition;
    Production {
        lhs: 19,
        production: &[ParseType::N(9), ParseType::N(22)],
    },
    // 1 - Prolog: StartDeclaration PrologList /* Vec */ PrologList0 /* Vec */;
    Production {
        lhs: 22,
        production: &[ParseType::N(24), ParseType::N(23), ParseType::N(34)],
    },
    // 2 - PrologList0: ScannerState PrologList0;
    Production {
        lhs: 24,
        production: &[ParseType::N(24), ParseType::N(28)],
    },
    // 3 - PrologList0: ;
    Production {
        lhs: 24,
        production: &[],
    },
    // 4 - PrologList: Declaration PrologList;
    Production {
        lhs: 23,
        production: &[ParseType::N(23), ParseType::N(6)],
    },
    // 5 - PrologList: ;
    Production {
        lhs: 23,
        production: &[],
    },
    // 6 - StartDeclaration: "%start" Identifier;
    Production {
        lhs: 34,
        production: &[ParseType::N(12), ParseType::T(5)],
    },
    // 7 - Declaration: "%title" String;
    Production {
        lhs: 6,
        production: &[ParseType::N(35), ParseType::T(6)],
    },
    // 8 - Declaration: "%comment" String;
    Production {
        lhs: 6,
        production: &[ParseType::N(35), ParseType::T(7)],
    },
    // 9 - Declaration: "%user_type" Identifier "=" UserTypeName;
    Production {
        lhs: 6,
        production: &[
            ParseType::N(41),
            ParseType::T(9),
            ParseType::N(12),
            ParseType::T(8),
        ],
    },
    // 10 - Declaration: '%grammar_type' LiteralString;
    Production {
        lhs: 6,
        production: &[ParseType::N(15), ParseType::T(10)],
    },
    // 11 - Declaration: ScannerDirectives;
    Production {
        lhs: 6,
        production: &[ParseType::N(27)],
    },
    // 12 - ScannerDirectives: "%line_comment" TokenLiteral;
    Production {
        lhs: 27,
        production: &[ParseType::N(37), ParseType::T(11)],
    },
    // 13 - ScannerDirectives: "%block_comment" TokenLiteral TokenLiteral;
    Production {
        lhs: 27,
        production: &[ParseType::N(37), ParseType::N(37), ParseType::T(12)],
    },
    // 14 - ScannerDirectives: "%auto_newline_off";
    Production {
        lhs: 27,
        production: &[ParseType::T(13)],
    },
    // 15 - ScannerDirectives: "%auto_ws_off";
    Production {
        lhs: 27,
        production: &[ParseType::T(14)],
    },
    // 16 - ScannerDirectives: '%on' IdentifierList '%enter' Identifier;
    Production {
        lhs: 27,
        production: &[
            ParseType::N(12),
            ParseType::T(16),
            ParseType::N(13),
            ParseType::T(15),
        ],
    },
    // 17 - GrammarDefinition: "%%" Production GrammarDefinitionList /* Vec */;
    Production {
        lhs: 9,
        production: &[ParseType::N(10), ParseType::N(20), ParseType::T(17)],
    },
    // 18 - GrammarDefinitionList: Production GrammarDefinitionList;
    Production {
        lhs: 10,
        production: &[ParseType::N(10), ParseType::N(20)],
    },
    // 19 - GrammarDefinitionList: ;
    Production {
        lhs: 10,
        production: &[],
    },
    // 20 - DoubleColon: "::";
    Production {
        lhs: 7,
        production: &[ParseType::T(18)],
    },
    // 21 - ProductionLHS: Identifier ":";
    Production {
        lhs: 21,
        production: &[ParseType::T(19), ParseType::N(12)],
    },
    // 22 - Production: ProductionLHS Alternations ";";
    Production {
        lhs: 20,
        production: &[ParseType::T(20), ParseType::N(3), ParseType::N(21)],
    },
    // 23 - Alternations: Alternation AlternationsList /* Vec */;
    Production {
        lhs: 3,
        production: &[ParseType::N(4), ParseType::N(1)],
    },
    // 24 - AlternationsList: '|' Alternation AlternationsList;
    Production {
        lhs: 4,
        production: &[ParseType::N(4), ParseType::N(1), ParseType::T(21)],
    },
    // 25 - AlternationsList: ;
    Production {
        lhs: 4,
        production: &[],
    },
    // 26 - Alternation: AlternationList /* Vec */;
    Production {
        lhs: 1,
        production: &[ParseType::N(2)],
    },
    // 27 - AlternationList: Factor AlternationList;
    Production {
        lhs: 2,
        production: &[ParseType::N(2), ParseType::N(8)],
    },
    // 28 - AlternationList: ;
    Production {
        lhs: 2,
        production: &[],
    },
    // 29 - Factor: Group;
    Production {
        lhs: 8,
        production: &[ParseType::N(11)],
    },
    // 30 - Factor: Repeat;
    Production {
        lhs: 8,
        production: &[ParseType::N(26)],
    },
    // 31 - Factor: Optional;
    Production {
        lhs: 8,
        production: &[ParseType::N(18)],
    },
    // 32 - Factor: Symbol;
    Production {
        lhs: 8,
        production: &[ParseType::N(36)],
    },
    // 33 - Symbol: NonTerminal;
    Production {
        lhs: 36,
        production: &[ParseType::N(16)],
    },
    // 34 - Symbol: SimpleToken;
    Production {
        lhs: 36,
        production: &[ParseType::N(32)],
    },
    // 35 - Symbol: TokenWithStates;
    Production {
        lhs: 36,
        production: &[ParseType::N(38)],
    },
    // 36 - Symbol: ScannerSwitch;
    Production {
        lhs: 36,
        production: &[ParseType::N(30)],
    },
    // 37 - TokenLiteral: String;
    Production {
        lhs: 37,
        production: &[ParseType::N(35)],
    },
    // 38 - TokenLiteral: LiteralString;
    Production {
        lhs: 37,
        production: &[ParseType::N(15)],
    },
    // 39 - TokenLiteral: Regex;
    Production {
        lhs: 37,
        production: &[ParseType::N(25)],
    },
    // 40 - SimpleToken: TokenLiteral SimpleTokenOpt /* Option */;
    Production {
        lhs: 32,
        production: &[ParseType::N(33), ParseType::N(37)],
    },
    // 41 - SimpleTokenOpt: ASTControl;
    Production {
        lhs: 33,
        production: &[ParseType::N(0)],
    },
    // 42 - SimpleTokenOpt: ;
    Production {
        lhs: 33,
        production: &[],
    },
    // 43 - TokenWithStates: "<" IdentifierList ">" TokenLiteral TokenWithStatesOpt /* Option */;
    Production {
        lhs: 38,
        production: &[
            ParseType::N(39),
            ParseType::N(37),
            ParseType::T(23),
            ParseType::N(13),
            ParseType::T(22),
        ],
    },
    // 44 - TokenWithStatesOpt: ASTControl;
    Production {
        lhs: 39,
        production: &[ParseType::N(0)],
    },
    // 45 - TokenWithStatesOpt: ;
    Production {
        lhs: 39,
        production: &[],
    },
    // 46 - Group: '(' Alternations ')';
    Production {
        lhs: 11,
        production: &[ParseType::T(25), ParseType::N(3), ParseType::T(24)],
    },
    // 47 - Optional: '[' Alternations ']';
    Production {
        lhs: 18,
        production: &[ParseType::T(27), ParseType::N(3), ParseType::T(26)],
    },
    // 48 - Repeat: '{' Alternations '}';
    Production {
        lhs: 26,
        production: &[ParseType::T(29), ParseType::N(3), ParseType::T(28)],
    },
    // 49 - NonTerminal: Identifier NonTerminalOpt /* Option */;
    Production {
        lhs: 16,
        production: &[ParseType::N(17), ParseType::N(12)],
    },
    // 50 - NonTerminalOpt: ASTControl;
    Production {
        lhs: 17,
        production: &[ParseType::N(0)],
    },
    // 51 - NonTerminalOpt: ;
    Production {
        lhs: 17,
        production: &[],
    },
    // 52 - Identifier: /[a-zA-Z_][a-zA-Z0-9_]*/;
    Production {
        lhs: 12,
        production: &[ParseType::T(30)],
    },
    // 53 - String: /"(\\.|[^\\])*?"/;
    Production {
        lhs: 35,
        production: &[ParseType::T(31)],
    },
    // 54 - LiteralString: /'(\\'|[^'])*?'/;
    Production {
        lhs: 15,
        production: &[ParseType::T(32)],
    },
    // 55 - ScannerState: "%scanner" Identifier '{' ScannerStateList /* Vec */ '}';
    Production {
        lhs: 28,
        production: &[
            ParseType::T(29),
            ParseType::N(29),
            ParseType::T(28),
            ParseType::N(12),
            ParseType::T(33),
        ],
    },
    // 56 - ScannerStateList: ScannerDirectives ScannerStateList;
    Production {
        lhs: 29,
        production: &[ParseType::N(29), ParseType::N(27)],
    },
    // 57 - ScannerStateList: ;
    Production {
        lhs: 29,
        production: &[],
    },
    // 58 - IdentifierList: Identifier IdentifierListList /* Vec */;
    Production {
        lhs: 13,
        production: &[ParseType::N(14), ParseType::N(12)],
    },
    // 59 - IdentifierListList: "," Identifier IdentifierListList;
    Production {
        lhs: 14,
        production: &[ParseType::N(14), ParseType::N(12), ParseType::T(34)],
    },
    // 60 - IdentifierListList: ;
    Production {
        lhs: 14,
        production: &[],
    },
    // 61 - ScannerSwitch: "%sc" '(' ScannerSwitchOpt /* Option */ ')';
    Production {
        lhs: 30,
        production: &[
            ParseType::T(25),
            ParseType::N(31),
            ParseType::T(24),
            ParseType::T(35),
        ],
    },
    // 62 - ScannerSwitch: "%push" '(' Identifier ')';
    Production {
        lhs: 30,
        production: &[
            ParseType::T(25),
            ParseType::N(12),
            ParseType::T(24),
            ParseType::T(36),
        ],
    },
    // 63 - ScannerSwitch: "%pop" '(' ')';
    Production {
        lhs: 30,
        production: &[ParseType::T(25), ParseType::T(24), ParseType::T(37)],
    },
    // 64 - ScannerSwitchOpt: Identifier;
    Production {
        lhs: 31,
        production: &[ParseType::N(12)],
    },
    // 65 - ScannerSwitchOpt: ;
    Production {
        lhs: 31,
        production: &[],
    },
    // 66 - ASTControl: CutOperator;
    Production {
        lhs: 0,
        production: &[ParseType::N(5)],
    },
    // 67 - ASTControl: UserTypeDeclaration;
    Production {
        lhs: 0,
        production: &[ParseType::N(40)],
    },
    // 68 - CutOperator: '^';
    Production {
        lhs: 5,
        production: &[ParseType::T(38)],
    },
    // 69 - UserTypeDeclaration: ":" UserTypeName;
    Production {
        lhs: 40,
        production: &[ParseType::N(41), ParseType::T(19)],
    },
    // 70 - UserTypeName: Identifier UserTypeNameList /* Vec */;
    Production {
        lhs: 41,
        production: &[ParseType::N(42), ParseType::N(12)],
    },
    // 71 - UserTypeNameList: DoubleColon Identifier UserTypeNameList;
    Production {
        lhs: 42,
        production: &[ParseType::N(42), ParseType::N(12), ParseType::N(7)],
    },
    // 72 - UserTypeNameList: ;
    Production {
        lhs: 42,
        production: &[],
    },
    // 73 - Regex: /\u{2f}(\\.|[^\\])*?\u{2f}/;
    Production {
        lhs: 25,
        production: &[ParseType::T(39)],
    },
];

static SCANNERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| {
    vec![ScannerConfig::new(
        "INITIAL",
        Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap(),
        &[],
    )]
});

pub fn parse<'t, T>(
    input: &'t str,
    file_name: T,
    user_actions: &mut ParolLsGrammar,
) -> Result<ParseTree<'t>, ParolError>
where
    T: AsRef<Path>,
{
    let mut llk_parser = LLKParser::new(
        19,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    llk_parser.trim_parse_tree();

    // Initialize wrapper
    let mut user_actions = ParolLsGrammarAuto::new(user_actions);
    llk_parser.parse(
        TokenStream::new(input, file_name, &SCANNERS, MAX_K).unwrap(),
        &mut user_actions,
    )
}
