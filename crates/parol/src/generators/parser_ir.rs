use crate::LRParseTable;
use crate::analysis::LookaheadDFA;
use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::generators::GrammarConfig;
use crate::grammar::{Symbol, SymbolAttribute, Terminal};
use crate::parser::parol_grammar::LookaheadExpression;
use anyhow::{Result, anyhow};
use parol_runtime::TerminalIndex;
use std::collections::BTreeMap;

pub(crate) enum ParserAlgorithmIR<'a> {
    Llk(&'a BTreeMap<String, LookaheadDFA>),
    Lalr1(&'a LRParseTable),
}

pub(crate) struct ParserCommonIR {
    pub(crate) non_terminal_names: Vec<String>,
    pub(crate) start_symbol_index: usize,
    pub(crate) ast_type_has_lifetime: bool,
}

pub(crate) enum ProductionSymbolIR {
    NonTerminal(usize),
    Terminal { index: TerminalIndex, clipped: bool },
}

pub(crate) struct ProductionIR {
    pub(crate) production_index: usize,
    pub(crate) lhs_index: usize,
    pub(crate) rhs: Vec<ProductionSymbolIR>,
    pub(crate) text: String,
}

pub(crate) struct ParserGenerationIR<'a, C>
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    pub(crate) grammar_config: &'a GrammarConfig,
    pub(crate) lexer_source: &'a str,
    pub(crate) config: &'a C,
    pub(crate) algorithm: ParserAlgorithmIR<'a>,
    pub(crate) common: ParserCommonIR,
}

impl<'a, C> ParserGenerationIR<'a, C>
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    pub(crate) fn new(
        grammar_config: &'a GrammarConfig,
        lexer_source: &'a str,
        config: &'a C,
        ast_type_has_lifetime: bool,
        algorithm: ParserAlgorithmIR<'a>,
    ) -> Result<Self> {
        let non_terminal_names = ordered_non_terminal_names(grammar_config);
        let start_symbol_index = find_start_symbol_index(&non_terminal_names, grammar_config)?;
        Ok(Self {
            grammar_config,
            lexer_source,
            config,
            algorithm,
            common: ParserCommonIR {
                non_terminal_names,
                start_symbol_index,
                ast_type_has_lifetime,
            },
        })
    }

    pub(crate) fn has_productions(&self) -> bool {
        !self.grammar_config.cfg.pr.is_empty()
    }
}

pub(crate) fn ordered_non_terminal_names(grammar_config: &GrammarConfig) -> Vec<String> {
    grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
}

pub(crate) fn find_start_symbol_index(
    non_terminal_names: &[String],
    grammar_config: &GrammarConfig,
) -> Result<usize> {
    non_terminal_names
        .iter()
        .position(|n| n == grammar_config.cfg.get_start_symbol())
        .ok_or_else(|| {
            anyhow!(
                "Start symbol '{}' is not part of the given grammar!",
                grammar_config.cfg.get_start_symbol()
            )
        })
}

