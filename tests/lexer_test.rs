#[macro_use]
extern crate lazy_static;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};
use parol_runtime::lexer::{Token, TokenStream, Tokenizer};
use std::cell::RefCell;

const PAROL_CFG_1: &'static str = r#"%start Grammar
%%

// Test grammar
// A simple grammar

/*  0 */ Grammar: Prolog GrammarDefinition;
/*  1 */ Prolog: "%start" Identifier;
/*  2 */ GrammarDefinition: "%%" Productions;
/*  3 */ Productions: Production Productions;
/*  4 */ Productions: Production;
/*  5 */ Production: Identifier ":" Symbols ";";
/*  6 */ Production: Identifier ":" ";";
/*  7 */ Symbols: Symbol Symbols;
/*  8 */ Symbols: Symbol;
/*  9 */ Symbol: Identifier;
/* 10 */ Symbol: String;
/* 11 */ Identifier: "[a-zA-Z_]\\w*";
/* 12 */ String: "\"([^\\]|(\\.))*?\"";


"#;

const PAROL_CFG_1_TOKENS: &[&str; 12] = &[
    /*  0 */ UNMATCHABLE_TOKEN, // token::EOI
    /*  1 */ NEW_LINE_TOKEN, // token::NEW_LINE
    /*  2 */ WHITESPACE_TOKEN, // token::WHITESPACE
    /*  3 */ r###"//.*"###, // token::LINE_COMMENT
    /*  4 */ r###"(?m)(/\*(.|[\r\n])*?\*/)(?-m)"###, // token::BLOCK_COMMENT
    /*  5 */ r###"%start"###, // token::FIRST_USER_TOKEN
    /*  6 */ r###"%%"###,
    /*  7 */ r###":"###,
    /*  8 */ r###";"###,
    /*  9 */ r###"[a-zA-Z_]\w*"###,
    /* 10 */ r###""([^\\]|(\\.))*?""###,
    /* 11 */ ERROR_TOKEN,
];

lazy_static! {
    static ref TOKENIZER: Tokenizer = Tokenizer::build(PAROL_CFG_1_TOKENS).unwrap();
}

#[test]
fn tokenizer_test() {
    assert_eq!(11, TOKENIZER.error_token_type, "Error token index is wrong");
}

#[test]
fn lexer_token_production() {
    let token_stream = RefCell::new(TokenStream::new(PAROL_CFG_1, "No file".to_owned(), &TOKENIZER, 1).unwrap());
    while !token_stream.borrow().all_input_consumed() {
        let tok = token_stream.borrow_mut().owned_lookahead(0).unwrap();
        print!("{:?}", tok);
        token_stream.borrow_mut().consume(1).unwrap();
    }
    assert_eq!(66, token_stream.borrow().tokens.len());
    assert_eq!(
        Token {
            symbol: ";",
            token_type: 8,
            line: 19,
            column: 39,
            length: 1
        },
        token_stream.borrow().tokens[64]
    );
    assert_eq!(Token::eoi(), token_stream.borrow().tokens[65]);
}

#[test]
#[should_panic(expected = "Lookahead exceeds its maximum")]
fn lookahead_must_fail() {
    let mut token_stream = TokenStream::new(PAROL_CFG_1, "No file".to_owned(), &TOKENIZER, 1).unwrap();
    let _tok = token_stream.lookahead(2).unwrap();
}

#[test]
#[should_panic(expected = "Lookahead exceeds token buffer length")]
fn lookahead_beyond_buffer_must_fail() {
    let token_stream = RefCell::new(TokenStream::new(PAROL_CFG_1, "No file".to_owned(), &TOKENIZER, 1).unwrap());
    while !token_stream.borrow().all_input_consumed() {
        if token_stream.borrow_mut().consume(1).is_ok() {
            let tok = token_stream.borrow_mut().owned_lookahead(0).unwrap();
            println!("{:?}", tok);
        }
    }
    token_stream.borrow_mut().owned_lookahead(1).unwrap();
}
