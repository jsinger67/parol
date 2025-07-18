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

use crate::boolean_grammar::BooleanGrammar;
use crate::boolean_grammar_trait::BooleanGrammarAuto;

pub const TERMINAL_NAMES: &[&str; 18] = &[
    /*  0 */ "EndOfInput",
    /*  1 */ "Newline",
    /*  2 */ "Whitespace",
    /*  3 */ "LineComment",
    /*  4 */ "BlockComment",
    /*  5 */ "AndOp",
    /*  6 */ "OrOp",
    /*  7 */ "XorOp",
    /*  8 */ "NorOp",
    /*  9 */ "NandOp",
    /* 10 */ "XnorOp",
    /* 11 */ "True",
    /* 12 */ "False",
    /* 13 */ "Not",
    /* 14 */ "Semicolon",
    /* 15 */ "LeftParenthesis",
    /* 16 */ "RightParenthesis",
    /* 17 */ "Error",
];

scanner! {
    BooleanGrammarScanner {
        mode INITIAL {
            token r"\r\n|\r|\n" => 1; // "Newline"
            token r"[\s--\r\n]+" => 2; // "Whitespace"
            token r"//.*(\r\n|\r|\n)?" => 3; // "LineComment"
            token r"\(\*([^*]|\*[^)])*\*\)" => 4; // "BlockComment"
            token r"[aA][nN][dD]" => 5; // "AndOp"
            token r"[oO][rR]" => 6; // "OrOp"
            token r"[xX][oO][rR]" => 7; // "XorOp"
            token r"[nN][oO][rR]" => 8; // "NorOp"
            token r"[nN][aA][nN][dD]" => 9; // "NandOp"
            token r"[xX][nN][oO][rR]" => 10; // "XnorOp"
            token r"[tT][rR][uU][eE]" => 11; // "True"
            token r"[fF][aA][lL][sS][eE]" => 12; // "False"
            token r"[nN][oO][tT]" => 13; // "Not"
            token r";" => 14; // "Semicolon"
            token r"\(" => 15; // "LeftParenthesis"
            token r"\)" => 16; // "RightParenthesis"
        }
    }
}

const MAX_K: usize = 2;

pub const NON_TERMINALS: &[&str; 25] = &[
    /*  0 */ "AndOp",
    /*  1 */ "BinaryOperator",
    /*  2 */ "Boolean",
    /*  3 */ "Expression",
    /*  4 */ "ExpressionList",
    /*  5 */ "Expressions",
    /*  6 */ "ExpressionsList",
    /*  7 */ "Factor",
    /*  8 */ "False",
    /*  9 */ "LeftParenthesis",
    /* 10 */ "NandOp",
    /* 11 */ "NorOp",
    /* 12 */ "Not",
    /* 13 */ "OrOp",
    /* 14 */ "Parenthesized",
    /* 15 */ "RightParenthesis",
    /* 16 */ "Semicolon",
    /* 17 */ "Term",
    /* 18 */ "TermOpt",
    /* 19 */ "TrailingSemicolon",
    /* 20 */ "TrailingSemicolonOpt",
    /* 21 */ "True",
    /* 22 */ "UnaryOperator",
    /* 23 */ "XnorOp",
    /* 24 */ "XorOp",
];

pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; 25] = &[
    /* 0 - "AndOp" */
    LookaheadDFA {
        prod0: 21,
        transitions: &[],
        k: 0,
    },
    /* 1 - "BinaryOperator" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 5, 1, 15),
            Trans(0, 6, 2, 16),
            Trans(0, 7, 3, 17),
            Trans(0, 8, 4, 18),
            Trans(0, 9, 5, 19),
            Trans(0, 10, 6, 20),
        ],
        k: 1,
    },
    /* 2 - "Boolean" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 11, 1, 12), Trans(0, 12, 2, 13)],
        k: 1,
    },
    /* 3 - "Expression" */
    LookaheadDFA {
        prod0: 6,
        transitions: &[],
        k: 0,
    },
    /* 4 - "ExpressionList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 2, 8),
            Trans(0, 5, 1, 7),
            Trans(0, 6, 1, 7),
            Trans(0, 7, 1, 7),
            Trans(0, 8, 1, 7),
            Trans(0, 9, 1, 7),
            Trans(0, 10, 1, 7),
            Trans(0, 14, 2, 8),
            Trans(0, 16, 2, 8),
        ],
        k: 1,
    },
    /* 5 - "Expressions" */
    LookaheadDFA {
        prod0: 0,
        transitions: &[],
        k: 0,
    },
    /* 6 - "ExpressionsList" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 0, 3, 2),
            Trans(0, 14, 1, -1),
            Trans(1, 0, 3, 2),
            Trans(1, 11, 2, 1),
            Trans(1, 12, 2, 1),
            Trans(1, 13, 2, 1),
            Trans(1, 15, 2, 1),
        ],
        k: 2,
    },
    /* 7 - "Factor" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 11, 1, 34),
            Trans(0, 12, 1, 34),
            Trans(0, 15, 2, 35),
        ],
        k: 1,
    },
    /* 8 - "False" */
    LookaheadDFA {
        prod0: 28,
        transitions: &[],
        k: 0,
    },
    /* 9 - "LeftParenthesis" */
    LookaheadDFA {
        prod0: 32,
        transitions: &[],
        k: 0,
    },
    /* 10 - "NandOp" */
    LookaheadDFA {
        prod0: 25,
        transitions: &[],
        k: 0,
    },
    /* 11 - "NorOp" */
    LookaheadDFA {
        prod0: 24,
        transitions: &[],
        k: 0,
    },
    /* 12 - "Not" */
    LookaheadDFA {
        prod0: 29,
        transitions: &[],
        k: 0,
    },
    /* 13 - "OrOp" */
    LookaheadDFA {
        prod0: 22,
        transitions: &[],
        k: 0,
    },
    /* 14 - "Parenthesized" */
    LookaheadDFA {
        prod0: 30,
        transitions: &[],
        k: 0,
    },
    /* 15 - "RightParenthesis" */
    LookaheadDFA {
        prod0: 33,
        transitions: &[],
        k: 0,
    },
    /* 16 - "Semicolon" */
    LookaheadDFA {
        prod0: 31,
        transitions: &[],
        k: 0,
    },
    /* 17 - "Term" */
    LookaheadDFA {
        prod0: 9,
        transitions: &[],
        k: 0,
    },
    /* 18 - "TermOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[
            Trans(0, 11, 2, 11),
            Trans(0, 12, 2, 11),
            Trans(0, 13, 1, 10),
            Trans(0, 15, 2, 11),
        ],
        k: 1,
    },
    /* 19 - "TrailingSemicolon" */
    LookaheadDFA {
        prod0: 3,
        transitions: &[],
        k: 0,
    },
    /* 20 - "TrailingSemicolonOpt" */
    LookaheadDFA {
        prod0: -1,
        transitions: &[Trans(0, 0, 2, 5), Trans(0, 14, 1, 4)],
        k: 1,
    },
    /* 21 - "True" */
    LookaheadDFA {
        prod0: 27,
        transitions: &[],
        k: 0,
    },
    /* 22 - "UnaryOperator" */
    LookaheadDFA {
        prod0: 14,
        transitions: &[],
        k: 0,
    },
    /* 23 - "XnorOp" */
    LookaheadDFA {
        prod0: 26,
        transitions: &[],
        k: 0,
    },
    /* 24 - "XorOp" */
    LookaheadDFA {
        prod0: 23,
        transitions: &[],
        k: 0,
    },
];

