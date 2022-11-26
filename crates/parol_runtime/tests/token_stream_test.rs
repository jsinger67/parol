//! This test is based on the scanner_states example of `parol`.
//! Scanner switching is tested and consistence of [miette::NamedSource] which is produced from
//! token stream and token spans is checked.

use miette::{NamedSource, SourceCode, SourceSpan};
use once_cell::sync::Lazy;
use parol_runtime::errors::FileSource;
use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};
use parol_runtime::lexer::{Token, TokenStream, Tokenizer};
use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

const INPUT: &str = r#"Id1
"1. String"
Id2
"2. \"String\t\" with \
escaped newline"

"3. String \nwith newline""#;

pub const TERMINALS: &[&str; 11] = &[
    /*  0 */ UNMATCHABLE_TOKEN,
    /*  1 */ UNMATCHABLE_TOKEN,
    /*  2 */ UNMATCHABLE_TOKEN,
    /*  3 */ UNMATCHABLE_TOKEN,
    /*  4 */ UNMATCHABLE_TOKEN,
    /*  5 */ r###"[a-zA-Z_]\w*"###,
    /*  6 */ r###"\u{5c}[\u{22}\u{5c}bfnt]"###,
    /*  7 */ r###"\u{5c}[\s^\n\r]*\r?\n"###,
    /*  8 */ r###"[^\u{22}\u{5c}]+"###,
    /*  9 */ r###"\u{22}"###,
    /* 10 */ ERROR_TOKEN,
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[usize; 2]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ NEW_LINE_TOKEN,
        /*  2 */ WHITESPACE_TOKEN,
        /*  3 */ r###"(//.*(\r\n|\r|\n|$))"###,
        /*  4 */ r###"((?ms)/\*.*?\*/)"###,
    ],
    &[5 /* Identifier */, 9 /* StringDelimiter */],
);

/* SCANNER_1: "String" */
const SCANNER_1: (&[&str; 5], &[usize; 4]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ UNMATCHABLE_TOKEN,
        /*  2 */ UNMATCHABLE_TOKEN,
        /*  3 */ UNMATCHABLE_TOKEN,
        /*  4 */ UNMATCHABLE_TOKEN,
    ],
    &[
        6, /* Escaped */
        7, /* EscapedLineEnd */
        8, /* NoneQuote */
        9, /* StringDelimiter */
    ],
);

const MAX_K: usize = 1;

static TOKENIZERS: Lazy<Vec<(&'static str, Tokenizer)>> = Lazy::new(|| {
    vec![
        (
            "INITIAL",
            Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap(),
        ),
        (
            "String",
            Tokenizer::build(TERMINALS, SCANNER_1.0, SCANNER_1.1).unwrap(),
        ),
    ]
});

#[test]
fn scanner_switch_and_named_source() {
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let stream = RefCell::new(TokenStream::new(INPUT, file_name, &TOKENIZERS, MAX_K).unwrap());

    assert_eq!(stream.borrow().current_scanner_index, 0);

    let mut prev_tok = Token::default();
    while !stream.borrow().all_input_consumed() {
        let tok = stream.borrow_mut().lookahead(0).unwrap();
        println!("{:?}", tok);

        // Check contents of named source
        let file_source: NamedSource = FileSource::from_stream(&stream.borrow()).into();
        let source_span: SourceSpan = (&tok).into();
        let span_contents = file_source.read_span(&source_span, 0, 0).unwrap();
        assert_eq!(span_contents.data(), tok.text().as_bytes());

        if tok.token_type == 9
        /* StringDelimiter */
        {
            let state = stream.borrow().current_scanner_index;
            if state == 1 {
                stream.borrow_mut().switch_scanner(0).unwrap();
            } else if state == 0 {
                stream.borrow_mut().switch_scanner(1).unwrap();
            }
        }

        stream.borrow_mut().consume().unwrap();
        prev_tok = tok;
    }

    assert_eq!(stream.borrow().current_scanner_index, 1);

    assert_eq!(prev_tok.text(), "\"");
    assert_eq!(prev_tok.location.line, 7);
    assert_eq!(prev_tok.location.column, 26);
}
