use crate::{GrammarConfig, StrVec, Symbol, Terminal};
use std::fmt::Debug;

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/nt_grammar_graph.dot"]
struct NtDotElements<'a> {
    title: &'a str,
    start_symbol: &'a str,
    productions: StrVec,
    terminals: StrVec,
    non_terminal_types: StrVec,
    non_terminal_instances: StrVec,
    non_terminal_to_production_edges: StrVec,
    inside_production_edges: StrVec,
    instances_to_types_edges: StrVec,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Formats the given Cfg in a special dot-format.
/// The basic graph type here resembles the same as the NtGrammarGraph type
/// used for detection of left-recursions.
/// The outputted dot-format is more suitable for this grammar graph type than
/// the dot format provides by the PetGraph itself.
///
pub fn render_nt_dot_string(grammar_config: &GrammarConfig) -> String {
    let cfg = &grammar_config.cfg;
    let non_terminal_positions = cfg.get_non_terminal_positions();
    let start_symbol = &cfg.st;

    let mut terminals = StrVec::new(4);
    cfg.get_terminal_positions().iter().for_each(|(p, t)| {
        let t_string = format!("{}", t);
        terminals.push(format!(
            "\"t{}_{}\" [label=\"{}\"];",
            p.pr_index(),
            p.sy_index(),
            t_string.escape_default()
        ));
    });

    let mut non_terminal_instances = StrVec::new(4);
    non_terminal_positions.iter().for_each(|(p, t)| {
        non_terminal_instances.push(format!(
            "\"n{}_{}\" [label=\"{}\"];",
            p.pr_index(),
            p.sy_index(),
            t
        ));
    });

    let mut non_terminal_types = StrVec::new(4);
    cfg.get_non_terminal_set().iter().for_each(|n| {
        non_terminal_types.push(format!("\"{}\";", n));
    });

    let mut productions = StrVec::new(4);
    let mut non_terminal_to_production_edges = StrVec::new(4);
    let mut inside_production_edges = StrVec::new(4);
    let mut instances_to_types_edges = StrVec::new(4);
    cfg.pr.iter().enumerate().for_each(|(pi, p)| {
        // Add nodes for every production
        let p_string = format!("{}", p);
        productions.push(format!(
            "\"{}\" [label=\"{}: {}\"];",
            pi,
            pi,
            p_string.escape_default()
        ));

        // Add edges from LHS non-terminals to their productions
        non_terminal_to_production_edges.push(format!("edge [label=\"{}\"];", pi));
        non_terminal_to_production_edges.push(format!("\"n{}_0\"->\"{}\";", pi, pi));

        // Add edges from LHS non-terminal type to their instance
        instances_to_types_edges.push(format!("\"{}\"->\"n{}_0\";", p.get_n(), pi));

        // Add edges within right-hand-sides of productions
        let mut from_node = format!("{}", pi);
        inside_production_edges.push(format!("edge [label=\"{}\"];", pi));
        p.get_r().iter().enumerate().for_each(|(si, s)| match s {
            Symbol::N(n, _) => {
                let to_node = format!("n{}_{}", pi, si + 1);
                inside_production_edges.push(format!("\"{}\"->\"{}\";", from_node, to_node));

                // Add edge from RHS non-terminal instance to its type
                instances_to_types_edges.push(format!("\"{}\"->\"{}\";", to_node, n));
                from_node = to_node;
            }
            Symbol::T(Terminal::Trm(_, _)) | Symbol::T(Terminal::End) => {
                let to_node = format!("t{}_{}", pi, si + 1);
                inside_production_edges.push(format!("\"{}\"->\"{}\";", from_node, to_node));
                from_node = to_node;
            }
            _ => panic!("Invalid symbol type on RHS of production"),
        });
    });

    let elements = NtDotElements {
        title: &grammar_config.title.clone().unwrap_or_default(),
        start_symbol,
        productions,
        terminals,
        non_terminal_types,
        non_terminal_instances,
        non_terminal_to_production_edges,
        inside_production_edges,
        instances_to_types_edges,
    };
    format!("{}", elements)
}

#[cfg(test)]
mod test {
    use crate::conversions::dot::render_nt_dot_string;
    use crate::{Cfg, GrammarConfig, Pr, ScannerConfig, Symbol};
    use regex::Regex;