pub(crate) fn build_production_ir(
    grammar_config: &GrammarConfig,
    non_terminal_names: &[String],
) -> Result<Vec<ProductionIR>> {
    let terminals = grammar_config.cfg.get_ordered_terminals();

    let get_non_terminal_index = |nt: &str| {
        non_terminal_names
            .iter()
            .position(|n| n == nt)
            .ok_or_else(|| anyhow!("Non-terminal '{}' not found", nt))
    };

    let get_terminal_index = |tr: &str, l: &Option<LookaheadExpression>| -> Result<TerminalIndex> {
        terminals
            .iter()
            .position(|(t, _, look, _)| *t == tr && look == l)
            .map(|i| i as TerminalIndex + parol_runtime::lexer::FIRST_USER_TOKEN)
            .ok_or_else(|| anyhow!("Terminal '{}' with lookahead not found", tr))
    };

    grammar_config
        .cfg
        .pr
        .iter()
        .enumerate()
        .map(|(production_index, pr)| {
            let lhs_index = get_non_terminal_index(pr.get_n_str())?;
            let rhs = pr
                .get_r()
                .iter()
                .map(|s| match s {
                    Symbol::N(n, ..) => {
                        get_non_terminal_index(n).map(ProductionSymbolIR::NonTerminal)
                    }
                    Symbol::T(Terminal::Trm(t, _, _, attr, _, _, l0)) => get_terminal_index(t, l0)
                        .map(|index| ProductionSymbolIR::Terminal {
                            index,
                            clipped: *attr == SymbolAttribute::Clipped,
                        }),
                    _ => Err(anyhow!("Unexpected symbol type in production")),
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(ProductionIR {
                production_index,
                lhs_index,
                rhs,
                text: format!("{pr}"),
            })
        })
        .collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::obtain_grammar_config;
    use parol_runtime::lexer::FIRST_USER_TOKEN;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn grammar_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/valid")
            .join(file_name)
    }

    #[test]
    fn production_ir_marks_clipped_terminals() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let non_terminal_names = ordered_non_terminal_names(&grammar_config);

        let production_ir = build_production_ir(&grammar_config, &non_terminal_names).unwrap();
        let rhs = &production_ir[0].rhs;

        assert_eq!(rhs.len(), 2);
        assert!(matches!(
            rhs[0],
            ProductionSymbolIR::Terminal {
                index: FIRST_USER_TOKEN,
                clipped: false
            }
        ));
        assert!(matches!(
            rhs[1],
            ProductionSymbolIR::Terminal {
                index,
                clipped: true
            } if index == FIRST_USER_TOKEN + 1
        ));
    }

    #[test]
    fn production_ir_errors_on_unknown_non_terminal() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let invalid_non_terminals = vec!["Unknown".to_string()];

        let error = match build_production_ir(&grammar_config, &invalid_non_terminals) {
            Ok(_) => panic!("Expected build_production_ir to fail for unknown non-terminal"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("Non-terminal 'Start' not found"));
    }

    #[test]
    fn production_ir_distinguishes_same_terminal_with_lookahead() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_grammar_file = std::env::temp_dir().join(format!(
            "parol_parser_ir_lookahead_{}_{}.par",
            std::process::id(),
            now
        ));
        let grammar = r#"%start S
%%
S: "a" ?= "b" | "a";
"#;
        fs::write(&temp_grammar_file, grammar).unwrap();

        let test_result = (|| {
            let grammar_config = obtain_grammar_config(temp_grammar_file.clone(), false)?;
            let non_terminal_names = ordered_non_terminal_names(&grammar_config);
            let production_ir = build_production_ir(&grammar_config, &non_terminal_names)?;

            anyhow::ensure!(
                production_ir.len() == 2,
                "Expected exactly two productions for test grammar"
            );

            let lookahead_terminal_index = grammar_config
                .cfg
                .get_ordered_terminals()
                .iter()
                .position(|(t, _, l, _)| *t == "a" && l.is_some())
                .map(|i| i as TerminalIndex + FIRST_USER_TOKEN)
                .ok_or_else(|| anyhow!("Failed to resolve lookahead terminal index"))?;
            let plain_terminal_index = grammar_config
                .cfg
                .get_ordered_terminals()
                .iter()
                .position(|(t, _, l, _)| *t == "a" && l.is_none())
                .map(|i| i as TerminalIndex + FIRST_USER_TOKEN)
                .ok_or_else(|| anyhow!("Failed to resolve plain terminal index"))?;

            anyhow::ensure!(
                lookahead_terminal_index != plain_terminal_index,
                "Lookahead and plain terminals must resolve to different indices"
            );

            let first_index = match production_ir[0].rhs.as_slice() {
                [ProductionSymbolIR::Terminal { index, .. }] => *index,
                _ => anyhow::bail!("Unexpected RHS for first production"),
            };
            let second_index = match production_ir[1].rhs.as_slice() {
                [ProductionSymbolIR::Terminal { index, .. }] => *index,
                _ => anyhow::bail!("Unexpected RHS for second production"),
            };

            anyhow::ensure!(
                first_index == lookahead_terminal_index,
                "First production should use lookahead-specific terminal index"
            );
            anyhow::ensure!(
                second_index == plain_terminal_index,
                "Second production should use plain terminal index"
            );
            Ok::<(), anyhow::Error>(())
        })();

        let _ = fs::remove_file(&temp_grammar_file);
        test_result.unwrap();
    }
}
