#[macro_use]
extern crate lazy_static;

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};
use parol_runtime::lexer::{Location, Token, TokenStream, Tokenizer};
use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

const PAROL_CFG_1: &str = r#"%start Grammar
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

const TERMINALS: &[&str; 12] = &[
    /*  0 */ UNMATCHABLE_TOKEN, // token::EOI
    /*  1 */ UNMATCHABLE_TOKEN, // token::NEW_LINE
    /*  2 */ UNMATCHABLE_TOKEN, // token::WHITESPACE
    /*  3 */ UNMATCHABLE_TOKEN, // token::LINE_COMMENT
    /*  4 */ UNMATCHABLE_TOKEN, // token::BLOCK_COMMENT
    /*  5 */ r###"%start"###, // token::FIRST_USER_TOKEN
    /*  6 */ r###"%%"###,
    /*  7 */ r###":"###,
    /*  8 */ r###";"###,
    /*  9 */ r###"[a-zA-Z_]\w*"###,
    /* 10 */ r###""([^\\]|(\\.))*?""###,
    /* 11 */ ERROR_TOKEN,
];

const SCANNER_0: &[&str; 5] = &[
    /*  0 */ UNMATCHABLE_TOKEN, // token::EOI
    /*  1 */ NEW_LINE_TOKEN, // token::NEW_LINE
    /*  2 */ WHITESPACE_TOKEN, // token::WHITESPACE
    /*  3 */ r###"//.*"###, // token::LINE_COMMENT
    /*  4 */ r###"(?m)(/\*(.|[\r\n])*?\*/)(?-m)"###, // token::BLOCK_COMMENT
];

lazy_static! {
    static ref TOKENIZERS: Vec<(&'static str, Tokenizer)> = vec![(
        "INITIAL",
        Tokenizer::build(TERMINALS, SCANNER_0, &[5, 6, 7, 8, 9, 10]).unwrap()
    ),];
}

#[test]
fn tokenizer_test() {
    assert_eq!(
        11, TOKENIZERS[0].1.error_token_type,
        "Error token index is wrong"
    );
}

#[test]
fn lexer_token_production() {
    let k = 3;
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let token_stream =
        RefCell::new(TokenStream::new(PAROL_CFG_1, file_name, &TOKENIZERS, k).unwrap());
    let mut tok = Token::default();
    while !token_stream.borrow().all_input_consumed() {
        tok = token_stream.borrow_mut().lookahead(0).unwrap();
        print!("{:?}", tok);
        token_stream.borrow_mut().consume().unwrap();
    }
    assert_eq!(k, token_stream.borrow().tokens.len());
    assert_eq!(
        Token::with(
            ";",
            8,
            Location::with(19, 39, 1, 0, 545, token_stream.borrow().file_name.clone())
        ),
        tok
    );
    assert_eq!(Token::eoi(), token_stream.borrow().tokens[0]);
}

#[test]
#[should_panic(expected = "Lookahead exceeds its maximum")]
fn lookahead_must_fail() {
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let mut token_stream = TokenStream::new(PAROL_CFG_1, file_name, &TOKENIZERS, 1).unwrap();
    let _tok = token_stream.lookahead(2).unwrap();
}

#[test]
#[should_panic(expected = "Lookahead exceeds token buffer length")]
fn lookahead_beyond_buffer_must_fail() {
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let token_stream =
        RefCell::new(TokenStream::new(PAROL_CFG_1, file_name, &TOKENIZERS, 1).unwrap());
    while !token_stream.borrow().all_input_consumed() {
        if token_stream.borrow_mut().consume().is_ok() {
            let tok = token_stream.borrow_mut().lookahead(0).unwrap();
            println!("{:?}", tok);
        }
    }
    token_stream.borrow_mut().lookahead(1).unwrap();
}
