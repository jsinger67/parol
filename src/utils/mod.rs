use crate::errors::*;
use crate::grammar::cfg::RX_NUM_SUFFIX;
use crate::parser::parol_grammar::ParolGrammar;
use crate::parser::parol_parser::parse;
use crate::GrammarConfig;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::hash::Hash;

pub mod str_vec;

/// Applies a key-generating function to each element of a vector and yields a vector of
/// pairs. Each pair consists of a unique key and a vector of all elements of the input
/// vector which did produce this key by applying the projection function.
/// The result vector is not sorted.
pub fn group_by<P, T, K>(data: &[T], projection: P) -> Vec<(K, Vec<T>)>
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
pub fn generate_name(exclusions: &[String], preferred_name: String) -> String {
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

// pub fn count_by<T, P>(data: &[T], pred: P) -> usize
// where
//     T: Eq,
//     P: Fn(&T) -> bool,
// {
//     data.iter().fold(0, |mut acc, e| {
//         if pred(e) {
//             acc += 1;
//         }
//         acc
//     })
// }

pub fn combine<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

pub fn short_cut_disjunction_combine<A, F, G>(f: F, g: G) -> impl Fn(&A) -> bool
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

pub fn short_cut_conjunction_combine<A, F, G>(f: F, g: G) -> impl Fn(&A) -> bool
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

pub fn obtain_cfg_ext(file_name: &str, verbose: bool) -> Result<GrammarConfig> {
    let input =
        fs::read_to_string(file_name).chain_err(|| format!("Can't read file {}", file_name))?;
    let mut parol_grammar = ParolGrammar::new();
    let _syntax_tree = parse(&input, file_name.to_owned(), &mut parol_grammar)
        .chain_err(|| format!("Failed parsing file {}", file_name))?;

    if verbose {
        println!("{}", parol_grammar);
    }

    GrammarConfig::try_from(parol_grammar)
}
