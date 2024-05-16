use crate::scanner_states_grammar_trait::{
    Content, ScannerStatesGrammarTrait, Start, StringContent, StringElement,
};
#[allow(unused_imports)]
use parol_runtime::{Result, Token};
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our Start grammar
///
#[derive(Debug, Default)]
pub struct ScannerStatesGrammar<'t> {
    pub start: Option<Start<'t>>,
}

impl ScannerStatesGrammar<'_> {
    pub fn new() -> Self {
        ScannerStatesGrammar::default()
    }
}

impl Display for Start<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        for cont in &self.start_list {
            writeln!(f, "{}", cont.content)?;
        }
        Ok(())
    }
}

impl Display for Content<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self {
            Content::Identifier(i) => write!(f, "{}", i.identifier.identifier.text()),
            Content::StringDelimiterStringContentStringDelimiter(s) => {
                write!(f, r#""{}""#, s.string_content)
            }
        }
    }
}

impl Display for StringContent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        if let StringContent::StringContentList(list) = &self {
            for elem in &list.string_content_list {
                write!(f, "{}", elem.string_element)?;
            }
        }
        Ok(())
    }
}

impl Display for StringElement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self {
            StringElement::Escaped(e) => write!(f, "\\{}", e.escaped.escaped.text()),
            StringElement::EscapedLineEnd(_) => write!(f, "\\"),
            StringElement::NoneQuote(c) => write!(f, "{}", c.none_quote.none_quote.text()),
        }
    }
}

impl Display for ScannerStatesGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.start {
            Some(start) => writeln!(f, "{}", start),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> ScannerStatesGrammarTrait<'t> for ScannerStatesGrammar<'t> {
    /// Semantic action for non-terminal 'Start'
    fn start(&mut self, arg: &Start<'t>) -> Result<()> {
        self.start = Some(arg.clone());
        Ok(())
    }
}
