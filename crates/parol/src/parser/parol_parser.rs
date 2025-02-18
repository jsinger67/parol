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

use crate::parser::parol_grammar::ParolGrammar;
use crate::parser::parol_grammar_trait::ParolGrammarAuto;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};

pub const TERMINALS: &[(&str, Option<(bool, &str)>); 44] = &[
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
    /* 10 */ (r"%grammar_type", None),
    /* 11 */ (r"%line_comment", None),
    /* 12 */ (r"%block_comment", None),
    /* 13 */ (r"%auto_newline_off", None),
    /* 14 */ (r"%auto_ws_off", None),
    /* 15 */ (r"%on", None),
    /* 16 */ (r"%enter", None),
    /* 17 */ (r"%%", None),
    /* 18 */ (r"::", None),
    /* 19 */ (r":", None),
    /* 20 */ (r";", None),
    /* 21 */ (r"\|", None),
    /* 22 */ (r"<", None),
    /* 23 */ (r">", None),
    /* 24 */ (r#""(\\.|[^"])*""#, None),
    /* 25 */ (r"'(\\.|[^'])*'", None),
    /* 26 */ (r"/(\\.|[^\/])*/", None),
    /* 27 */ (r"\(", None),
    /* 28 */ (r"\)", None),
    /* 29 */ (r"\[", None),
    /* 30 */ (r"\]", None),
    /* 31 */ (r"\{", None),
    /* 32 */ (r"\}", None),
    /* 33 */ (r"[a-zA-Z_][a-zA-Z0-9_]*", None),
    /* 34 */ (r"%scanner", None),
    /* 35 */ (r",", None),
    /* 36 */ (r"%sc", None),
    /* 37 */ (r"%push", None),
    /* 38 */ (r"%pop", None),
    /* 39 */ (r"@", None),
    /* 40 */ (r"\^", None),
    /* 41 */ (r"\?=", None),
    /* 42 */ (r"\?!", None),
    /* 43 */ (ERROR_TOKEN, None),
];

pub const TERMINAL_NAMES: &[&str; 44] = &[
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
    /* 24 */ "String",
    /* 25 */ "RawString",
    /* 26 */ "Regex",
    /* 27 */ "LParen",
    /* 28 */ "RParen",
    /* 29 */ "LBracket",
    /* 30 */ "RBracket",
    /* 31 */ "LBrace",
    /* 32 */ "RBrace",
    /* 33 */ "Identifier",
    /* 34 */ "PercentScanner",
    /* 35 */ "Comma",
    /* 36 */ "PercentSc",
    /* 37 */ "PercentPush",
    /* 38 */ "PercentPop",
    /* 39 */ "At",
    /* 40 */ "CutOperator",
    /* 41 */ "PositiveLookahead",
    /* 42 */ "NegativeLookahead",
    /* 43 */ "Error",
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 38]) = (
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
        24, /* String */
        25, /* RawString */
        26, /* Regex */
        27, /* LParen */
        28, /* RParen */
        29, /* LBracket */
        30, /* RBracket */
        31, /* LBrace */
        32, /* RBrace */
        33, /* Identifier */
        34, /* PercentScanner */
        35, /* Comma */
        36, /* PercentSc */
        37, /* PercentPush */
        38, /* PercentPop */
        39, /* At */
        40, /* CutOperator */
        41, /* PositiveLookahead */
        42, /* NegativeLookahead */
    ],
);

const MAX_K: usize = 1;

