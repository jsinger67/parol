// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

use parol_runtime::{
    parser::{
        parse_tree_type::TreeConstruct, LLKParser, LookaheadDFA, ParseType, Production, Trans,
    },
    ParolError, ParseTree, TokenStream,
};
use scnr2::scanner;
use std::path::Path;

use crate::basic_grammar::BasicGrammar;
use crate::basic_grammar_trait::BasicGrammarAuto;

pub const TERMINAL_NAMES: &[&str; 32] = &[
    /*  0 */ "EndOfInput",
    /*  1 */ "Newline",
    /*  2 */ "Whitespace",
    /*  3 */ "LineComment",
    /*  4 */ "BlockComment",
    /*  5 */ "Colon",
    /*  6 */ "LineNumber",
    /*  7 */ "Comma",
    /*  8 */ "EndOfLine",
    /*  9 */ "Float1",
    /* 10 */ "Float2",
    /* 11 */ "Integer",
    /* 12 */ "Rem",
    /* 13 */ "If",
    /* 14 */ "Then",
    /* 15 */ "Goto",
    /* 16 */ "Let",
    /* 17 */ "Print",
    /* 18 */ "End",
    /* 19 */ "AssignOp",
    /* 20 */ "LogicalOrOp",
    /* 21 */ "LogicalAndOp",
    /* 22 */ "LogicalNotOp",
    /* 23 */ "RelationalOp",
    /* 24 */ "Plus",
    /* 25 */ "Minus",
    /* 26 */ "MulOp",
    /* 27 */ "LParen",
    /* 28 */ "RParen",
    /* 29 */ "Comment",
    /* 30 */ "Variable",
    /* 31 */ "Error",
];

scanner! {
    BasicGrammarScanner {
        mode INITIAL {
            token r"[\s--\r\n]+" => 2; // "Whitespace"
            token r":" => 5; // "Colon"
            token r"[0 ]*[1-9] *(?:[0-9] *){1,4}|[0 ]+" => 6; // "LineNumber"
            token r"," => 7; // "Comma"
            token r"(?:\r?\n|\r)+" => 8; // "EndOfLine"
            token r"REM" => 12; // "Rem"
            token r"IF" => 13; // "If"
            token r"THEN" => 14; // "Then"
            token r"GOTO" => 15; // "Goto"
            token r"LET" => 16; // "Let"
            token r"PRINT|\?" => 17; // "Print"
            token r"END" => 18; // "End"
            token r"=" => 19; // "AssignOp"
            token r"[A-Z][0-9A-Z]*" => 30; // "Variable"
            on 12 enter Cmnt;
            on 13 enter Expr;
            on 17 enter Expr;
            on 19 enter Expr;
        }
        mode Cmnt {
            token r"[\s--\r\n]+" => 2; // "Whitespace"
            token r"(?:\r?\n|\r)+" => 8; // "EndOfLine"
            token r"[^\r\n]+" => 29; // "Comment"
            on 8 enter INITIAL;
        }
        mode Expr {
            token r"[\s--\r\n]+" => 2; // "Whitespace"
            token r":" => 5; // "Colon"
            token r"," => 7; // "Comma"
            token r"(?:\r?\n|\r)+" => 8; // "EndOfLine"
            token r"(?:(?:[0-9] *)+)?\. *(?:(?:[0-9] *)+)? *(?:E *[-+]? *(?:[0-9] *)+)?" => 9; // "Float1"
            token r"(?:[0-9] *)+E *[-+]? *(?:[0-9] *)+" => 10; // "Float2"
            token r"(?:[0-9] *)+" => 11; // "Integer"
            token r"THEN" => 14; // "Then"
            token r"GOTO" => 15; // "Goto"
            token r"N?OR" => 20; // "LogicalOrOp"
            token r"AND" => 21; // "LogicalAndOp"
            token r"NOT" => 22; // "LogicalNotOp"
            token r"<\s*>|<\s*=|<|>\s*=|>|=" => 23; // "RelationalOp"
            token r"\+" => 24; // "Plus"
            token r"\-" => 25; // "Minus"
            token r"\*|\u{2F}" => 26; // "MulOp"
            token r"\(" => 27; // "LParen"
            token r"\)" => 28; // "RParen"
            token r"[A-Z][0-9A-Z]*" => 30; // "Variable"
            on 8 enter INITIAL;
            on 14 enter INITIAL;
            on 15 enter INITIAL;
        }
    }
}

