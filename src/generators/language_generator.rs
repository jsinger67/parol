use crate::{Cfg, Pr, Symbol, Terminal};
use log::trace;
use miette::IntoDiagnostic;
use miette::{miette, Diagnostic, Result};
use rand::Rng;
use std::collections::HashMap;

const MAX_RESULT_SIZE: usize = 100000;
const MAX_REPEAT: u32 = 8;

/// Error "Generation does not terminate in good time"
#[derive(Error, Diagnostic, Debug)]
#[error("Stopping generation to prevent endless recursion at size {len}")]
#[diagnostic(
    help("Generation does not terminate in good time"),
    code("parol::generators::language_generator::source_size_exceeded")
)]
pub struct SourceSizeExceeded {
    len: usize,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Provides the possibility to generate random sentences of a given grammar.
///
#[derive(Debug)]
pub struct LanguageGenerator<'a> {
    generator_stack: Vec<Symbol>,
    cfg: &'a Cfg,
    cache: HashMap<String, rand_regex::Regex>,
}

impl<'a> LanguageGenerator<'a> {
    /// Creates a new item
    pub fn new(cfg: &'a Cfg) -> Self {
        Self {
            generator_stack: Vec::new(),
            cfg,
            cache: HashMap::new(),
        }
    }

    /// Generates a sentence
    pub fn generate(&mut self, max_result_length: Option<usize>) -> Result<String> {
        let mut result = String::new();
        let termination_threshold = max_result_length.unwrap_or(MAX_RESULT_SIZE) / 2;
        trace!(
            "Try to terminate at result length {}",
            termination_threshold
        );
        self.process_non_terminal(self.cfg.get_start_symbol(), false)?;
        while let Some(symbol) = self.generator_stack.pop() {
            match symbol {
                Symbol::N(n, _) => {
                    self.process_non_terminal(&n, result.len() > termination_threshold)
                }
                Symbol::T(Terminal::Trm(t, _)) => {
                    self.process_terminal(t.clone(), &mut result, max_result_length)
                }
                _ => Ok(()),
            }?
        }
        Ok(result)
    }

    fn process_non_terminal(&mut self, non_terminal: &str, terminate: bool) -> Result<()> {
        let productions_of_nt = self.cfg.matching_productions(non_terminal);
        let chosen_index = if terminate {
            Self::chose_minimal_expanding_production(&productions_of_nt)
        } else {
            rand::thread_rng().gen_range(0..productions_of_nt.len())
        };
        trace!(
            "/* {} */ {} {}/{} {}",
            productions_of_nt[chosen_index].0,
            productions_of_nt[chosen_index].1,
            chosen_index + 1,
            productions_of_nt.len(),
            if terminate { "term" } else { "" }
        );
        productions_of_nt[chosen_index]
            .1
            .get_r()
            .iter()
            .rev()
            .for_each(|s| self.generator_stack.push(s.clone()));
        Ok(())
    }

    fn process_terminal(
        &mut self,
        terminal: String,
        result: &mut String,
        max_result_length: Option<usize>,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();
        let utf8_gen = self.get_regex(terminal)?;
        let generated = rng.sample::<String, _>(&utf8_gen);
        trace!("gen: {}", generated);
        result.push_str(&generated);
        result.push(' ');
        let len = result.len();
        if len > max_result_length.unwrap_or(MAX_RESULT_SIZE) {
            Err(miette!(SourceSizeExceeded { len }))
        } else {
            Ok(())
        }
    }

    fn get_regex<'b, 'c>(&'b mut self, terminal: String) -> Result<&'c rand_regex::Regex>
    where
        'b: 'c,
    {
        let exist = self.cache.get(&terminal).is_some();

        if exist {
            let regex = self.cache.get(&terminal).unwrap();
            trace!("Reusing cached regex for: {}", terminal);
            return Ok(regex);
        }

        match regex_syntax::ParserBuilder::new().build().parse(&terminal) {
            Ok(utf8_hir) => {
                let utf8_gen =
                    rand_regex::Regex::with_hir(utf8_hir, MAX_REPEAT).into_diagnostic()?;
                trace!("Caching regex for: {}", terminal);
                self.cache.insert(terminal.clone(), utf8_gen);
                self.get_regex(terminal)
            }
            Err(err) => Err(miette!(err)),
        }
    }

    fn chose_minimal_expanding_production(productions_of_nt: &[(usize, &Pr)]) -> usize {
        // The strategy here is to select the production with least number of non-terminals on the
        // right-hand side.
        // This should force the generation process to stop eventually.
        let production_index = productions_of_nt
            .iter()
            .min_by(|(_, a), (_, b)| {
                let a_nt_count = a.get_r().iter().fold(0, |mut acc, s| {
                    if s.is_n() {
                        acc += 1
                    }
                    acc
                });
                let b_nt_count = b.get_r().iter().fold(0, |mut acc, s| {
                    if s.is_n() {
                        acc += 1
                    }
                    acc
                });
                a_nt_count.cmp(&b_nt_count)
            })
            .unwrap()
            .0;
        productions_of_nt
            .iter()
            .position(|(idx, _)| *idx == production_index)
            .unwrap()
    }
}