pub const NON_TERMINALS: &[&str; 50] = &[
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
    /* 16 */ "LookAhead",
    /* 17 */ "LookAheadGroup",
    /* 18 */ "MemberName",
    /* 19 */ "NegativeLookahead",
    /* 20 */ "NonTerminal",
    /* 21 */ "NonTerminalOpt",
    /* 22 */ "Optional",
    /* 23 */ "Parol",
    /* 24 */ "PositiveLookahead",
    /* 25 */ "Production",
    /* 26 */ "Prolog",
    /* 27 */ "PrologList",
    /* 28 */ "PrologList0",
    /* 29 */ "RawString",
    /* 30 */ "Regex",
    /* 31 */ "Repeat",
    /* 32 */ "ScannerDirectives",
    /* 33 */ "ScannerState",
    /* 34 */ "ScannerStateList",
    /* 35 */ "ScannerSwitch",
    /* 36 */ "ScannerSwitchOpt",
    /* 37 */ "SimpleToken",
    /* 38 */ "SimpleTokenOpt",
    /* 39 */ "StartDeclaration",
    /* 40 */ "String",
    /* 41 */ "Symbol",
    /* 42 */ "TokenExpression",
    /* 43 */ "TokenExpressionOpt",
    /* 44 */ "TokenLiteral",
    /* 45 */ "TokenWithStates",
    /* 46 */ "TokenWithStatesOpt",
    /* 47 */ "UserTypeDeclaration",
    /* 48 */ "UserTypeName",
    /* 49 */ "UserTypeNameList",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 50] = &[
    /* 0 - "ASTControl" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 3, 71),
            Trans(0, 39, 2, 70),
            Trans(0, 40, 1, 69),
        ],
        k: 1,
    },
    /* 1 - "ASTControlOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 72),
            Trans(0, 20, 2, 73),
            Trans(0, 21, 2, 73),
            Trans(0, 22, 2, 73),
            Trans(0, 24, 2, 73),
            Trans(0, 25, 2, 73),
            Trans(0, 26, 2, 73),
            Trans(0, 27, 2, 73),
            Trans(0, 28, 2, 73),
            Trans(0, 29, 2, 73),
            Trans(0, 30, 2, 73),
            Trans(0, 31, 2, 73),
            Trans(0, 32, 2, 73),
            Trans(0, 33, 2, 73),
            Trans(0, 36, 2, 73),
            Trans(0, 37, 2, 73),
            Trans(0, 38, 2, 73),
        ],
        k: 1,
    },
    /* 2 - "Alternation" */
    LookaheadDFA {
        prod0: 25,
        transitions: &[],
        k: 0,
    },
    /* 3 - "AlternationList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 2, 27),
            Trans(0, 21, 2, 27),
            Trans(0, 22, 1, 26),
            Trans(0, 24, 1, 26),
            Trans(0, 25, 1, 26),
            Trans(0, 26, 1, 26),
            Trans(0, 27, 1, 26),
            Trans(0, 28, 2, 27),
            Trans(0, 29, 1, 26),
            Trans(0, 30, 2, 27),
            Trans(0, 31, 1, 26),
            Trans(0, 32, 2, 27),
            Trans(0, 33, 1, 26),
            Trans(0, 36, 1, 26),
            Trans(0, 37, 1, 26),
            Trans(0, 38, 1, 26),
        ],
        k: 1,
    },
    /* 4 - "Alternations" */
    LookaheadDFA {
        prod0: 22,
        transitions: &[],
        k: 0,
    },
    /* 5 - "AlternationsList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 20, 2, 24),
            Trans(0, 21, 1, 23),
            Trans(0, 28, 2, 24),
            Trans(0, 30, 2, 24),
            Trans(0, 32, 2, 24),
        ],
        k: 1,
    },
    /* 6 - "CutOperator" */
    LookaheadDFA {
        prod0: 75,
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
            Trans(0, 12, 5, 11),
            Trans(0, 13, 5, 11),
            Trans(0, 14, 5, 11),
            Trans(0, 15, 5, 11),
        ],
        k: 1,
    },
    /* 8 - "DoubleColon" */
    LookaheadDFA {
        prod0: 20,
        transitions: &[],
        k: 0,
    },
    /* 9 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 22, 4, 31),
            Trans(0, 24, 4, 31),
            Trans(0, 25, 4, 31),
            Trans(0, 26, 4, 31),
            Trans(0, 27, 1, 28),
            Trans(0, 29, 3, 30),
            Trans(0, 31, 2, 29),
            Trans(0, 33, 4, 31),
            Trans(0, 36, 4, 31),
            Trans(0, 37, 4, 31),
            Trans(0, 38, 4, 31),
        ],
        k: 1,
    },
    /* 10 - "GrammarDefinition" */
    LookaheadDFA {
        prod0: 17,
        transitions: &[],
        k: 0,
    },
    /* 11 - "GrammarDefinitionList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 19), Trans(0, 33, 1, 18)],
        k: 1,
    },
    /* 12 - "Group" */
    LookaheadDFA {
        prod0: 51,
        transitions: &[],
        k: 0,
    },
    /* 13 - "Identifier" */
    LookaheadDFA {
        prod0: 57,
        transitions: &[],
        k: 0,
    },
    /* 14 - "IdentifierList" */
    LookaheadDFA {
        prod0: 61,
        transitions: &[],
        k: 0,
    },
    /* 15 - "IdentifierListList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 63),
            Trans(0, 23, 2, 63),
            Trans(0, 35, 1, 62),
        ],
        k: 1,
    },
    /* 16 - "LookAhead" */
    LookaheadDFA {
        prod0: 80,
        transitions: &[],
        k: 0,
    },
    /* 17 - "LookAheadGroup" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 41, 1, 81), Trans(0, 42, 2, 82)],
        k: 1,
    },
    /* 18 - "MemberName" */
    LookaheadDFA {
        prod0: 74,
        transitions: &[],
        k: 0,
    },
    /* 19 - "NegativeLookahead" */
    LookaheadDFA {
        prod0: 84,
        transitions: &[],
        k: 0,
    },
    /* 20 - "NonTerminal" */
    LookaheadDFA {
        prod0: 54,
        transitions: &[],
        k: 0,
    },
    /* 21 - "NonTerminalOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 55),
            Trans(0, 20, 2, 56),
            Trans(0, 21, 2, 56),
            Trans(0, 22, 2, 56),
            Trans(0, 24, 2, 56),
            Trans(0, 25, 2, 56),
            Trans(0, 26, 2, 56),
            Trans(0, 27, 2, 56),
            Trans(0, 28, 2, 56),
            Trans(0, 29, 2, 56),
            Trans(0, 30, 2, 56),
            Trans(0, 31, 2, 56),
            Trans(0, 32, 2, 56),
            Trans(0, 33, 2, 56),
            Trans(0, 36, 2, 56),
            Trans(0, 37, 2, 56),
            Trans(0, 38, 2, 56),
            Trans(0, 39, 1, 55),
            Trans(0, 40, 1, 55),
        ],
        k: 1,
    },
    /* 22 - "Optional" */
    LookaheadDFA {
        prod0: 52,
        transitions: &[],
        k: 0,
    },
    /* 23 - "Parol" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 24 - "PositiveLookahead" */
    LookaheadDFA {
        prod0: 83,
        transitions: &[],
        k: 0,
    },
    /* 25 - "Production" */
    LookaheadDFA {
        prod0: 21,
        transitions: &[],
        k: 0,
    },
    /* 26 - "Prolog" */
    LookaheadDFA {
        prod0: 1,
        transitions: &[],
        k: 0,
    },
    /* 27 - "PrologList" */
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
            Trans(0, 34, 2, 5),
        ],
        k: 1,
    },
    /* 28 - "PrologList0" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 17, 2, 3), Trans(0, 34, 1, 2)],
        k: 1,
    },
    /* 29 - "RawString" */
    LookaheadDFA {
        prod0: 49,
        transitions: &[],
        k: 0,
    },
    /* 30 - "Regex" */
    LookaheadDFA {
        prod0: 50,
        transitions: &[],
        k: 0,
    },
    /* 31 - "Repeat" */
    LookaheadDFA {
        prod0: 53,
        transitions: &[],
        k: 0,
    },
    /* 32 - "ScannerDirectives" */
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
    /* 33 - "ScannerState" */
    LookaheadDFA {
        prod0: 58,
        transitions: &[],
        k: 0,
    },
    /* 34 - "ScannerStateList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 11, 1, 59),
            Trans(0, 12, 1, 59),
            Trans(0, 13, 1, 59),
            Trans(0, 14, 1, 59),
            Trans(0, 15, 1, 59),
            Trans(0, 32, 2, 60),
        ],
        k: 1,
    },
    /* 35 - "ScannerSwitch" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 36, 1, 64),
            Trans(0, 37, 2, 65),
            Trans(0, 38, 3, 66),
        ],
        k: 1,
    },
    /* 36 - "ScannerSwitchOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 28, 2, 68), Trans(0, 33, 1, 67)],
        k: 1,
    },
    /* 37 - "SimpleToken" */
    LookaheadDFA {
        prod0: 42,
        transitions: &[],
        k: 0,
    },
    /* 38 - "SimpleTokenOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 43),
            Trans(0, 20, 2, 44),
            Trans(0, 21, 2, 44),
            Trans(0, 22, 2, 44),
            Trans(0, 24, 2, 44),
            Trans(0, 25, 2, 44),
            Trans(0, 26, 2, 44),
            Trans(0, 27, 2, 44),
            Trans(0, 28, 2, 44),
            Trans(0, 29, 2, 44),
            Trans(0, 30, 2, 44),
            Trans(0, 31, 2, 44),
            Trans(0, 32, 2, 44),
            Trans(0, 33, 2, 44),
            Trans(0, 36, 2, 44),
            Trans(0, 37, 2, 44),
            Trans(0, 38, 2, 44),
            Trans(0, 39, 1, 43),
            Trans(0, 40, 1, 43),
        ],
        k: 1,
    },
    /* 39 - "StartDeclaration" */
    LookaheadDFA {
        prod0: 6,
        transitions: &[],
        k: 0,
    },
    /* 40 - "String" */
    LookaheadDFA {
        prod0: 48,
        transitions: &[],
        k: 0,
    },
    /* 41 - "Symbol" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 22, 3, 34),
            Trans(0, 24, 2, 33),
            Trans(0, 25, 2, 33),
            Trans(0, 26, 2, 33),
            Trans(0, 33, 1, 32),
            Trans(0, 36, 4, 35),
            Trans(0, 37, 4, 35),
            Trans(0, 38, 4, 35),
        ],
        k: 1,
    },
    /* 42 - "TokenExpression" */
    LookaheadDFA {
        prod0: 39,
        transitions: &[],
        k: 0,
    },
    /* 43 - "TokenExpressionOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 2, 41),
            Trans(0, 20, 2, 41),
            Trans(0, 21, 2, 41),
            Trans(0, 22, 2, 41),
            Trans(0, 24, 2, 41),
            Trans(0, 25, 2, 41),
            Trans(0, 26, 2, 41),
            Trans(0, 27, 2, 41),
            Trans(0, 28, 2, 41),
            Trans(0, 29, 2, 41),
            Trans(0, 30, 2, 41),
            Trans(0, 31, 2, 41),
            Trans(0, 32, 2, 41),
            Trans(0, 33, 2, 41),
            Trans(0, 36, 2, 41),
            Trans(0, 37, 2, 41),
            Trans(0, 38, 2, 41),
            Trans(0, 39, 2, 41),
            Trans(0, 40, 2, 41),
            Trans(0, 41, 1, 40),
            Trans(0, 42, 1, 40),
        ],
        k: 1,
    },
    /* 44 - "TokenLiteral" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 24, 1, 36),
            Trans(0, 25, 2, 37),
            Trans(0, 26, 3, 38),
        ],
        k: 1,
    },
    /* 45 - "TokenWithStates" */
    LookaheadDFA {
        prod0: 45,
        transitions: &[],
        k: 0,
    },
    /* 46 - "TokenWithStatesOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 19, 1, 46),
            Trans(0, 20, 2, 47),
            Trans(0, 21, 2, 47),
            Trans(0, 22, 2, 47),
            Trans(0, 24, 2, 47),
            Trans(0, 25, 2, 47),
            Trans(0, 26, 2, 47),
            Trans(0, 27, 2, 47),
            Trans(0, 28, 2, 47),
            Trans(0, 29, 2, 47),
            Trans(0, 30, 2, 47),
            Trans(0, 31, 2, 47),
            Trans(0, 32, 2, 47),
            Trans(0, 33, 2, 47),
            Trans(0, 36, 2, 47),
            Trans(0, 37, 2, 47),
            Trans(0, 38, 2, 47),
            Trans(0, 39, 1, 46),
            Trans(0, 40, 1, 46),
        ],
        k: 1,
    },
    /* 47 - "UserTypeDeclaration" */
    LookaheadDFA {
        prod0: 76,
        transitions: &[],
        k: 0,
    },
    /* 48 - "UserTypeName" */
    LookaheadDFA {
        prod0: 77,
        transitions: &[],
        k: 0,
    },
    /* 49 - "UserTypeNameList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 6, 2, 79),
            Trans(0, 7, 2, 79),
            Trans(0, 8, 2, 79),
            Trans(0, 10, 2, 79),
            Trans(0, 11, 2, 79),
            Trans(0, 12, 2, 79),
            Trans(0, 13, 2, 79),
            Trans(0, 14, 2, 79),
            Trans(0, 15, 2, 79),
            Trans(0, 17, 2, 79),
            Trans(0, 18, 1, 78),
            Trans(0, 20, 2, 79),
            Trans(0, 21, 2, 79),
            Trans(0, 22, 2, 79),
            Trans(0, 24, 2, 79),
            Trans(0, 25, 2, 79),
            Trans(0, 26, 2, 79),
            Trans(0, 27, 2, 79),
            Trans(0, 28, 2, 79),
            Trans(0, 29, 2, 79),
            Trans(0, 30, 2, 79),
            Trans(0, 31, 2, 79),
            Trans(0, 32, 2, 79),
            Trans(0, 33, 2, 79),
            Trans(0, 34, 2, 79),
            Trans(0, 36, 2, 79),
            Trans(0, 37, 2, 79),
            Trans(0, 38, 2, 79),
        ],
        k: 1,
    },
];

pub const PRODUCTIONS: &[Production; 85] = &[
    // 0 - Parol: Prolog GrammarDefinition;
    Production {
        lhs: 23,
        production: &[ParseType::N(10), ParseType::N(26)],
    },
    // 1 - Prolog: StartDeclaration PrologList /* Vec */ PrologList0 /* Vec */;
    Production {
        lhs: 26,
        production: &[ParseType::N(28), ParseType::N(27), ParseType::N(39)],
    },
    // 2 - PrologList0: ScannerState : crate::parser::parol_grammar::ScannerConfig  PrologList0;
    Production {
        lhs: 28,
        production: &[ParseType::N(28), ParseType::N(33)],
    },
    // 3 - PrologList0: ;
    Production {
        lhs: 28,
        production: &[],
    },
    // 4 - PrologList: Declaration PrologList;
    Production {
        lhs: 27,
        production: &[ParseType::N(27), ParseType::N(7)],
    },
    // 5 - PrologList: ;
    Production {
        lhs: 27,
        production: &[],
    },
    // 6 - StartDeclaration: '%start'^ /* Clipped */ Identifier;
    Production {
        lhs: 39,
        production: &[ParseType::N(13), ParseType::T(5)],
    },
    // 7 - Declaration: '%title'^ /* Clipped */ String;
    Production {
        lhs: 7,
        production: &[ParseType::N(40), ParseType::T(6)],
    },
    // 8 - Declaration: '%comment'^ /* Clipped */ String;
    Production {
        lhs: 7,
        production: &[ParseType::N(40), ParseType::T(7)],
    },
    // 9 - Declaration: '%user_type'^ /* Clipped */ Identifier '='^ /* Clipped */ UserTypeName : crate::parser::parol_grammar::UserDefinedTypeName ;
    Production {
        lhs: 7,
        production: &[
            ParseType::N(48),
            ParseType::T(9),
            ParseType::N(13),
            ParseType::T(8),
        ],
    },
    // 10 - Declaration: '%grammar_type'^ /* Clipped */ RawString;
    Production {
        lhs: 7,
        production: &[ParseType::N(29), ParseType::T(10)],
    },
    // 11 - Declaration: ScannerDirectives;
    Production {
        lhs: 7,
        production: &[ParseType::N(32)],
    },
    // 12 - ScannerDirectives: '%line_comment'^ /* Clipped */ TokenLiteral;
    Production {
        lhs: 32,
        production: &[ParseType::N(44), ParseType::T(11)],
    },
    // 13 - ScannerDirectives: '%block_comment'^ /* Clipped */ TokenLiteral TokenLiteral;
    Production {
        lhs: 32,
        production: &[ParseType::N(44), ParseType::N(44), ParseType::T(12)],
    },
    // 14 - ScannerDirectives: '%auto_newline_off'^ /* Clipped */;
    Production {
        lhs: 32,
        production: &[ParseType::T(13)],
    },
    // 15 - ScannerDirectives: '%auto_ws_off'^ /* Clipped */;
    Production {
        lhs: 32,
        production: &[ParseType::T(14)],
    },
    // 16 - ScannerDirectives: '%on'^ /* Clipped */ IdentifierList '%enter'^ /* Clipped */ Identifier;
    Production {
        lhs: 32,
        production: &[
            ParseType::N(13),
            ParseType::T(16),
            ParseType::N(14),
            ParseType::T(15),
        ],
    },
    // 17 - GrammarDefinition: '%%'^ /* Clipped */ Production GrammarDefinitionList /* Vec */;
    Production {
        lhs: 10,
        production: &[ParseType::N(11), ParseType::N(25), ParseType::T(17)],
    },
    // 18 - GrammarDefinitionList: Production GrammarDefinitionList;
    Production {
        lhs: 11,
        production: &[ParseType::N(11), ParseType::N(25)],
    },
    // 19 - GrammarDefinitionList: ;
    Production {
        lhs: 11,
        production: &[],
    },
    // 20 - DoubleColon: '::';
    Production {
        lhs: 8,
        production: &[ParseType::T(18)],
    },
    // 21 - Production: Identifier ':'^ /* Clipped */ Alternations ';'^ /* Clipped */;
    Production {
        lhs: 25,
        production: &[
            ParseType::T(20),
            ParseType::N(4),
            ParseType::T(19),
            ParseType::N(13),
        ],
    },
    // 22 - Alternations: Alternation AlternationsList /* Vec */;
    Production {
        lhs: 4,
        production: &[ParseType::N(5), ParseType::N(2)],
    },
    // 23 - AlternationsList: '|'^ /* Clipped */ Alternation AlternationsList;
    Production {
        lhs: 5,
        production: &[ParseType::N(5), ParseType::N(2), ParseType::T(21)],
    },
    // 24 - AlternationsList: ;
    Production {
        lhs: 5,
        production: &[],
    },
    // 25 - Alternation: AlternationList /* Vec */;
    Production {
        lhs: 2,
        production: &[ParseType::N(3)],
    },
    // 26 - AlternationList: Factor AlternationList;
    Production {
        lhs: 3,
        production: &[ParseType::N(3), ParseType::N(9)],
    },
    // 27 - AlternationList: ;
    Production {
        lhs: 3,
        production: &[],
    },
    // 28 - Factor: Group;
    Production {
        lhs: 9,
        production: &[ParseType::N(12)],
    },
    // 29 - Factor: Repeat;
    Production {
        lhs: 9,
        production: &[ParseType::N(31)],
    },
    // 30 - Factor: Optional;
    Production {
        lhs: 9,
        production: &[ParseType::N(22)],
    },
    // 31 - Factor: Symbol;
    Production {
        lhs: 9,
        production: &[ParseType::N(41)],
    },
    // 32 - Symbol: NonTerminal;
    Production {
        lhs: 41,
        production: &[ParseType::N(20)],
    },
    // 33 - Symbol: SimpleToken;
    Production {
        lhs: 41,
        production: &[ParseType::N(37)],
    },
    // 34 - Symbol: TokenWithStates;
    Production {
        lhs: 41,
        production: &[ParseType::N(45)],
    },
    // 35 - Symbol: ScannerSwitch;
    Production {
        lhs: 41,
        production: &[ParseType::N(35)],
    },
    // 36 - TokenLiteral: String;
    Production {
        lhs: 44,
        production: &[ParseType::N(40)],
    },
    // 37 - TokenLiteral: RawString;
    Production {
        lhs: 44,
        production: &[ParseType::N(29)],
    },
    // 38 - TokenLiteral: Regex;
    Production {
        lhs: 44,
        production: &[ParseType::N(30)],
    },
    // 39 - TokenExpression: TokenLiteral TokenExpressionOpt /* Option */;
    Production {
        lhs: 42,
        production: &[ParseType::N(43), ParseType::N(44)],
    },
    // 40 - TokenExpressionOpt: LookAhead;
    Production {
        lhs: 43,
        production: &[ParseType::N(16)],
    },
    // 41 - TokenExpressionOpt: ;
    Production {
        lhs: 43,
        production: &[],
    },
    // 42 - SimpleToken: TokenExpression SimpleTokenOpt /* Option */;
    Production {
        lhs: 37,
        production: &[ParseType::N(38), ParseType::N(42)],
    },
    // 43 - SimpleTokenOpt: ASTControl;
    Production {
        lhs: 38,
        production: &[ParseType::N(0)],
    },
    // 44 - SimpleTokenOpt: ;
    Production {
        lhs: 38,
        production: &[],
    },
    // 45 - TokenWithStates: '<'^ /* Clipped */ IdentifierList '>'^ /* Clipped */ TokenExpression TokenWithStatesOpt /* Option */;
    Production {
        lhs: 45,
        production: &[
            ParseType::N(46),
            ParseType::N(42),
            ParseType::T(23),
            ParseType::N(14),
            ParseType::T(22),
        ],
    },
    // 46 - TokenWithStatesOpt: ASTControl;
    Production {
        lhs: 46,
        production: &[ParseType::N(0)],
    },
    // 47 - TokenWithStatesOpt: ;
    Production {
        lhs: 46,
        production: &[],
    },
    // 48 - String: /"(\\.|[^"])*"/;
    Production {
        lhs: 40,
        production: &[ParseType::T(24)],
    },
    // 49 - RawString: /'(\\.|[^'])*'/;
    Production {
        lhs: 29,
        production: &[ParseType::T(25)],
    },
    // 50 - Regex: "/(\\.|[^\/])*/";
    Production {
        lhs: 30,
        production: &[ParseType::T(26)],
    },
    // 51 - Group: '(' Alternations ')';
    Production {
        lhs: 12,
        production: &[ParseType::T(28), ParseType::N(4), ParseType::T(27)],
    },
    // 52 - Optional: '[' Alternations ']';
    Production {
        lhs: 22,
        production: &[ParseType::T(30), ParseType::N(4), ParseType::T(29)],
    },
    // 53 - Repeat: '{' Alternations '}';
    Production {
        lhs: 31,
        production: &[ParseType::T(32), ParseType::N(4), ParseType::T(31)],
    },
    // 54 - NonTerminal: Identifier NonTerminalOpt /* Option */;
    Production {
        lhs: 20,
        production: &[ParseType::N(21), ParseType::N(13)],
    },
    // 55 - NonTerminalOpt: ASTControl;
    Production {
        lhs: 21,
        production: &[ParseType::N(0)],
    },
    // 56 - NonTerminalOpt: ;
    Production {
        lhs: 21,
        production: &[],
    },
    // 57 - Identifier: /[a-zA-Z_][a-zA-Z0-9_]*/;
    Production {
        lhs: 13,
        production: &[ParseType::T(33)],
    },
    // 58 - ScannerState: '%scanner'^ /* Clipped */ Identifier@state_name '{'^ /* Clipped */ ScannerStateList /* Vec */ '}'^ /* Clipped */;
    Production {
        lhs: 33,
        production: &[
            ParseType::T(32),
            ParseType::N(34),
            ParseType::T(31),
            ParseType::N(13),
            ParseType::T(34),
        ],
    },
    // 59 - ScannerStateList: ScannerDirectives ScannerStateList;
    Production {
        lhs: 34,
        production: &[ParseType::N(34), ParseType::N(32)],
    },
    // 60 - ScannerStateList: ;
    Production {
        lhs: 34,
        production: &[],
    },
    // 61 - IdentifierList: Identifier IdentifierListList /* Vec */;
    Production {
        lhs: 14,
        production: &[ParseType::N(15), ParseType::N(13)],
    },
    // 62 - IdentifierListList: ','^ /* Clipped */ Identifier IdentifierListList;
    Production {
        lhs: 15,
        production: &[ParseType::N(15), ParseType::N(13), ParseType::T(35)],
    },
    // 63 - IdentifierListList: ;
    Production {
        lhs: 15,
        production: &[],
    },
    // 64 - ScannerSwitch: '%sc' '('^ /* Clipped */ ScannerSwitchOpt /* Option */ ')'^ /* Clipped */;
    Production {
        lhs: 35,
        production: &[
            ParseType::T(28),
            ParseType::N(36),
            ParseType::T(27),
            ParseType::T(36),
        ],
    },
    // 65 - ScannerSwitch: '%push' '('^ /* Clipped */ Identifier ')'^ /* Clipped */;
    Production {
        lhs: 35,
        production: &[
            ParseType::T(28),
            ParseType::N(13),
            ParseType::T(27),
            ParseType::T(37),
        ],
    },
    // 66 - ScannerSwitch: '%pop' '('^ /* Clipped */ ')'^ /* Clipped */;
    Production {
        lhs: 35,
        production: &[ParseType::T(28), ParseType::T(27), ParseType::T(38)],
    },
    // 67 - ScannerSwitchOpt: Identifier;
    Production {
        lhs: 36,
        production: &[ParseType::N(13)],
    },
    // 68 - ScannerSwitchOpt: ;
    Production {
        lhs: 36,
        production: &[],
    },
    // 69 - ASTControl: CutOperator;
    Production {
        lhs: 0,
        production: &[ParseType::N(6)],
    },
    // 70 - ASTControl: MemberName ASTControlOpt /* Option */;
    Production {
        lhs: 0,
        production: &[ParseType::N(1), ParseType::N(18)],
    },
    // 71 - ASTControl: UserTypeDeclaration;
    Production {
        lhs: 0,
        production: &[ParseType::N(47)],
    },
    // 72 - ASTControlOpt: UserTypeDeclaration;
    Production {
        lhs: 1,
        production: &[ParseType::N(47)],
    },
    // 73 - ASTControlOpt: ;
    Production {
        lhs: 1,
        production: &[],
    },
    // 74 - MemberName: '@'^ /* Clipped */ Identifier;
    Production {
        lhs: 18,
        production: &[ParseType::N(13), ParseType::T(39)],
    },
    // 75 - CutOperator: '^'^ /* Clipped */;
    Production {
        lhs: 6,
        production: &[ParseType::T(40)],
    },
    // 76 - UserTypeDeclaration: ':'^ /* Clipped */ UserTypeName : crate::parser::parol_grammar::UserDefinedTypeName ;
    Production {
        lhs: 47,
        production: &[ParseType::N(48), ParseType::T(19)],
    },
    // 77 - UserTypeName: Identifier UserTypeNameList /* Vec */;
    Production {
        lhs: 48,
        production: &[ParseType::N(49), ParseType::N(13)],
    },
    // 78 - UserTypeNameList: DoubleColon^ /* Clipped */ Identifier UserTypeNameList;
    Production {
        lhs: 49,
        production: &[ParseType::N(49), ParseType::N(13), ParseType::N(8)],
    },
    // 79 - UserTypeNameList: ;
    Production {
        lhs: 49,
        production: &[],
    },
    // 80 - LookAhead: LookAheadGroup TokenLiteral;
    Production {
        lhs: 16,
        production: &[ParseType::N(44), ParseType::N(17)],
    },
    // 81 - LookAheadGroup: PositiveLookahead;
    Production {
        lhs: 17,
        production: &[ParseType::N(24)],
    },
    // 82 - LookAheadGroup: NegativeLookahead;
    Production {
        lhs: 17,
        production: &[ParseType::N(19)],
    },
    // 83 - PositiveLookahead: '?='^ /* Clipped */;
    Production {
        lhs: 24,
        production: &[ParseType::T(41)],
    },
    // 84 - NegativeLookahead: '?!'^ /* Clipped */;
    Production {
        lhs: 19,
        production: &[ParseType::T(42)],
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
    user_actions: &mut ParolGrammar<'t>,
) -> Result<ParseTree, ParolError>
where
    T: AsRef<Path>,
{
    let mut llk_parser = LLKParser::new(
        23,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    // Initialize wrapper
    let mut user_actions = ParolGrammarAuto::new(user_actions);
    llk_parser.parse(
        TokenStream::new(input, file_name, &SCANNERS, MAX_K).unwrap(),
        &mut user_actions,
    )
}
