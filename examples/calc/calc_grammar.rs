use crate::assign_operator::AssignOperator;
use crate::binary_operator::BinaryOperator;
use crate::calc_grammar_trait::CalcGrammarTrait;
use crate::errors::CalcError;
use crate::unary_operator::UnaryOperator;
use parol_macros::{bail, parol};
use parol_runtime::{errors::FileSource, log::trace, Location, ParseTreeType, Result};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// The value range for the supported calculations
///
pub type DefinitionRange = isize;

#[derive(Debug, Clone)]
pub struct AssignItem(pub String, pub AssignOperator);

impl Display for AssignItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "({} {})", self.0, self.1)
    }
}

#[derive(Debug, Clone)]
pub struct RightItem(pub BinaryOperator, pub DefinitionRange);

impl Display for RightItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Left({} {})", self.0, self.1)
    }
}

///
/// Data structure used to build up a calc structure item during parsing
///
#[derive(Debug, Clone)]
pub enum CalcGrammarItem {
    Num(DefinitionRange),
    Id(String, Location),
    AssignOp(AssignOperator),
    AssignItem(AssignItem),
    AssignItems(Vec<AssignItem>),
    UnaryOp(UnaryOperator),
    BinaryOp(BinaryOperator),
    RightItem(RightItem),
    RightItems(Vec<RightItem>),
}

