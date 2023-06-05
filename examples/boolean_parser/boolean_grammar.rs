use crate::boolean_grammar_trait::BooleanGrammarTrait;
use parol_macros::parol;
use parol_runtime::parser::ParseTreeType;
use parol_runtime::{log::trace, Result, Token};
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

    pub fn on_comment(&mut self, _token: Token<'_>) {}

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
            _ => Err(parol!("{}: unexpected ({:?}, {:?}", context, op, value)),
        }
    }

    fn process_unary_operator(&mut self, context: &str) -> Result<()> {
        let context = format!("process_unary_operator {}", context);
        if self.item_stack.len() >= 2
            && matches!(
                self.item_stack[self.item_stack.len() - 2..],
                [
                    BooleanGrammarItem::Val(_),
                    BooleanGrammarItem::UnaryOp(UnaryOp::Not)
                ]
            )
        {
            self.pop(&context); // Remove the unary operator from the stack
            if let BooleanGrammarItem::Val(val) = self.pop(&context).unwrap() {
                // Invert the value
                self.push(BooleanGrammarItem::Val(!val), &context);
            }
        }
        Ok(())
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
    fn expression(
        &mut self,
        _term: &ParseTreeType<'_>,
        _tail_expression: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "expression";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_associations(context)
    }

    /// Semantic action for production 9:
    ///
    /// Term: TermOpt Factor;
    ///
    fn term(&mut self, _term_opt: &ParseTreeType<'_>, _factor: &ParseTreeType<'_>) -> Result<()> {
        let context = "term";
        trace!("{}", self.trace_item_stack(context));
        self.process_unary_operator(context)
    }

    /// Semantic action for production 10:
    ///
    /// TermOpt: UnaryOperator;
    ///
    fn term_opt_0(&mut self, _unary_operator: &ParseTreeType<'_>) -> Result<()> {
        let context = "term_opt_0";
        trace!("{}", self.trace_item_stack(context));
        self.push(BooleanGrammarItem::UnaryOp(UnaryOp::Not), context);
        Ok(())
    }

    /// Semantic action for production 14:
    ///
    /// BinaryOperator: AndOp;
    ///
    fn binary_operator_0(&mut self, _and_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "binary_operator_0";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 15:
    ///
    /// BinaryOperator: OrOp;
    ///
    fn binary_operator_1(&mut self, _or_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "binary_operator_1";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 16:
    ///
    /// BinaryOperator: XorOp;
    ///
    fn binary_operator_2(&mut self, _xor_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "binary_operator_2";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 17:
    ///
    /// BinaryOperator: NorOp;
    ///
    fn binary_operator_3(&mut self, _nor_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "binary_operator_3";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 18:
    ///
    /// BinaryOperator: NandOp;
    ///
    fn binary_operator_4(&mut self, _nand_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "binary_operator_4";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 19:
    ///
    /// BinaryOperator: XnorOp;
    ///
    fn binary_operator_5(&mut self, _xnor_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "binary_operator_5";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_assoc(context)
    }

    /// Semantic action for production 20:
    ///
    /// AndOp: "(?i)AND";
    ///
    fn and_op(&mut self, _and_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "and_op";
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
    fn or_op(&mut self, _or_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "or_op";
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
    fn xor_op(&mut self, _xor_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "xor_op";
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
    fn nor_op(&mut self, _nor_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "nor_op";
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
    fn nand_op(&mut self, _nand_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "nand_op";
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
    fn xnor_op(&mut self, _xnor_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "xnor_op";
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
    fn r#true(&mut self, _true: &ParseTreeType<'_>) -> Result<()> {
        let context = "r#true";
        let val = BooleanGrammarItem::Val(true);
        self.record_expression(&val);
        self.push(val, context);
        Ok(())
    }

    /// Semantic action for production 27:
    ///
    /// False: "(?i)FALSE";
    ///
    fn r#false(&mut self, _false: &ParseTreeType<'_>) -> Result<()> {
        let context = "r#false";
        let val = BooleanGrammarItem::Val(false);
        self.record_expression(&val);
        self.push(val, context);
        Ok(())
    }

    /// Semantic action for production 28:
    ///
    /// Not: "(?i)NOT";
    ///
    fn not(&mut self, _not: &ParseTreeType<'_>) -> Result<()> {
        let context = "not";
        let op = UnaryOp::Not;
        self.record_expression(&op);
        self.push(BooleanGrammarItem::UnaryOp(op), context);
        Ok(())
    }

    /// Semantic action for production 30:
    ///
    /// Semicolon: ";";
    ///
    fn semicolon(&mut self, _semicolon: &ParseTreeType<'_>) -> Result<()> {
        self.expression_stack.push(String::new());
        Ok(())
    }

    /// Semantic action for production 31:
    ///
    /// LeftParenthesis: "\(";
    ///
    fn left_parenthesis(&mut self, _left_parenthesis: &ParseTreeType<'_>) -> Result<()> {
        self.record_expression(&"(");
        Ok(())
    }

    /// Semantic action for production 32:
    ///
    /// RightParenthesis: "\)";
    ///
    fn right_parenthesis(&mut self, _right_parenthesis: &ParseTreeType<'_>) -> Result<()> {
        self.record_expression(&")");
        Ok(())
    }
}
