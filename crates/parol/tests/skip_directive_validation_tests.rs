use parol::{ParolGrammar, ParolParserError, parse};
use parol_runtime::ParolError;
use std::path::Path;

#[test]
fn skip_requires_primary_non_terminal() {
    let grammar = r#"
%start Start
%skip NotPrimary
%%
Start: A;
A: "a";
B: "b";
NotPrimary: A B;
"#;

    let mut parol_grammar = ParolGrammar::new();
    let err = parse(
        grammar,
        Path::new("skip_requires_primary_non_terminal.par"),
        &mut parol_grammar,
    )
    .unwrap_err();

    let parser_err = match &err {
        ParolError::UserError(anyhow_err) => anyhow_err
            .downcast_ref::<ParolParserError>()
            .expect("expected ParolParserError"),
        other => panic!("unexpected error type: {other:?}"),
    };

    match parser_err {
        ParolParserError::InvalidTokenInTransition { token, .. } => {
            assert_eq!(token, "NotPrimary");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn skip_token_must_exist_in_target_scanner() {
    let grammar = r#"
%start Start
%on CommentStart %enter COMMENT

%scanner COMMENT {
    %skip Identifier
    %on CommentEnd %pop
}

%%

Start: Identifier;
Identifier: <INITIAL> "id";
CommentStart: <INITIAL> "/*";
CommentEnd: <COMMENT> "*/";
"#;

    let mut parol_grammar = ParolGrammar::new();
    let err = parse(
        grammar,
        Path::new("skip_token_must_exist_in_target_scanner.par"),
        &mut parol_grammar,
    )
    .unwrap_err();

    let parser_err = match &err {
        ParolError::UserError(anyhow_err) => anyhow_err
            .downcast_ref::<ParolParserError>()
            .expect("expected ParolParserError"),
        other => panic!("unexpected error type: {other:?}"),
    };

    match parser_err {
        ParolParserError::TokenIsNotInScanner { scanner, token, .. } => {
            assert_eq!(scanner, "COMMENT");
            assert_eq!(token, "Identifier");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
