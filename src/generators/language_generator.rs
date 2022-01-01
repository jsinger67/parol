use crate::{Cfg, Symbol, Terminal};
use log::trace;
use miette::{miette, IntoDiagnostic, Result};
use rand::prelude::*;
use regex_generate::{Generator, DEFAULT_MAX_REPEAT};
use std::io::Write;

#[derive(Debug)]
pub struct LanguageGenerator<'a> {
    generator_stack: Vec<Symbol>,
    cfg: &'a Cfg,
}

impl<'a> LanguageGenerator<'a> {
    pub fn new(cfg: &'a Cfg) -> Self {
        Self {
            generator_stack: Vec::new(),
            cfg,
        }
    }

    pub fn generate<W>(&mut self, buffer: &mut W, max_repeat: u32) -> Result<()>
    where
        W: Write,
    {
        self.process_non_terminal(self.cfg.get_start_symbol())?;
        while let Some(symbol) = self.generator_stack.pop() {
            match symbol {
                Symbol::N(n) => self.process_non_terminal(&n),
                Symbol::T(Terminal::Trm(t, _)) => self.process_terminal(&t, buffer, max_repeat),
                _ => Ok(()),
            }?
        }
        Ok(())
    }

    fn process_non_terminal(&mut self, non_terminal: &str) -> Result<()> {
        let productions_of_nt = self.cfg.matching_productions(non_terminal);
        let chosen_index = rand::thread_rng().gen_range(0, productions_of_nt.len());
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

    fn process_terminal<W>(&mut self, terminal: &str, buffer: &mut W, max_repeat: u32) -> Result<()>
    where
        W: Write,
    {
        let mut gen = Generator::new(
            terminal,
            ThreadRng::default(),
            std::cmp::max(max_repeat, DEFAULT_MAX_REPEAT),
        )
        .map_err(|error| miette!("{}", error))?;
        gen.generate(buffer).map_err(|error| miette!("{}", error))?;
        buffer.write(b" ").into_diagnostic()?;
        Ok(())
    }
}
