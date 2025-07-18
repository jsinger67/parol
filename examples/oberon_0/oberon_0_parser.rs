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

use crate::oberon_0_grammar::Oberon0Grammar;
use crate::oberon_0_grammar_trait::Oberon0GrammarAuto;

pub const TERMINAL_NAMES: &[&str; 42] = &[
    /*  0 */ "EndOfInput",
    /*  1 */ "Newline",
    /*  2 */ "Whitespace",
    /*  3 */ "LineComment",
    /*  4 */ "BlockComment",
    /*  5 */ "Dot",
    /*  6 */ "LBracket",
    /*  7 */ "RBracket",
    /*  8 */ "LParen",
    /*  9 */ "RParen",
    /* 10 */ "Tilde",
    /* 11 */ "ColonEqu",
    /* 12 */ "RelationalOps",
    /* 13 */ "Equ",
    /* 14 */ "Comma",
    /* 15 */ "ELSE",
    /* 16 */ "END",
    /* 17 */ "IF",
    /* 18 */ "THEN",
    /* 19 */ "ELSIF",
    /* 20 */ "WHILE",
    /* 21 */ "DO",
    /* 22 */ "REPEAT",
    /* 23 */ "UNTIL",
    /* 24 */ "Semicolon",
    /* 25 */ "ARRAY",
    /* 26 */ "OF",
    /* 27 */ "Colon",
    /* 28 */ "RECORD",
    /* 29 */ "VAR",
    /* 30 */ "PROCEDURE",
    /* 31 */ "BEGIN",
    /* 32 */ "RETURN",
    /* 33 */ "TYPE",
    /* 34 */ "CONST",
    /* 35 */ "MODULE",
    /* 36 */ "MulOperator",
    /* 37 */ "AddOperator",
    /* 38 */ "UnaryOp",
    /* 39 */ "Ident",
    /* 40 */ "Integer",
    /* 41 */ "Error",
];

scanner! {
    Oberon0GrammarScanner {
        mode INITIAL {
            token r"\r\n|\r|\n" => 1; // "Newline"
            token r"[\s--\r\n]+" => 2; // "Whitespace"
            token r"\(\*([^*]|\*[^)])*\*\)" => 4; // "BlockComment"
            token r"\." => 5; // "Dot"
            token r"\[" => 6; // "LBracket"
            token r"]" => 7; // "RBracket"
            token r"\(" => 8; // "LParen"
            token r"\)" => 9; // "RParen"
            token r"~" => 10; // "Tilde"
            token r":=" => 11; // "ColonEqu"
            token r">=|<=|\#|<|>" => 12; // "RelationalOps"
            token r"=" => 13; // "Equ"
            token r"," => 14; // "Comma"
            token r"ELSE" => 15; // "ELSE"
            token r"END" => 16; // "END"
            token r"IF" => 17; // "IF"
            token r"THEN" => 18; // "THEN"
            token r"ELSIF" => 19; // "ELSIF"
            token r"WHILE" => 20; // "WHILE"
            token r"DO" => 21; // "DO"
            token r"REPEAT" => 22; // "REPEAT"
            token r"UNTIL" => 23; // "UNTIL"
            token r";" => 24; // "Semicolon"
            token r"ARRAY" => 25; // "ARRAY"
            token r"OF" => 26; // "OF"
            token r":" => 27; // "Colon"
            token r"RECORD" => 28; // "RECORD"
            token r"VAR" => 29; // "VAR"
            token r"PROCEDURE" => 30; // "PROCEDURE"
            token r"BEGIN" => 31; // "BEGIN"
            token r"RETURN" => 32; // "RETURN"
            token r"TYPE" => 33; // "TYPE"
            token r"CONST" => 34; // "CONST"
            token r"MODULE" => 35; // "MODULE"
            token r"\*|/|DIV|MOD|&" => 36; // "MulOperator"
            token r"\+|-|OR" => 37; // "AddOperator"
            token r"\+|-" => 38; // "UnaryOp"
            token r"[a-zA-Z][a-zA-Z0-9]*" => 39; // "Ident"
            token r"[0-9]+" => 40; // "Integer"
        }
    }
}

const MAX_K: usize = 2;

