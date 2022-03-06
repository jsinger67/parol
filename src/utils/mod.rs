use crate::grammar::cfg::RX_NUM_SUFFIX;
use crate::parser::parol_grammar::ParolGrammar;
use crate::parser::parol_parser::parse;
use crate::GrammarConfig;
use id_tree::Tree;
use id_tree_layout::Layouter;
use miette::{IntoDiagnostic, Result, WrapErr};
use parol_runtime::parser::ParseTreeType;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::path::Path;

pub mod str_vec;

/// Applies a key-generating function to each element of a vector and yields a vector of
/// pairs. Each pair consists of a unique key and a vector of all elements of the input
/// vector which did produce this key by applying the projection function.
/// The result vector is not sorted.
pub(crate) fn group_by<P, T, K>(data: &[T], projection: P) -> Vec<(K, Vec<T>)>
where
    P: Fn(&T) -> K,
    K: Eq + Hash,
    T: Clone,
{
    let mut grouping: HashMap<K, Vec<T>> = HashMap::new();
    data.iter()
        .fold(&mut grouping, |acc, t| {
            let key = projection(t);
            if let Some(vt) = acc.get_mut(&key) {
                vt.push(t.clone());
            } else {
                acc.insert(key, vec![t.clone()]);
            }
            acc
        })
        .drain()
        .collect()
}

/// Generates a new unique name avoiding collisions with the names given in the 'exclusions'.
/// It takes a preferred name and if it collides it adds an increasing suffix number.
/// If the preferred name already has a suffix number it starts counting up from this number.
pub(crate) fn generate_name(exclusions: &[String], preferred_name: String) -> String {
    fn gen_name(exclusions: &[String], prefix: String, start_num: usize) -> String {
        let mut num = start_num;
        let mut new_name = format!("{}{}", prefix, num);
        while exclusions.contains(&new_name) {
            num += 1;
            new_name = format!("{}{}", prefix, num);
        }
        new_name
    }

    if exclusions.contains(&preferred_name) {
        let (suffix_number, prefix) = {
            if let Some(match_) = RX_NUM_SUFFIX.find(&preferred_name) {
                let num = match_.as_str().parse::<usize>().unwrap_or(1);
                (num, preferred_name[0..match_.start()].to_string())
            } else {
                (1, preferred_name.clone())
            }
        };
        gen_name(exclusions, prefix, suffix_number)
    } else {
        preferred_name
    }
}

pub(crate) fn combine<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

pub(crate) fn short_cut_disjunction_combine<A, F, G>(f: F, g: G) -> impl Fn(&A) -> bool
where
    F: Fn(&A) -> bool,
    G: Fn(&A) -> bool,
{
    move |x| {
        let r = f(x);
        if r {
            r
        } else {
            g(x)
        }
    }
}

pub(crate) fn short_cut_conjunction_combine<A, F, G>(f: F, g: G) -> impl Fn(&A) -> bool
where
    F: Fn(&A) -> bool,
    G: Fn(&A) -> bool,
{
    move |x| {
        let r = f(x);
        if !r {
            r
        } else {
            g(x)
        }
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Utility function to parse a file with a grammar in PAR syntax.
///
pub fn obtain_grammar_config<T>(file_name: T, verbose: bool) -> Result<GrammarConfig>
where
    T: AsRef<Path> + Debug,
{
    let input = fs::read_to_string(&file_name)
        .into_diagnostic()
        .wrap_err(format!("Can't read file {:?}", file_name))?;
    obtain_grammar_config_from_string(&input, verbose)
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Utility function to parse a text with a grammar in PAR syntax.
///
pub fn obtain_grammar_config_from_string(input: &str, verbose: bool) -> Result<GrammarConfig> {
    let mut parol_grammar = ParolGrammar::new();
    let _syntax_tree = parse(input, "No file", &mut parol_grammar)
        .wrap_err(format!("Failed parsing text {}", input))?;

    if verbose {
        println!("{}", parol_grammar);
    }

    GrammarConfig::try_from(parol_grammar)
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Utility function for generating tree layouts
///  
pub fn generate_tree_layout<T>(syntax_tree: &Tree<ParseTreeType>, input_file_name: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let mut svg_full_file_name = input_file_name.as_ref().to_path_buf();
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(std::path::Path::new(&svg_full_file_name))
        .write()
        .into_diagnostic()
        .wrap_err("Failed writing layout")
}
