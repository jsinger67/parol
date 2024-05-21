use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, Range, Uri,
};
use parol::{
    analysis::lalr1_parse_table::LRResolvedConflict, GrammarAnalysisError, ParolParserError,
};
use parol_runtime::{ParolError, ParserError, SyntaxError};
use std::error::Error;

use crate::{
    document_state::{DocumentState, LocatedDocumentState},
    server::Server,
    utils::{location_to_location, location_to_range},
};

#[derive(Debug)]
pub struct Diagnostics {}

impl Diagnostics {
    pub(crate) fn to_diagnostics(
        uri: &Uri,
        document_state: &DocumentState,
        err: anyhow::Error,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut range = Range::default();
        let source = Some("parol-ls".to_string());
        let mut related_information = vec![];
        let message = err.to_string();

        // Extract additional information from certain errors
        if let Some(e) = err.downcast_ref::<ParolError>() {
            match e {
                ParolError::ParserError(err) => {
                    if let ParserError::SyntaxErrors { entries } = err {
                        extract_syntax_errors(entries, &mut diagnostics, uri);
                        return diagnostics;
                    }
                }
                ParolError::LexerError(_) => (),
                ParolError::UserError(err) => {
                    if let Some(e) = err.downcast_ref::<GrammarAnalysisError>() {
                        let located_document_state = LocatedDocumentState::new(uri, document_state);
                        extract_grammar_analysis_error(
                            e,
                            &located_document_state,
                            &mut diagnostics,
                            uri,
                        );
                        return diagnostics;
                    } else if let Some(e) = err.downcast_ref::<ParolParserError>() {
                        let located_document_state = LocatedDocumentState::new(uri, document_state);
                        extract_parser_error(
                            e,
                            &located_document_state,
                            &mut range,
                            &mut related_information,
                        );
                    }
                }
            }
        }

        let diagnostic = Diagnostic {
            source,
            code: None,
            range,
            message,
            related_information: if related_information.is_empty() {
                None
            } else {
                Some(related_information)
            },
            ..Default::default()
        };
        diagnostics.push(diagnostic);
        diagnostics
    }

    pub(crate) fn to_resolved_confict_warning(
        uri: &Uri,
        warnings: Vec<LRResolvedConflict>,
    ) -> Diagnostic {
        let range = Range::default();
        let source = Some("parol-ls".to_string());
        let message = format!("{} automatically resolved conflicts", warnings.len());

        let related_information: Option<Vec<DiagnosticRelatedInformation>> =
            Some(warnings.into_iter().fold(vec![], |mut acc, w| {
                acc.push(DiagnosticRelatedInformation {
                    location: Location::new(uri.clone(), Range::default()),
                    message: w.to_string(),
                });
                acc
            }));

        Diagnostic {
            source,
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
            range,
            message,
            related_information,
            ..Default::default()
        }
    }
}

fn extract_syntax_errors(entries: &[SyntaxError], diagnostics: &mut Vec<Diagnostic>, uri: &Uri) {
    for e in entries {
        let range = if e.unexpected_tokens.is_empty() {
            location_to_range(&e.error_location)
        } else {
            e.unexpected_tokens
                .iter()
                .fold(Range::default(), |mut acc, un| {
                    if acc.start == lsp_types::Position::default() {
                        acc.start = lsp_types::Position {
                            line: un.token.start_line,
                            character: un.token.start_column,
                        };
                        acc.end = lsp_types::Position {
                            line: un.token.end_line,
                            character: un.token.end_column,
                        };
                    }
                    acc
                })
        };
        let mut related_information: Vec<DiagnosticRelatedInformation> = vec![];
        for u in &e.unexpected_tokens {
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(&u.token, uri),
                message: format!("Unexpected token {}", u),
            })
        }
        related_information.push(DiagnosticRelatedInformation {
            location: location_to_location(&e.unexpected_tokens[0].token, uri),
            message: format!("Expecting {}", e.expected_tokens),
        });
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(lsp_types::NumberOrString::String(
                "parol_runtime::parser::syntax_error".to_owned(),
            )),
            code_description: None,
            source: e.source.as_ref().map(|s| s.to_string()),
            message: e.cause.clone(),
            related_information: Some(related_information),
            tags: None,
            data: None,
        });
    }
}

