// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

// Disable clippy warnings that can result in the way how parol generates code.
#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]

use crate::scanner_states_grammar::ScannerStatesGrammar;
use parol_runtime::parser::{ParseTreeType, UserActionsTrait};
use parol_runtime::{ParserError, Result, Token};
///
/// The `ScannerStatesGrammarTrait` trait is automatically generated for the
/// given grammar.
/// All functions have default implementations.
///
pub trait ScannerStatesGrammarTrait {
    /// Semantic action for production 0:
    ///
    /// `Start: StartList /* Vec */;`
    ///
    fn start(&mut self, _start_list: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 1:
    ///
    /// `StartList /* Vec<T>::Push */: Content StartList;`
    ///
    fn start_list_0(
        &mut self,
        _content: &ParseTreeType,
        _start_list: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 2:
    ///
    /// `StartList /* Vec<T>::New */: ;`
    ///
    fn start_list_1(&mut self) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 3:
    ///
    /// `Content: Identifier;`
    ///
    fn content_0(&mut self, _identifier: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// `Content: StringDelimiter %push(String) StringContent StringDelimiter %pop();`
    ///
    fn content_1(
        &mut self,
        _string_delimiter: &ParseTreeType,
        _string_content: &ParseTreeType,
        _string_delimiter0: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// `StringContent: StringElement StringContent;`
    ///
    fn string_content_0(
        &mut self,
        _string_element: &ParseTreeType,
        _string_content: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// `StringContent: ;`
    ///
    fn string_content_1(&mut self) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 7:
    ///
    /// `StringElement: Escaped;`
    ///
    fn string_element_0(&mut self, _escaped: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 8:
    ///
    /// `StringElement: EscapedLineEnd;`
    ///
    fn string_element_1(&mut self, _escaped_line_end: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 9:
    ///
    /// `StringElement: NoneQuote;`
    ///
    fn string_element_2(&mut self, _none_quote: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// `Identifier: /[a-zA-Z_]\w*/;`
    ///
    fn identifier(&mut self, _identifier: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// `Escaped: <String>/\["\\bfnt]/;`
    ///
    fn escaped(&mut self, _escaped: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 12:
    ///
    /// `EscapedLineEnd: <String>/\[\s--\n\r]*\r?\n/;`
    ///
    fn escaped_line_end(&mut self, _escaped_line_end: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 13:
    ///
    /// `NoneQuote: <String>/[^"\\]+/;`
    ///
    fn none_quote(&mut self, _none_quote: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 14:
    ///
    /// `StringDelimiter: <INITIAL, String>/"/;`
    ///
    fn string_delimiter(&mut self, _string_delimiter: &ParseTreeType) -> Result<()> {
        Ok(())
    }
}

impl UserActionsTrait<'_> for ScannerStatesGrammar {
    ///
    /// This function is implemented automatically for the user's item ScannerStatesGrammar.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeType],
    ) -> Result<()> {
        match prod_num {
            0 => self.start(&children[0]),
            1 => self.start_list_0(&children[0], &children[1]),
            2 => self.start_list_1(),
            3 => self.content_0(&children[0]),
            4 => self.content_1(&children[0], &children[1], &children[2]),
            5 => self.string_content_0(&children[0], &children[1]),
            6 => self.string_content_1(),
            7 => self.string_element_0(&children[0]),
            8 => self.string_element_1(&children[0]),
            9 => self.string_element_2(&children[0]),
            10 => self.identifier(&children[0]),
            11 => self.escaped(&children[0]),
            12 => self.escaped_line_end(&children[0]),
            13 => self.none_quote(&children[0]),
            14 => self.string_delimiter(&children[0]),
            _ => Err(ParserError::InternalError(format!(
                "Unhandled production number: {}",
                prod_num
            ))
            .into()),
        }
    }
    fn on_comment_parsed(&mut self, _token: Token<'_>) {
        // This is currently only supported for auto generate mode.
        // Please, file an issue if need arises.
    }
}
