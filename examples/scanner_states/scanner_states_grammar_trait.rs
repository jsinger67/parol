// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

use crate::scanner_states_grammar::ScannerStatesGrammar;
use id_tree::Tree;
use parol_runtime::errors::*;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType, ScannerAccess, UserActionsTrait};
use std::cell::RefMut;

///
/// The `ScannerStatesGrammarTrait` trait is automatically generated for the
/// given grammar.
/// All functions have default implementations.
///
pub trait ScannerStatesGrammarTrait {
    /// Semantic action for production 0:
    ///
    /// Start: StartRest;
    ///
    fn start_0(
        &mut self,
        _start_rest_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 1:
    ///
    /// Start: ;
    ///
    fn start_1(
        &mut self,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 2:
    ///
    /// StartRest: Content StartRestSuffix;
    ///
    fn start_rest_2(
        &mut self,
        _content_0: &ParseTreeStackEntry,
        _start_rest_suffix_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 3:
    ///
    /// StartRestSuffix: StartRest;
    ///
    fn start_rest_suffix_3(
        &mut self,
        _start_rest_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// StartRestSuffix: ;
    ///
    fn start_rest_suffix_4(
        &mut self,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// Content: Identifier;
    ///
    fn content_5(
        &mut self,
        _identifier_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// Content: StringDelimiter StringContent StringDelimiter;
    ///
    fn content_6(
        &mut self,
        _string_delimiter_0: &ParseTreeStackEntry,
        _string_content_1: &ParseTreeStackEntry,
        _string_delimiter_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 7:
    ///
    /// StringContent: StringContentRest;
    ///
    fn string_content_7(
        &mut self,
        _string_content_rest_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 8:
    ///
    /// StringContent: ;
    ///
    fn string_content_8(
        &mut self,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 9:
    ///
    /// StringContentRest: StringContentRestGroup StringContentRest;
    ///
    fn string_content_rest_9(
        &mut self,
        _string_content_rest_group_0: &ParseTreeStackEntry,
        _string_content_rest_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// StringContentRestGroup: NoneQuote;
    ///
    fn string_content_rest_group_10(
        &mut self,
        _none_quote_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// StringContentRestGroup: EscapedLineEnd;
    ///
    fn string_content_rest_group_11(
        &mut self,
        _escaped_line_end_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 12:
    ///
    /// StringContentRestGroup: Escaped;
    ///
    fn string_content_rest_group_12(
        &mut self,
        _escaped_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 13:
    ///
    /// StringContentRest: StringContentRestGroup1;
    ///
    fn string_content_rest_13(
        &mut self,
        _string_content_rest_group1_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 14:
    ///
    /// StringContentRestGroup1: NoneQuote;
    ///
    fn string_content_rest_group1_14(
        &mut self,
        _none_quote_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 15:
    ///
    /// StringContentRestGroup1: EscapedLineEnd;
    ///
    fn string_content_rest_group1_15(
        &mut self,
        _escaped_line_end_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 16:
    ///
    /// StringContentRestGroup1: Escaped;
    ///
    fn string_content_rest_group1_16(
        &mut self,
        _escaped_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// Identifier: "[a-zA-Z_]\w*";
    ///
    fn identifier_17(
        &mut self,
        _end_of_input_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 18:
    ///
    /// Escaped: "\\[\\bft]";
    ///
    fn escaped_18(
        &mut self,
        _newline_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 19:
    ///
    /// EscapedLineEnd: "\\[\s*]\r?\n";
    ///
    fn escaped_line_end_19(
        &mut self,
        _whitespace_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 20:
    ///
    /// NoneQuote: "[^\u{22}]";
    ///
    fn none_quote_20(
        &mut self,
        _line_comment_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 21:
    ///
    /// StringDelimiter: "\u{22}";
    ///
    fn string_delimiter_21(
        &mut self,
        _block_comment_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        Ok(())
    }
}

impl UserActionsTrait for ScannerStatesGrammar {
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeStackEntry],
        parse_tree: &Tree<ParseTreeType>,
        scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        match prod_num {
            0 => self.start_0(&children[0], parse_tree, scanner_access),

            1 => self.start_1(parse_tree, scanner_access),

            2 => self.start_rest_2(&children[0], &children[1], parse_tree, scanner_access),

            3 => self.start_rest_suffix_3(&children[0], parse_tree, scanner_access),

            4 => self.start_rest_suffix_4(parse_tree, scanner_access),

            5 => self.content_5(&children[0], parse_tree, scanner_access),

            6 => self.content_6(
                &children[0],
                &children[1],
                &children[2],
                parse_tree,
                scanner_access,
            ),

            7 => self.string_content_7(&children[0], parse_tree, scanner_access),

            8 => self.string_content_8(parse_tree, scanner_access),

            9 => self.string_content_rest_9(&children[0], &children[1], parse_tree, scanner_access),

            10 => self.string_content_rest_group_10(&children[0], parse_tree, scanner_access),

            11 => self.string_content_rest_group_11(&children[0], parse_tree, scanner_access),

            12 => self.string_content_rest_group_12(&children[0], parse_tree, scanner_access),

            13 => self.string_content_rest_13(&children[0], parse_tree, scanner_access),

            14 => self.string_content_rest_group1_14(&children[0], parse_tree, scanner_access),

            15 => self.string_content_rest_group1_15(&children[0], parse_tree, scanner_access),

            16 => self.string_content_rest_group1_16(&children[0], parse_tree, scanner_access),

            17 => self.identifier_17(&children[0], parse_tree, scanner_access),

            18 => self.escaped_18(&children[0], parse_tree, scanner_access),

            19 => self.escaped_line_end_19(&children[0], parse_tree, scanner_access),

            20 => self.none_quote_20(&children[0], parse_tree, scanner_access),

            21 => self.string_delimiter_21(&children[0], parse_tree, scanner_access),

            _ => panic!("Unhandled production number: {}", prod_num),
        }
    }
}
