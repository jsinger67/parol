//! This test is based on the scanner_states example of `parol`.
//! Scanner switching is tested and token spans are checked.

use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};
use parol_runtime::once_cell::sync::Lazy;
use parol_runtime::{FileSource, ScannerConfig, TerminalIndex, Token, TokenStream, Tokenizer};
use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub const TERMINALS: &[(&str, Option<(bool, &str)>); 11] = &[
    /*  0 */ (UNMATCHABLE_TOKEN, None),
    /*  1 */ (UNMATCHABLE_TOKEN, None),
    /*  2 */ (UNMATCHABLE_TOKEN, None),
    /*  3 */ (UNMATCHABLE_TOKEN, None),
    /*  4 */ (UNMATCHABLE_TOKEN, None),
    /*  5 */ (r"[a-zA-Z_]\w*", None),
    /*  6 */ (r#"\["\\bfnt]"#, None),
    /*  7 */ (r"\[\s--\n\r]*\r?\n", None),
    /*  8 */ (r#"[^"\\]+"#, None),
    /*  9 */ (r#"""#, None),
    /* 10 */ (ERROR_TOKEN, None),
];

/* SCANNER_0: "INITIAL" */
const SCANNER_0: (&[&str; 5], &[TerminalIndex; 2]) = (
    &[
        /*  0 */ UNMATCHABLE_TOKEN,
        /*  1 */ NEW_LINE_TOKEN,
        /*  2 */ WHITESPACE_TOKEN,
        /*  3 */ r"//.*(\r\n|\r|\n)",
        /*  4 */ r"/\*([.\r\n--*]|\*[^/])*\*/",
    ],
    &[5 /* Identifier */, 9 /* StringDelimiter */],
);

/* SCANNER_1: "String" */
const SCANNER_1: (&[&str; 5], &[TerminalIndex; 4]) = (
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

static SCANNERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| {
    vec![
        ScannerConfig::new(
            "INITIAL",
            Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap(),
            &[],
        ),
        ScannerConfig::new(
            "String",
            Tokenizer::build(TERMINALS, SCANNER_1.0, SCANNER_1.1).unwrap(),
            &[],
        ),
    ]
});

const INPUT: &str = r#"Id1
"1. String"
Id2
"2. \"String\t\" with \
escaped newline"

"3. String \nwith newline""#;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn scanner_switch_and_named_source() {
    init();
    let file_name: Cow<'static, Path> = Cow::Owned(PathBuf::default());
    let stream = RefCell::new(TokenStream::new(INPUT, file_name, &SCANNERS, MAX_K, false).unwrap());
    eprintln!("'{INPUT:#?}'");

    assert_eq!(stream.borrow().current_scanner_index(), 0);
    let mut prev_tok = Token::default();
    let mut token_count = 0;
    while !stream.borrow().all_input_consumed() {
        let tok = stream.borrow_mut().lookahead(0).unwrap();
        println!("{:?}", tok);

        // Check contents of file source
        let file_source = FileSource::from_stream(&stream.borrow());
        let source_span: std::ops::Range<usize> = (&tok).into();
        let span_contents: &str = &file_source.input.as_str()[source_span.clone()];
        assert_eq!(span_contents, tok.text());
        assert_eq!(span_contents, &INPUT[source_span]);

        if tok.token_type == 9 {
            // StringDelimiter
            let state = stream.borrow().current_scanner_index();
            let new_state = if state == 0 { 1 } else { 0 };
            stream.borrow_mut().switch_scanner(new_state).unwrap();
            eprintln!("switched to scanner {new_state}");
        }

        // Consume the token which will update the iterator position where to reset the scanner
        // after clearing the token buffer.
        stream.borrow_mut().consume().unwrap();
        token_count += 1;

        if token_count > 30 {
            panic!("Too much tokens processed");
        }

        prev_tok = tok;
    }

    assert_eq!(stream.borrow().current_scanner_index(), 0);

    assert_eq!(prev_tok.text(), "\"");
    assert_eq!(prev_tok.location.start_line, 7);
    assert_eq!(prev_tok.location.start_column, 26);
}