    #[test]
    fn check_dot_format() {
        let rx_newline: Regex = Regex::new(r"\r\n|\r\n").unwrap();
        let g = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("S", vec![Symbol::t("a", vec![0]), Symbol::n("X")]))
            .add_pr(Pr::new("X", vec![Symbol::t("b", vec![0]), Symbol::n("S")]))
            .add_pr(Pr::new(
                "X",
                vec![
                    Symbol::t("a", vec![0]),
                    Symbol::n("Y"),
                    Symbol::t("b", vec![0]),
                    Symbol::n("Y"),
                ],
            ))
            .add_pr(Pr::new(
                "Y",
                vec![Symbol::t("b", vec![0]), Symbol::t("a", vec![0])],
            ))
            .add_pr(Pr::new("Y", vec![Symbol::t("a", vec![0]), Symbol::n("Z")]))
            .add_pr(Pr::new(
                "Z",
                vec![Symbol::t("a", vec![0]), Symbol::n("Z"), Symbol::n("X")],
            ));

        let title = Some("Test grammar".to_owned());
        let comment = Some("A simple grammar".to_owned());

        let scanner_config = ScannerConfig::default()
            .with_line_comments(vec!["//".to_owned()])
            .with_block_comments(vec![(r#"/\*"#.to_owned(), r#"\*/"#.to_owned())]);

        let grammar_config = GrammarConfig::new(g, 1)
            .with_title(title)
            .with_comment(comment)
            .add_scanner(scanner_config);

        let dot_str = render_nt_dot_string(&grammar_config);
        let expected = r#"digraph G {
    rankdir=LR;
    label="Test grammar";

    // S T A R T   S Y M B O L
    node [shape=point, style=invis]; ""
    node [shape=ellipse, color=cyan, style=solid];
    "" -> "S"

    // P R O D U C T I O N S
    node [shape=rectangle, color=green];
    "0" [label="0: S: \"a\" X;"];
    "1" [label="1: X: \"b\" S;"];
    "2" [label="2: X: \"a\" Y \"b\" Y;"];
    "3" [label="3: Y: \"b\" \"a\";"];
    "4" [label="4: Y: \"a\" Z;"];
    "5" [label="5: Z: \"a\" Z X;"];

    // T E R M I N A L S
    node [shape=diamond, color=blue];
    "t0_1" [label="\"a\""];
    "t1_1" [label="\"b\""];
    "t2_1" [label="\"a\""];
    "t2_3" [label="\"b\""];
    "t3_1" [label="\"b\""];
    "t3_2" [label="\"a\""];
    "t4_1" [label="\"a\""];
    "t5_1" [label="\"a\""];

    // N O N - T E R M I N A L S
    // TYPES
    node [shape=ellipse, color=cyan];
    "S";
    "X";
    "Y";
    "Z";

    // INSTANCES
    node [color=red];
    "n0_0" [label="S"];
    "n0_2" [label="X"];
    "n1_0" [label="X"];
    "n1_2" [label="S"];
    "n2_0" [label="X"];
    "n2_2" [label="Y"];
    "n2_4" [label="Y"];
    "n3_0" [label="Y"];
    "n4_0" [label="Y"];
    "n4_2" [label="Z"];
    "n5_0" [label="Z"];
    "n5_2" [label="Z"];
    "n5_3" [label="X"];

    // E D G E S
    // Nt TO PRODUCTIONS
    edge [color=blue];
    edge [label="0"];
    "n0_0"->"0";
    edge [label="1"];
    "n1_0"->"1";
    edge [label="2"];
    "n2_0"->"2";
    edge [label="3"];
    "n3_0"->"3";
    edge [label="4"];
    "n4_0"->"4";
    edge [label="5"];
    "n5_0"->"5";

    // INSIDE PRODUCTIONS
    edge [color=green, fontcolor=green];
    edge [label="0"];
    "0"->"t0_1";
    "t0_1"->"n0_2";
    edge [label="1"];
    "1"->"t1_1";
    "t1_1"->"n1_2";
    edge [label="2"];
    "2"->"t2_1";
    "t2_1"->"n2_2";
    "n2_2"->"t2_3";
    "t2_3"->"n2_4";
    edge [label="3"];
    "3"->"t3_1";
    "t3_1"->"t3_2";
    edge [label="4"];
    "4"->"t4_1";
    "t4_1"->"n4_2";
    edge [label="5"];
    "5"->"t5_1";
    "t5_1"->"n5_2";
    "n5_2"->"n5_3";

    // Nt INSTANCES <=> Nt TYPES
    edge [color=cyan, label=""];
    "S"->"n0_0";
    "n0_2"->"X";
    "X"->"n1_0";
    "n1_2"->"S";
    "X"->"n2_0";
    "n2_2"->"Y";
    "n2_4"->"Y";
    "Y"->"n3_0";
    "Y"->"n4_0";
    "n4_2"->"Z";
    "Z"->"n5_0";
    "n5_2"->"Z";
    "n5_3"->"X";
}
"#;
        assert_eq!(
            rx_newline.replace_all(expected, "\n"),
            rx_newline.replace_all(&dot_str, "\n")
        );
    }
}
