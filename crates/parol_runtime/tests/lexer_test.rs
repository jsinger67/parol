use log::trace;
use parol_runtime::LocationBuilder;
use parol_runtime::{Token, TokenStream};
use scnr2::scanner;
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

scanner!(
    ParolScanner {
        mode INITIAL {
            token r"\r\n|\r|\n" => 1; // token::NEW_LINE
            token r"[\s--\r\n]+" => 2; // token::WHITESPACE
            token r"//.*" => 3; // token::LINE_COMMENT
            token r"/\*([^*]|\*[^/])*\*/" => 4; // token::BLOCK_COMMENT
            token r"%start" => 5; // token::FIRST_USER_TOKEN
            token r"%%" => 6;
            token r":" => 7;
            token r";" => 8;
            token r"[a-zA-Z_]\w*" => 9; // Identifier
            token r#""(\\.|[^"])*""# => 10; // String
            token r"." => 11; // token::ERROR_TOKEN
        }
    }
);
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn lexer_token_production() {
    init();
    let k = 3;
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let scanner = parol_scanner::ParolScanner::new();
    let token_stream = RefCell::new(
        TokenStream::new(
            PAROL_CFG_1,
            file_name,
            scanner.scanner_impl.clone(),
            &parol_scanner::ParolScanner::match_function,
            k,
        )
        .unwrap(),
    );
    let mut tok = Token::default();
    let mut token_count = 0;
    while !token_stream.borrow().all_input_consumed() {
        print_skip_tokens(&token_stream);
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
                .start(541)
                .end(542)
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
                .file_name(token_stream.borrow().file_name.clone())
                .build()
                .unwrap()
        ),
        *token_stream.borrow().tokens.non_skip_token_at(0).unwrap()
    );
    assert_eq!(
        Token::eoi(82).with_location(
            LocationBuilder::default()
                .file_name(token_stream.borrow().file_name.clone())
                .build()
                .unwrap()
        ),
        *token_stream.borrow().tokens.non_skip_token_at(1).unwrap()
    );
}

fn print_skip_tokens<F: Fn(char) -> Option<usize> + Clone>(
    token_stream: &RefCell<TokenStream<'_, F>>,
) {
    // Print the skip tokens
    token_stream
        .borrow_mut()
        .take_skip_tokens()
        .into_iter()
        .for_each(|t| println!("Skipped: {:?}", t));
}

#[test]
#[should_panic(expected = "LookaheadExceedsMaximum")]
fn lookahead_must_fail() {
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let scanner = parol_scanner::ParolScanner::new();
    let mut token_stream = TokenStream::new(
        PAROL_CFG_1,
        file_name,
        scanner.scanner_impl.clone(),
        &parol_scanner::ParolScanner::match_function,
        1,
    )
    .unwrap();
    let _tok = token_stream.lookahead(1).unwrap();
}

#[test]
fn lookahead_beyond_buffer_must_not_fail() {
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let scanner = parol_scanner::ParolScanner::new();
    let token_stream = RefCell::new(
        TokenStream::new(
            PAROL_CFG_1,
            file_name,
            scanner.scanner_impl.clone(),
            &parol_scanner::ParolScanner::match_function,
            1,
        )
        .unwrap(),
    );
    while !token_stream.borrow().all_input_consumed() {
        print_skip_tokens(&token_stream);
        if token_stream.borrow_mut().consume().is_ok() {
            let tok = token_stream.borrow_mut().lookahead(0).unwrap();
            println!("{:?}", tok);
        }
    }
    print_skip_tokens(&token_stream);
    // Consume the EOI token
    println!("{:?}", token_stream.borrow_mut().consume().unwrap());
    // This must not fail because EOI tokens are generated on the fly
    println!("{:?}", token_stream.borrow_mut().lookahead(0).unwrap());
}
