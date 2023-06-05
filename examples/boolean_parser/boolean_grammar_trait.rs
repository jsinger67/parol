// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

// Disable clippy warnings that can result in the way how parol generates code.
#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]

use crate::boolean_grammar::BooleanGrammar;
use parol_runtime::parser::{ParseTreeType, UserActionsTrait};
use parol_runtime::{ParserError, Result, Token};
///
/// The `BooleanGrammarTrait` trait is automatically generated for the
/// given grammar.
/// All functions have default implementations.
///
pub trait BooleanGrammarTrait {
    /// Semantic action for production 0:
    ///
    /// Expressions: Expression ExpressionsList /* Vec */ ExpressionsOpt /* Option */;
    ///
    fn expressions(
        &mut self,
        _expression: &ParseTreeType,
        _expressions_list: &ParseTreeType,
        _expressions_opt: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 1:
    ///
    /// ExpressionsList /* `Vec<T>::Push` */: Semicolon Expression ExpressionsList;
    ///
    fn expressions_list_0(
        &mut self,
        _semicolon: &ParseTreeType,
        _expression: &ParseTreeType,
        _expressions_list: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 2:
    ///
    /// ExpressionsList /* `Vec<T>::New` */: ;
    ///
    fn expressions_list_1(&mut self) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 3:
    ///
    /// ExpressionsOpt /* `Option<T>::Some` */: Semicolon;
    ///
    fn expressions_opt_0(&mut self, _semicolon: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// ExpressionsOpt /* `Option<T>::None` */: ;
    ///
    fn expressions_opt_1(&mut self) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// Expression: Term TailExpression;
    ///
    fn expression(
        &mut self,
        _term: &ParseTreeType,
        _tail_expression: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// TailExpression: TailExpressionList /* Vec */;
    ///
    fn tail_expression(&mut self, _tail_expression_list: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 7:
    ///
    /// TailExpressionList /* `Vec<T>::Push` */: BinaryOperator Term TailExpressionList;
    ///
    fn tail_expression_list_0(
        &mut self,
        _binary_operator: &ParseTreeType,
        _term: &ParseTreeType,
        _tail_expression_list: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 8:
    ///
    /// TailExpressionList /* `Vec<T>::New` */: ;
    ///
    fn tail_expression_list_1(&mut self) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 9:
    ///
    /// Term: TermOpt /* Option */ Factor;
    ///
    fn term(&mut self, _term_opt: &ParseTreeType, _factor: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// TermOpt /* `Option<T>::Some` */: UnaryOperator;
    ///
    fn term_opt_0(&mut self, _unary_operator: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// TermOpt /* `Option<T>::None` */: ;
    ///
    fn term_opt_1(&mut self) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 12:
    ///
    /// Boolean: True;
    ///
    fn boolean_0(&mut self, _true: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 13:
    ///
    /// Boolean: False;
    ///
    fn boolean_1(&mut self, _false: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 14:
    ///
    /// UnaryOperator: Not;
    ///
    fn unary_operator(&mut self, _not: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 15:
    ///
    /// BinaryOperator: AndOp;
    ///
    fn binary_operator_0(&mut self, _and_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 16:
    ///
    /// BinaryOperator: OrOp;
    ///
    fn binary_operator_1(&mut self, _or_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// BinaryOperator: XorOp;
    ///
    fn binary_operator_2(&mut self, _xor_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 18:
    ///
    /// BinaryOperator: NorOp;
    ///
    fn binary_operator_3(&mut self, _nor_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 19:
    ///
    /// BinaryOperator: NandOp;
    ///
    fn binary_operator_4(&mut self, _nand_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 20:
    ///
    /// BinaryOperator: XnorOp;
    ///
    fn binary_operator_5(&mut self, _xnor_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 21:
    ///
    /// AndOp: "(?i)AND";
    ///
    fn and_op(&mut self, _and_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 22:
    ///
    /// OrOp: "(?i)OR";
    ///
    fn or_op(&mut self, _or_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 23:
    ///
    /// XorOp: "(?i)XOR";
    ///
    fn xor_op(&mut self, _xor_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 24:
    ///
    /// NorOp: "(?i)NOR";
    ///
    fn nor_op(&mut self, _nor_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 25:
    ///
    /// NandOp: "(?i)NAND";
    ///
    fn nand_op(&mut self, _nand_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 26:
    ///
    /// XnorOp: "(?i)XNOR";
    ///
    fn xnor_op(&mut self, _xnor_op: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 27:
    ///
    /// True: "(?i)TRUE";
    ///
    fn r#true(&mut self, _true: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 28:
    ///
    /// False: "(?i)FALSE";
    ///
    fn r#false(&mut self, _false: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 29:
    ///
    /// Not: "(?i)NOT";
    ///
    fn not(&mut self, _not: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 30:
    ///
    /// Parenthesized: LeftParenthesis Expression RightParenthesis;
    ///
    fn parenthesized(
        &mut self,
        _left_parenthesis: &ParseTreeType,
        _expression: &ParseTreeType,
        _right_parenthesis: &ParseTreeType,
    ) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 31:
    ///
    /// Semicolon: ";";
    ///
    fn semicolon(&mut self, _semicolon: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 32:
    ///
    /// LeftParenthesis: "\(";
    ///
    fn left_parenthesis(&mut self, _left_parenthesis: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 33:
    ///
    /// RightParenthesis: "\)";
    ///
    fn right_parenthesis(&mut self, _right_parenthesis: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 34:
    ///
    /// Factor: Boolean;
    ///
    fn factor_0(&mut self, _boolean: &ParseTreeType) -> Result<()> {
        Ok(())
    }

    /// Semantic action for production 35:
    ///
    /// Factor: Parenthesized;
    ///
    fn factor_1(&mut self, _parenthesized: &ParseTreeType) -> Result<()> {
        Ok(())
    }
}

impl UserActionsTrait<'_> for BooleanGrammar {
    ///
    /// This function is implemented automatically for the user's item BooleanGrammar.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeType],
    ) -> Result<()> {
        match prod_num {
            0 => self.expressions(&children[0], &children[1], &children[2]),
            1 => self.expressions_list_0(&children[0], &children[1], &children[2]),
            2 => self.expressions_list_1(),
            3 => self.expressions_opt_0(&children[0]),
            4 => self.expressions_opt_1(),
            5 => self.expression(&children[0], &children[1]),
            6 => self.tail_expression(&children[0]),
            7 => self.tail_expression_list_0(&children[0], &children[1], &children[2]),
            8 => self.tail_expression_list_1(),
            9 => self.term(&children[0], &children[1]),
            10 => self.term_opt_0(&children[0]),
            11 => self.term_opt_1(),
            12 => self.boolean_0(&children[0]),
            13 => self.boolean_1(&children[0]),
            14 => self.unary_operator(&children[0]),
            15 => self.binary_operator_0(&children[0]),
            16 => self.binary_operator_1(&children[0]),
            17 => self.binary_operator_2(&children[0]),
            18 => self.binary_operator_3(&children[0]),
            19 => self.binary_operator_4(&children[0]),
            20 => self.binary_operator_5(&children[0]),
            21 => self.and_op(&children[0]),
            22 => self.or_op(&children[0]),
            23 => self.xor_op(&children[0]),
            24 => self.nor_op(&children[0]),
            25 => self.nand_op(&children[0]),
            26 => self.xnor_op(&children[0]),
            27 => self.r#true(&children[0]),
            28 => self.r#false(&children[0]),
            29 => self.not(&children[0]),
            30 => self.parenthesized(&children[0], &children[1], &children[2]),
            31 => self.semicolon(&children[0]),
            32 => self.left_parenthesis(&children[0]),
            33 => self.right_parenthesis(&children[0]),
            34 => self.factor_0(&children[0]),
            35 => self.factor_1(&children[0]),
            _ => Err(ParserError::InternalError(format!(
                "Unhandled production number: {}",
                prod_num
            ))
            .into()),
        }
    }
    fn on_comment_parsed(&mut self, _token: Token<'_>) {}
}
