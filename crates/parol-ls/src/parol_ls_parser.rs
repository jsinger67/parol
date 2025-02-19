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

pub const TERMINALS: &[(&str, Option<(bool, &str)>); 45] = &[
    /*  0 */ (UNMATCHABLE_TOKEN, None),
    /*  1 */ (UNMATCHABLE_TOKEN, None),
    /*  2 */ (UNMATCHABLE_TOKEN, None),
    /*  3 */ (UNMATCHABLE_TOKEN, None),
    /*  4 */ (UNMATCHABLE_TOKEN, None),
    /*  5 */ (r"%start", None),
    /*  6 */ (r"%title", None),
    /*  7 */ (r"%comment", None),
    /*  8 */ (r"%user_type", None),
    /*  9 */ (r"=", None),
    /* 10 */ (r"%production_type", None),
    /* 11 */ (r"%grammar_type", None),
    /* 12 */ (r"%line_comment", None),
    /* 13 */ (r"%block_comment", None),
    /* 14 */ (r"%auto_newline_off", None),
    /* 15 */ (r"%auto_ws_off", None),
    /* 16 */ (r"%on", None),
    /* 17 */ (r"%enter", None),
    /* 18 */ (r"%%", None),
    /* 19 */ (r"::", None),
    /* 20 */ (r":", None),
    /* 21 */ (r";", None),
    /* 22 */ (r"\|", None),
    /* 23 */ (r"<", None),
    /* 24 */ (r">", None),
    /* 25 */ (r"\(", None),
    /* 26 */ (r"\)", None),
    /* 27 */ (r"\[", None),
    /* 28 */ (r"\]", None),
    /* 29 */ (r"\{", None),
    /* 30 */ (r"\}", None),
    /* 31 */ (r"[a-zA-Z_][a-zA-Z0-9_]*", None),
    /* 32 */ (r#""(\\.|[^"])*""#, None),
    /* 33 */ (r"'(\\.|[^'])*'", None),
    /* 34 */ (r"%scanner", None),
    /* 35 */ (r",", None),
    /* 36 */ (r"%sc", None),
    /* 37 */ (r"%push", None),
    /* 38 */ (r"%pop", None),
    /* 39 */ (r"@", None),
    /* 40 */ (r"\^", None),
    /* 41 */ (r"/(\\.|[^\/])*/", None),
    /* 42 */ (r"\?=", None),
    /* 43 */ (r"\?!", None),
    /* 44 */ (ERROR_TOKEN, None),
];

pub const TERMINAL_NAMES: &[&str; 45] = &[
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
    /* 10 */ "PercentProductionUnderscoreType",
    /* 11 */ "PercentGrammarUnderscoreType",
    /* 12 */ "PercentLineUnderscoreComment",
    /* 13 */ "PercentBlockUnderscoreComment",
    /* 14 */ "PercentAutoUnderscoreNewlineUnderscoreOff",
    /* 15 */ "PercentAutoUnderscoreWsUnderscoreOff",
    /* 16 */ "PercentOn",
    /* 17 */ "PercentEnter",
    /* 18 */ "PercentPercent",
    /* 19 */ "DoubleColon",
    /* 20 */ "Colon",
    /* 21 */ "Semicolon",
    /* 22 */ "Or",
    /* 23 */ "LT",
    /* 24 */ "GT",
    /* 25 */ "LParen",
    /* 26 */ "RParen",
    /* 27 */ "LBracket",
    /* 28 */ "RBracket",
    /* 29 */ "LBrace",
    /* 30 */ "RBrace",
    /* 31 */ "Identifier",
    /* 32 */ "String",
    /* 33 */ "LiteralString",
    /* 34 */ "PercentScanner",
    /* 35 */ "Comma",
    /* 36 */ "PercentSc",
    /* 37 */ "PercentPush",
    /* 38 */ "PercentPop",
    /* 39 */ "At",
    /* 40 */ "CutOperator",
    /* 41 */ "Regex",
    /* 42 */ "PositiveLookahead",
    /* 43 */ "NegativeLookahead",
    /* 44 */ "Error",
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 39]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ NEW_LINE_TOKEN,
        /*  2 */ WHITESPACE_TOKEN,
        /*  3 */ r"//.*(\r\n|\r|\n)?",
        /*  4 */ r"/\*([^*]|\*[^/])*\*/",
    ],
    &[
        5,  /* PercentStart */
        6,  /* PercentTitle */
        7,  /* PercentComment */
        8,  /* PercentUserUnderscoreType */
        9,  /* Equ */
        10, /* PercentProductionUnderscoreType */
        11, /* PercentGrammarUnderscoreType */
        12, /* PercentLineUnderscoreComment */
        13, /* PercentBlockUnderscoreComment */
        14, /* PercentAutoUnderscoreNewlineUnderscoreOff */
        15, /* PercentAutoUnderscoreWsUnderscoreOff */
        16, /* PercentOn */
        17, /* PercentEnter */
        18, /* PercentPercent */
        19, /* DoubleColon */
        20, /* Colon */
        21, /* Semicolon */
        22, /* Or */
        23, /* LT */
        24, /* GT */
        25, /* LParen */
        26, /* RParen */
        27, /* LBracket */
        28, /* RBracket */
        29, /* LBrace */
        30, /* RBrace */
        31, /* Identifier */
        32, /* String */
        33, /* LiteralString */
        34, /* PercentScanner */
        35, /* Comma */
        36, /* PercentSc */
        37, /* PercentPush */
        38, /* PercentPop */
        39, /* At */
        40, /* CutOperator */
        41, /* Regex */
        42, /* PositiveLookahead */
        43, /* NegativeLookahead */
    ],
);