const MAX_K: usize = 2;

pub const NON_TERMINALS: &[&str; 59] = &[
    /*  0 */ "AssignOp",
    /*  1 */ "Assignment",
    /*  2 */ "AssignmentOpt",
    /*  3 */ "Basic",
    /*  4 */ "BasicList",
    /*  5 */ "BasicOpt",
    /*  6 */ "BasicOpt0",
    /*  7 */ "Comment",
    /*  8 */ "End",
    /*  9 */ "EndOfLine",
    /* 10 */ "EndStatement",
    /* 11 */ "Expression",
    /* 12 */ "Factor",
    /* 13 */ "Float",
    /* 14 */ "Float1",
    /* 15 */ "Float2",
    /* 16 */ "Goto",
    /* 17 */ "GotoStatement",
    /* 18 */ "If",
    /* 19 */ "IfBody",
    /* 20 */ "IfStatement",
    /* 21 */ "Integer",
    /* 22 */ "LParen",
    /* 23 */ "Let",
    /* 24 */ "Line",
    /* 25 */ "LineList",
    /* 26 */ "LineNumber",
    /* 27 */ "Literal",
    /* 28 */ "LogicalAnd",
    /* 29 */ "LogicalAndList",
    /* 30 */ "LogicalAndOp",
    /* 31 */ "LogicalNot",
    /* 32 */ "LogicalNotOp",
    /* 33 */ "LogicalNotOpt",
    /* 34 */ "LogicalOr",
    /* 35 */ "LogicalOrList",
    /* 36 */ "LogicalOrOp",
    /* 37 */ "Minus",
    /* 38 */ "MulOp",
    /* 39 */ "Multiplication",
    /* 40 */ "MultiplicationList",
    /* 41 */ "Number",
    /* 42 */ "Plus",
    /* 43 */ "Print",
    /* 44 */ "PrintStatement",
    /* 45 */ "PrintStatementList",
    /* 46 */ "RParen",
    /* 47 */ "Relational",
    /* 48 */ "RelationalList",
    /* 49 */ "RelationalOp",
    /* 50 */ "Rem",
    /* 51 */ "Remark",
    /* 52 */ "RemarkOpt",
    /* 53 */ "Statement",
    /* 54 */ "Summation",
    /* 55 */ "SummationList",
    /* 56 */ "SummationListGroup",
    /* 57 */ "Then",
    /* 58 */ "Variable",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 59] = &[
    /* 0 - "AssignOp" */
    LookaheadDFA {
        prod0: 47,
        transitions: &[],
        k: 0,
    },
    /* 1 - "Assignment" */
    LookaheadDFA {
        prod0: 22,
        transitions: &[],
        k: 0,
    },
    /* 2 - "AssignmentOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 16, 1, 23), Trans(0, 30, 2, 24)],
        k: 1,
    },
    /* 3 - "Basic" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 4 - "BasicList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 3, 2),
            Trans(0, 8, 1, -1),
            Trans(1, 0, 3, 2),
            Trans(1, 6, 2, 1),
        ],
        k: 2,
    },
    /* 5 - "BasicOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 6, 2, 6), Trans(0, 8, 1, 5)],
        k: 1,
    },
    /* 6 - "BasicOpt0" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 4), Trans(0, 8, 1, 3)],
        k: 1,
    },
    /* 7 - "Comment" */
    LookaheadDFA {
        prod0: 57,
        transitions: &[],
        k: 0,
    },
    /* 8 - "End" */
    LookaheadDFA {
        prod0: 46,
        transitions: &[],
        k: 0,
    },
    /* 9 - "EndOfLine" */
    LookaheadDFA {
        prod0: 31,
        transitions: &[],
        k: 0,
    },
    /* 10 - "EndStatement" */
    LookaheadDFA {
        prod0: 30,
        transitions: &[],
        k: 0,
    },
    /* 11 - "Expression" */
    LookaheadDFA {
        prod0: 59,
        transitions: &[],
        k: 0,
    },
    /* 12 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 9, 1, 80),
            Trans(0, 10, 1, 80),
            Trans(0, 11, 1, 80),
            Trans(0, 25, 3, 82),
            Trans(0, 27, 4, 83),
            Trans(0, 30, 2, 81),
        ],
        k: 1,
    },
    /* 13 - "Float" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 9, 1, 35), Trans(0, 10, 2, 36)],
        k: 1,
    },
    /* 14 - "Float1" */
    LookaheadDFA {
        prod0: 37,
        transitions: &[],
        k: 0,
    },
    /* 15 - "Float2" */
    LookaheadDFA {
        prod0: 38,
        transitions: &[],
        k: 0,
    },
    /* 16 - "Goto" */
    LookaheadDFA {
        prod0: 43,
        transitions: &[],
        k: 0,
    },
    /* 17 - "GotoStatement" */
    LookaheadDFA {
        prod0: 20,
        transitions: &[],
        k: 0,
    },
    /* 18 - "If" */
    LookaheadDFA {
        prod0: 41,
        transitions: &[],
        k: 0,
    },
    /* 19 - "IfBody" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 14, 1, 25), Trans(0, 15, 2, 26)],
        k: 1,
    },
    /* 20 - "IfStatement" */
    LookaheadDFA {
        prod0: 21,
        transitions: &[],
        k: 0,
    },
    /* 21 - "Integer" */
    LookaheadDFA {
        prod0: 39,
        transitions: &[],
        k: 0,
    },
    /* 22 - "LParen" */
    LookaheadDFA {
        prod0: 55,
        transitions: &[],
        k: 0,
    },
    /* 23 - "Let" */
    LookaheadDFA {
        prod0: 44,
        transitions: &[],
        k: 0,
    },
    /* 24 - "Line" */
    LookaheadDFA {
        prod0: 7,
        transitions: &[],
        k: 0,
    },
    /* 25 - "LineList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 9), Trans(0, 5, 1, 8), Trans(0, 8, 2, 9)],
        k: 1,
    },
    /* 26 - "LineNumber" */
    LookaheadDFA {
        prod0: 10,
        transitions: &[],
        k: 0,
    },
    /* 27 - "Literal" */
    LookaheadDFA {
        prod0: 32,
        transitions: &[],
        k: 0,
    },
    /* 28 - "LogicalAnd" */
    LookaheadDFA {
        prod0: 63,
        transitions: &[],
        k: 0,
    },
    /* 29 - "LogicalAndList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 65),
            Trans(0, 5, 2, 65),
            Trans(0, 7, 2, 65),
            Trans(0, 8, 2, 65),
            Trans(0, 14, 2, 65),
            Trans(0, 15, 2, 65),
            Trans(0, 20, 2, 65),
            Trans(0, 21, 1, 64),
            Trans(0, 28, 2, 65),
        ],
        k: 1,
    },
    /* 30 - "LogicalAndOp" */
    LookaheadDFA {
        prod0: 49,
        transitions: &[],
        k: 0,
    },
    /* 31 - "LogicalNot" */
    LookaheadDFA {
        prod0: 66,
        transitions: &[],
        k: 0,
    },
    /* 32 - "LogicalNotOp" */
    LookaheadDFA {
        prod0: 50,
        transitions: &[],
        k: 0,
    },
    /* 33 - "LogicalNotOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 9, 2, 68),
            Trans(0, 10, 2, 68),
            Trans(0, 11, 2, 68),
            Trans(0, 22, 1, 67),
            Trans(0, 25, 2, 68),
            Trans(0, 27, 2, 68),
            Trans(0, 30, 2, 68),
        ],
        k: 1,
    },
    /* 34 - "LogicalOr" */
    LookaheadDFA {
        prod0: 60,
        transitions: &[],
        k: 0,
    },
    /* 35 - "LogicalOrList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 62),
            Trans(0, 5, 2, 62),
            Trans(0, 7, 2, 62),
            Trans(0, 8, 2, 62),
            Trans(0, 14, 2, 62),
            Trans(0, 15, 2, 62),
            Trans(0, 20, 1, 61),
            Trans(0, 28, 2, 62),
        ],
        k: 1,
    },
    /* 36 - "LogicalOrOp" */
    LookaheadDFA {
        prod0: 48,
        transitions: &[],
        k: 0,
    },
    /* 37 - "Minus" */
    LookaheadDFA {
        prod0: 53,
        transitions: &[],
        k: 0,
    },
    /* 38 - "MulOp" */
    LookaheadDFA {
        prod0: 54,
        transitions: &[],
        k: 0,
    },
    /* 39 - "Multiplication" */
    LookaheadDFA {
        prod0: 77,
        transitions: &[],
        k: 0,
    },
    /* 40 - "MultiplicationList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 79),
            Trans(0, 5, 2, 79),
            Trans(0, 7, 2, 79),
            Trans(0, 8, 2, 79),
            Trans(0, 14, 2, 79),
            Trans(0, 15, 2, 79),
            Trans(0, 20, 2, 79),
            Trans(0, 21, 2, 79),
            Trans(0, 23, 2, 79),
            Trans(0, 24, 2, 79),
            Trans(0, 25, 2, 79),
            Trans(0, 26, 1, 78),
            Trans(0, 28, 2, 79),
        ],
        k: 1,
    },
    /* 41 - "Number" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 9, 1, 33), Trans(0, 10, 1, 33), Trans(0, 11, 2, 34)],
        k: 1,
    },
    /* 42 - "Plus" */
    LookaheadDFA {
        prod0: 52,
        transitions: &[],
        k: 0,
    },
    /* 43 - "Print" */
    LookaheadDFA {
        prod0: 45,
        transitions: &[],
        k: 0,
    },
    /* 44 - "PrintStatement" */
    LookaheadDFA {
        prod0: 27,
        transitions: &[],
        k: 0,
    },
    /* 45 - "PrintStatementList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 29),
            Trans(0, 5, 2, 29),
            Trans(0, 7, 1, 28),
            Trans(0, 8, 2, 29),
        ],
        k: 1,
    },
    /* 46 - "RParen" */
    LookaheadDFA {
        prod0: 56,
        transitions: &[],
        k: 0,
    },
    /* 47 - "Relational" */
    LookaheadDFA {
        prod0: 69,
        transitions: &[],
        k: 0,
    },
    /* 48 - "RelationalList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 71),
            Trans(0, 5, 2, 71),
            Trans(0, 7, 2, 71),
            Trans(0, 8, 2, 71),
            Trans(0, 14, 2, 71),
            Trans(0, 15, 2, 71),
            Trans(0, 20, 2, 71),
            Trans(0, 21, 2, 71),
            Trans(0, 23, 1, 70),
            Trans(0, 28, 2, 71),
        ],
        k: 1,
    },
    /* 49 - "RelationalOp" */
    LookaheadDFA {
        prod0: 51,
        transitions: &[],
        k: 0,
    },
    /* 50 - "Rem" */
    LookaheadDFA {
        prod0: 40,
        transitions: &[],
        k: 0,
    },
    /* 51 - "Remark" */
    LookaheadDFA {
        prod0: 17,
        transitions: &[],
        k: 0,
    },
    /* 52 - "RemarkOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 19),
            Trans(0, 5, 2, 19),
            Trans(0, 8, 2, 19),
            Trans(0, 29, 1, 18),
        ],
        k: 1,
    },
    /* 53 - "Statement" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 12, 1, 11),
            Trans(0, 13, 3, 13),
            Trans(0, 15, 2, 12),
            Trans(0, 16, 4, 14),
            Trans(0, 17, 5, 15),
            Trans(0, 18, 6, 16),
            Trans(0, 30, 4, 14),
        ],
        k: 1,
    },
    /* 54 - "Summation" */
    LookaheadDFA {
        prod0: 72,
        transitions: &[],
        k: 0,
    },
    /* 55 - "SummationList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 76),
            Trans(0, 5, 2, 76),
            Trans(0, 7, 2, 76),
            Trans(0, 8, 2, 76),
            Trans(0, 14, 2, 76),
            Trans(0, 15, 2, 76),
            Trans(0, 20, 2, 76),
            Trans(0, 21, 2, 76),
            Trans(0, 23, 2, 76),
            Trans(0, 24, 1, 73),
            Trans(0, 25, 1, 73),
            Trans(0, 28, 2, 76),
        ],
        k: 1,
    },
    /* 56 - "SummationListGroup" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 24, 1, 74), Trans(0, 25, 2, 75)],
        k: 1,
    },
    /* 57 - "Then" */
    LookaheadDFA {
        prod0: 42,
        transitions: &[],
        k: 0,
    },
    /* 58 - "Variable" */
    LookaheadDFA {
        prod0: 58,
        transitions: &[],
        k: 0,
    },
];