impl Display for CalcGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Num(n) => write!(f, "Num({})", n),
            Self::Id(t, _) => write!(f, "Id({})", t),
            Self::AssignOp(o) => write!(f, "AssignOp({})", o),
            Self::AssignItem(a) => write!(f, "AssignItem({})", a),
            Self::AssignItems(l) => write!(
                f,
                "AssignItems[{}]",
                l.iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::UnaryOp(o) => write!(f, "UOp({})", o),
            Self::BinaryOp(o) => write!(f, "Op({})", o),
            Self::RightItem(i) => write!(f, "{}", i),
            Self::RightItems(l) => write!(
                f,
                "RightItems[{}]",
                l.iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

///
/// Data structure used to build up a calc structure during parsing
///
#[derive(Debug, Default)]
pub struct CalcGrammar {
    pub item_stack: Vec<CalcGrammarItem>,
    env: BTreeMap<String, DefinitionRange>,
}

impl CalcGrammar {
    pub fn new() -> Self {
        CalcGrammar::default()
    }

    fn push(&mut self, item: CalcGrammarItem, context: &str) {
        trace!("push    {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn pop(&mut self, context: &str) -> Option<CalcGrammarItem> {
        if !self.item_stack.is_empty() {
            let item = self.item_stack.pop();
            if let Some(ref item) = item {
                trace!("pop     {}: {}", context, item);
            }
            item
        } else {
            None
        }
    }

    fn value(&self, id: &str) -> Option<DefinitionRange> {
        self.env.get(id).cloned()
    }

    fn declare(&mut self, id: &str, context: &str) {
        if !self.env.contains_key(id) {
            trace!("declare {}: {}", context, id);
            self.env.insert(id.to_owned(), 0);
        }
    }

    fn assign(
        &mut self,
        item: &AssignItem,
        num: DefinitionRange,
        context: &str,
    ) -> Result<DefinitionRange> {
        if let Some(var) = self.env.get_mut(&item.0) {
            trace!("assign: to variable {}", item.0);
            Self::apply_assign_item(var, &item.1, num, context)?;
            Ok(*var)
        } else {
            Err(parol!("assign: undeclared variable {}", item.0))
        }
    }

    fn apply_assign_item(
        lhs: &mut DefinitionRange,
        op: &AssignOperator,
        rhs: DefinitionRange,
        context: &str,
    ) -> Result<()> {
        trace!("apply_assign_item: {}: {} {} {}", context, lhs, op, rhs);
        match op {
            AssignOperator::Assign => *lhs = rhs,
            AssignOperator::Plus => *lhs += rhs,
            AssignOperator::Minus => *lhs -= rhs,
            AssignOperator::Mul => *lhs *= rhs,
            AssignOperator::Div => {
                if rhs == 0 {
                    bail!("Division by zero detected!");
                }
                *lhs /= rhs
            }
            AssignOperator::Mod => *lhs %= rhs,
            AssignOperator::ShiftLeft => *lhs <<= rhs,
            AssignOperator::ShiftRight => *lhs >>= rhs,
            AssignOperator::BitwiseAnd => *lhs &= rhs,
            AssignOperator::BitwiseXOr => *lhs = rhs,
            AssignOperator::BitwiseOr => *lhs |= rhs,
        }
        trace!("apply_assign_item:      = {}", lhs);
        Ok(())
    }

    fn apply_binary_operation(
        lhs: DefinitionRange,
        rhs: &RightItem,
        context: &str,
    ) -> Result<DefinitionRange> {
        trace!(
            "apply_binary_operation: {}: {} {} {}",
            context,
            lhs,
            rhs.0,
            rhs.1
        );
        let result = match rhs.0 {
            BinaryOperator::Add => lhs + rhs.1,
            BinaryOperator::Sub => lhs - rhs.1,
            BinaryOperator::Mul => lhs * rhs.1,
            BinaryOperator::Div => {
                if rhs.1 == 0 {
                    bail!("Division by zero detected!");
                }
                lhs / rhs.1
            }
            BinaryOperator::Mod => lhs % rhs.1,
            BinaryOperator::Pow => {
                if let Ok(exponent) = rhs.1.try_into() {
                    lhs.pow(exponent)
                } else {
                    bail!("Exponent {} can't be converted to u32!", rhs);
                }
            }
            BinaryOperator::Eq => (lhs == rhs.1) as DefinitionRange,
            BinaryOperator::Ne => (lhs != rhs.1) as DefinitionRange,
            BinaryOperator::Lt => (lhs < rhs.1) as DefinitionRange,
            BinaryOperator::Le => (lhs <= rhs.1) as DefinitionRange,
            BinaryOperator::Gt => (lhs > rhs.1) as DefinitionRange,
            BinaryOperator::Ge => (lhs >= rhs.1) as DefinitionRange,
            BinaryOperator::BitShl => lhs << rhs.1,
            BinaryOperator::BitShr => lhs >> rhs.1,
            BinaryOperator::BitAnd => lhs & rhs.1,
            BinaryOperator::BitOr => lhs | rhs.1,
            BinaryOperator::LogAnd => ((lhs != 0) && (rhs.1 != 0)) as DefinitionRange,
            BinaryOperator::LogOr => ((lhs != 0) || (rhs.1 != 0)) as DefinitionRange,
        };

        trace!("apply_binary_operation:      = {}", result);

        Ok(result)
    }

    fn process_left_associative_operation_list(&mut self, context: &str) -> Result<()> {
        let list = self.pop(context);
        let value = self.pop(context);
        match (&list, &value) {
            (Some(CalcGrammarItem::RightItems(list)), Some(CalcGrammarItem::Num(num)))
                if !list.is_empty() =>
            {
                let mut value = *num;
                // The value is sequentially calculated from left to right
                // because the operations are left associative.
                // The list is in reverse ordering (right to left) so we have to
                // reverse it.
                for l in list.iter().rev() {
                    value = Self::apply_binary_operation(value, l, context)?;
                }
                self.push(CalcGrammarItem::Num(value), context);
                Ok(())
            }
            (Some(CalcGrammarItem::RightItems(_)), Some(CalcGrammarItem::Num(value))) => {
                // No operation to apply.
                // Recreate the number on  the user stack.
                self.push(CalcGrammarItem::Num(*value), context);
                Ok(())
            }
            // _ => Ok(()),
            _ => Err(parol!("{}: unexpected ({:?}, {:?})", context, list, value)),
        }
    }

    fn process_right_associative_operation_list(&mut self, context: &str) -> Result<()> {
        let value = self.pop(context);
        let left_lst = self.pop(context);
        match (&value, &left_lst) {
            (Some(CalcGrammarItem::RightItems(list)), Some(CalcGrammarItem::Num(num)))
                if !list.is_empty() =>
            {
                let mut value = 0;
                // The value is sequentially calculated from right to left
                // because the power operation is right associative.
                // The list is already in reverse ordering (right to left)
                // but we need to access the "previous" element to obtain the
                // left-hand side of the operation.
                for i in 0..list.len() {
                    if i + 1 < list.len() {
                        value = list[i + 1].1;
                        value = Self::apply_binary_operation(value, &list[i], context)?;
                    }
                }
                // At the end we apply the result of the operations to the very
                // first number in the chain.
                value = Self::apply_binary_operation(
                    *num,
                    &RightItem(BinaryOperator::Pow, value),
                    context,
                )?;
                self.push(CalcGrammarItem::Num(value), context);
                Ok(())
            }
            (Some(CalcGrammarItem::RightItems(_)), Some(CalcGrammarItem::Num(value))) => {
                // No power operation to apply.
                // Recreate the number on  the user stack.
                self.push(CalcGrammarItem::Num(*value), context);
                Ok(())
            }
            _ => Err(parol!(
                "{}: unexpected ({:?}, {:?})",
                context,
                value,
                left_lst
            )),
        }
    }

    fn process_binary_operator(
        &mut self,
        stack_entry: &ParseTreeType<'_>,
        context: &str,
    ) -> Result<()> {
        let symbol = stack_entry.text()?;
        let op: BinaryOperator = symbol.into();
        self.push(CalcGrammarItem::BinaryOp(op), context);
        Ok(())
    }

    fn process_right_items(&mut self, context: &str) -> Result<()> {
        let right_lst = self.pop(context);
        let right_item = self.pop(context);
        match (&right_item, &right_lst) {
            (Some(CalcGrammarItem::RightItem(item)), Some(CalcGrammarItem::RightItems(list))) => {
                let mut list = list.clone();
                list.push(item.clone());
                self.push(CalcGrammarItem::RightItems(list.to_vec()), context);
                Ok(())
            }
            _ => Err(parol!(
                "{}: unexpected ({:?}, {:?}",
                context,
                right_item,
                right_lst
            )),
        }
    }

    fn process_right_item(&mut self, context: &str) -> Result<()> {
        let value = self.pop(context);
        let op = self.pop(context);
        match (&value, &op) {
            (Some(CalcGrammarItem::Num(num)), Some(CalcGrammarItem::BinaryOp(op))) => {
                self.push(
                    CalcGrammarItem::RightItem(RightItem(op.clone(), *num)),
                    context,
                );
                Ok(())
            }
            _ => Err(parol!("{}: unexpected ({:?}, {:?}", context, value, op)),
        }
    }

    #[allow(dead_code)]
    // Use this function for debugging purposes:
    // trace!("{}", self.trace_item_stack(context));
    fn trace_item_stack(&self, context: &str) -> String {
        format!(
            "Item stack at {}:\n{}",
            context,
            self.item_stack
                .iter()
                .rev()
                .map(|s| format!("  {}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Display for CalcGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(
            f,
            "Stack\n{}",
            self.item_stack
                .iter()
                .rev()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\n")
        )?;
        writeln!(
            f,
            "\nEnv\n{}",
            self.env
                .iter()
                .map(|(i, v)| format!("{} = {}", i, v))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl CalcGrammarTrait for CalcGrammar {
    /// Semantic action for production 6:
    ///
    /// EqualityOp: "==|!=";
    ///
    fn equality_op(&mut self, tk_equality_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "equality_op";
        self.process_binary_operator(tk_equality_op, context)
    }

    /// Semantic action for production 7:
    ///
    /// AssignOp: "(\+|-|\*|/|%|<<|>>|&|\|\|)?=";
    ///
    fn assign_op(&mut self, tk_assign_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "assign_op";
        let symbol = tk_assign_op.text()?;
        let assign_op: AssignOperator = symbol.into();
        self.push(CalcGrammarItem::AssignOp(assign_op), context);
        Ok(())
    }

    /// Semantic action for production 8:
    ///
    /// AssignItem: Id AssignOp;
    ///
    fn assign_item(
        &mut self,
        _id: &ParseTreeType<'_>,
        _assign_op: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "assign_item";
        let top_of_stack1 = self.pop(context);
        let top_of_stack2 = self.pop(context);
        match (&top_of_stack1, &top_of_stack2) {
            (Some(CalcGrammarItem::AssignOp(op)), Some(CalcGrammarItem::Id(text, _))) => {
                self.declare(text, context);
                self.push(
                    CalcGrammarItem::AssignItem(AssignItem(text.to_string(), op.clone())),
                    context,
                );
                Ok(())
            }
            _ => Err(parol!(
                "{}: unexpected ({:?}, {:?}",
                context,
                top_of_stack1,
                top_of_stack2
            )),
        }
    }

    /// Semantic action for production 9:
    ///
    /// Assignment: AssignItem AssignmentLst1 LogicalOr;
    ///
    fn assignment(
        &mut self,
        _assign_item: &ParseTreeType<'_>,
        _assignment_lst: &ParseTreeType<'_>,
        _logical_or: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "assignment";
        let value = self.pop(context);
        let assignment_lst = self.pop(context);
        let assign_item = self.pop(context);
        match (&value, &assignment_lst, &assign_item) {
            (
                Some(CalcGrammarItem::Num(num)),
                Some(CalcGrammarItem::AssignItems(list)),
                Some(CalcGrammarItem::AssignItem(item)),
            ) => {
                let mut value = *num;
                // The value is sequentially calculated and assigned from right
                // to left because the assignment operations are right
                // associative.
                for i in list {
                    value = self.assign(i, value, context)?;
                }
                self.assign(item, value, context)?;
                Ok(())
            }
            //_ => Ok(())
            _ => Err(parol!(
                "{}: unexpected ({:?}, {:?}, {:?})",
                context,
                value,
                assignment_lst,
                assign_item
            )),
        }
    }

    /// Semantic action for production 10:
    ///
    /// AssignmentLst1: AssignmentLst1Itm1 AssignmentLst1;
    ///
    fn assignment_lst1_0(
        &mut self,
        _assignment_lst1_itm1: &ParseTreeType<'_>,
        _assignment_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "assignment_lst1_0";
        let top_of_stack1 = self.pop(context);
        let top_of_stack2 = self.pop(context);
        match (&top_of_stack1, &top_of_stack2) {
            (Some(CalcGrammarItem::AssignItems(list)), Some(CalcGrammarItem::AssignItem(item))) => {
                let mut list = list.clone();
                list.push(item.clone());
                self.push(CalcGrammarItem::AssignItems(list.to_vec()), context);
                Ok(())
            }
            _ => Err(parol!(
                "{}: unexpected ({:?}, {:?}",
                context,
                top_of_stack1,
                top_of_stack2
            )),
        }
    }

    /// Semantic action for production 12:
    ///
    /// AssignmentLst1: ;
    ///
    fn assignment_lst1_1(&mut self) -> Result<()> {
        let context = "assignment_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::AssignItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 13:
    ///
    /// LogicalOr: LogicalAnd LogicalOrLst1;
    ///
    fn logical_or(
        &mut self,
        _logical_and: &ParseTreeType<'_>,
        _logical_or_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "logical_or";
        trace!("{}", self.trace_item_stack(context));
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 14:
    ///
    /// LogicalOrLst1: LogicalOrLst1Itm1 LogicalOrLst1;
    ///
    fn logical_or_lst1_0(
        &mut self,
        _logical_or_lst1_itm1: &ParseTreeType<'_>,
        _logical_or_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "logical_or_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 16:
    ///
    /// LogicalOrLst1: ;
    ///
    fn logical_or_lst1_1(&mut self) -> Result<()> {
        let context = "logical_or_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// LogicalOrOp: "\|\|";
    ///
    fn logical_or_op(&mut self, _logical_or_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "logical_or_op";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::LogOr), context);
        Ok(())
    }

    /// Semantic action for production 18:
    ///
    /// LogicalOrItem: LogicalOrOp LogicalAnd;
    ///
    fn logical_or_item(
        &mut self,
        _logical_or_op: &ParseTreeType<'_>,
        _logical_and: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "logical_or_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 19:
    ///
    /// LogicalAnd: BitwiseOr LogicalAndLst1;
    ///
    fn logical_and(
        &mut self,
        _bitwise_or: &ParseTreeType<'_>,
        _logical_and_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "logical_and";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 20:
    ///
    /// LogicalAndLst1: LogicalAndLst1Itm1 LogicalAndLst1;
    ///
    fn logical_and_lst1_0(
        &mut self,
        _logical_and_lst1_itm1: &ParseTreeType<'_>,
        _logical_and_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "logical_and_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 22:
    ///
    /// LogicalAndLst1: ;
    ///
    fn logical_and_lst1_1(&mut self) -> Result<()> {
        let context = "logical_and_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 23:
    ///
    /// LogicalAndOp: "&&";
    ///
    fn logical_and_op(&mut self, _logical_and_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "logical_and_op";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::LogAnd), context);
        Ok(())
    }

    /// Semantic action for production 24:
    ///
    /// LogicalAndItem: LogicalAndOp BitwiseOr;
    ///
    fn logical_and_item(
        &mut self,
        _logical_and_op: &ParseTreeType<'_>,
        _bitwise_or: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "logical_and_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 25:
    ///
    /// BitwiseOr: BitwiseAnd BitwiseOrLst1;
    ///
    fn bitwise_or(
        &mut self,
        _bitwise_and: &ParseTreeType<'_>,
        _bitwise_or_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_or";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 26:
    ///
    /// BitwiseOrLst1: BitwiseOrLst1Itm1 BitwiseOrLst1;
    ///
    fn bitwise_or_lst1_0(
        &mut self,
        _bitwise_or_lst1_itm1: &ParseTreeType<'_>,
        _bitwise_or_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_or_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 28:
    ///
    /// BitwiseOrLst1: ;
    ///
    fn bitwise_or_lst1_1(&mut self) -> Result<()> {
        let context = "bitwise_or_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 29:
    ///
    /// BitwiseOrOp: "\|";
    ///
    fn bitwise_or_op(&mut self, _bitwise_or_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "bitwise_or_op";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::BitOr), context);
        Ok(())
    }

    /// Semantic action for production 30:
    ///
    /// BitwiseOrItem: BitwiseOrOp BitwiseAnd;
    ///
    fn bitwise_or_item(
        &mut self,
        _bitwise_or_op: &ParseTreeType<'_>,
        _bitwise_and: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_or_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 31:
    ///
    /// BitwiseAnd: Equality BitwiseAndLst1;
    ///
    fn bitwise_and(
        &mut self,
        _equality: &ParseTreeType<'_>,
        _bitwise_and_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_and";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 32:
    ///
    /// BitwiseAndLst1: BitwiseAndLst1Itm1 BitwiseAndLst1;
    ///
    fn bitwise_and_lst1_0(
        &mut self,
        _bitwise_and_lst1_itm1: &ParseTreeType<'_>,
        _bitwise_and_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_and_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 34:
    ///
    /// BitwiseAndLst1: ;
    ///
    fn bitwise_and_lst1_1(&mut self) -> Result<()> {
        let context = "bitwise_and_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 35:
    ///
    /// BitwiseAndOp: "&";
    ///
    fn bitwise_and_op(&mut self, _bitwise_and_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "bitwise_and_op";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::BitAnd), context);
        Ok(())
    }

    /// Semantic action for production 36:
    ///
    /// BitwiseAndItem: BitwiseAndOp Equality;
    ///
    fn bitwise_and_item(
        &mut self,
        _bitwise_and_op: &ParseTreeType<'_>,
        _equality: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_and_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 37:
    ///
    /// Equality: Relational EqualityLst1;
    ///
    fn equality(
        &mut self,
        _relational: &ParseTreeType<'_>,
        _equality_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "equality";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 38:
    ///
    /// EqualityLst1: EqualityLst1Itm1 EqualityLst1;
    ///
    fn equality_lst1_0(
        &mut self,
        _equality_lst1_itm1: &ParseTreeType<'_>,
        _equality_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "equality_lst1";
        self.process_right_items(context)
    }

    /// Semantic action for production 40:
    ///
    /// EqualityLst1: ;
    ///
    fn equality_lst1_1(&mut self) -> Result<()> {
        let context = "equality_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 41:
    ///
    /// EqualityItem: EqualityOp Relational;
    ///
    fn equality_item(
        &mut self,
        _equality_op: &ParseTreeType<'_>,
        _relational: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "equality_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 42:
    ///
    /// BitwiseShiftOp: "<<|>>";
    ///
    fn bitwise_shift_op(&mut self, tk_bitwise_shift_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "bitwise_shift_op";
        self.process_binary_operator(tk_bitwise_shift_op, context)
    }

    /// Semantic action for production 43:
    ///
    /// Relational: BitwiseShift RelationalLst1;
    ///
    fn relational(
        &mut self,
        _bitwise_shift: &ParseTreeType<'_>,
        _relational_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "relational";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 44:
    ///
    /// RelationalLst1: RelationalLst1Itm1 RelationalLst1;
    ///
    fn relational_lst1_0(
        &mut self,
        _relational_lst1_itm1: &ParseTreeType<'_>,
        _relational_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "relational_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 46:
    ///
    /// RelationalLst1: ;
    ///
    fn relational_lst1_1(&mut self) -> Result<()> {
        let context = "relational_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 47:
    ///
    /// RelationalOp: "<=|<|>=|>";
    ///
    fn relational_op(&mut self, tk_relational_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "relational_op";
        self.process_binary_operator(tk_relational_op, context)
    }

    /// Semantic action for production 48:
    ///
    /// RelationalItem: RelationalOp BitwiseShift;
    ///
    fn relational_item(
        &mut self,
        _relational_op: &ParseTreeType<'_>,
        _bitwise_shift: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "relational_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 49:
    ///
    /// BitwiseShift: Summ BitwiseShiftLst1;
    ///
    fn bitwise_shift(
        &mut self,
        _summ: &ParseTreeType<'_>,
        _bitwise_shift_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_shift";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 50:
    ///
    /// BitwiseShiftLst1: BitwiseShiftLst1Itm1 BitwiseShiftLst1;
    ///
    fn bitwise_shift_lst1_0(
        &mut self,
        _bitwise_shift_lst1_itm1: &ParseTreeType<'_>,
        _bitwise_shift_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_shift_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 52:
    ///
    /// BitwiseShiftLst1: ;
    ///
    fn bitwise_shift_lst1_1(&mut self) -> Result<()> {
        let context = "bitwise_shift_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 53:
    ///
    /// BitwiseShiftItem: BitwiseShiftOp Summ;
    ///
    fn bitwise_shift_item(
        &mut self,
        _bitwise_shift_op: &ParseTreeType<'_>,
        _summ: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "bitwise_shift_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 54:
    ///
    /// Summ: Mult SummLst1;
    ///
    fn summ(&mut self, _mult: &ParseTreeType<'_>, _summ_lst1: &ParseTreeType<'_>) -> Result<()> {
        let context = "summ";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 55:
    ///
    /// SummLst1: SummLst1Itm1 SummLst1;
    ///
    fn summ_lst1_0(
        &mut self,
        _summ_lst1_itm1: &ParseTreeType<'_>,
        _summ_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "summ_lst1";
        self.process_right_items(context)
    }

    /// Semantic action for production 57:
    ///
    /// SummLst1: ;
    ///
    fn summ_lst1_1(&mut self) -> Result<()> {
        let context = "summ_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 58:
    ///
    /// Plus: "\+";
    ///
    fn plus(&mut self, _plus: &ParseTreeType<'_>) -> Result<()> {
        let context = "plus";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::Add), context);
        Ok(())
    }

    /// Semantic action for production 59:
    ///
    /// Minus: "-";
    ///
    fn minus(&mut self, _minus: &ParseTreeType<'_>) -> Result<()> {
        let context = "minus";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::Sub), context);
        Ok(())
    }

    /// Semantic action for production 62:
    ///
    /// SummItem: AddOp Mult;
    ///
    fn summ_item(&mut self, _add_op: &ParseTreeType<'_>, _mult: &ParseTreeType<'_>) -> Result<()> {
        let context = "summ_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 63:
    ///
    /// PowOp: "\*\*";
    ///
    fn pow_op(&mut self, _pow_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "pow_op";
        self.push(CalcGrammarItem::BinaryOp(BinaryOperator::Pow), context);
        Ok(())
    }

    /// Semantic action for production 64:
    ///
    /// Mult: Power MultLst1;
    ///
    fn mult(&mut self, _power: &ParseTreeType<'_>, _mult_lst1: &ParseTreeType<'_>) -> Result<()> {
        let context = "mult";
        self.process_left_associative_operation_list(context)
    }

    /// Semantic action for production 65:
    ///
    /// MultLst1: MultLst1Itm1 MultLst1;
    ///
    fn mult_lst1_0(
        &mut self,
        _mult_lst1_itm1: &ParseTreeType<'_>,
        _mult_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "mult_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 67:
    ///
    /// MultLst1: ;
    ///
    fn mult_lst1_1(&mut self) -> Result<()> {
        let context = "mult_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 68:
    ///
    /// MultOp: "\*|/|%";
    ///
    fn mult_op(&mut self, tk_mult_op: &ParseTreeType<'_>) -> Result<()> {
        let context = "mult_op";
        self.process_binary_operator(tk_mult_op, context)
    }

    /// Semantic action for production 69:
    ///
    /// MultItem: MultOp Power;
    ///
    fn mult_item(
        &mut self,
        _mult_op: &ParseTreeType<'_>,
        _power: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "mult_item";
        self.process_right_item(context)
    }

    /// Semantic action for production 70:
    ///
    /// Power: Factor PowerLst1;
    ///
    fn power(
        &mut self,
        _factor: &ParseTreeType<'_>,
        _power_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "power";
        self.process_right_associative_operation_list(context)
    }

    /// Semantic action for production 71:
    ///
    /// PowerLst1: PowerLst1Itm1 PowerLst1;
    ///
    fn power_lst1_0(
        &mut self,
        _power_lst1_itm1: &ParseTreeType<'_>,
        _power_lst1: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "power_lst1_0";
        self.process_right_items(context)
    }

    /// Semantic action for production 72:
    ///
    /// PowerLst1Itm1: PowOp Factor;
    ///
    fn power_lst1_itm1(
        &mut self,
        _pow_op: &ParseTreeType<'_>,
        _factor: &ParseTreeType<'_>,
    ) -> Result<()> {
        let context = "power_lst1_itm1";
        self.process_right_item(context)
    }

    /// Semantic action for production 73:
    ///
    /// PowerLst1: ;
    ///
    fn power_lst1_1(&mut self) -> Result<()> {
        let context = "power_lst1_1";
        // Start with an empty list here
        self.push(CalcGrammarItem::RightItems(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 74:
    ///
    /// Negate: Minus;
    ///
    fn negate(&mut self, _minus: &ParseTreeType<'_>) -> Result<()> {
        let context = "negate";
        let minus = self.pop(context);
        if let Some(CalcGrammarItem::BinaryOp(BinaryOperator::Sub)) = minus {
            self.push(CalcGrammarItem::UnaryOp(UnaryOperator::Negation), context);
            Ok(())
        } else {
            Err(parol!("{}: unexpected {:?}", context, minus))
        }
    }

    /// Semantic action for production 77:
    ///
    /// Factor: Negate Factor;
    ///
    fn factor_2(&mut self, _negate: &ParseTreeType<'_>, _factor: &ParseTreeType<'_>) -> Result<()> {
        let context = "factor_2";
        let number = self.pop(context);
        let negate = self.pop(context);
        match (&number, &negate) {
            (
                Some(CalcGrammarItem::Num(num)),
                Some(CalcGrammarItem::UnaryOp(UnaryOperator::Negation)),
            ) => {
                self.push(CalcGrammarItem::Num(-num), context);
                Ok(())
            }
            _ => Err(parol!("{}: unexpected {:?} {:?}", context, negate, number)),
        }
    }

    /// Semantic action for production 79:
    ///
    /// Number: "[0-9]+";
    ///
    fn number(&mut self, tk_number: &ParseTreeType<'_>) -> Result<()> {
        let context = "number";
        let symbol = tk_number.text()?;
        let number = match symbol.parse::<DefinitionRange>() {
            Ok(number) => number,
            Err(error) => {
                bail!(CalcError::ParseISizeFailed {
                    context: context.to_owned(),
                    input: FileSource::try_new(tk_number.token()?.location.file_name.clone())
                        .map_err(|e| parol!(e))?,
                    token: tk_number.token()?.into(),
                    source: anyhow::anyhow!(error),
                })
            }
        };
        self.push(CalcGrammarItem::Num(number), context);
        Ok(())
    }

    /// Semantic action for production 80:
    ///
    /// IdRef: Id;
    ///
    fn id_ref(&mut self, _id: &ParseTreeType<'_>) -> Result<()> {
        let context = "idref";
        let top_of_stack = self.pop(context);
        match top_of_stack {
            Some(CalcGrammarItem::Id(text, location)) => {
                if let Some(val) = self.value(&text) {
                    self.push(CalcGrammarItem::Num(val), context);
                } else {
                    bail!(CalcError::UndeclaredVariable {
                        context: context.to_owned(),
                        input: FileSource::try_new(location.file_name.clone())
                            .map_err(|e| parol!(e))?,
                        token: location,
                    });
                }
                Ok(())
            }
            _ => Err(parol!("{}: unexpected {:?}", context, top_of_stack)),
        }
    }

    /// Semantic action for production 81:
    ///
    /// id: "[a-zA-Z_]\w*";
    ///
    fn id(&mut self, tk_id: &ParseTreeType<'_>) -> Result<()> {
        let context = "id";
        let id = tk_id.token()?;
        self.push(
            CalcGrammarItem::Id(id.text().to_string(), id.location.clone()),
            context,
        );
        Ok(())
    }
}
