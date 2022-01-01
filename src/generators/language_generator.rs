use crate::{Cfg, Symbol, Terminal};
use log::trace;
use miette::{miette, ErrReport, Result};
use rand::Rng;
//use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug)]
pub struct LanguageGenerator<'a> {
    generator_stack: Vec<Symbol>,
    cfg: &'a Cfg,
    //cache: HashMap<&'a str, rand_regex::Regex>,
}

impl<'a> LanguageGenerator<'a> {
    pub fn new(cfg: &'a Cfg) -> Self {
        Self {
            generator_stack: Vec::new(),
            cfg,
            // cache: HashMap::new(),
        }
    }

    pub fn generate(&mut self, max_repeat: u32) -> Result<String> {
        let mut result = String::new();
        self.process_non_terminal(self.cfg.get_start_symbol())?;
        while let Some(symbol) = self.generator_stack.pop() {
            match symbol {
                Symbol::N(n) => self.process_non_terminal(&n),
                Symbol::T(Terminal::Trm(t, _)) => {
                    self.process_terminal(&t, &mut result, max_repeat)
                }
                _ => Ok(()),
            }?
        }
        Ok(result)
    }

    fn process_non_terminal(&mut self, non_terminal: &str) -> Result<()> {
        let productions_of_nt = self.cfg.matching_productions(non_terminal);
        let chosen_index = rand::thread_rng().gen_range(0..productions_of_nt.len());
        trace!(
            "/* {} */ {} {}/{}",
            productions_of_nt[chosen_index].0,
            productions_of_nt[chosen_index].1,
            chosen_index + 1,
            productions_of_nt.len()
        );
        productions_of_nt[chosen_index]
            .1
            .get_r()
            .iter()
            .rev()
            .for_each(|s| self.generator_stack.push(s.clone()));
        Ok(())
    }

    fn to_miette<T: Display>(e: T) -> ErrReport {
        miette!("{}", e)
    }

    fn process_terminal(
        &mut self,
        terminal: &str,
        result: &mut String,
        max_repeat: u32,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();
        regex_syntax::ParserBuilder::new()
            .build()
            .parse(terminal)
            .map_err(Self::to_miette)
            .and_then(|utf8_hir| {
                rand_regex::Regex::with_hir(utf8_hir, max_repeat)
                    .map_err(Self::to_miette)
                    .and_then(|utf8_gen| {
                        let generated = rng.sample::<String, _>(&utf8_gen);
                        trace!("gen: {}", generated);
                        result.push_str(&generated);
                        result.push_str(" ");
                        Ok(())
                    })
            })
    }

    // fn get_regex(&mut self, terminal: &'a str, max_repeat: u32) -> Result<&rand_regex::Regex> {
    //     if let Some(regex) = self.cache.get(terminal) {
    //         Ok(regex)
    //     } else {
    //         regex_syntax::ParserBuilder::new()
    //             .build()
    //             .parse(terminal)
    //             .map_err(Self::to_miette)
    //             .and_then(move |utf8_hir| {
    //                 rand_regex::Regex::with_hir(utf8_hir, max_repeat)
    //                     .map_err(Self::to_miette)
    //                     .and_then(|utf8_gen| {
    //                         self.cache.insert(terminal, utf8_gen);
    //                         Ok(self.cache.get(terminal).unwrap())
    //                     })
    //             })
    //     }
    // }
}
