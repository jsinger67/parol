use crate::boolean_grammar_trait::BooleanGrammarTrait;
use id_tree::Tree;
use log::trace;
use miette::{miette, Result};
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};

///
/// The value range for the supported calculations
///
pub type DefinitionRange = bool;

///
/// Binary operators
///
#[derive(Debug, Clone)]
pub enum BinaryOp {
    And,
    Or,
    Xor,
    Nor,
    Nand,
    Xnor,
}

impl BinaryOp {
    pub fn _calc(&self, lhs: DefinitionRange, rhs: DefinitionRange) -> DefinitionRange {
        match self {
            Self::And => lhs & rhs,
            Self::Or => lhs | rhs,
            Self::Xor => lhs ^ rhs,
            Self::Nor => !(lhs | rhs),
            Self::Nand => !(lhs & rhs),
            Self::Xnor => lhs == rhs,
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
            Self::Xor => write!(f, "XOR"),
            Self::Nor => write!(f, "NOR"),
            Self::Nand => write!(f, "NAND"),
            Self::Xnor => write!(f, "XNOR"),
        }
    }
}

///
/// Unary operators
///
#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Not => write!(f, "NOT"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LeftAssoc(pub DefinitionRange, pub BinaryOp);

impl LeftAssoc {
    pub fn _calc(&self, lhs: DefinitionRange) -> DefinitionRange {
        self.1._calc(lhs, self.0)
    }
}

impl Display for LeftAssoc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Left({} {})", self.0, self.1)
    }
}

///
/// Data structure used to build up boolean calculations during parsing
///
#[derive(Debug, Clone)]
pub enum BooleanGrammarItem {
    Val(DefinitionRange),
    BinOp(BinaryOp),
    UnaryOp(UnaryOp),
    LeftAssociations(Vec<LeftAssoc>),
}