pub const PRODUCTIONS: &[Production; 84] = &[
    // 0 - Basic: BasicOpt /* Option */ Line BasicList /* Vec */ BasicOpt0 /* Option */;
    Production {
        lhs: 3,
        production: &[
            ParseType::N(6),
            ParseType::N(4),
            ParseType::N(24),
            ParseType::N(5),
        ],
    },
    // 1 - BasicList: EndOfLine Line BasicList;
    Production {
        lhs: 4,
        production: &[ParseType::N(4), ParseType::N(24), ParseType::N(9)],
    },
    // 2 - BasicList: ;
    Production {
        lhs: 4,
        production: &[],
    },
    // 3 - BasicOpt0: EndOfLine;
    Production {
        lhs: 6,
        production: &[ParseType::N(9)],
    },
    // 4 - BasicOpt0: ;
    Production {
        lhs: 6,
        production: &[],
    },
    // 5 - BasicOpt: EndOfLine;
    Production {
        lhs: 5,
        production: &[ParseType::N(9)],
    },
    // 6 - BasicOpt: ;
    Production {
        lhs: 5,
        production: &[],
    },
    // 7 - Line: LineNumber Statement LineList /* Vec */;
    Production {
        lhs: 24,
        production: &[ParseType::N(25), ParseType::N(53), ParseType::N(26)],
    },
    // 8 - LineList: ':'^ /* Clipped */ Statement LineList;
    Production {
        lhs: 25,
        production: &[ParseType::N(25), ParseType::N(53), ParseType::T(5)],
    },
    // 9 - LineList: ;
    Production {
        lhs: 25,
        production: &[],
    },
    // 10 - LineNumber: /[0 ]*[1-9] *(?:[0-9] *){1,4}|[0 ]+/;
    Production {
        lhs: 26,
        production: &[ParseType::T(6)],
    },
    // 11 - Statement: Remark;
    Production {
        lhs: 53,
        production: &[ParseType::N(51)],
    },
    // 12 - Statement: GotoStatement;
    Production {
        lhs: 53,
        production: &[ParseType::N(17)],
    },
    // 13 - Statement: IfStatement;
    Production {
        lhs: 53,
        production: &[ParseType::N(20)],
    },
    // 14 - Statement: Assignment;
    Production {
        lhs: 53,
        production: &[ParseType::N(1)],
    },
    // 15 - Statement: PrintStatement;
    Production {
        lhs: 53,
        production: &[ParseType::N(44)],
    },
    // 16 - Statement: EndStatement;
    Production {
        lhs: 53,
        production: &[ParseType::N(10)],
    },
    // 17 - Remark: Rem RemarkOpt /* Option */;
    Production {
        lhs: 51,
        production: &[ParseType::N(52), ParseType::N(50)],
    },
    // 18 - RemarkOpt: Comment;
    Production {
        lhs: 52,
        production: &[ParseType::N(7)],
    },
    // 19 - RemarkOpt: ;
    Production {
        lhs: 52,
        production: &[],
    },
    // 20 - GotoStatement: Goto LineNumber;
    Production {
        lhs: 17,
        production: &[ParseType::N(26), ParseType::N(16)],
    },
    // 21 - IfStatement: If Expression IfBody;
    Production {
        lhs: 20,
        production: &[ParseType::N(19), ParseType::N(11), ParseType::N(18)],
    },
    // 22 - Assignment: AssignmentOpt /* Option */ Variable AssignOp Expression;
    Production {
        lhs: 1,
        production: &[
            ParseType::N(11),
            ParseType::N(0),
            ParseType::N(58),
            ParseType::N(2),
        ],
    },
    // 23 - AssignmentOpt: Let;
    Production {
        lhs: 2,
        production: &[ParseType::N(23)],
    },
    // 24 - AssignmentOpt: ;
    Production {
        lhs: 2,
        production: &[],
    },
    // 25 - IfBody: Then Statement;
    Production {
        lhs: 19,
        production: &[ParseType::N(53), ParseType::N(57)],
    },
    // 26 - IfBody: Goto LineNumber;
    Production {
        lhs: 19,
        production: &[ParseType::N(26), ParseType::N(16)],
    },
    // 27 - PrintStatement: Print Expression PrintStatementList /* Vec */;
    Production {
        lhs: 44,
        production: &[ParseType::N(45), ParseType::N(11), ParseType::N(43)],
    },
    // 28 - PrintStatementList: ','^ /* Clipped */ Expression PrintStatementList;
    Production {
        lhs: 45,
        production: &[ParseType::N(45), ParseType::N(11), ParseType::T(7)],
    },
    // 29 - PrintStatementList: ;
    Production {
        lhs: 45,
        production: &[],
    },
    // 30 - EndStatement: End;
    Production {
        lhs: 10,
        production: &[ParseType::N(8)],
    },
    // 31 - EndOfLine: /(?:\r?\n|\r)+/^ /* Clipped */;
    Production {
        lhs: 9,
        production: &[ParseType::T(8)],
    },
    // 32 - Literal: Number;
    Production {
        lhs: 27,
        production: &[ParseType::N(41)],
    },
    // 33 - Number: Float;
    Production {
        lhs: 41,
        production: &[ParseType::N(13)],
    },
    // 34 - Number: Integer;
    Production {
        lhs: 41,
        production: &[ParseType::N(21)],
    },
    // 35 - Float: Float1;
    Production {
        lhs: 13,
        production: &[ParseType::N(14)],
    },
    // 36 - Float: Float2;
    Production {
        lhs: 13,
        production: &[ParseType::N(15)],
    },
    // 37 - Float1: /(?:(?:[0-9] *)+)?\. *(?:(?:[0-9] *)+)? *(?:E *[-+]? *(?:[0-9] *)+)?/;
    Production {
        lhs: 14,
        production: &[ParseType::T(9)],
    },
    // 38 - Float2: /(?:[0-9] *)+E *[-+]? *(?:[0-9] *)+/;
    Production {
        lhs: 15,
        production: &[ParseType::T(10)],
    },
    // 39 - Integer: /(?:[0-9] *)+/;
    Production {
        lhs: 21,
        production: &[ParseType::T(11)],
    },
    // 40 - Rem: 'REM'^ /* Clipped */;
    Production {
        lhs: 50,
        production: &[ParseType::T(12)],
    },
    // 41 - If: 'IF'^ /* Clipped */;
    Production {
        lhs: 18,
        production: &[ParseType::T(13)],
    },
    // 42 - Then: 'THEN'^ /* Clipped */;
    Production {
        lhs: 57,
        production: &[ParseType::T(14)],
    },
    // 43 - Goto: 'GOTO'^ /* Clipped */;
    Production {
        lhs: 16,
        production: &[ParseType::T(15)],
    },
    // 44 - Let: 'LET'^ /* Clipped */;
    Production {
        lhs: 23,
        production: &[ParseType::T(16)],
    },
    // 45 - Print: /PRINT|\?/^ /* Clipped */;
    Production {
        lhs: 43,
        production: &[ParseType::T(17)],
    },
    // 46 - End: 'END'^ /* Clipped */;
    Production {
        lhs: 8,
        production: &[ParseType::T(18)],
    },
    // 47 - AssignOp: '='^ /* Clipped */;
    Production {
        lhs: 0,
        production: &[ParseType::T(19)],
    },
    // 48 - LogicalOrOp: /N?OR/;
    Production {
        lhs: 36,
        production: &[ParseType::T(20)],
    },
    // 49 - LogicalAndOp: 'AND';
    Production {
        lhs: 30,
        production: &[ParseType::T(21)],
    },
    // 50 - LogicalNotOp: 'NOT';
    Production {
        lhs: 32,
        production: &[ParseType::T(22)],
    },
    // 51 - RelationalOp: /<\s*>|<\s*=|<|>\s*=|>|=/;
    Production {
        lhs: 49,
        production: &[ParseType::T(23)],
    },
    // 52 - Plus: '+';
    Production {
        lhs: 42,
        production: &[ParseType::T(24)],
    },
    // 53 - Minus: '-';
    Production {
        lhs: 37,
        production: &[ParseType::T(25)],
    },
    // 54 - MulOp: /\*|\u{2F}/;
    Production {
        lhs: 38,
        production: &[ParseType::T(26)],
    },
    // 55 - LParen: '(';
    Production {
        lhs: 22,
        production: &[ParseType::T(27)],
    },
    // 56 - RParen: ')';
    Production {
        lhs: 46,
        production: &[ParseType::T(28)],
    },
    // 57 - Comment: /[^\r\n]+/;
    Production {
        lhs: 7,
        production: &[ParseType::T(29)],
    },
    // 58 - Variable: /[A-Z][0-9A-Z]*/;
    Production {
        lhs: 58,
        production: &[ParseType::T(30)],
    },
    // 59 - Expression: LogicalOr;
    Production {
        lhs: 11,
        production: &[ParseType::N(34)],
    },
    // 60 - LogicalOr: LogicalAnd LogicalOrList /* Vec */;
    Production {
        lhs: 34,
        production: &[ParseType::N(35), ParseType::N(28)],
    },
    // 61 - LogicalOrList: LogicalOrOp LogicalAnd LogicalOrList;
    Production {
        lhs: 35,
        production: &[ParseType::N(35), ParseType::N(28), ParseType::N(36)],
    },
    // 62 - LogicalOrList: ;
    Production {
        lhs: 35,
        production: &[],
    },
    // 63 - LogicalAnd: LogicalNot LogicalAndList /* Vec */;
    Production {
        lhs: 28,
        production: &[ParseType::N(29), ParseType::N(31)],
    },
    // 64 - LogicalAndList: LogicalAndOp LogicalNot LogicalAndList;
    Production {
        lhs: 29,
        production: &[ParseType::N(29), ParseType::N(31), ParseType::N(30)],
    },
    // 65 - LogicalAndList: ;
    Production {
        lhs: 29,
        production: &[],
    },
    // 66 - LogicalNot: LogicalNotOpt /* Option */ Relational;
    Production {
        lhs: 31,
        production: &[ParseType::N(47), ParseType::N(33)],
    },
    // 67 - LogicalNotOpt: LogicalNotOp;
    Production {
        lhs: 33,
        production: &[ParseType::N(32)],
    },
    // 68 - LogicalNotOpt: ;
    Production {
        lhs: 33,
        production: &[],
    },
    // 69 - Relational: Summation RelationalList /* Vec */;
    Production {
        lhs: 47,
        production: &[ParseType::N(48), ParseType::N(54)],
    },
    // 70 - RelationalList: RelationalOp Summation RelationalList;
    Production {
        lhs: 48,
        production: &[ParseType::N(48), ParseType::N(54), ParseType::N(49)],
    },
    // 71 - RelationalList: ;
    Production {
        lhs: 48,
        production: &[],
    },
    // 72 - Summation: Multiplication SummationList /* Vec */;
    Production {
        lhs: 54,
        production: &[ParseType::N(55), ParseType::N(39)],
    },
    // 73 - SummationList: SummationListGroup Multiplication SummationList;
    Production {
        lhs: 55,
        production: &[ParseType::N(55), ParseType::N(39), ParseType::N(56)],
    },
    // 74 - SummationListGroup: Plus;
    Production {
        lhs: 56,
        production: &[ParseType::N(42)],
    },
    // 75 - SummationListGroup: Minus;
    Production {
        lhs: 56,
        production: &[ParseType::N(37)],
    },
    // 76 - SummationList: ;
    Production {
        lhs: 55,
        production: &[],
    },
    // 77 - Multiplication: Factor MultiplicationList /* Vec */;
    Production {
        lhs: 39,
        production: &[ParseType::N(40), ParseType::N(12)],
    },
    // 78 - MultiplicationList: MulOp Factor MultiplicationList;
    Production {
        lhs: 40,
        production: &[ParseType::N(40), ParseType::N(12), ParseType::N(38)],
    },
    // 79 - MultiplicationList: ;
    Production {
        lhs: 40,
        production: &[],
    },
    // 80 - Factor: Literal;
    Production {
        lhs: 12,
        production: &[ParseType::N(27)],
    },
    // 81 - Factor: Variable;
    Production {
        lhs: 12,
        production: &[ParseType::N(58)],
    },
    // 82 - Factor: Minus Factor;
    Production {
        lhs: 12,
        production: &[ParseType::N(12), ParseType::N(37)],
    },
    // 83 - Factor: LParen Expression RParen;
    Production {
        lhs: 12,
        production: &[ParseType::N(46), ParseType::N(11), ParseType::N(22)],
    },
];

