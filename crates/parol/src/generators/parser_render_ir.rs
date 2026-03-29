use crate::LRParseTable;
use crate::generators::GrammarConfig;
use crate::generators::parser_model::{
    ProductionModel, ProductionSymbolModel, build_lalr_parse_table_model,
};
use crate::parser::parol_grammar::LookaheadExpression;
use parol_runtime::TerminalIndex;
use parol_runtime::lexer::{
    BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, WHITESPACE,
};
use std::collections::BTreeMap;

pub(crate) struct CSharpLlkProductionRenderIR {
    pub(crate) production_index: usize,
    pub(crate) lhs_index: usize,
    pub(crate) text: String,
    pub(crate) symbols: Vec<String>,
}

pub(crate) struct LalrProductionRenderIR {
    pub(crate) production_index: usize,
    pub(crate) lhs_index: usize,
    pub(crate) text: String,
    pub(crate) rhs_len: usize,
    pub(crate) semantic_children: Vec<bool>,
}

pub(crate) struct LalrStateActionRenderIR {
    pub(crate) terminal: TerminalIndex,
    pub(crate) action_index: usize,
    pub(crate) terminal_label: String,
}

pub(crate) struct LalrStateGotoRenderIR {
    pub(crate) non_terminal: usize,
    pub(crate) goto_state: usize,
    pub(crate) non_terminal_name: String,
}

pub(crate) struct LalrStateRenderIR {
    pub(crate) state_index: usize,
    pub(crate) actions: Vec<LalrStateActionRenderIR>,
    pub(crate) gotos: Vec<LalrStateGotoRenderIR>,
}

pub(crate) struct LalrParseTableRenderIR {
    pub(crate) actions: Vec<crate::LRAction>,
    pub(crate) states: Vec<LalrStateRenderIR>,
}

pub(crate) struct RustLalrStateActionRenderIR {
    pub(crate) terminal: TerminalIndex,
    pub(crate) action_index: usize,
    pub(crate) terminal_label: String,
    pub(crate) action_comment: String,
}

pub(crate) struct RustLalrStateGotoRenderIR {
    pub(crate) non_terminal: usize,
    pub(crate) goto_state: usize,
    pub(crate) non_terminal_name: String,
}

pub(crate) struct RustLalrStateRenderIR {
    pub(crate) state_index: usize,
    pub(crate) actions: Vec<RustLalrStateActionRenderIR>,
    pub(crate) gotos: Vec<RustLalrStateGotoRenderIR>,
}

pub(crate) struct RustLalrParseTableRenderIR {
    pub(crate) actions: Vec<String>,
    pub(crate) states: Vec<RustLalrStateRenderIR>,
}

pub(crate) struct CSharpLalrParseTableRenderIR {
    pub(crate) actions: Vec<String>,
    pub(crate) states: Vec<CSharpLalrStateRenderIR>,
}

pub(crate) struct CSharpLalrParseTableSectionRenderIR {
    pub(crate) action_rows: Vec<String>,
    pub(crate) state_rows: Vec<String>,
}

pub(crate) struct CSharpLalrStateRenderIR {
    pub(crate) state_index: usize,
    pub(crate) action_refs: Vec<String>,
    pub(crate) gotos: Vec<String>,
}

pub(crate) struct CSharpNonTerminalNamesRenderIR {
    pub(crate) rows: Vec<String>,
}

pub(crate) struct NonTerminalMetadataIR {
    pub(crate) names: Vec<String>,
    pub(crate) indexed_rows: Vec<String>,
}