impl Display for BooleanGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Val(v) => write!(f, "{}", if *v { "TRUE" } else { "FALSE" }),
            Self::BinOp(op) => write!(f, "{}", op),
            Self::UnaryOp(op) => write!(f, "{}", op),
            Self::LeftAssociations(l) => write!(
                f,
                "LeftAssociations[{}]",
                l.iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

///
/// Data structure that implements the semantic actions for our boolean grammar
///
#[derive(Debug, Default)]
pub struct BooleanGrammar {
    pub item_stack: Vec<BooleanGrammarItem>,
    pub expression_stack: Vec<String>,
}

impl BooleanGrammar {
    pub fn new() -> Self {
        BooleanGrammar::default()
    }

    fn push(&mut self, item: BooleanGrammarItem, context: &str) {
        trace!("push   {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn pop(&mut self, context: &str) -> Option<BooleanGrammarItem> {
        if !self.item_stack.is_empty() {
            let item = self.item_stack.pop();
            if let Some(ref item) = item {
                trace!("pop    {}: {}", context, item);
            }
            item
        } else {
            trace!("pop    {}: None", context);
            None
        }
    }

    #[allow(dead_code)]
    // Use this function for debugging purposes:
    // $env:RUST_LOG="json_parser::json_grammar=trace"
    // trace!("{}", self.trace_item_stack(context));
    fn trace_item_stack(&self, context: &str) -> String {
        format!(
            "Ast stack at {}:\n{}",
            context,
            self.item_stack
                .iter()
                .rev()
                .map(|s| format!("  {}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    fn process_left_assoc(&mut self, context: &str) -> Result<()> {
        let context = format!("process_left_assoc {}", context);

        let op = self.pop(&context);
        let value = self.pop(&context);
        let list = self.pop(&context);
        match (&value, &op) {
            (Some(BooleanGrammarItem::Val(val)), Some(BooleanGrammarItem::BinOp(op))) => {
                let left_assoc = LeftAssoc(*val, op.clone());
                match list {
                    Some(BooleanGrammarItem::LeftAssociations(mut l)) => {
                        l.push(left_assoc);
                        self.push(BooleanGrammarItem::LeftAssociations(l), &context);
                    }
                    Some(item) => {
                        self.push(item, &context);
                        self.push(
                            BooleanGrammarItem::LeftAssociations(vec![left_assoc]),
                            &context,
                        );
                    }
                    _ => {
                        self.push(
                            BooleanGrammarItem::LeftAssociations(vec![left_assoc]),
                            &context,
                        );
                    }
                }
                Ok(())
            }
            _ => Err(miette!("{}: unexpected ({:?}, {:?}", context, op, value)),
        }
    }

    fn process_unary_operator(&mut self, context: &str) -> Result<()> {
        let context = format!("process_unary_operator {}", context);
        let value = self.pop(&context);
        let op = self.pop(&context);
        match (&value, &op) {
            (
                Some(BooleanGrammarItem::Val(val)),
                Some(BooleanGrammarItem::UnaryOp(UnaryOp::Not)),
            ) => {
                self.push(BooleanGrammarItem::Val(!val), &context);
                Ok(())
            }
            _ => Err(miette!("{}: unexpected ({:?}, {:?}", context, op, value)),
        }
    }

    fn process_left_associations(&mut self, context: &str) -> Result<()> {
        let context = format!("process_left_associations {}", context);

        let value = self.pop(&context);
        let list = self.pop(&context);
        match (&list, &value) {
            (
                Some(BooleanGrammarItem::LeftAssociations(list)),
                Some(BooleanGrammarItem::Val(val)),
            ) if !list.is_empty() => {
                let mut value = *val;
                // The value is sequentially calculated from left to right
                // because the operations are left associative.
                // The list is in reverse ordering (right to left) so we have to
                // iterate it in reverse order.
                for l in list.iter().rev() {
                    value = l._calc(value);
                }
                self.push(BooleanGrammarItem::Val(value), &context);
                Ok(())
            }
            (
                Some(BooleanGrammarItem::LeftAssociations(_)),
                Some(BooleanGrammarItem::Val(value)),
            ) => {
                // No operation to apply.
                // Recreate the value on the item stack.
                self.push(BooleanGrammarItem::Val(*value), &context);
                Ok(())
            }
            _ => {
                // No match, recreate stack as it was before
                if let Some(list) = list {
                    self.push(list, &context);
                }

                if let Some(value) = value {
                    self.push(value, &context);
                }
                Ok(())
            }
        }
    }

    fn record_expression(&mut self, item: &dyn Display) {
        if let Some(last) = self.expression_stack.last_mut() {
            last.push_str(format!("{} ", item).as_str());
        } else {
            self.expression_stack.push(format!("{} ", item));
        }
    }
}

impl Display for BooleanGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(
            f,
            "{}",
            self.expression_stack
                .iter()
                .zip(self.item_stack.iter())
                .map(|(e, r)| format!("{}= {};", e, r))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl BooleanGrammarTrait for BooleanGrammar {
    /// Semantic action for production 5:
    ///
    /// Expression: Term TailExpression;
    ///
    fn expression_0(
        &mut self,
        _term_0: &ParseTreeStackEntry,
        _tail_expression_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "expression_0";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_associations(context)
    }

    /// Semantic action for production 9:
    ///
    /// Term: UnaryOperator Factor;
    ///
    fn term_0(
        &mut self,
        _unary_operator_0: &ParseTreeStackEntry,
        _factor_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "term_0";
        trace!("{}", self.trace_item_stack(context));
        self.process_unary_operator(context)
    }

    /// Semantic action for production 14:
    ///
    /// BinaryOperator: AndOp;
    ///
    fn binary_operator_0(
        &mut self,
        _and_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "binary_operator_0";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 15:
    ///
    /// BinaryOperator: OrOp;
    ///
    fn binary_operator_1(
        &mut self,
        _or_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "binary_operator_1";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 16:
    ///
    /// BinaryOperator: XorOp;
    ///
    fn binary_operator_2(
        &mut self,
        _xor_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "binary_operator_2";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 17:
    ///
    /// BinaryOperator: NorOp;
    ///
    fn binary_operator_3(
        &mut self,
        _nor_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "binary_operator_3";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 18:
    ///
    /// BinaryOperator: NandOp;
    ///
    fn binary_operator_4(
        &mut self,
        _nand_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "binary_operator_4";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 19:
    ///
    /// BinaryOperator: XnorOp;
    ///
    fn binary_operator_5(
        &mut self,
        _xnor_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "binary_operator_5";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 20:
    ///
    /// AndOp: "(?i)AND";
    ///
    fn and_op_0(
        &mut self,
        _and_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "and_op_0";
        trace!("{}", self.trace_item_stack(context));
        let op = BinaryOp::And;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::BinOp(op), context);
        Ok(())
    }

    /// Semantic action for production 21:
    ///
    /// OrOp: "(?i)OR";
    ///
    fn or_op_0(
        &mut self,
        _or_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "or_op_0";
        trace!("{}", self.trace_item_stack(context));
        let op = BinaryOp::Or;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::BinOp(op), context);
        Ok(())
    }

    /// Semantic action for production 21:
    ///
    /// XorOp: "(?i)XOR";
    ///
    fn xor_op_0(
        &mut self,
        _xor_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "xor_op_0";
        trace!("{}", self.trace_item_stack(context));
        let op = BinaryOp::Xor;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::BinOp(op), context);
        Ok(())
    }

    /// Semantic action for production 23:
    ///
    /// NorOp: "(?i)NOR";
    ///
    fn nor_op_0(
        &mut self,
        _nor_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "nor_op_0";
        trace!("{}", self.trace_item_stack(context));
        let op = BinaryOp::Nor;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::BinOp(op), context);
        Ok(())
    }

    /// Semantic action for production 24:
    ///
    /// NandOp: "(?i)NAND";
    ///
    fn nand_op_0(
        &mut self,
        _nand_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "nand_op_0";
        trace!("{}", self.trace_item_stack(context));
        let op = BinaryOp::Nand;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::BinOp(op), context);
        Ok(())
    }

    /// Semantic action for production 25:
    ///
    /// XnorOp: "(?i)XNOR";
    ///
    fn xnor_op_0(
        &mut self,
        _xnor_op_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "xnor_op_0";
        trace!("{}", self.trace_item_stack(context));
        let op = BinaryOp::Xnor;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::BinOp(op), context);
        Ok(())
    }

    /// Semantic action for production 26:
    ///
    /// True: "(?i)TRUE";
    ///
    fn true_0(
        &mut self,
        _true_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "true_0";
        let val = BooleanGrammarItem::Val(true);
        self.record_expression(&val);
        self.push(val, context);
        Ok(())
    }

    /// Semantic action for production 27:
    ///
    /// False: "(?i)FALSE";
    ///
    fn false_0(
        &mut self,
        _false_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "false_0";
        let val = BooleanGrammarItem::Val(false);
        self.record_expression(&val);
        self.push(val, context);
        Ok(())
    }

    /// Semantic action for production 28:
    ///
    /// Not: "(?i)NOT";
    ///
    fn not_0(
        &mut self,
        _not_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "not_0";
        let op = UnaryOp::Not;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::UnaryOp(op), context);
        Ok(())
    }

    /// Semantic action for production 30:
    ///
    /// Semicolon: ";";
    ///
    fn semicolon_0(
        &mut self,
        _semicolon_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        self.expression_stack.push(String::new());
        Ok(())
    }

    /// Semantic action for production 31:
    ///
    /// LeftParenthesis: "\(";
    ///
    fn left_parenthesis_0(
        &mut self,
        _left_parenthesis_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        self.record_expression(&"(");
        Ok(())
    }

    /// Semantic action for production 32:
    ///
    /// RightParenthesis: "\)";
    ///
    fn right_parenthesis_0(
        &mut self,
        _right_parenthesis_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        self.record_expression(&")");
        Ok(())
    }
}
