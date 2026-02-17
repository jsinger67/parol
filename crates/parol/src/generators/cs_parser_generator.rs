use crate::analysis::LookaheadDFA;
use crate::analysis::compiled_la_dfa::CompiledDFA;
use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::generators::{GrammarConfig, NamingHelper};
use crate::grammar::SymbolAttribute;
use crate::{LRParseTable, Symbol, Terminal};
use anyhow::Result;
use parol_runtime::TerminalIndex;
use parol_runtime::lexer::FIRST_USER_TOKEN;
use std::collections::BTreeMap;
use std::fmt::Write;

/// Generates the parser part of the parser output file for C# (LL(k)).
pub fn generate_parser_source<C: CommonGeneratorConfig + ParserGeneratorConfig>(
    grammar_config: &GrammarConfig,
    _lexer_source: &str, // Ignored, we regenerate the data class
    config: &C,
    la_dfa: &BTreeMap<String, LookaheadDFA>,
    _ast_type_has_lifetime: bool,
) -> Result<String> {
    let mut source = String::new();

    let original_non_terminals = grammar_config.cfg.get_non_terminal_set();
    let non_terminals = original_non_terminals.iter().collect::<Vec<_>>();

    let parser_type_name = NamingHelper::to_upper_camel_case(config.user_type_name()) + "Parser";

    writeln!(source, "using System;")?;
    writeln!(source, "using System.Collections.Generic;")?;
    writeln!(source, "using Parol.Runtime;")?;
    writeln!(source, "using Parol.Runtime.Scanner;")?;
    writeln!(source)?;
    writeln!(source, "namespace {} {{", config.module_name())?;
    writeln!(source, "    public class {} {{", parser_type_name)?;

    // Lexer Source (Scanner data class only)
    let scanner_data =
        crate::generators::cs_lexer_generator::generate_scanner_data(grammar_config, config)?;
    writeln!(source, "{}", scanner_data)?;
    writeln!(source)?;

    // Max K
    writeln!(
        source,
        "        public const int MaxK = {};",
        grammar_config.lookahead_size
    )?;
    writeln!(source)?;

    // Non-Terminal Names
    writeln!(
        source,
        "        public static readonly string[] NonTerminalNames = {{"
    )?;
    for name in &non_terminals {
        writeln!(source, "            \"{}\",", name)?;
    }
    writeln!(source, "        }};")?;
    writeln!(source)?;

    // Lookahead Automata
    generate_lookahead_automata(&mut source, la_dfa, &non_terminals)?;
    writeln!(source)?;

    // Productions
    generate_productions(&mut source, grammar_config, &non_terminals)?;
    writeln!(source)?;

    // Parse Methods
    generate_parse_methods(&mut source, grammar_config, config, &parser_type_name)?;

    writeln!(source, "    }}")?;
    writeln!(source, "}}")?;

    Ok(source)
}

fn generate_lookahead_automata(
    source: &mut String,
    la_dfa: &BTreeMap<String, LookaheadDFA>,
    non_terminals: &[&String],
) -> Result<()> {
    writeln!(
        source,
        "        public static readonly LookaheadDfa[] LookaheadAutomata = {{"
    )?;
    for (i, nt_name) in non_terminals.iter().enumerate() {
        if let Some(dfa) = la_dfa.get(*nt_name) {
            let compiled_dfa = CompiledDFA::from_lookahead_dfa(dfa);
            writeln!(source, "            /* {} - \"{}\" */", i, nt_name)?;
            writeln!(source, "            new LookaheadDfa(")?;
            writeln!(source, "                {},", compiled_dfa.prod0)?;
            writeln!(source, "                new Trans[] {{")?;
            for t in &compiled_dfa.transitions {
                writeln!(
                    source,
                    "                    new Trans({}, {}, {}, {}),",
                    t.from_state, t.term, t.to_state, t.prod_num
                )?;
            }
            writeln!(source, "                }},")?;
            writeln!(source, "                {} // k", compiled_dfa.k)?;
            writeln!(source, "            ),")?;
        }
    }
    writeln!(source, "        }};")?;
    Ok(())
}

