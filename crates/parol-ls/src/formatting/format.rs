use lsp_types::{FormattingOptions, Range, TextEdit};
use std::vec;

use crate::{
    parol_ls_grammar_trait::{
        Declaration, ParolLs, Production, ProductionLHS, ScannerStateDirectives, Symbol,
    },
    rng::Rng,
};

use super::{
    Comments, FmtOptions, FormatterContext, Indent, Line, LineEnd, Padding,
    context_for_declaration, context_for_scanner_directive, declaration_delimiter,
    scanner_directive_indent, semicolon_starts_on_new_line,
};

mod declaration_fmt;
mod dispatch;
mod entry_user_fmt;
mod grammar_core_fmt;
mod helpers;
mod last_token;
mod production_fmt;
mod prolog_fmt;
mod scalar_fmt;
mod scanner_fmt;
mod scanner_state_fmt;
mod token_expr_fmt;

// This is the actual start column for each production (alternation) line
const START_LINE_OFFSET: usize = 6;

pub(crate) trait Format {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit>;
}

impl Format for &ParolLs {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit> {
        // We use the complete document's range for the edit to ensure that the whole document is
        // replaced. This is necessary to avoid problems with comments at the start and the end of
        // the document.
        let range = Rng::new(Range::default()).extend_to_end().0;
        let fmt_options = options.into();
        let (new_text, comments) = self.txt(&fmt_options, comments);
        debug_assert!(comments.is_empty());
        vec![TextEdit { range, new_text }]
    }
}

pub(crate) trait Fmt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments);
}

#[cfg(test)]
mod test {
    use std::{ffi::OsStr, fs};

    use parol_runtime::Report;

    use crate::{
        formatting::{FmtOptions, LineEnd, Padding, fmt_options::Trimming, format::Fmt},
        parol_ls_grammar::ParolLsGrammar,
        parol_ls_parser::parse,
        utils::RX_NEW_LINE,
    };

    struct LsErrorReporter;
    impl Report for LsErrorReporter {}

    const INPUT_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/input");
    const ACTUAL_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/actual");

    // Use this to skip certain tests if they are not ready yet
    const SKIP_LIST: &[&str] = &[]; //&["complex1.par"];

    // Use this if you only want to debug a view tests
    const SELECTED_TESTS: &[&str] = &[]; //&["single_group.par"];

    const TEST_DATA: &[(FmtOptions, &str)] = &[
        (
            FmtOptions {
                empty_line_after_prod: true,
                prod_semicolon_on_nl: true,
                max_line_length: 100,
                padding: Padding::None,
                line_end: LineEnd::Unchanged,
                trimming: Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(env!("CARGO_MANIFEST_DIR"), "/data/expected/options_default"),
        ),
        (
            FmtOptions {
                empty_line_after_prod: true,
                prod_semicolon_on_nl: false,
                max_line_length: 100,
                padding: Padding::None,
                line_end: LineEnd::Unchanged,
                trimming: Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/expected/prod_semicolon_on_nl_false"
            ),
        ),
        (
            FmtOptions {
                empty_line_after_prod: false,
                prod_semicolon_on_nl: true,
                max_line_length: 100,
                padding: Padding::None,
                line_end: LineEnd::Unchanged,
                trimming: Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/expected/empty_line_after_prod_false"
            ),
        ),
    ];

    #[test]
    // #[ignore = "Not ready yet"]
    fn test_formatting() {
        let mut error_count = 0;
        let mut tests_run = 0;

        for (fmt_options, expected_folder) in TEST_DATA {
            eprintln!("from folder {INPUT_FOLDER}:");
            for entry in std::path::Path::new(INPUT_FOLDER)
                .read_dir()
                .unwrap()
                .flatten()
            {
                if skip_test(&entry.file_name()) {
                    continue;
                }
                if entry.path().extension().unwrap().to_str().unwrap() == "par" {
                    eprintln!("\nParsing {}...", entry.path().display());
                    if !process_single_file(
                        entry.file_name().as_os_str(),
                        fmt_options,
                        expected_folder,
                    ) {
                        error_count += 1;
                    }
                    tests_run += 1;
                }
            }
        }
        eprintln!("Found {error_count} formatting error(s) in {tests_run} tests.");
        assert_eq!(0, error_count);
    }

    fn process_single_file(
        file_name: &OsStr,
        fmt_options: &FmtOptions,
        expected_folder: &str,
    ) -> bool {
        let mut input_file = std::path::PathBuf::from(INPUT_FOLDER);
        input_file.push(file_name);
        let input_grammar = fs::read_to_string(input_file.clone()).unwrap();
        let mut grammar = ParolLsGrammar::new();

        if let Err(e) = parse(&input_grammar, input_file.clone(), &mut grammar) {
            LsErrorReporter::report_error(&e, input_file).unwrap();
            panic!("Parsing failed!")
        } else {
            // We generate the new formatting by calling Fmt::txt()
            let (formatted_grammar, _comments) = grammar
                .grammar
                .unwrap()
                .txt(fmt_options, grammar.comments.clone());
            // assert!(comments.is_empty());

            let mut expected_file = std::path::PathBuf::from(expected_folder);

            // Only to support debugging we write out the currently generated source
            let mut actual_file = std::path::PathBuf::from(ACTUAL_FOLDER);
            let expected_sub_folder = expected_file.iter().next_back().unwrap();
            actual_file.push(expected_sub_folder);
            fs::DirBuilder::new()
                .recursive(true)
                .create(actual_file.clone())
                .unwrap();

            actual_file.push(file_name);
            fs::write(actual_file, formatted_grammar.clone()).unwrap();

            // Read the fixed expectation file into a string
            expected_file.push(file_name);
            eprintln!("expected_file: '{}'", expected_file.display());
            let expected_format = fs::read_to_string(expected_file).unwrap();

            // Compare result with expectation
            let expected_format = RX_NEW_LINE.replace_all(&expected_format, "\n");
            let formatted_grammar = RX_NEW_LINE.replace_all(&formatted_grammar, "\n");

            if expected_format != formatted_grammar {
                eprintln!("=====================================================");
                eprintln!("expecting:\n'{expected_format}'");
                eprintln!("-----------------------------------------------------");
                eprintln!("received:\n'{formatted_grammar}'");
                eprintln!("=====================================================");
                false
            } else {
                true
            }
        }
    }

    #[allow(clippy::const_is_empty)]
    fn skip_test(file_name: &OsStr) -> bool {
        SKIP_LIST.contains(&file_name.to_str().unwrap())
            || (!SELECTED_TESTS.is_empty()
                && !SELECTED_TESTS.contains(&file_name.to_str().unwrap()))
    }
}