pub const NON_TERMINALS: &[&str; 59] = &[
    /*  0 */ "ActualParameters",
    /*  1 */ "ActualParametersSuffix",
    /*  2 */ "AddExpression",
    /*  3 */ "AddOperator",
    /*  4 */ "ArrayType",
    /*  5 */ "AssignOp",
    /*  6 */ "Assignment",
    /*  7 */ "ConstDecls",
    /*  8 */ "Declarations",
    /*  9 */ "DeclarationsSuffix",
    /* 10 */ "DeclarationsSuffix0",
    /* 11 */ "DeclarationsSuffix1",
    /* 12 */ "ElseIfList",
    /* 13 */ "Expression",
    /* 14 */ "ExpressionListRest",
    /* 15 */ "ExpressionSuffix",
    /* 16 */ "FPSection",
    /* 17 */ "FPSectionRest",
    /* 18 */ "Factor",
    /* 19 */ "FieldList",
    /* 20 */ "FieldListRest",
    /* 21 */ "FormalParameters",
    /* 22 */ "FormalParametersSuffix",
    /* 23 */ "Ident",
    /* 24 */ "IdentList",
    /* 25 */ "IdentListRest",
    /* 26 */ "IfPrefix",
    /* 27 */ "IfStatement",
    /* 28 */ "IfStatementSuffix",
    /* 29 */ "Integer",
    /* 30 */ "Module",
    /* 31 */ "ModuleSuffix",
    /* 32 */ "MulExpression",
    /* 33 */ "MulOperator",
    /* 34 */ "ProcedureBody",
    /* 35 */ "ProcedureBodySuffix",
    /* 36 */ "ProcedureBodySuffix0",
    /* 37 */ "ProcedureCall",
    /* 38 */ "ProcedureCallSuffix",
    /* 39 */ "ProcedureDeclaration",
    /* 40 */ "ProcedureDeclarationList",
    /* 41 */ "ProcedureHeading",
    /* 42 */ "ProcedureHeadingSuffix",
    /* 43 */ "RecordType",
    /* 44 */ "RelationOp",
    /* 45 */ "RelationalOps",
    /* 46 */ "RepeatStatement",
    /* 47 */ "Selector",
    /* 48 */ "SelectorList",
    /* 49 */ "SimpleExpression",
    /* 50 */ "Statement",
    /* 51 */ "StatementSequence",
    /* 52 */ "StatementSequenceRest",
    /* 53 */ "Term",
    /* 54 */ "Type",
    /* 55 */ "TypeDecls",
    /* 56 */ "UnaryOp",
    /* 57 */ "VarDecls",
    /* 58 */ "WhileStatement",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 59] = &[
    /* 0 - "ActualParameters" */
    LookaheadDFA {
        prod0: 24,
        transitions: &[],
        k: 0,
    },
    /* 1 - "ActualParametersSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 8, 2, 26),
            Trans(0, 9, 1, 25),
            Trans(0, 10, 2, 26),
            Trans(0, 38, 2, 26),
            Trans(0, 39, 2, 26),
            Trans(0, 40, 2, 26),
        ],
        k: 1,
    },
    /* 2 - "AddExpression" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 7, 2, 14),
            Trans(0, 9, 2, 14),
            Trans(0, 11, 2, 14),
            Trans(0, 12, 2, 14),
            Trans(0, 13, 2, 14),
            Trans(0, 14, 2, 14),
            Trans(0, 15, 2, 14),
            Trans(0, 16, 2, 14),
            Trans(0, 18, 2, 14),
            Trans(0, 19, 2, 14),
            Trans(0, 21, 2, 14),
            Trans(0, 23, 2, 14),
            Trans(0, 24, 2, 14),
            Trans(0, 26, 2, 14),
            Trans(0, 32, 2, 14),
            Trans(0, 37, 1, 13),
        ],
        k: 1,
    },
    /* 3 - "AddOperator" */
    LookaheadDFA {
        prod0: 101,
        transitions: &[],
        k: 0,
    },
    /* 4 - "ArrayType" */
    LookaheadDFA {
        prod0: 52,
        transitions: &[],
        k: 0,
    },
    /* 5 - "AssignOp" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 11, 1, 15), Trans(0, 13, 2, 19)],
        k: 1,
    },
    /* 6 - "Assignment" */
    LookaheadDFA {
        prod0: 23,
        transitions: &[],
        k: 0,
    },
    /* 7 - "ConstDecls" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 92),
            Trans(0, 29, 2, 92),
            Trans(0, 30, 2, 92),
            Trans(0, 31, 2, 92),
            Trans(0, 32, 2, 92),
            Trans(0, 33, 2, 92),
            Trans(0, 39, 1, 91),
        ],
        k: 1,
    },
    /* 8 - "Declarations" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 4, 83),
            Trans(0, 29, 3, 82),
            Trans(0, 30, 4, 83),
            Trans(0, 31, 4, 83),
            Trans(0, 32, 4, 83),
            Trans(0, 33, 1, 78),
            Trans(0, 34, 2, 79),
        ],
        k: 1,
    },
    /* 9 - "DeclarationsSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 88),
            Trans(0, 29, 1, 87),
            Trans(0, 30, 2, 88),
            Trans(0, 31, 2, 88),
            Trans(0, 32, 2, 88),
        ],
        k: 1,
    },
    /* 10 - "DeclarationsSuffix0" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 3, 86),
            Trans(0, 29, 2, 85),
            Trans(0, 30, 3, 86),
            Trans(0, 31, 3, 86),
            Trans(0, 32, 3, 86),
            Trans(0, 33, 1, 84),
        ],
        k: 1,
    },
    /* 11 - "DeclarationsSuffix1" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 81),
            Trans(0, 29, 1, 80),
            Trans(0, 30, 2, 81),
            Trans(0, 31, 2, 81),
            Trans(0, 32, 2, 81),
        ],
        k: 1,
    },
    /* 12 - "ElseIfList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 15, 2, 37),
            Trans(0, 16, 2, 37),
            Trans(0, 19, 1, 36),
        ],
        k: 1,
    },
    /* 13 - "Expression" */
    LookaheadDFA {
        prod0: 20,
        transitions: &[],
        k: 0,
    },
    /* 14 - "ExpressionListRest" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 9, 2, 28), Trans(0, 14, 1, 27)],
        k: 1,
    },
    /* 15 - "ExpressionSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 7, 2, 22),
            Trans(0, 9, 2, 22),
            Trans(0, 11, 1, 21),
            Trans(0, 12, 1, 21),
            Trans(0, 13, 1, 21),
            Trans(0, 14, 2, 22),
            Trans(0, 15, 2, 22),
            Trans(0, 16, 2, 22),
            Trans(0, 18, 2, 22),
            Trans(0, 19, 2, 22),
            Trans(0, 21, 2, 22),
            Trans(0, 23, 2, 22),
            Trans(0, 24, 2, 22),
            Trans(0, 26, 2, 22),
            Trans(0, 32, 2, 22),
        ],
        k: 1,
    },
    /* 16 - "FPSection" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 29, 1, 61), Trans(0, 39, 2, 62)],
        k: 1,
    },
    /* 17 - "FPSectionRest" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 9, 2, 67), Trans(0, 24, 1, 66)],
        k: 1,
    },
    /* 18 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 8, 3, 6),
            Trans(0, 10, 4, 7),
            Trans(0, 38, 5, 8),
            Trans(0, 39, 1, 4),
            Trans(0, 40, 2, 5),
        ],
        k: 1,
    },
    /* 19 - "FieldList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 54),
            Trans(0, 24, 2, 54),
            Trans(0, 39, 1, 53),
        ],
        k: 1,
    },
    /* 20 - "FieldListRest" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 16, 2, 57), Trans(0, 24, 1, 56)],
        k: 1,
    },
    /* 21 - "FormalParameters" */
    LookaheadDFA {
        prod0: 63,
        transitions: &[],
        k: 0,
    },
    /* 22 - "FormalParametersSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 9, 1, 64), Trans(0, 29, 2, 65), Trans(0, 39, 2, 65)],
        k: 1,
    },
    /* 23 - "Ident" */
    LookaheadDFA {
        prod0: 103,
        transitions: &[],
        k: 0,
    },
    /* 24 - "IdentList" */
    LookaheadDFA {
        prod0: 49,
        transitions: &[],
        k: 0,
    },
    /* 25 - "IdentListRest" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 14, 1, 50), Trans(0, 27, 2, 51)],
        k: 1,
    },
    /* 26 - "IfPrefix" */
    LookaheadDFA {
        prod0: 35,
        transitions: &[],
        k: 0,
    },
    /* 27 - "IfStatement" */
    LookaheadDFA {
        prod0: 32,
        transitions: &[],
        k: 0,
    },
    /* 28 - "IfStatementSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 15, 1, 33), Trans(0, 16, 2, 34)],
        k: 1,
    },
    /* 29 - "Integer" */
    LookaheadDFA {
        prod0: 104,
        transitions: &[],
        k: 0,
    },
    /* 30 - "Module" */
    LookaheadDFA {
        prod0: 97,
        transitions: &[],
        k: 0,
    },
    /* 31 - "ModuleSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 16, 2, 99), Trans(0, 31, 1, 98)],
        k: 1,
    },
    /* 32 - "MulExpression" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 7, 2, 11),
            Trans(0, 9, 2, 11),
            Trans(0, 11, 2, 11),
            Trans(0, 12, 2, 11),
            Trans(0, 13, 2, 11),
            Trans(0, 14, 2, 11),
            Trans(0, 15, 2, 11),
            Trans(0, 16, 2, 11),
            Trans(0, 18, 2, 11),
            Trans(0, 19, 2, 11),
            Trans(0, 21, 2, 11),
            Trans(0, 23, 2, 11),
            Trans(0, 24, 2, 11),
            Trans(0, 26, 2, 11),
            Trans(0, 32, 2, 11),
            Trans(0, 36, 1, 10),
            Trans(0, 37, 2, 11),
        ],
        k: 1,
    },
    /* 33 - "MulOperator" */
    LookaheadDFA {
        prod0: 100,
        transitions: &[],
        k: 0,
    },
    /* 34 - "ProcedureBody" */
    LookaheadDFA {
        prod0: 71,
        transitions: &[],
        k: 0,
    },
    /* 35 - "ProcedureBodySuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 16, 2, 76), Trans(0, 32, 1, 75)],
        k: 1,
    },
    /* 36 - "ProcedureBodySuffix0" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 3, 74),
            Trans(0, 31, 1, 72),
            Trans(0, 32, 2, 73),
        ],
        k: 1,
    },
    /* 37 - "ProcedureCall" */
    LookaheadDFA {
        prod0: 29,
        transitions: &[],
        k: 0,
    },
    /* 38 - "ProcedureCallSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 8, 1, 30),
            Trans(0, 15, 2, 31),
            Trans(0, 16, 2, 31),
            Trans(0, 19, 2, 31),
            Trans(0, 23, 2, 31),
            Trans(0, 24, 2, 31),
            Trans(0, 32, 2, 31),
        ],
        k: 1,
    },
    /* 39 - "ProcedureDeclaration" */
    LookaheadDFA {
        prod0: 77,
        transitions: &[],
        k: 0,
    },
    /* 40 - "ProcedureDeclarationList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 90),
            Trans(0, 30, 1, 89),
            Trans(0, 31, 2, 90),
            Trans(0, 32, 2, 90),
        ],
        k: 1,
    },
    /* 41 - "ProcedureHeading" */
    LookaheadDFA {
        prod0: 68,
        transitions: &[],
        k: 0,
    },
    /* 42 - "ProcedureHeadingSuffix" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 8, 1, 69), Trans(0, 24, 2, 70)],
        k: 1,
    },
    /* 43 - "RecordType" */
    LookaheadDFA {
        prod0: 55,
        transitions: &[],
        k: 0,
    },
    /* 44 - "RelationOp" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 11, 1, 16),
            Trans(0, 12, 2, 17),
            Trans(0, 13, 1, 16),
        ],
        k: 1,
    },
    /* 45 - "RelationalOps" */
    LookaheadDFA {
        prod0: 18,
        transitions: &[],
        k: 0,
    },
    /* 46 - "RepeatStatement" */
    LookaheadDFA {
        prod0: 39,
        transitions: &[],
        k: 0,
    },
    /* 47 - "Selector" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 48 - "SelectorList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 1, 1),
            Trans(0, 6, 2, 2),
            Trans(0, 7, 3, 3),
            Trans(0, 9, 3, 3),
            Trans(0, 11, 3, 3),
            Trans(0, 12, 3, 3),
            Trans(0, 13, 3, 3),
            Trans(0, 14, 3, 3),
            Trans(0, 15, 3, 3),
            Trans(0, 16, 3, 3),
            Trans(0, 18, 3, 3),
            Trans(0, 19, 3, 3),
            Trans(0, 21, 3, 3),
            Trans(0, 23, 3, 3),
            Trans(0, 24, 3, 3),
            Trans(0, 26, 3, 3),
            Trans(0, 32, 3, 3),
            Trans(0, 36, 3, 3),
            Trans(0, 37, 3, 3),
        ],
        k: 1,
    },
    /* 49 - "SimpleExpression" */
    LookaheadDFA {
        prod0: 12,
        transitions: &[],
        k: 0,
    },
    /* 50 - "Statement" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 15, 10, -1),
            Trans(0, 16, 11, -1),
            Trans(0, 17, 4, -1),
            Trans(0, 19, 12, -1),
            Trans(0, 20, 6, -1),
            Trans(0, 22, 8, -1),
            Trans(0, 23, 12, -1),
            Trans(0, 24, 13, -1),
            Trans(0, 32, 12, -1),
            Trans(0, 39, 1, -1),
            Trans(1, 5, 2, 40),
            Trans(1, 6, 2, 40),
            Trans(1, 8, 3, 41),
            Trans(1, 11, 2, 40),
            Trans(1, 13, 2, 40),
            Trans(1, 15, 3, 41),
            Trans(1, 16, 3, 41),
            Trans(1, 19, 3, 41),
            Trans(1, 23, 3, 41),
            Trans(1, 24, 3, 41),
            Trans(1, 32, 3, 41),
            Trans(4, 8, 5, 42),
            Trans(4, 10, 5, 42),
            Trans(4, 38, 5, 42),
            Trans(4, 39, 5, 42),
            Trans(4, 40, 5, 42),
            Trans(6, 8, 7, 43),
            Trans(6, 10, 7, 43),
            Trans(6, 38, 7, 43),
            Trans(6, 39, 7, 43),
            Trans(6, 40, 7, 43),
            Trans(8, 17, 9, 44),
            Trans(8, 20, 9, 44),
            Trans(8, 22, 9, 44),
            Trans(8, 23, 9, 44),
            Trans(8, 24, 9, 44),
            Trans(8, 39, 9, 44),
            Trans(10, 16, 14, 45),
            Trans(10, 17, 14, 45),
            Trans(10, 20, 14, 45),
            Trans(10, 22, 14, 45),
            Trans(10, 24, 14, 45),
            Trans(10, 39, 14, 45),
            Trans(11, 15, 14, 45),
            Trans(11, 16, 14, 45),
            Trans(11, 19, 14, 45),
            Trans(11, 23, 14, 45),
            Trans(11, 24, 14, 45),
            Trans(11, 32, 14, 45),
            Trans(11, 39, 14, 45),
            Trans(12, 8, 14, 45),
            Trans(12, 10, 14, 45),
            Trans(12, 38, 14, 45),
            Trans(12, 39, 14, 45),
            Trans(12, 40, 14, 45),
            Trans(13, 15, 14, 45),
            Trans(13, 16, 14, 45),
            Trans(13, 17, 14, 45),
            Trans(13, 19, 14, 45),
            Trans(13, 20, 14, 45),
            Trans(13, 22, 14, 45),
            Trans(13, 23, 14, 45),
            Trans(13, 24, 14, 45),
            Trans(13, 32, 14, 45),
            Trans(13, 39, 14, 45),
        ],
        k: 2,
    },
    /* 51 - "StatementSequence" */
    LookaheadDFA {
        prod0: 46,
        transitions: &[],
        k: 0,
    },
    /* 52 - "StatementSequenceRest" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 15, 2, 48),
            Trans(0, 16, 2, 48),
            Trans(0, 19, 2, 48),
            Trans(0, 23, 2, 48),
            Trans(0, 24, 1, 47),
            Trans(0, 32, 2, 48),
        ],
        k: 1,
    },
    /* 53 - "Term" */
    LookaheadDFA {
        prod0: 9,
        transitions: &[],
        k: 0,
    },
    /* 54 - "Type" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 25, 2, 59),
            Trans(0, 28, 3, 60),
            Trans(0, 39, 1, 58),
        ],
        k: 1,
    },
    /* 55 - "TypeDecls" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 94),
            Trans(0, 29, 2, 94),
            Trans(0, 30, 2, 94),
            Trans(0, 31, 2, 94),
            Trans(0, 32, 2, 94),
            Trans(0, 39, 1, 93),
        ],
        k: 1,
    },
    /* 56 - "UnaryOp" */
    LookaheadDFA {
        prod0: 102,
        transitions: &[],
        k: 0,
    },
    /* 57 - "VarDecls" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 16, 2, 96),
            Trans(0, 30, 2, 96),
            Trans(0, 31, 2, 96),
            Trans(0, 32, 2, 96),
            Trans(0, 39, 1, 95),
        ],
        k: 1,
    },
    /* 58 - "WhileStatement" */
    LookaheadDFA {
        prod0: 38,
        transitions: &[],
        k: 0,
    },
];

