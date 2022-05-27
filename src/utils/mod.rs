use crate::grammar::cfg::RX_NUM_SUFFIX;
use crate::parser::parol_grammar::ParolGrammar;
use crate::parser::parol_parser::parse;
use crate::GrammarConfig;
use id_tree::Tree;
use id_tree_layout::Layouter;
use miette::{IntoDiagnostic, Result, WrapErr};
use parol_runtime::parser::ParseTreeType;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

pub mod str_vec;

/// Generates a new unique name avoiding collisions with the names given in the 'exclusions'.
/// It takes a preferred name and if it collides it adds an increasing suffix number.
/// If the preferred name already has a suffix number it starts counting up from this number.
pub(crate) fn generate_name<T>(exclusions: &[T], preferred_name: String) -> String
where
    T: AsRef<str>,
{
    fn gen_name<T>(exclusions: &[T], prefix: String, start_num: usize) -> String
    where
        T: AsRef<str>,
    {
        let mut num = start_num;
        let mut new_name = format!("{}{}", prefix, num);
        while exclusions.iter().any(|n| n.as_ref() == new_name) {
            num += 1;
            new_name = format!("{}{}", prefix, num);
        }
        new_name
    }

    if exclusions.iter().any(|n| n.as_ref() == preferred_name) {
        let (suffix_number, prefix) = {
            if let Some(match_) = RX_NUM_SUFFIX.find(&preferred_name) {
                let num = match_.as_str().parse::<usize>().unwrap_or(1);
                (num, preferred_name[0..match_.start()].to_string())
            } else {
                (0, preferred_name.clone())
            }
        };
        gen_name(exclusions, prefix, suffix_number)
    } else {
        preferred_name
    }
}

/// Generates a new function h that is the composition of the two given functions f and g.
/// The result is h = f ∘ g, i.e h(x) = g(f(x)).
/// The result type of the first function f and the input type of the second function g must be the
/// same to make the functions combinable.
pub(crate) fn combine<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

/// Optimized combinator on boolean functions
/// Applies short cut disjunction
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

/// Optimized combinator on boolean functions
/// Applies short cut conjunction
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
        .wrap_err(format!("Failed parsing text {}", input.escape_default()))?;

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