fn generate_productions(
    source: &mut String,
    grammar_config: &GrammarConfig,
    non_terminals: &[&String],
) -> Result<()> {
    let terminals = grammar_config.cfg.get_ordered_terminals();
    let get_non_terminal_index = |nt: &str| non_terminals.iter().position(|n| *n == nt).unwrap();
    let get_terminal_index =
        |tr: &str, l: &Option<crate::parser::parol_grammar::LookaheadExpression>| {
            terminals
                .iter()
                .position(|(t, _, look, _)| *t == tr && look == l)
                .unwrap() as TerminalIndex
                + FIRST_USER_TOKEN
        };

    writeln!(
        source,
        "        public static readonly Production[] Productions = {{"
    )?;
    for (i, pr) in grammar_config.cfg.pr.iter().enumerate() {
        let lhs = get_non_terminal_index(pr.get_n_str());
        writeln!(source, "            // {} - {}", i, pr)?;
        writeln!(source, "            new Production(")?;
        writeln!(source, "                {},", lhs)?;
        writeln!(source, "                new ParseItem[] {{")?;
        for s in pr.get_r() {
            match s {
                Symbol::N(n, ..) => {
                    writeln!(
                        source,
                        "                    new ParseItem(ParseType.N, {}),",
                        get_non_terminal_index(n)
                    )?;
                }
                Symbol::T(Terminal::Trm(t, _, _, attr, _, _, l0)) => {
                    let parse_type = if *attr == SymbolAttribute::Clipped {
                        "ParseType.C"
                    } else {
                        "ParseType.T"
                    };
                    writeln!(
                        source,
                        "                    new ParseItem({}, {}),",
                        parse_type,
                        get_terminal_index(t, l0)
                    )?;
                }
                _ => panic!("Unexpected symbol type in production!"),
            }
        }
        writeln!(source, "                }}")?;
        writeln!(source, "            ),")?;
    }
    writeln!(source, "        }};")?;
    Ok(())
}

fn generate_parse_methods(
    source: &mut String,
    grammar_config: &GrammarConfig,
    config: &impl CommonGeneratorConfig,
    _parser_type_name: &str,
) -> Result<()> {
    let scanner_type_name = NamingHelper::to_upper_camel_case(config.user_type_name()) + "Scanner";
    let actions_interface_name = format!(
        "I{}Actions",
        NamingHelper::to_upper_camel_case(config.user_type_name())
    );
    let start_symbol_index = grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .position(|n| *n == grammar_config.cfg.get_start_symbol())
        .unwrap();

    writeln!(
        source,
        "        public static void Parse(string input, string fileName, {} userActions) {{",
        actions_interface_name
    )?;
    writeln!(
        source,
        "            ParseInternal(input, fileName, userActions);"
    )?;
    writeln!(source, "        }}")?;
    writeln!(source)?;

    writeln!(
        source,
        "        public static void Parse(string input, string fileName, IUserActions userActions) {{"
    )?;
    writeln!(
        source,
        "            ParseInternal(input, fileName, userActions);"
    )?;
    writeln!(source, "        }}")?;
    writeln!(source)?;

    writeln!(
        source,
        "        private static void ParseInternal(string input, string fileName, IUserActions userActions) {{"
    )?;
    writeln!(source, "            var parser = new LLKParser(")?;
    writeln!(source, "                {},", start_symbol_index)?;
    writeln!(source, "                LookaheadAutomata,")?;
    writeln!(source, "                Productions,")?;
    writeln!(
        source,
        "                {}Data.TerminalNames,",
        scanner_type_name
    )?;
    writeln!(source, "                NonTerminalNames")?;
    writeln!(source, "            );")?;
    writeln!(source)?;
    writeln!(
        source,
        "            var tokens = Scanner.Scan(input, fileName, {}Data.MatchFunction, {}Data.ScannerModes);",
        scanner_type_name, scanner_type_name
    )?;
    writeln!(source, "            parser.Parse(tokens, userActions);")?;
    writeln!(source, "        }}")?;

    Ok(())
}

/// Generates the parser part of the parser output file for C# (LALR(1)).
pub fn generate_lalr1_parser_source<C: CommonGeneratorConfig + ParserGeneratorConfig>(
    _grammar_config: &GrammarConfig,
    _lexer_source: &str,
    _config: &C,
    _parse_table: &LRParseTable,
    _ast_type_has_lifetime: bool,
) -> Result<String> {
    Ok("// C# LALR(1) Parser Source (TODO)".to_string())
}