pub const PRODUCTIONS: &[Production; 105] = &[
    // 0 - Selector: SelectorList;
    Production {
        lhs: 47,
        production: &[ParseType::N(48)],
    },
    // 1 - SelectorList: "\." Ident SelectorList;
    Production {
        lhs: 48,
        production: &[ParseType::N(48), ParseType::N(23), ParseType::T(5)],
    },
    // 2 - SelectorList: "\[" Expression "]" SelectorList;
    Production {
        lhs: 48,
        production: &[
            ParseType::N(48),
            ParseType::T(7),
            ParseType::N(13),
            ParseType::T(6),
        ],
    },
    // 3 - SelectorList: ;
    Production {
        lhs: 48,
        production: &[],
    },
    // 4 - Factor: Ident Selector;
    Production {
        lhs: 18,
        production: &[ParseType::N(47), ParseType::N(23)],
    },
    // 5 - Factor: Integer;
    Production {
        lhs: 18,
        production: &[ParseType::N(29)],
    },
    // 6 - Factor: "\(" Expression "\)";
    Production {
        lhs: 18,
        production: &[ParseType::T(9), ParseType::N(13), ParseType::T(8)],
    },
    // 7 - Factor: "~" Factor;
    Production {
        lhs: 18,
        production: &[ParseType::N(18), ParseType::T(10)],
    },
    // 8 - Factor: UnaryOp Factor;
    Production {
        lhs: 18,
        production: &[ParseType::N(18), ParseType::N(56)],
    },
    // 9 - Term: Factor MulExpression;
    Production {
        lhs: 53,
        production: &[ParseType::N(32), ParseType::N(18)],
    },
    // 10 - MulExpression: MulOperator Factor MulExpression;
    Production {
        lhs: 32,
        production: &[ParseType::N(32), ParseType::N(18), ParseType::N(33)],
    },
    // 11 - MulExpression: ;
    Production {
        lhs: 32,
        production: &[],
    },
    // 12 - SimpleExpression: Term AddExpression;
    Production {
        lhs: 49,
        production: &[ParseType::N(2), ParseType::N(53)],
    },
    // 13 - AddExpression: AddOperator Term AddExpression;
    Production {
        lhs: 2,
        production: &[ParseType::N(2), ParseType::N(53), ParseType::N(3)],
    },
    // 14 - AddExpression: ;
    Production {
        lhs: 2,
        production: &[],
    },
    // 15 - AssignOp: ":=";
    Production {
        lhs: 5,
        production: &[ParseType::T(11)],
    },
    // 16 - RelationOp: AssignOp;
    Production {
        lhs: 44,
        production: &[ParseType::N(5)],
    },
    // 17 - RelationOp: RelationalOps;
    Production {
        lhs: 44,
        production: &[ParseType::N(45)],
    },
    // 18 - RelationalOps: ">=|<=|\#|<|>";
    Production {
        lhs: 45,
        production: &[ParseType::T(12)],
    },
    // 19 - AssignOp: "=";
    Production {
        lhs: 5,
        production: &[ParseType::T(13)],
    },
    // 20 - Expression: SimpleExpression ExpressionSuffix;
    Production {
        lhs: 13,
        production: &[ParseType::N(15), ParseType::N(49)],
    },
    // 21 - ExpressionSuffix: RelationOp SimpleExpression;
    Production {
        lhs: 15,
        production: &[ParseType::N(49), ParseType::N(44)],
    },
    // 22 - ExpressionSuffix: ;
    Production {
        lhs: 15,
        production: &[],
    },
    // 23 - Assignment: Ident Selector AssignOp Expression;
    Production {
        lhs: 6,
        production: &[
            ParseType::N(13),
            ParseType::N(5),
            ParseType::N(47),
            ParseType::N(23),
        ],
    },
    // 24 - ActualParameters: "\(" ActualParametersSuffix;
    Production {
        lhs: 0,
        production: &[ParseType::N(1), ParseType::T(8)],
    },
    // 25 - ActualParametersSuffix: "\)";
    Production {
        lhs: 1,
        production: &[ParseType::T(9)],
    },
    // 26 - ActualParametersSuffix: Expression ExpressionListRest "\)";
    Production {
        lhs: 1,
        production: &[ParseType::T(9), ParseType::N(14), ParseType::N(13)],
    },
    // 27 - ExpressionListRest: "," Expression ExpressionListRest;
    Production {
        lhs: 14,
        production: &[ParseType::N(14), ParseType::N(13), ParseType::T(14)],
    },
    // 28 - ExpressionListRest: ;
    Production {
        lhs: 14,
        production: &[],
    },
    // 29 - ProcedureCall: Ident ProcedureCallSuffix;
    Production {
        lhs: 37,
        production: &[ParseType::N(38), ParseType::N(23)],
    },
    // 30 - ProcedureCallSuffix: ActualParameters;
    Production {
        lhs: 38,
        production: &[ParseType::N(0)],
    },
    // 31 - ProcedureCallSuffix: ;
    Production {
        lhs: 38,
        production: &[],
    },
    // 32 - IfStatement: IfPrefix IfStatementSuffix;
    Production {
        lhs: 27,
        production: &[ParseType::N(28), ParseType::N(26)],
    },
    // 33 - IfStatementSuffix: "ELSE" StatementSequence "END";
    Production {
        lhs: 28,
        production: &[ParseType::T(16), ParseType::N(51), ParseType::T(15)],
    },
    // 34 - IfStatementSuffix: "END";
    Production {
        lhs: 28,
        production: &[ParseType::T(16)],
    },
    // 35 - IfPrefix: "IF" Expression "THEN" StatementSequence ElseIfList;
    Production {
        lhs: 26,
        production: &[
            ParseType::N(12),
            ParseType::N(51),
            ParseType::T(18),
            ParseType::N(13),
            ParseType::T(17),
        ],
    },
    // 36 - ElseIfList: "ELSIF" Expression "THEN" StatementSequence ElseIfList;
    Production {
        lhs: 12,
        production: &[
            ParseType::N(12),
            ParseType::N(51),
            ParseType::T(18),
            ParseType::N(13),
            ParseType::T(19),
        ],
    },
    // 37 - ElseIfList: ;
    Production {
        lhs: 12,
        production: &[],
    },
    // 38 - WhileStatement: "WHILE" Expression "DO" StatementSequence "END";
    Production {
        lhs: 58,
        production: &[
            ParseType::T(16),
            ParseType::N(51),
            ParseType::T(21),
            ParseType::N(13),
            ParseType::T(20),
        ],
    },
    // 39 - RepeatStatement: "REPEAT" StatementSequence "UNTIL" Expression;
    Production {
        lhs: 46,
        production: &[
            ParseType::N(13),
            ParseType::T(23),
            ParseType::N(51),
            ParseType::T(22),
        ],
    },
    // 40 - Statement: Assignment;
    Production {
        lhs: 50,
        production: &[ParseType::N(6)],
    },
    // 41 - Statement: ProcedureCall;
    Production {
        lhs: 50,
        production: &[ParseType::N(37)],
    },
    // 42 - Statement: IfStatement;
    Production {
        lhs: 50,
        production: &[ParseType::N(27)],
    },
    // 43 - Statement: WhileStatement;
    Production {
        lhs: 50,
        production: &[ParseType::N(58)],
    },
    // 44 - Statement: RepeatStatement;
    Production {
        lhs: 50,
        production: &[ParseType::N(46)],
    },
    // 45 - Statement: ;
    Production {
        lhs: 50,
        production: &[],
    },
    // 46 - StatementSequence: Statement StatementSequenceRest;
    Production {
        lhs: 51,
        production: &[ParseType::N(52), ParseType::N(50)],
    },
    // 47 - StatementSequenceRest: ";" Statement StatementSequenceRest;
    Production {
        lhs: 52,
        production: &[ParseType::N(52), ParseType::N(50), ParseType::T(24)],
    },
    // 48 - StatementSequenceRest: ;
    Production {
        lhs: 52,
        production: &[],
    },
    // 49 - IdentList: Ident IdentListRest;
    Production {
        lhs: 24,
        production: &[ParseType::N(25), ParseType::N(23)],
    },
    // 50 - IdentListRest: "," Ident IdentListRest;
    Production {
        lhs: 25,
        production: &[ParseType::N(25), ParseType::N(23), ParseType::T(14)],
    },
    // 51 - IdentListRest: ;
    Production {
        lhs: 25,
        production: &[],
    },
    // 52 - ArrayType: "ARRAY" Expression "OF" Type;
    Production {
        lhs: 4,
        production: &[
            ParseType::N(54),
            ParseType::T(26),
            ParseType::N(13),
            ParseType::T(25),
        ],
    },
    // 53 - FieldList: IdentList ":" Type;
    Production {
        lhs: 19,
        production: &[ParseType::N(54), ParseType::T(27), ParseType::N(24)],
    },
    // 54 - FieldList: ;
    Production {
        lhs: 19,
        production: &[],
    },
    // 55 - RecordType: "RECORD" FieldList FieldListRest "END";
    Production {
        lhs: 43,
        production: &[
            ParseType::T(16),
            ParseType::N(20),
            ParseType::N(19),
            ParseType::T(28),
        ],
    },
    // 56 - FieldListRest: ";" FieldList FieldListRest;
    Production {
        lhs: 20,
        production: &[ParseType::N(20), ParseType::N(19), ParseType::T(24)],
    },
    // 57 - FieldListRest: ;
    Production {
        lhs: 20,
        production: &[],
    },
    // 58 - Type: Ident;
    Production {
        lhs: 54,
        production: &[ParseType::N(23)],
    },
    // 59 - Type: ArrayType;
    Production {
        lhs: 54,
        production: &[ParseType::N(4)],
    },
    // 60 - Type: RecordType;
    Production {
        lhs: 54,
        production: &[ParseType::N(43)],
    },
    // 61 - FPSection: "VAR" IdentList ":" Type;
    Production {
        lhs: 16,
        production: &[
            ParseType::N(54),
            ParseType::T(27),
            ParseType::N(24),
            ParseType::T(29),
        ],
    },
    // 62 - FPSection: IdentList ":" Type;
    Production {
        lhs: 16,
        production: &[ParseType::N(54), ParseType::T(27), ParseType::N(24)],
    },
    // 63 - FormalParameters: "\(" FormalParametersSuffix;
    Production {
        lhs: 21,
        production: &[ParseType::N(22), ParseType::T(8)],
    },
    // 64 - FormalParametersSuffix: "\)";
    Production {
        lhs: 22,
        production: &[ParseType::T(9)],
    },
    // 65 - FormalParametersSuffix: FPSection FPSectionRest "\)";
    Production {
        lhs: 22,
        production: &[ParseType::T(9), ParseType::N(17), ParseType::N(16)],
    },
    // 66 - FPSectionRest: ";" FPSection FPSectionRest;
    Production {
        lhs: 17,
        production: &[ParseType::N(17), ParseType::N(16), ParseType::T(24)],
    },
    // 67 - FPSectionRest: ;
    Production {
        lhs: 17,
        production: &[],
    },
    // 68 - ProcedureHeading: "PROCEDURE" Ident ProcedureHeadingSuffix;
    Production {
        lhs: 41,
        production: &[ParseType::N(42), ParseType::N(23), ParseType::T(30)],
    },
    // 69 - ProcedureHeadingSuffix: FormalParameters;
    Production {
        lhs: 42,
        production: &[ParseType::N(21)],
    },
    // 70 - ProcedureHeadingSuffix: ;
    Production {
        lhs: 42,
        production: &[],
    },
    // 71 - ProcedureBody: Declarations ProcedureBodySuffix0;
    Production {
        lhs: 34,
        production: &[ParseType::N(36), ParseType::N(8)],
    },
    // 72 - ProcedureBodySuffix0: "BEGIN" StatementSequence ProcedureBodySuffix;
    Production {
        lhs: 36,
        production: &[ParseType::N(35), ParseType::N(51), ParseType::T(31)],
    },
    // 73 - ProcedureBodySuffix0: "RETURN" Expression "END" Ident;
    Production {
        lhs: 36,
        production: &[
            ParseType::N(23),
            ParseType::T(16),
            ParseType::N(13),
            ParseType::T(32),
        ],
    },
    // 74 - ProcedureBodySuffix0: "END" Ident;
    Production {
        lhs: 36,
        production: &[ParseType::N(23), ParseType::T(16)],
    },
    // 75 - ProcedureBodySuffix: "RETURN" Expression "END" Ident;
    Production {
        lhs: 35,
        production: &[
            ParseType::N(23),
            ParseType::T(16),
            ParseType::N(13),
            ParseType::T(32),
        ],
    },
    // 76 - ProcedureBodySuffix: "END" Ident;
    Production {
        lhs: 35,
        production: &[ParseType::N(23), ParseType::T(16)],
    },
    // 77 - ProcedureDeclaration: ProcedureHeading ";" ProcedureBody;
    Production {
        lhs: 39,
        production: &[ParseType::N(34), ParseType::T(24), ParseType::N(41)],
    },
    // 78 - Declarations: "TYPE" TypeDecls DeclarationsSuffix1;
    Production {
        lhs: 8,
        production: &[ParseType::N(11), ParseType::N(55), ParseType::T(33)],
    },
    // 79 - Declarations: "CONST" ConstDecls DeclarationsSuffix0;
    Production {
        lhs: 8,
        production: &[ParseType::N(10), ParseType::N(7), ParseType::T(34)],
    },
    // 80 - DeclarationsSuffix1: "VAR" VarDecls ProcedureDeclarationList;
    Production {
        lhs: 11,
        production: &[ParseType::N(40), ParseType::N(57), ParseType::T(29)],
    },
    // 81 - DeclarationsSuffix1: ProcedureDeclarationList;
    Production {
        lhs: 11,
        production: &[ParseType::N(40)],
    },
    // 82 - Declarations: "VAR" VarDecls ProcedureDeclarationList;
    Production {
        lhs: 8,
        production: &[ParseType::N(40), ParseType::N(57), ParseType::T(29)],
    },
    // 83 - Declarations: ProcedureDeclarationList;
    Production {
        lhs: 8,
        production: &[ParseType::N(40)],
    },
    // 84 - DeclarationsSuffix0: "TYPE" TypeDecls DeclarationsSuffix;
    Production {
        lhs: 10,
        production: &[ParseType::N(9), ParseType::N(55), ParseType::T(33)],
    },
    // 85 - DeclarationsSuffix0: "VAR" VarDecls ProcedureDeclarationList;
    Production {
        lhs: 10,
        production: &[ParseType::N(40), ParseType::N(57), ParseType::T(29)],
    },
    // 86 - DeclarationsSuffix0: ProcedureDeclarationList;
    Production {
        lhs: 10,
        production: &[ParseType::N(40)],
    },
    // 87 - DeclarationsSuffix: "VAR" VarDecls ProcedureDeclarationList;
    Production {
        lhs: 9,
        production: &[ParseType::N(40), ParseType::N(57), ParseType::T(29)],
    },
    // 88 - DeclarationsSuffix: ProcedureDeclarationList;
    Production {
        lhs: 9,
        production: &[ParseType::N(40)],
    },
    // 89 - ProcedureDeclarationList: ProcedureDeclaration ";" ProcedureDeclarationList;
    Production {
        lhs: 40,
        production: &[ParseType::N(40), ParseType::T(24), ParseType::N(39)],
    },
    // 90 - ProcedureDeclarationList: ;
    Production {
        lhs: 40,
        production: &[],
    },
    // 91 - ConstDecls: Ident AssignOp Expression ";" ConstDecls;
    Production {
        lhs: 7,
        production: &[
            ParseType::N(7),
            ParseType::T(24),
            ParseType::N(13),
            ParseType::N(5),
            ParseType::N(23),
        ],
    },
    // 92 - ConstDecls: ;
    Production {
        lhs: 7,
        production: &[],
    },
    // 93 - TypeDecls: Ident AssignOp Type ";" TypeDecls;
    Production {
        lhs: 55,
        production: &[
            ParseType::N(55),
            ParseType::T(24),
            ParseType::N(54),
            ParseType::N(5),
            ParseType::N(23),
        ],
    },
    // 94 - TypeDecls: ;
    Production {
        lhs: 55,
        production: &[],
    },
    // 95 - VarDecls: IdentList ":" Type ";" VarDecls;
    Production {
        lhs: 57,
        production: &[
            ParseType::N(57),
            ParseType::T(24),
            ParseType::N(54),
            ParseType::T(27),
            ParseType::N(24),
        ],
    },
    // 96 - VarDecls: ;
    Production {
        lhs: 57,
        production: &[],
    },
    // 97 - Module: "MODULE" Ident ";" Declarations ModuleSuffix;
    Production {
        lhs: 30,
        production: &[
            ParseType::N(31),
            ParseType::N(8),
            ParseType::T(24),
            ParseType::N(23),
            ParseType::T(35),
        ],
    },
    // 98 - ModuleSuffix: "BEGIN" StatementSequence "END" Ident "\.";
    Production {
        lhs: 31,
        production: &[
            ParseType::T(5),
            ParseType::N(23),
            ParseType::T(16),
            ParseType::N(51),
            ParseType::T(31),
        ],
    },
    // 99 - ModuleSuffix: "END" Ident "\.";
    Production {
        lhs: 31,
        production: &[ParseType::T(5), ParseType::N(23), ParseType::T(16)],
    },
    // 100 - MulOperator: "\*|/|DIV|MOD|&";
    Production {
        lhs: 33,
        production: &[ParseType::T(36)],
    },
    // 101 - AddOperator: "\+|-|OR";
    Production {
        lhs: 3,
        production: &[ParseType::T(37)],
    },
    // 102 - UnaryOp: "\+|-";
    Production {
        lhs: 56,
        production: &[ParseType::T(38)],
    },
    // 103 - Ident: "[a-zA-Z][a-zA-Z0-9]*";
    Production {
        lhs: 23,
        production: &[ParseType::T(39)],
    },
    // 104 - Integer: "[0-9]+";
    Production {
        lhs: 29,
        production: &[ParseType::T(40)],
    },
];

pub fn parse<'t, T>(
    input: &'t str,
    file_name: T,
    user_actions: &mut Oberon0Grammar<'t>,
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
    user_actions: &mut Oberon0Grammar<'t>,
) -> Result<(), ParolError>
where
    ParolError: From<T::Error>,
{
    use oberon0_grammar_scanner::Oberon0GrammarScanner;
    let mut llk_parser = LLKParser::new(
        30,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    llk_parser.trim_parse_tree();
    let scanner = Oberon0GrammarScanner::new();
    // Initialize wrapper
    let mut user_actions = Oberon0GrammarAuto::new(user_actions);
    llk_parser.parse_into(
        tree_builder,
        TokenStream::new(
            input,
            file_name,
            scanner.scanner_impl.clone(),
            &Oberon0GrammarScanner::match_function,
            MAX_K,
        )
        .unwrap(),
        &mut user_actions,
    )
}