pub(crate) fn build_non_terminal_metadata_ir(
    grammar_config: &GrammarConfig,
) -> NonTerminalMetadataIR {
    let names = grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let width = (names.len() as f32).log10() as usize + 1;
    let indexed_rows = names
        .iter()
        .enumerate()
        .map(|(i, n)| format!(r#"/* {i:width$} */ "{n}","#))
        .collect::<Vec<_>>();

    NonTerminalMetadataIR {
        names,
        indexed_rows,
    }
}

pub(crate) fn build_terminal_label_map(
    terminals: &[(&str, Option<LookaheadExpression>)],
) -> BTreeMap<TerminalIndex, String> {
    let mut labels = BTreeMap::new();

    labels.insert(EOI, "<$>".to_string());
    labels.insert(NEW_LINE, "<NL>".to_string());
    labels.insert(WHITESPACE, "<WS>".to_string());
    labels.insert(LINE_COMMENT, "<LC>".to_string());
    labels.insert(BLOCK_COMMENT, "<BC>".to_string());

    for (i, (terminal_name, _)) in terminals.iter().enumerate() {
        labels.insert(
            i as TerminalIndex + FIRST_USER_TOKEN,
            (*terminal_name).to_string(),
        );
    }

    labels
}

pub(crate) fn build_csharp_non_terminal_names_render_ir(
    non_terminal_names: &[String],
) -> CSharpNonTerminalNamesRenderIR {
    let rows = non_terminal_names
        .iter()
        .map(|n| format!("\"{}\",", n))
        .collect::<Vec<_>>();

    CSharpNonTerminalNamesRenderIR { rows }
}

fn csharp_llk_symbol_source(symbol_ir: &ProductionSymbolModel) -> String {
    match symbol_ir {
        ProductionSymbolModel::NonTerminal(index) => format!("new(ParseType.N, {})", index),
        ProductionSymbolModel::Terminal { index, clipped } => {
            let parse_type = if *clipped {
                "ParseType.C"
            } else {
                "ParseType.T"
            };
            format!("new({}, {})", parse_type, index)
        }
    }
}

pub(crate) fn build_csharp_llk_production_render_ir(
    production_ir: &[ProductionModel],
) -> Vec<CSharpLlkProductionRenderIR> {
    production_ir
        .iter()
        .map(|p| CSharpLlkProductionRenderIR {
            production_index: p.production_index,
            lhs_index: p.lhs_index,
            text: p.text.clone(),
            symbols: p
                .rhs
                .iter()
                .map(csharp_llk_symbol_source)
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>()
}

pub(crate) fn build_lalr_production_render_ir(
    production_ir: &[ProductionModel],
) -> Vec<LalrProductionRenderIR> {
    production_ir
        .iter()
        .map(|p| LalrProductionRenderIR {
            production_index: p.production_index,
            lhs_index: p.lhs_index,
            text: p.text.clone(),
            rhs_len: p.rhs.len(),
            semantic_children: p
                .rhs
                .iter()
                .map(|s| match s {
                    ProductionSymbolModel::NonTerminal(..) => true,
                    ProductionSymbolModel::Terminal { clipped, .. } => !*clipped,
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>()
}

fn csharp_render_action(action: &crate::LRAction) -> String {
    match action {
        crate::LRAction::Shift(state) => format!("new LRAction.Shift({})", state),
        crate::LRAction::Reduce(non_terminal, production) => {
            format!("new LRAction.Reduce({}, {})", non_terminal, production)
        }
        crate::LRAction::Accept => "new LRAction.Accept()".to_string(),
    }
}

fn rust_render_action(action: &crate::LRAction, non_terminal_names: &[String]) -> String {
    match action {
        crate::LRAction::Shift(s) => format!("LRAction::Shift({s})"),
        crate::LRAction::Reduce(n, p) => {
            format!(
                "LRAction::Reduce({} /* {} */, {})",
                n, non_terminal_names[*n], p
            )
        }
        crate::LRAction::Accept => "LRAction::Accept".to_string(),
    }
}

fn rust_render_action_comment(action: &crate::LRAction, non_terminal_names: &[String]) -> String {
    match action {
        crate::LRAction::Shift(s) => format!("LRAction::Shift({s})"),
        crate::LRAction::Reduce(n, p) => {
            format!("LRAction::Reduce({}, {})", non_terminal_names[*n], p)
        }
        crate::LRAction::Accept => "LRAction::Accept".to_string(),
    }
}

pub(crate) fn build_lalr_parse_table_render_ir(
    parse_table: &LRParseTable,
    terminal_labels: &BTreeMap<TerminalIndex, String>,
    non_terminal_names: &[String],
) -> LalrParseTableRenderIR {
    let parse_table_ir = build_lalr_parse_table_model(parse_table);

    let states = parse_table_ir
        .states
        .iter()
        .enumerate()
        .map(|(state_index, state)| LalrStateRenderIR {
            state_index,
            actions: state
                .actions
                .iter()
                .map(|(terminal, action_index)| LalrStateActionRenderIR {
                    terminal: *terminal,
                    action_index: *action_index,
                    terminal_label: terminal_labels
                        .get(terminal)
                        .cloned()
                        .unwrap_or_else(|| "<?>".to_string()),
                })
                .collect::<Vec<_>>(),
            gotos: state
                .gotos
                .iter()
                .map(|(non_terminal, goto_state)| LalrStateGotoRenderIR {
                    non_terminal: *non_terminal,
                    goto_state: *goto_state,
                    non_terminal_name: non_terminal_names
                        .get(*non_terminal)
                        .cloned()
                        .unwrap_or_else(|| "<?>".to_string()),
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    LalrParseTableRenderIR {
        actions: parse_table_ir.actions,
        states,
    }
}

pub(crate) fn build_rust_lalr_parse_table_render_ir(
    parse_table: &LRParseTable,
    terminal_labels: &BTreeMap<TerminalIndex, String>,
    non_terminal_names: &[String],
) -> RustLalrParseTableRenderIR {
    let render_ir =
        build_lalr_parse_table_render_ir(parse_table, terminal_labels, non_terminal_names);

    let actions = render_ir
        .actions
        .iter()
        .map(|action| rust_render_action(action, non_terminal_names))
        .collect::<Vec<_>>();

    let states = render_ir
        .states
        .iter()
        .map(|state| RustLalrStateRenderIR {
            state_index: state.state_index,
            actions: state
                .actions
                .iter()
                .map(|action| RustLalrStateActionRenderIR {
                    terminal: action.terminal,
                    action_index: action.action_index,
                    terminal_label: action.terminal_label.clone(),
                    action_comment: rust_render_action_comment(
                        &render_ir.actions[action.action_index],
                        non_terminal_names,
                    ),
                })
                .collect::<Vec<_>>(),
            gotos: state
                .gotos
                .iter()
                .map(|goto| RustLalrStateGotoRenderIR {
                    non_terminal: goto.non_terminal,
                    goto_state: goto.goto_state,
                    non_terminal_name: goto.non_terminal_name.clone(),
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    RustLalrParseTableRenderIR { actions, states }
}

pub(crate) fn build_csharp_lalr_parse_table_render_ir(
    parse_table: &LRParseTable,
    terminal_labels: &BTreeMap<TerminalIndex, String>,
    non_terminal_names: &[String],
) -> CSharpLalrParseTableRenderIR {
    let render_ir =
        build_lalr_parse_table_render_ir(parse_table, terminal_labels, non_terminal_names);

    let actions = render_ir
        .actions
        .iter()
        .map(csharp_render_action)
        .collect::<Vec<_>>();

    let states = render_ir
        .states
        .iter()
        .map(|state| CSharpLalrStateRenderIR {
            state_index: state.state_index,
            action_refs: state
                .actions
                .iter()
                .map(|a| format!("new LRActionRef({}, {})", a.terminal, a.action_index))
                .collect::<Vec<_>>(),
            gotos: state
                .gotos
                .iter()
                .map(|g| format!("new LRGoto({}, {})", g.non_terminal, g.goto_state))
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    CSharpLalrParseTableRenderIR { actions, states }
}

pub(crate) fn build_csharp_lalr_parse_table_section_render_ir(
    parse_table: &LRParseTable,
    terminal_labels: &BTreeMap<TerminalIndex, String>,
    non_terminal_names: &[String],
) -> CSharpLalrParseTableSectionRenderIR {
    let render_ir =
        build_csharp_lalr_parse_table_render_ir(parse_table, terminal_labels, non_terminal_names);

    let action_rows = render_ir
        .actions
        .iter()
        .enumerate()
        .map(|(i, action_source)| format!("/* {} */ {},", i, action_source))
        .collect::<Vec<_>>();

    let state_rows = render_ir
        .states
        .iter()
        .map(|state| {
            let action_refs = state
                .action_refs
                .iter()
                .map(|a| format!("                        {},", a))
                .collect::<Vec<_>>()
                .join("\n");
            let gotos = state
                .gotos
                .iter()
                .map(|g| format!("                        {},", g))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "                // State {}\n                new LR1State(\n                    [\n{}\n                    ],\n                    [\n{}\n                    ]\n                ),",
                state.state_index, action_refs, gotos
            )
        })
        .collect::<Vec<_>>();

    CSharpLalrParseTableSectionRenderIR {
        action_rows,
        state_rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::parser_model;
    use crate::utils::obtain_grammar_config;
    use parol_runtime::lexer::{EOI, FIRST_USER_TOKEN, NEW_LINE};
    use std::path::PathBuf;

    fn grammar_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/valid")
            .join(file_name)
    }

    #[test]
    fn non_terminal_metadata_produces_indexed_rows() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let metadata = build_non_terminal_metadata_ir(&grammar_config);

        assert!(!metadata.names.is_empty());
        assert_eq!(metadata.names.len(), metadata.indexed_rows.len());
        assert!(metadata.indexed_rows[0].contains("/* 0"));
        assert!(metadata.indexed_rows[0].contains("\"Start\""));
    }

    #[test]
    fn terminal_label_map_contains_builtins_and_user_terminals() {
        let terminals = vec![("A", None), ("B", None)];
        let labels = build_terminal_label_map(&terminals);

        assert_eq!(labels.get(&EOI).unwrap(), "<$>");
        assert_eq!(labels.get(&NEW_LINE).unwrap(), "<NL>");
        assert_eq!(labels.get(&(FIRST_USER_TOKEN)).unwrap(), "A");
        assert_eq!(labels.get(&(FIRST_USER_TOKEN + 1)).unwrap(), "B");
    }

    #[test]
    fn lalr_production_render_ir_contains_semantic_child_flags() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let non_terminal_names = parser_model::ordered_non_terminal_names(&grammar_config);
        let production_model =
            parser_model::build_production_model(&grammar_config, &non_terminal_names).unwrap();

        let render_ir = build_lalr_production_render_ir(&production_model);

        assert_eq!(render_ir.len(), 1);
        assert_eq!(render_ir[0].rhs_len, 2);
        assert_eq!(render_ir[0].semantic_children, vec![true, false]);
    }

    #[test]
    fn rust_lalr_render_ir_contains_resolved_labels_and_comments() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate_lr.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let parse_table = crate::calculate_lalr1_parse_table(&grammar_config)
            .unwrap()
            .0;

        let non_terminal_metadata = build_non_terminal_metadata_ir(&grammar_config);
        let terminals = grammar_config
            .cfg
            .get_ordered_terminals()
            .iter()
            .map(|(t, _, l, _)| (*t, l.clone()))
            .collect::<Vec<_>>();
        let terminal_labels = build_terminal_label_map(&terminals);

        let render_ir = build_rust_lalr_parse_table_render_ir(
            &parse_table,
            &terminal_labels,
            &non_terminal_metadata.names,
        );

        assert!(!render_ir.actions.is_empty());
        assert_eq!(render_ir.states.len(), parse_table.states.len());
        assert!(render_ir.actions.iter().any(|a| a.contains("LRAction::")));
        assert!(
            render_ir
                .states
                .iter()
                .flat_map(|s| s.actions.iter())
                .any(|a| !a.terminal_label.is_empty() && !a.action_comment.is_empty())
        );
    }

    #[test]
    fn csharp_lalr_render_ir_contains_action_sources() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate_lr.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let parse_table = crate::calculate_lalr1_parse_table(&grammar_config)
            .unwrap()
            .0;

        let non_terminal_metadata = build_non_terminal_metadata_ir(&grammar_config);
        let terminals = grammar_config
            .cfg
            .get_ordered_terminals()
            .iter()
            .map(|(t, _, l, _)| (*t, l.clone()))
            .collect::<Vec<_>>();
        let terminal_labels = build_terminal_label_map(&terminals);

        let render_ir = build_csharp_lalr_parse_table_render_ir(
            &parse_table,
            &terminal_labels,
            &non_terminal_metadata.names,
        );

        assert!(!render_ir.actions.is_empty());
        assert_eq!(render_ir.states.len(), parse_table.states.len());
        assert!(
            render_ir
                .actions
                .iter()
                .any(|a| a.contains("new LRAction."))
        );
        assert!(
            render_ir
                .states
                .iter()
                .flat_map(|s| s.action_refs.iter())
                .any(|a| a.contains("new LRActionRef("))
        );
        assert!(
            render_ir
                .states
                .iter()
                .flat_map(|s| s.gotos.iter())
                .any(|g| g.contains("new LRGoto("))
        );
    }

    #[test]
    fn csharp_lalr_parse_table_section_render_ir_contains_state_blocks() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate_lr.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let parse_table = crate::calculate_lalr1_parse_table(&grammar_config)
            .unwrap()
            .0;

        let non_terminal_metadata = build_non_terminal_metadata_ir(&grammar_config);
        let terminals = grammar_config
            .cfg
            .get_ordered_terminals()
            .iter()
            .map(|(t, _, l, _)| (*t, l.clone()))
            .collect::<Vec<_>>();
        let terminal_labels = build_terminal_label_map(&terminals);

        let render_ir = build_csharp_lalr_parse_table_section_render_ir(
            &parse_table,
            &terminal_labels,
            &non_terminal_metadata.names,
        );

        assert!(!render_ir.action_rows.is_empty());
        assert_eq!(render_ir.state_rows.len(), parse_table.states.len());
        assert!(
            render_ir
                .state_rows
                .iter()
                .any(|s| s.contains("new LR1State("))
        );
    }

    #[test]
    fn csharp_llk_production_render_ir_contains_parse_type_sources() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let non_terminal_names = parser_model::ordered_non_terminal_names(&grammar_config);
        let production_model =
            parser_model::build_production_model(&grammar_config, &non_terminal_names).unwrap();

        let render_ir = build_csharp_llk_production_render_ir(&production_model);

        assert_eq!(render_ir.len(), 1);
        assert_eq!(render_ir[0].lhs_index, 0);
        assert_eq!(render_ir[0].symbols.len(), 2);
        assert_eq!(
            render_ir[0].symbols[0],
            format!("new(ParseType.T, {})", FIRST_USER_TOKEN)
        );
        assert_eq!(
            render_ir[0].symbols[1],
            format!("new(ParseType.C, {})", FIRST_USER_TOKEN + 1)
        );
    }

    #[test]
    fn csharp_non_terminal_name_rows_are_quoted_and_comma_terminated() {
        let render_ir =
            build_csharp_non_terminal_names_render_ir(&["Start".to_string(), "Expr".to_string()]);

        assert_eq!(
            render_ir.rows,
            vec!["\"Start\",".to_string(), "\"Expr\",".to_string()]
        );
    }
}