const MAX_K: usize = 1;

pub const NON_TERMINALS: &[&str; 51] = &[
    /*  0 */ "ASTControl",
    /*  1 */ "ASTControlOpt",
    /*  2 */ "Alternation",
    /*  3 */ "AlternationList",
    /*  4 */ "Alternations",
    /*  5 */ "AlternationsList",
    /*  6 */ "CutOperator",
    /*  7 */ "Declaration",
    /*  8 */ "DoubleColon",
    /*  9 */ "Factor",
    /* 10 */ "GrammarDefinition",
    /* 11 */ "GrammarDefinitionList",
    /* 12 */ "Group",
    /* 13 */ "Identifier",
    /* 14 */ "IdentifierList",
    /* 15 */ "IdentifierListList",
    /* 16 */ "LiteralString",
    /* 17 */ "LookAhead",
    /* 18 */ "LookAheadGroup",
    /* 19 */ "MemberName",
    /* 20 */ "NegativeLookahead",
    /* 21 */ "NonTerminal",
    /* 22 */ "NonTerminalOpt",
    /* 23 */ "Optional",
    /* 24 */ "ParolLs",
    /* 25 */ "PositiveLookahead",
    /* 26 */ "Production",
    /* 27 */ "ProductionLHS",
    /* 28 */ "Prolog",
    /* 29 */ "PrologList",
    /* 30 */ "PrologList0",
    /* 31 */ "Regex",
    /* 32 */ "Repeat",
    /* 33 */ "ScannerDirectives",
    /* 34 */ "ScannerState",
    /* 35 */ "ScannerStateList",
    /* 36 */ "ScannerSwitch",
    /* 37 */ "ScannerSwitchOpt",
    /* 38 */ "SimpleToken",
    /* 39 */ "SimpleTokenOpt",
    /* 40 */ "StartDeclaration",
    /* 41 */ "String",
    /* 42 */ "Symbol",
    /* 43 */ "TokenExpression",
    /* 44 */ "TokenExpressionOpt",
    /* 45 */ "TokenLiteral",
    /* 46 */ "TokenWithStates",
    /* 47 */ "TokenWithStatesOpt",
    /* 48 */ "UserTypeDeclaration",
    /* 49 */ "UserTypeName",
    /* 50 */ "UserTypeNameList",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 51] = &[
    /* 0 - "ASTControl" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 3, 72),
            Trans(0, 39, 2, 71),
            Trans(0, 40, 1, 70),
        ],
        k: 1,
    },
    /* 1 - "ASTControlOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 1, 73),
            Trans(0, 21, 2, 74),
            Trans(0, 22, 2, 74),
            Trans(0, 23, 2, 74),
            Trans(0, 25, 2, 74),
            Trans(0, 26, 2, 74),
            Trans(0, 27, 2, 74),
            Trans(0, 28, 2, 74),
            Trans(0, 29, 2, 74),
            Trans(0, 30, 2, 74),
            Trans(0, 31, 2, 74),
            Trans(0, 32, 2, 74),
            Trans(0, 33, 2, 74),
            Trans(0, 36, 2, 74),
            Trans(0, 37, 2, 74),
            Trans(0, 38, 2, 74),
            Trans(0, 41, 2, 74),
        ],
        k: 1,
    },
    /* 2 - "Alternation" */
    LookaheadDFA {
        prod0: 27,
        transitions: &[],
        k: 0,
    },
    /* 3 - "AlternationList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 21, 2, 29),
            Trans(0, 22, 2, 29),
            Trans(0, 23, 1, 28),
            Trans(0, 25, 1, 28),
            Trans(0, 26, 2, 29),
            Trans(0, 27, 1, 28),
            Trans(0, 28, 2, 29),
            Trans(0, 29, 1, 28),
            Trans(0, 30, 2, 29),
            Trans(0, 31, 1, 28),
            Trans(0, 32, 1, 28),
            Trans(0, 33, 1, 28),
            Trans(0, 36, 1, 28),
            Trans(0, 37, 1, 28),
            Trans(0, 38, 1, 28),
            Trans(0, 41, 1, 28),
        ],
        k: 1,
    },
    /* 4 - "Alternations" */
    LookaheadDFA {
        prod0: 24,
        transitions: &[],
        k: 0,
    },
    /* 5 - "AlternationsList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 21, 2, 26),
            Trans(0, 22, 1, 25),
            Trans(0, 26, 2, 26),
            Trans(0, 28, 2, 26),
            Trans(0, 30, 2, 26),
        ],
        k: 1,
    },
    /* 6 - "CutOperator" */
    LookaheadDFA {
        prod0: 76,
        transitions: &[],
        k: 0,
    },
    /* 7 - "Declaration" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 6, 1, 7),
            Trans(0, 7, 2, 8),
            Trans(0, 8, 3, 9),
            Trans(0, 10, 4, 10),
            Trans(0, 11, 5, 11),
            Trans(0, 12, 6, 12),
            Trans(0, 13, 6, 12),
            Trans(0, 14, 6, 12),
            Trans(0, 15, 6, 12),
            Trans(0, 16, 6, 12),
        ],
        k: 1,
    },
    /* 8 - "DoubleColon" */
    LookaheadDFA {
        prod0: 21,
        transitions: &[],
        k: 0,
    },
    /* 9 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 23, 4, 33),
            Trans(0, 25, 1, 30),
            Trans(0, 27, 3, 32),
            Trans(0, 29, 2, 31),
            Trans(0, 31, 4, 33),
            Trans(0, 32, 4, 33),
            Trans(0, 33, 4, 33),
            Trans(0, 36, 4, 33),
            Trans(0, 37, 4, 33),
            Trans(0, 38, 4, 33),
            Trans(0, 41, 4, 33),
        ],
        k: 1,
    },
    /* 10 - "GrammarDefinition" */
    LookaheadDFA {
        prod0: 18,
        transitions: &[],
        k: 0,
    },
    /* 11 - "GrammarDefinitionList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 20), Trans(0, 31, 1, 19)],
        k: 1,
    },
    /* 12 - "Group" */
    LookaheadDFA {
        prod0: 50,
        transitions: &[],
        k: 0,
    },
    /* 13 - "Identifier" */
    LookaheadDFA {
        prod0: 56,
        transitions: &[],
        k: 0,
    },
    /* 14 - "IdentifierList" */
    LookaheadDFA {
        prod0: 62,
        transitions: &[],
        k: 0,
    },
    /* 15 - "IdentifierListList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 17, 2, 64),
            Trans(0, 24, 2, 64),
            Trans(0, 35, 1, 63),
        ],
        k: 1,
    },
    /* 16 - "LiteralString" */
    LookaheadDFA {
        prod0: 58,
        transitions: &[],
        k: 0,
    },
    /* 17 - "LookAhead" */
    LookaheadDFA {
        prod0: 82,
        transitions: &[],
        k: 0,
    },
    /* 18 - "LookAheadGroup" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 42, 1, 83), Trans(0, 43, 2, 84)],
        k: 1,
    },
    /* 19 - "MemberName" */
    LookaheadDFA {
        prod0: 75,
        transitions: &[],
        k: 0,
    },
    /* 20 - "NegativeLookahead" */
    LookaheadDFA {
        prod0: 86,
        transitions: &[],
        k: 0,
    },
    /* 21 - "NonTerminal" */
    LookaheadDFA {
        prod0: 53,
        transitions: &[],
        k: 0,
    },
    /* 22 - "NonTerminalOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 1, 54),
            Trans(0, 21, 2, 55),
            Trans(0, 22, 2, 55),
            Trans(0, 23, 2, 55),
            Trans(0, 25, 2, 55),
            Trans(0, 26, 2, 55),
            Trans(0, 27, 2, 55),
            Trans(0, 28, 2, 55),
            Trans(0, 29, 2, 55),
            Trans(0, 30, 2, 55),
            Trans(0, 31, 2, 55),
            Trans(0, 32, 2, 55),
            Trans(0, 33, 2, 55),
            Trans(0, 36, 2, 55),
            Trans(0, 37, 2, 55),
            Trans(0, 38, 2, 55),
            Trans(0, 39, 1, 54),
            Trans(0, 40, 1, 54),
            Trans(0, 41, 2, 55),
        ],
        k: 1,
    },
    /* 23 - "Optional" */
    LookaheadDFA {
        prod0: 51,
        transitions: &[],
        k: 0,
    },
    /* 24 - "ParolLs" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 25 - "PositiveLookahead" */
    LookaheadDFA {
        prod0: 85,
        transitions: &[],
        k: 0,
    },
    /* 26 - "Production" */
    LookaheadDFA {
        prod0: 23,
        transitions: &[],
        k: 0,
    },
    /* 27 - "ProductionLHS" */
    LookaheadDFA {
        prod0: 22,
        transitions: &[],
        k: 0,
    },
    /* 28 - "Prolog" */
    LookaheadDFA {
        prod0: 1,
        transitions: &[],
        k: 0,
    },
    /* 29 - "PrologList" */
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
            Trans(0, 16, 1, 4),
            Trans(0, 18, 2, 5),
            Trans(0, 34, 2, 5),
        ],
        k: 1,
    },
    /* 30 - "PrologList0" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 18, 2, 3), Trans(0, 34, 1, 2)],
        k: 1,
    },
    /* 31 - "Regex" */
    LookaheadDFA {
        prod0: 81,
        transitions: &[],
        k: 0,
    },
    /* 32 - "Repeat" */
    LookaheadDFA {
        prod0: 52,
        transitions: &[],
        k: 0,
    },
    /* 33 - "ScannerDirectives" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 12, 1, 13),
            Trans(0, 13, 2, 14),
            Trans(0, 14, 3, 15),
            Trans(0, 15, 4, 16),
            Trans(0, 16, 5, 17),
        ],
        k: 1,
    },
    /* 34 - "ScannerState" */
    LookaheadDFA {
        prod0: 59,
        transitions: &[],
        k: 0,
    },
    /* 35 - "ScannerStateList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 12, 1, 60),
            Trans(0, 13, 1, 60),
            Trans(0, 14, 1, 60),
            Trans(0, 15, 1, 60),
            Trans(0, 16, 1, 60),
            Trans(0, 30, 2, 61),
        ],
        k: 1,
    },
    /* 36 - "ScannerSwitch" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 36, 1, 65),
            Trans(0, 37, 2, 66),
            Trans(0, 38, 3, 67),
        ],
        k: 1,
    },
    /* 37 - "ScannerSwitchOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 26, 2, 69), Trans(0, 31, 1, 68)],
        k: 1,
    },
    /* 38 - "SimpleToken" */
    LookaheadDFA {
        prod0: 44,
        transitions: &[],
        k: 0,
    },
    /* 39 - "SimpleTokenOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 1, 45),
            Trans(0, 21, 2, 46),
            Trans(0, 22, 2, 46),
            Trans(0, 23, 2, 46),
            Trans(0, 25, 2, 46),
            Trans(0, 26, 2, 46),
            Trans(0, 27, 2, 46),
            Trans(0, 28, 2, 46),
            Trans(0, 29, 2, 46),
            Trans(0, 30, 2, 46),
            Trans(0, 31, 2, 46),
            Trans(0, 32, 2, 46),
            Trans(0, 33, 2, 46),
            Trans(0, 36, 2, 46),
            Trans(0, 37, 2, 46),
            Trans(0, 38, 2, 46),
            Trans(0, 39, 1, 45),
            Trans(0, 40, 1, 45),
            Trans(0, 41, 2, 46),
        ],
        k: 1,
    },
    /* 40 - "StartDeclaration" */
    LookaheadDFA {
        prod0: 6,
        transitions: &[],
        k: 0,
    },
    /* 41 - "String" */
    LookaheadDFA {
        prod0: 57,
        transitions: &[],
        k: 0,
    },
    /* 42 - "Symbol" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 23, 3, 36),
            Trans(0, 31, 1, 34),
            Trans(0, 32, 2, 35),
            Trans(0, 33, 2, 35),
            Trans(0, 36, 4, 37),
            Trans(0, 37, 4, 37),
            Trans(0, 38, 4, 37),
            Trans(0, 41, 2, 35),
        ],
        k: 1,
    },
    /* 43 - "TokenExpression" */
    LookaheadDFA {
        prod0: 41,
        transitions: &[],
        k: 0,
    },
    /* 44 - "TokenExpressionOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 2, 43),
            Trans(0, 21, 2, 43),
            Trans(0, 22, 2, 43),
            Trans(0, 23, 2, 43),
            Trans(0, 25, 2, 43),
            Trans(0, 26, 2, 43),
            Trans(0, 27, 2, 43),
            Trans(0, 28, 2, 43),
            Trans(0, 29, 2, 43),
            Trans(0, 30, 2, 43),
            Trans(0, 31, 2, 43),
            Trans(0, 32, 2, 43),
            Trans(0, 33, 2, 43),
            Trans(0, 36, 2, 43),
            Trans(0, 37, 2, 43),
            Trans(0, 38, 2, 43),
            Trans(0, 39, 2, 43),
            Trans(0, 40, 2, 43),
            Trans(0, 41, 2, 43),
            Trans(0, 42, 1, 42),
            Trans(0, 43, 1, 42),
        ],
        k: 1,
    },
    /* 45 - "TokenLiteral" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 32, 1, 38),
            Trans(0, 33, 2, 39),
            Trans(0, 41, 3, 40),
        ],
        k: 1,
    },
    /* 46 - "TokenWithStates" */
    LookaheadDFA {
        prod0: 47,
        transitions: &[],
        k: 0,
    },
    /* 47 - "TokenWithStatesOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 1, 48),
            Trans(0, 21, 2, 49),
            Trans(0, 22, 2, 49),
            Trans(0, 23, 2, 49),
            Trans(0, 25, 2, 49),
            Trans(0, 26, 2, 49),
            Trans(0, 27, 2, 49),
            Trans(0, 28, 2, 49),
            Trans(0, 29, 2, 49),
            Trans(0, 30, 2, 49),
            Trans(0, 31, 2, 49),
            Trans(0, 32, 2, 49),
            Trans(0, 33, 2, 49),
            Trans(0, 36, 2, 49),
            Trans(0, 37, 2, 49),
            Trans(0, 38, 2, 49),
            Trans(0, 39, 1, 48),
            Trans(0, 40, 1, 48),
            Trans(0, 41, 2, 49),
        ],
        k: 1,
    },
    /* 48 - "UserTypeDeclaration" */
    LookaheadDFA {
        prod0: 77,
        transitions: &[],
        k: 0,
    },
    /* 49 - "UserTypeName" */
    LookaheadDFA {
        prod0: 78,
        transitions: &[],
        k: 0,
    },
    /* 50 - "UserTypeNameList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 6, 2, 80),
            Trans(0, 7, 2, 80),
            Trans(0, 8, 2, 80),
            Trans(0, 10, 2, 80),
            Trans(0, 11, 2, 80),
            Trans(0, 12, 2, 80),
            Trans(0, 13, 2, 80),
            Trans(0, 14, 2, 80),
            Trans(0, 15, 2, 80),
            Trans(0, 16, 2, 80),
            Trans(0, 18, 2, 80),
            Trans(0, 19, 1, 79),
            Trans(0, 21, 2, 80),
            Trans(0, 22, 2, 80),
            Trans(0, 23, 2, 80),
            Trans(0, 25, 2, 80),
            Trans(0, 26, 2, 80),
            Trans(0, 27, 2, 80),
            Trans(0, 28, 2, 80),
            Trans(0, 29, 2, 80),
            Trans(0, 30, 2, 80),
            Trans(0, 31, 2, 80),
            Trans(0, 32, 2, 80),
            Trans(0, 33, 2, 80),
            Trans(0, 34, 2, 80),
            Trans(0, 36, 2, 80),
            Trans(0, 37, 2, 80),
            Trans(0, 38, 2, 80),
            Trans(0, 41, 2, 80),
        ],
        k: 1,
    },
];

pub const PRODUCTIONS: &[Production; 87] = &[
    // 0 - ParolLs: Prolog GrammarDefinition;
    Production {
        lhs: 24,
        production: &[ParseType::N(10), ParseType::N(28)],
    },
    // 1 - Prolog: StartDeclaration PrologList /* Vec */ PrologList0 /* Vec */;
    Production {
        lhs: 28,
        production: &[ParseType::N(30), ParseType::N(29), ParseType::N(40)],
    },
    // 2 - PrologList0: ScannerState PrologList0;
    Production {
        lhs: 30,
        production: &[ParseType::N(30), ParseType::N(34)],
    },
    // 3 - PrologList0: ;
    Production {
        lhs: 30,
        production: &[],
    },
    // 4 - PrologList: Declaration PrologList;
    Production {
        lhs: 29,
        production: &[ParseType::N(29), ParseType::N(7)],
    },
    // 5 - PrologList: ;
    Production {
        lhs: 29,
        production: &[],
    },
    // 6 - StartDeclaration: "%start" Identifier;
    Production {
        lhs: 40,
        production: &[ParseType::N(13), ParseType::T(5)],
    },
    // 7 - Declaration: "%title" String;
    Production {
        lhs: 7,
        production: &[ParseType::N(41), ParseType::T(6)],
    },
    // 8 - Declaration: "%comment" String;
    Production {
        lhs: 7,
        production: &[ParseType::N(41), ParseType::T(7)],
    },
    // 9 - Declaration: "%user_type" Identifier "=" UserTypeName;
    Production {
        lhs: 7,
        production: &[
            ParseType::N(49),
            ParseType::T(9),
            ParseType::N(13),
            ParseType::T(8),
        ],
    },
    // 10 - Declaration: "%production_type" Identifier@prod_name "=" UserTypeName@prod_type;
    Production {
        lhs: 7,
        production: &[
            ParseType::N(49),
            ParseType::T(9),
            ParseType::N(13),
            ParseType::T(10),
        ],
    },
    // 11 - Declaration: '%grammar_type' LiteralString;
    Production {
        lhs: 7,
        production: &[ParseType::N(16), ParseType::T(11)],
    },
    // 12 - Declaration: ScannerDirectives;
    Production {
        lhs: 7,
        production: &[ParseType::N(33)],
    },
    // 13 - ScannerDirectives: "%line_comment" TokenLiteral;
    Production {
        lhs: 33,
        production: &[ParseType::N(45), ParseType::T(12)],
    },
    // 14 - ScannerDirectives: "%block_comment" TokenLiteral TokenLiteral;
    Production {
        lhs: 33,
        production: &[ParseType::N(45), ParseType::N(45), ParseType::T(13)],
    },
    // 15 - ScannerDirectives: "%auto_newline_off";
    Production {
        lhs: 33,
        production: &[ParseType::T(14)],
    },
    // 16 - ScannerDirectives: "%auto_ws_off";
    Production {
        lhs: 33,
        production: &[ParseType::T(15)],
    },
    // 17 - ScannerDirectives: '%on' IdentifierList '%enter' Identifier;
    Production {
        lhs: 33,
        production: &[
            ParseType::N(13),
            ParseType::T(17),
            ParseType::N(14),
            ParseType::T(16),
        ],
    },
    // 18 - GrammarDefinition: "%%" Production GrammarDefinitionList /* Vec */;
    Production {
        lhs: 10,
        production: &[ParseType::N(11), ParseType::N(26), ParseType::T(18)],
    },
    // 19 - GrammarDefinitionList: Production GrammarDefinitionList;
    Production {
        lhs: 11,
        production: &[ParseType::N(11), ParseType::N(26)],
    },
    // 20 - GrammarDefinitionList: ;
    Production {
        lhs: 11,
        production: &[],
    },
    // 21 - DoubleColon: "::";
    Production {
        lhs: 8,
        production: &[ParseType::T(19)],
    },
    // 22 - ProductionLHS: Identifier ":";
    Production {
        lhs: 27,
        production: &[ParseType::T(20), ParseType::N(13)],
    },
    // 23 - Production: ProductionLHS Alternations ";";
    Production {
        lhs: 26,
        production: &[ParseType::T(21), ParseType::N(4), ParseType::N(27)],
    },
    // 24 - Alternations: Alternation AlternationsList /* Vec */;
    Production {
        lhs: 4,
        production: &[ParseType::N(5), ParseType::N(2)],
    },
    // 25 - AlternationsList: '|' Alternation AlternationsList;
    Production {
        lhs: 5,
        production: &[ParseType::N(5), ParseType::N(2), ParseType::T(22)],
    },
    // 26 - AlternationsList: ;
    Production {
        lhs: 5,
        production: &[],
    },
    // 27 - Alternation: AlternationList /* Vec */;
    Production {
        lhs: 2,
        production: &[ParseType::N(3)],
    },
    // 28 - AlternationList: Factor AlternationList;
    Production {
        lhs: 3,
        production: &[ParseType::N(3), ParseType::N(9)],
    },
    // 29 - AlternationList: ;
    Production {
        lhs: 3,
        production: &[],
    },
    // 30 - Factor: Group;
    Production {
        lhs: 9,
        production: &[ParseType::N(12)],
    },
    // 31 - Factor: Repeat;
    Production {
        lhs: 9,
        production: &[ParseType::N(32)],
    },
    // 32 - Factor: Optional;
    Production {
        lhs: 9,
        production: &[ParseType::N(23)],
    },
    // 33 - Factor: Symbol;
    Production {
        lhs: 9,
        production: &[ParseType::N(42)],
    },
    // 34 - Symbol: NonTerminal;
    Production {
        lhs: 42,
        production: &[ParseType::N(21)],
    },
    // 35 - Symbol: SimpleToken;
    Production {
        lhs: 42,
        production: &[ParseType::N(38)],
    },
    // 36 - Symbol: TokenWithStates;
    Production {
        lhs: 42,
        production: &[ParseType::N(46)],
    },
    // 37 - Symbol: ScannerSwitch;
    Production {
        lhs: 42,
        production: &[ParseType::N(36)],
    },
    // 38 - TokenLiteral: String;
    Production {
        lhs: 45,
        production: &[ParseType::N(41)],
    },
    // 39 - TokenLiteral: LiteralString;
    Production {
        lhs: 45,
        production: &[ParseType::N(16)],
    },
    // 40 - TokenLiteral: Regex;
    Production {
        lhs: 45,
        production: &[ParseType::N(31)],
    },
    // 41 - TokenExpression: TokenLiteral TokenExpressionOpt /* Option */;
    Production {
        lhs: 43,
        production: &[ParseType::N(44), ParseType::N(45)],
    },
    // 42 - TokenExpressionOpt: LookAhead;
    Production {
        lhs: 44,
        production: &[ParseType::N(17)],
    },
    // 43 - TokenExpressionOpt: ;
    Production {
        lhs: 44,
        production: &[],
    },
    // 44 - SimpleToken: TokenExpression SimpleTokenOpt /* Option */;
    Production {
        lhs: 38,
        production: &[ParseType::N(39), ParseType::N(43)],
    },
    // 45 - SimpleTokenOpt: ASTControl;
    Production {
        lhs: 39,
        production: &[ParseType::N(0)],
    },
    // 46 - SimpleTokenOpt: ;
    Production {
        lhs: 39,
        production: &[],
    },
    // 47 - TokenWithStates: "<" IdentifierList ">" TokenExpression TokenWithStatesOpt /* Option */;
    Production {
        lhs: 46,
        production: &[
            ParseType::N(47),
            ParseType::N(43),
            ParseType::T(24),
            ParseType::N(14),
            ParseType::T(23),
        ],
    },
    // 48 - TokenWithStatesOpt: ASTControl;
    Production {
        lhs: 47,
        production: &[ParseType::N(0)],
    },
    // 49 - TokenWithStatesOpt: ;
    Production {
        lhs: 47,
        production: &[],
    },
    // 50 - Group: '(' Alternations ')';
    Production {
        lhs: 12,
        production: &[ParseType::T(26), ParseType::N(4), ParseType::T(25)],
    },
    // 51 - Optional: '[' Alternations ']';
    Production {
        lhs: 23,
        production: &[ParseType::T(28), ParseType::N(4), ParseType::T(27)],
    },
    // 52 - Repeat: '{' Alternations '}';
    Production {
        lhs: 32,
        production: &[ParseType::T(30), ParseType::N(4), ParseType::T(29)],
    },
    // 53 - NonTerminal: Identifier NonTerminalOpt /* Option */;
    Production {
        lhs: 21,
        production: &[ParseType::N(22), ParseType::N(13)],
    },
    // 54 - NonTerminalOpt: ASTControl;
    Production {
        lhs: 22,
        production: &[ParseType::N(0)],
    },
    // 55 - NonTerminalOpt: ;
    Production {
        lhs: 22,
        production: &[],
    },
    // 56 - Identifier: /[a-zA-Z_][a-zA-Z0-9_]*/;
    Production {
        lhs: 13,
        production: &[ParseType::T(31)],
    },
    // 57 - String: /"(\\.|[^"])*"/;
    Production {
        lhs: 41,
        production: &[ParseType::T(32)],
    },
    // 58 - LiteralString: /'(\\.|[^'])*'/;
    Production {
        lhs: 16,
        production: &[ParseType::T(33)],
    },
    // 59 - ScannerState: "%scanner" Identifier '{' ScannerStateList /* Vec */ '}';
    Production {
        lhs: 34,
        production: &[
            ParseType::T(30),
            ParseType::N(35),
            ParseType::T(29),
            ParseType::N(13),
            ParseType::T(34),
        ],
    },
    // 60 - ScannerStateList: ScannerDirectives ScannerStateList;
    Production {
        lhs: 35,
        production: &[ParseType::N(35), ParseType::N(33)],
    },
    // 61 - ScannerStateList: ;
    Production {
        lhs: 35,
        production: &[],
    },
    // 62 - IdentifierList: Identifier IdentifierListList /* Vec */;
    Production {
        lhs: 14,
        production: &[ParseType::N(15), ParseType::N(13)],
    },
    // 63 - IdentifierListList: "," Identifier IdentifierListList;
    Production {
        lhs: 15,
        production: &[ParseType::N(15), ParseType::N(13), ParseType::T(35)],
    },
    // 64 - IdentifierListList: ;
    Production {
        lhs: 15,
        production: &[],
    },
    // 65 - ScannerSwitch: "%sc" '(' ScannerSwitchOpt /* Option */ ')';
    Production {
        lhs: 36,
        production: &[
            ParseType::T(26),
            ParseType::N(37),
            ParseType::T(25),
            ParseType::T(36),
        ],
    },
    // 66 - ScannerSwitch: "%push" '(' Identifier ')';
    Production {
        lhs: 36,
        production: &[
            ParseType::T(26),
            ParseType::N(13),
            ParseType::T(25),
            ParseType::T(37),
        ],
    },
    // 67 - ScannerSwitch: "%pop" '(' ')';
    Production {
        lhs: 36,
        production: &[ParseType::T(26), ParseType::T(25), ParseType::T(38)],
    },
    // 68 - ScannerSwitchOpt: Identifier;
    Production {
        lhs: 37,
        production: &[ParseType::N(13)],
    },
    // 69 - ScannerSwitchOpt: ;
    Production {
        lhs: 37,
        production: &[],
    },
    // 70 - ASTControl: CutOperator;
    Production {
        lhs: 0,
        production: &[ParseType::N(6)],
    },
    // 71 - ASTControl: MemberName ASTControlOpt /* Option */;
    Production {
        lhs: 0,
        production: &[ParseType::N(1), ParseType::N(19)],
    },
    // 72 - ASTControl: UserTypeDeclaration;
    Production {
        lhs: 0,
        production: &[ParseType::N(48)],
    },
    // 73 - ASTControlOpt: UserTypeDeclaration;
    Production {
        lhs: 1,
        production: &[ParseType::N(48)],
    },
    // 74 - ASTControlOpt: ;
    Production {
        lhs: 1,
        production: &[],
    },
    // 75 - MemberName: '@'^ /* Clipped */ Identifier;
    Production {
        lhs: 19,
        production: &[ParseType::N(13), ParseType::T(39)],
    },
    // 76 - CutOperator: '^';
    Production {
        lhs: 6,
        production: &[ParseType::T(40)],
    },
    // 77 - UserTypeDeclaration: ":" UserTypeName;
    Production {
        lhs: 48,
        production: &[ParseType::N(49), ParseType::T(20)],
    },
    // 78 - UserTypeName: Identifier UserTypeNameList /* Vec */;
    Production {
        lhs: 49,
        production: &[ParseType::N(50), ParseType::N(13)],
    },
    // 79 - UserTypeNameList: DoubleColon Identifier UserTypeNameList;
    Production {
        lhs: 50,
        production: &[ParseType::N(50), ParseType::N(13), ParseType::N(8)],
    },
    // 80 - UserTypeNameList: ;
    Production {
        lhs: 50,
        production: &[],
    },
    // 81 - Regex: "/(\\.|[^\/])*/";
    Production {
        lhs: 31,
        production: &[ParseType::T(41)],
    },
    // 82 - LookAhead: LookAheadGroup TokenLiteral;
    Production {
        lhs: 17,
        production: &[ParseType::N(45), ParseType::N(18)],
    },
    // 83 - LookAheadGroup: PositiveLookahead;
    Production {
        lhs: 18,
        production: &[ParseType::N(25)],
    },
    // 84 - LookAheadGroup: NegativeLookahead;
    Production {
        lhs: 18,
        production: &[ParseType::N(20)],
    },
    // 85 - PositiveLookahead: '?=';
    Production {
        lhs: 25,
        production: &[ParseType::T(42)],
    },
    // 86 - NegativeLookahead: '?!';
    Production {
        lhs: 20,
        production: &[ParseType::T(43)],
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
    user_actions: &mut ParolLsGrammar,
) -> Result<ParseTree, ParolError>
where
    T: AsRef<Path>,
{
    let mut llk_parser = LLKParser::new(
        24,
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
