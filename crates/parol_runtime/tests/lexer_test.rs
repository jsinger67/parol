use log::trace;
use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};
use parol_runtime::once_cell::sync::Lazy;
use parol_runtime::{LocationBuilder, ScannerConfig};
use parol_runtime::{Token, TokenStream, Tokenizer};
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
/* 12 */ String: "\"(\\.|[^\"])*\"";


"#;

const TERMINALS: &[(&str, Option<(bool, &str)>); 12] = &[
    /*  0 */ (UNMATCHABLE_TOKEN, None), // token::EOI
    /*  1 */ (UNMATCHABLE_TOKEN, None), // token::NEW_LINE
    /*  2 */ (UNMATCHABLE_TOKEN, None), // token::WHITESPACE
    /*  3 */ (UNMATCHABLE_TOKEN, None), // token::LINE_COMMENT
    /*  4 */ (UNMATCHABLE_TOKEN, None), // token::BLOCK_COMMENT
    /*  5 */ (r###"%start"###, None), // token::FIRST_USER_TOKEN
    /*  6 */ (r###"%%"###, None),
    /*  7 */ (r###":"###, None),
    /*  8 */ (r###";"###, None),
    /*  9 */ (r"[a-zA-Z_]\w*", None),
    /* 10 */ (r#""(\\.|[^"])*""#, None),
    /* 11 */ (ERROR_TOKEN, None),
];

const SCANNER_0: &[&str; 5] = &[
    /*  0 */ UNMATCHABLE_TOKEN, // token::EOI
    /*  1 */ NEW_LINE_TOKEN, // token::NEW_LINE
    /*  2 */ WHITESPACE_TOKEN, // token::WHITESPACE
    /*  3 */ r###"//.*"###, // token::LINE_COMMENT
    /*  4 */ r###"/\*([.--*]|\*[^/])*\*/"###, // token::BLOCK_COMMENT
];

static TOKENIZERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| {
    vec![ScannerConfig {
        name: "INITIAL",
        tokenizer: Tokenizer::build(TERMINALS, SCANNER_0, &[5, 6, 7, 8, 9, 10]).unwrap(),
        transitions: &[],
    }]
});

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn tokenizer_test() {
    assert_eq!(
        11, TOKENIZERS[0].tokenizer.error_token_type,
        "Error token index is wrong"
    );
}

#[test]
fn lexer_token_production() {
    init();
    let k = 3;
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let token_stream =
        RefCell::new(TokenStream::new(PAROL_CFG_1, file_name, &TOKENIZERS, k).unwrap());
    let mut tok = Token::default();
    let mut token_count = 0;
    while !token_stream.borrow().all_input_consumed() {
        tok = token_stream.borrow_mut().consume().unwrap();
        println!("{:w$}: {:?}", token_count, tok, w = 3);
        token_count += 1;
    }
    trace!("Token buffer: {:?}", token_stream.borrow().tokens);
    assert_eq!(65, token_count);
    assert_eq!(k, token_stream.borrow().tokens.len());
    assert_eq!(
        Token::with(
            ";",
            8,
            LocationBuilder::default()
                .start_line(19)
                .start_column(36)
                .end_line(19)
                .end_column(37)
                .length(1)
                .offset(542)
                .file_name(token_stream.borrow().file_name.clone())
                .build()
                .unwrap(),
            79
        ),
        tok
    );
    assert_eq!(
        Token::eoi(81).with_location(
            LocationBuilder::default()
                .start_line(21)
                .start_column(1)
                .end_line(21)
                .end_column(3)
                .length(1)
                .offset(545)
                .file_name(token_stream.borrow().file_name.clone())
                .build()
                .unwrap()
        ),
        token_stream.borrow().tokens[0]
    );
    assert_eq!(
        Token::eoi(82).with_location(
            LocationBuilder::default()
                .start_line(21)
                .start_column(1)
                .end_line(21)
                .end_column(4)
                .length(1)
                .offset(545)
                .file_name(token_stream.borrow().file_name.clone())
                .build()
                .unwrap()
        ),
        token_stream.borrow().tokens[1]
    );
}

#[test]
#[should_panic(expected = "LookaheadExceedsMaximum")]
fn lookahead_must_fail() {
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let mut token_stream = TokenStream::new(PAROL_CFG_1, file_name, &TOKENIZERS, 1).unwrap();
    let _tok = token_stream.lookahead(1).unwrap();
}

#[test]
#[should_panic(expected = "LookaheadExceedsTokenBufferLength")]
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
    // Consume the EOI token
    println!("{:?}", token_stream.borrow_mut().consume().unwrap());
    // This must fail
    println!("{:?}", token_stream.borrow_mut().lookahead(0).unwrap());
}