fn extract_grammar_analysis_error(
    error: &GrammarAnalysisError,
    located_document_state: &LocatedDocumentState,
    diagnostics: &mut Vec<Diagnostic>,
    uri: &Uri,
) {
    match error {
        GrammarAnalysisError::LeftRecursion { recursions } => {
            for rec in recursions.iter() {
                if let Some(non_terminals) = Server::find_non_terminal_definitions(
                    located_document_state.document_state,
                    &rec.name,
                ) {
                    for rang in non_terminals {
                        let (range, message) =
                            (rang, format!("Left-recursive non-terminal: {}", rec.name));
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "parol::analysis::left_recursion".to_owned(),
                            )),
                            code_description: None,
                            source: error.source().map(|s| s.to_string()),
                            message,
                            related_information: Some(vec![DiagnosticRelatedInformation {
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                message: format!(
                                    "Left-recursions are not allowed in LL grammars: {}",
                                    rec.name
                                ),
                            }]),
                            data: None,
                            tags: None,
                        });
                    }
                }
            }
        }
        GrammarAnalysisError::UnreachableNonTerminals { non_terminals }
        | GrammarAnalysisError::NonProductiveNonTerminals { non_terminals } => {
            let (error_spec, code) =
                if let GrammarAnalysisError::UnreachableNonTerminals { .. } = error {
                    (
                        "Unreachable",
                        Some(lsp_types::NumberOrString::String(
                            "parol::analysis::unreachable_non_terminal".to_owned(),
                        )),
                    )
                } else {
                    (
                        "Nonproductive",
                        Some(lsp_types::NumberOrString::String(
                            "parol::analysis::nonproductive_non_terminal".to_owned(),
                        )),
                    )
                };
            for nt in non_terminals {
                if let Some(non_terminals) = Server::find_non_terminal_definitions(
                    located_document_state.document_state,
                    &nt.hint,
                ) {
                    for rng in non_terminals {
                        let (range, message) =
                            (rng, format!("{} non-terminal: {}", error_spec, nt.hint));
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: code.clone(),
                            code_description: None,
                            source: error.source().map(|s| s.to_string()),
                            message,
                            related_information: Some(vec![DiagnosticRelatedInformation {
                                location: Location {
                                    uri: uri.clone(),
                                    range,
                                },
                                message: format!("{} non-terminals are not allowed", error_spec),
                            }]),
                            data: None,
                            tags: None,
                        });
                    }
                }
            }
        }
        GrammarAnalysisError::MaxKExceeded { max_k: _ } => {
            // No additional information attached
        }
        GrammarAnalysisError::LALR1ParseTableConstructionFailed { conflict } => {
            diagnostics.push(Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String(
                    "parol::analysis::lalr1_parse_table_conflict".to_owned(),
                )),
                code_description: None,
                source: error.source().map(|s| s.to_string()),
                message: format!("{:?}", conflict),
                related_information: Some(vec![DiagnosticRelatedInformation {
                    location: Location {
                        uri: uri.clone(),
                        range: Range::default(),
                    },
                    message: "Conflict was not automatically resolved".to_owned(),
                }]),
                data: None,
                tags: None,
            });
        }
    }
}

fn extract_parser_error(
    error: &ParolParserError,
    located_document_state: &LocatedDocumentState,
    range: &mut Range,
    related_information: &mut Vec<DiagnosticRelatedInformation>,
) {
    match error {
        ParolParserError::UnknownScanner { name, token, .. } => {
            *range = location_to_range(token);
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(token, located_document_state.uri),
                message: name.to_string(),
            });
        }
        ParolParserError::EmptyGroup { start, end, .. } => {
            *range = location_to_range(&parol_runtime::Location {
                start_line: start.start_line,
                start_column: start.start_column,
                end_line: end.end_line,
                end_column: end.end_column,
                ..Default::default()
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(start, located_document_state.uri),
                message: "Start".to_string(),
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(end, located_document_state.uri),
                message: "End".to_string(),
            });
        }
        ParolParserError::EmptyOptional { start, end, .. } => {
            *range = location_to_range(&parol_runtime::Location {
                start_line: start.start_line,
                start_column: start.start_column,
                end_line: end.end_line,
                end_column: end.end_column,
                ..Default::default()
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(start, located_document_state.uri),
                message: "Start".to_string(),
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(end, located_document_state.uri),
                message: "End".to_string(),
            });
        }
        ParolParserError::EmptyRepetition { start, end, .. } => {
            *range = location_to_range(&parol_runtime::Location {
                start_line: start.start_line,
                start_column: start.start_column,
                end_line: end.end_line,
                end_column: end.end_column,
                ..Default::default()
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(start, located_document_state.uri),
                message: "Start".to_string(),
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(end, located_document_state.uri),
                message: "End".to_string(),
            });
        }
        ParolParserError::ConflictingTokenAliases {
            first_alias,
            second_alias,
            first,
            second,
            ..
        } => {
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(first, located_document_state.uri),
                message: first_alias.to_string(),
            });
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(second, located_document_state.uri),
                message: second_alias.to_string(),
            });
        }
        ParolParserError::EmptyScanners { .. } => {
            // No additional information attached
        }
        ParolParserError::UnsupportedGrammarType {
            grammar_type,
            token,
            ..
        } => {
            *range = location_to_range(token);
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(token, located_document_state.uri),
                message: grammar_type.to_string(),
            });
        }
        ParolParserError::UnsupportedFeature {
            feature,
            token,
            hint,
            ..
        } => {
            *range = location_to_range(token);
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(token, located_document_state.uri),
                message: format!("{feature} - Feature is not yet supported\n{hint}"),
            });
        }
        ParolParserError::InvalidTokenInTransition {
            context,
            token,
            input,
            location,
        } => {
            *range = location_to_range(location);
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(location, located_document_state.uri),
                message: format!(
                    "Context: {}, Token: {}, Input: {}",
                    context,
                    token,
                    input.display()
                ),
            });
        }
        ParolParserError::TokenIsNotInScanner {
            context,
            scanner,
            token,
            input,
            location,
        } => {
            *range = location_to_range(location);
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(location, located_document_state.uri),
                message: format!(
                    "Context: {}, Scanner: {}, Token: {}, Input: {}",
                    context,
                    scanner,
                    token,
                    input.display()
                ),
            });
        }
        ParolParserError::MixedScannerSwitching {
            context,
            input,
            location,
        } => {
            *range = location_to_range(location);
            related_information.push(DiagnosticRelatedInformation {
                location: location_to_location(location, located_document_state.uri),
                message: format!("Context: {}, Input: {}", context, input.display()),
            });
        }
    }
}