pub fn parse<'t, T>(
    input: &'t str,
    file_name: T,
    user_actions: &mut BasicGrammar<'t>,
) -> Result<ParseTree, ParolError>
where
    T: AsRef<Path>,
{
    use parol_runtime::{
        parser::{parse_tree_type::SynTree, parser_types::SynTreeFlavor},
        syntree::Builder,
    };
    let mut builder = Builder::<SynTree, SynTreeFlavor>::new_with();
    parse_into(input, &mut builder, file_name, user_actions)?;
    Ok(builder.build()?)
}
#[allow(dead_code)]
pub fn parse_into<'t, T: TreeConstruct<'t>>(
    input: &'t str,
    tree_builder: &mut T,
    file_name: impl AsRef<Path>,
    user_actions: &mut BasicGrammar<'t>,
) -> Result<(), ParolError>
where
    ParolError: From<T::Error>,
{
    use basic_grammar_scanner::BasicGrammarScanner;
    let mut llk_parser = LLKParser::new(
        3,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    let scanner = BasicGrammarScanner::new();
    // Initialize wrapper
    let mut user_actions = BasicGrammarAuto::new(user_actions);
    llk_parser.parse_into(
        tree_builder,
        TokenStream::new(
            input,
            file_name,
            scanner.scanner_impl.clone(),
            &BasicGrammarScanner::match_function,
            MAX_K,
        )
        .unwrap(),
        &mut user_actions,
    )
}
