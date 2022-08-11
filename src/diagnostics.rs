use lsp_types::{Diagnostic, DiagnosticRelatedInformation, Location, NumberOrString, Range, Url};
use parol::analysis::GrammarAnalysisError;
use std::collections::HashSet;
use std::fmt::Write as _;

use crate::{
    document_state::{DocumentState, LocatedDocumentState},
    server::Server,
    utils::source_code_span_to_range,
};

#[derive(Debug)]
pub struct Diagnostics {}

impl Diagnostics {
    pub(crate) fn to_diagnostics(
        uri: &Url,
        document_state: &DocumentState,
        err: miette::ErrReport,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let miette_diagnostic: &dyn miette::Diagnostic = err.as_ref();
        let range = if let Some(mut labels) = miette_diagnostic.labels() {
            labels.next().map_or(Range::default(), |src| {
                source_code_span_to_range(&document_state.input, src.inner())
            })
        } else {
            Range::default()
        };
        let source = Some("parol-ls".to_string());
        let code = miette_diagnostic
            .code()
            .map(|d| NumberOrString::String(format!("{d}")));
        let mut related_information = vec![];
        let message = miette_diagnostic
            .help()
            .map_or(format!("{err}"), |help| format!("{err}:\nHelp: {help}"));

        // Extract additional information from certain errors
        if let Some(e) = err.downcast_ref::<GrammarAnalysisError>() {
            let located_document_state = LocatedDocumentState::new(uri, document_state);
            extract_grammar_analysis_error(
                e,
                range,
                &located_document_state,
                &mut related_information,
            );
        }

        let diagnostic = Diagnostic {
            source,
            code,
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
}

fn extract_grammar_analysis_error(
    error: &GrammarAnalysisError,
    range: Range,
    located_document_state: &LocatedDocumentState,
    related_information: &mut Vec<DiagnosticRelatedInformation>,
) {
    match error {
        GrammarAnalysisError::LeftRecursion { recursions } => {
            for (i, rec) in recursions.iter().enumerate() {
                related_information.push(DiagnosticRelatedInformation {
                    location: Location {
                        uri: located_document_state.uri.to_owned(),
                        range,
                    },
                    message: format!("Recursion #{}:", i + 1),
                });
                for hint in &rec.hints {
                    eprintln!("{}", hint);
                    if let Some(non_terminals) = Server::find_non_terminal_definitions(
                        located_document_state.document_state,
                        &hint.hint,
                    ) {
                        // We use a Set to avoid duplicate entries which can occur due to the use of
                        // an expanded version of the grammar.
                        // If you write a production P like this:
                        // P: A | B | C;
                        // You will eventually end up with three productions for P:
                        // P: A;
                        // P: B;
                        // P: C;
                        let mut ranges: HashSet<String> = HashSet::new();
                        for rng in non_terminals {
                            let (range, message) = (*rng, format!("Non-terminal: {}", hint.hint));
                            if ranges.insert(hint.hint.clone()) {
                                related_information.push(DiagnosticRelatedInformation {
                                    location: Location {
                                        uri: located_document_state.uri.to_owned(),
                                        range,
                                    },
                                    message,
                                })
                            }
                        }
                    } else if let Some(rel) = related_information.last_mut() {
                        if rel.message.ends_with(':') {
                            let _ = write!(rel.message, " {}", hint.hint);
                        } else {
                            let _ = write!(rel.message, " => {}", hint.hint);
                        }
                    }
                }
            }
        }
        GrammarAnalysisError::UnreachableNonTerminals { non_terminals }
        | GrammarAnalysisError::NonProductiveNonTerminals { non_terminals } => {
            for hint in non_terminals {
                eprintln!("{}", hint);
                if let Some(non_terminals) = Server::find_non_terminal_definitions(
                    located_document_state.document_state,
                    &hint.hint,
                ) {
                    for rng in non_terminals {
                        let (range, message) = (*rng, format!("Non-terminal: {}", hint.hint));
                        related_information.push(DiagnosticRelatedInformation {
                            location: Location {
                                uri: located_document_state.uri.to_owned(),
                                range,
                            },
                            message,
                        })
                    }
                }
            }
        }
        GrammarAnalysisError::MaxKExceeded { max_k: _ } => {
            // No additional information attached
        }
    }
}