pub const PRODUCTIONS: &[Production; 36] = &[
    // 0 - Expressions: Expression ExpressionsList /* Vec */ TrailingSemicolon^ /* Clipped */;
    Production {
        lhs: 5,
        production: &[ParseType::N(19), ParseType::N(6), ParseType::N(3)],
    },
    // 1 - ExpressionsList: Semicolon^ /* Clipped */ Expression ExpressionsList;
    Production {
        lhs: 6,
        production: &[ParseType::N(6), ParseType::N(3), ParseType::N(16)],
    },
    // 2 - ExpressionsList: ;
    Production {
        lhs: 6,
        production: &[],
    },
    // 3 - TrailingSemicolon: TrailingSemicolonOpt /* Option */;
    Production {
        lhs: 19,
        production: &[ParseType::N(20)],
    },
    // 4 - TrailingSemicolonOpt: Semicolon;
    Production {
        lhs: 20,
        production: &[ParseType::N(16)],
    },
    // 5 - TrailingSemicolonOpt: ;
    Production {
        lhs: 20,
        production: &[],
    },
    // 6 - Expression: Term ExpressionList /* Vec */;
    Production {
        lhs: 3,
        production: &[ParseType::N(4), ParseType::N(17)],
    },
    // 7 - ExpressionList: BinaryOperator Term ExpressionList;
    Production {
        lhs: 4,
        production: &[ParseType::N(4), ParseType::N(17), ParseType::N(1)],
    },
    // 8 - ExpressionList: ;
    Production {
        lhs: 4,
        production: &[],
    },
    // 9 - Term: TermOpt /* Option */ Factor;
    Production {
        lhs: 17,
        production: &[ParseType::N(7), ParseType::N(18)],
    },
    // 10 - TermOpt: UnaryOperator;
    Production {
        lhs: 18,
        production: &[ParseType::N(22)],
    },
    // 11 - TermOpt: ;
    Production {
        lhs: 18,
        production: &[],
    },
    // 12 - Boolean: True;
    Production {
        lhs: 2,
        production: &[ParseType::N(21)],
    },
    // 13 - Boolean: False;
    Production {
        lhs: 2,
        production: &[ParseType::N(8)],
    },
    // 14 - UnaryOperator: Not;
    Production {
        lhs: 22,
        production: &[ParseType::N(12)],
    },
    // 15 - BinaryOperator: AndOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(0)],
    },
    // 16 - BinaryOperator: OrOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(13)],
    },
    // 17 - BinaryOperator: XorOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(24)],
    },
    // 18 - BinaryOperator: NorOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(11)],
    },
    // 19 - BinaryOperator: NandOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(10)],
    },
    // 20 - BinaryOperator: XnorOp;
    Production {
        lhs: 1,
        production: &[ParseType::N(23)],
    },
    // 21 - AndOp: "[aA][nN][dD]"^ /* Clipped */;
    Production {
        lhs: 0,
        production: &[ParseType::T(5)],
    },
    // 22 - OrOp: "[oO][rR]"^ /* Clipped */;
    Production {
        lhs: 13,
        production: &[ParseType::T(6)],
    },
    // 23 - XorOp: "[xX][oO][rR]"^ /* Clipped */;
    Production {
        lhs: 24,
        production: &[ParseType::T(7)],
    },
    // 24 - NorOp: "[nN][oO][rR]"^ /* Clipped */;
    Production {
        lhs: 11,
        production: &[ParseType::T(8)],
    },
    // 25 - NandOp: "[nN][aA][nN][dD]"^ /* Clipped */;
    Production {
        lhs: 10,
        production: &[ParseType::T(9)],
    },
    // 26 - XnorOp: "[xX][nN][oO][rR]"^ /* Clipped */;
    Production {
        lhs: 23,
        production: &[ParseType::T(10)],
    },
    // 27 - True: "[tT][rR][uU][eE]"^ /* Clipped */;
    Production {
        lhs: 21,
        production: &[ParseType::T(11)],
    },
    // 28 - False: "[fF][aA][lL][sS][eE]"^ /* Clipped */;
    Production {
        lhs: 8,
        production: &[ParseType::T(12)],
    },
    // 29 - Not: "[nN][oO][tT]"^ /* Clipped */;
    Production {
        lhs: 12,
        production: &[ParseType::T(13)],
    },
    // 30 - Parenthesized: LeftParenthesis^ /* Clipped */ Expression RightParenthesis^ /* Clipped */;
    Production {
        lhs: 14,
        production: &[ParseType::N(15), ParseType::N(3), ParseType::N(9)],
    },
    // 31 - Semicolon: ';';
    Production {
        lhs: 16,
        production: &[ParseType::T(14)],
    },
    // 32 - LeftParenthesis: '(';
    Production {
        lhs: 9,
        production: &[ParseType::T(15)],
    },
    // 33 - RightParenthesis: ')';
    Production {
        lhs: 15,
        production: &[ParseType::T(16)],
    },
    // 34 - Factor: Boolean;
    Production {
        lhs: 7,
        production: &[ParseType::N(2)],
    },
    // 35 - Factor: Parenthesized;
    Production {
        lhs: 7,
        production: &[ParseType::N(14)],
    },
];

pub fn parse<'t, T>(
    input: &'t str,
    file_name: T,
    user_actions: &mut BooleanGrammar<'t>,
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
    user_actions: &mut BooleanGrammar<'t>,
) -> Result<(), ParolError>
where
    ParolError: From<T::Error>,
{
    use boolean_grammar_scanner::BooleanGrammarScanner;
    let mut llk_parser = LLKParser::new(
        5,
        LOOKAHEAD_AUTOMATA,
        PRODUCTIONS,
        TERMINAL_NAMES,
        NON_TERMINALS,
    );
    let scanner = BooleanGrammarScanner::new();
    // Initialize wrapper
    let mut user_actions = BooleanGrammarAuto::new(user_actions);
    llk_parser.parse_into(
        tree_builder,
        TokenStream::new(
            input,
            file_name,
            scanner.scanner_impl.clone(),
            &BooleanGrammarScanner::match_function,
            MAX_K,
        )
        .unwrap(),
        &mut user_actions,
    )
}
