use crate::{
    assign_operator::AssignOperator,
    binary_operator::BinaryOperator,
    calc_grammar_trait::{
        AddOp, Assignment, AssignmentList, BitwiseAnd, BitwiseOr, BitwiseShift, Calc,
        CalcGrammarTrait, CalcList, Equality, Factor, Factor54, Factor55, Factor56, Factor57,
        Instruction, LogicalAnd, LogicalOr, Mult, Power, Relational, Summ,
    },
    errors::CalcError,
};
use log::trace;
use miette::{miette, Result, WrapErr};
use parol_runtime::{errors::FileSource, lexer::OwnedToken};
use std::{
    collections::BTreeMap,
    convert::TryInto,
    fmt::{Debug, Display, Error, Formatter},
    path::{Path, PathBuf},
};

///
/// The value range for the supported calculations
///
pub type DefinitionRange = isize;

///
/// Data structure that implements the semantic actions for our calc grammar
///
#[derive(Debug, Default)]
pub struct CalcGrammar {
    pub calc_results: Vec<DefinitionRange>,
    pub env: BTreeMap<String, DefinitionRange>,
    file_name: PathBuf,
}

impl CalcGrammar {
    pub fn new() -> Self {
        CalcGrammar::default()
    }

    fn value(&self, id: &OwnedToken) -> Result<DefinitionRange> {
        self.env
            .get(&id.symbol)
            .cloned()
            .ok_or(miette!(CalcError::UndeclaredVariable {
                context: "value".to_owned(),
                input: FileSource::try_new(self.file_name.clone())?.into(),
                token: id.into()
            }))
    }

    fn declare(&mut self, id: &str, context: &str) {
        if !self.env.contains_key(id) {
            trace!("declare {}: {}", context, id);
            self.env.insert(id.to_owned(), 0);
        }
    }

    fn parse_number(&self, context: &str, token: &OwnedToken) -> Result<DefinitionRange> {
        match token.symbol.parse::<DefinitionRange>() {
            Ok(number) => Ok(number),
            Err(error) => Err(miette!(CalcError::ParseISizeFailed {
                context: context.to_owned(),
                input: FileSource::try_new(self.file_name.clone())?.into(),
                token: token.into()
            }))
            .wrap_err(miette!(error)),
        }
    }

    fn apply_assign_operation(
        lhs: &mut DefinitionRange,
        op: &AssignOperator,
        rhs: DefinitionRange,
        context: &str,
    ) -> Result<DefinitionRange> {
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
            AssignOperator::BitwiseXOr => *lhs ^= rhs,
            AssignOperator::BitwiseOr => *lhs |= rhs,
        }
        trace!("apply_assign_item:      = {}", lhs);
        Ok(*lhs)
    }

    fn apply_binary_operation(
        lhs: DefinitionRange,
        op: &BinaryOperator,
        rhs: DefinitionRange,
        context: &str,
    ) -> Result<DefinitionRange> {
        trace!(
            "apply_binary_operation: {}: {} {} {}",
            context,
            lhs,
            op,
            rhs
        );
        let result = match op {
            BinaryOperator::Add => lhs + rhs,
            BinaryOperator::Sub => lhs - rhs,
            BinaryOperator::Mul => lhs * rhs,
            BinaryOperator::Div => {
                if rhs == 0 {
                    bail!("Division by zero detected!");
                }
                lhs / rhs
            }
            BinaryOperator::Mod => lhs % rhs,
            BinaryOperator::Pow => {
                if let Ok(exponent) = rhs.try_into() {
                    lhs.pow(exponent)
                } else {
                    bail!("Exponent {} can't be converted to u32!", rhs);
                }
            }
            BinaryOperator::Eq => (lhs == rhs) as DefinitionRange,
            BinaryOperator::Ne => (lhs != rhs) as DefinitionRange,
            BinaryOperator::Lt => (lhs < rhs) as DefinitionRange,
            BinaryOperator::Le => (lhs <= rhs) as DefinitionRange,
            BinaryOperator::Gt => (lhs > rhs) as DefinitionRange,
            BinaryOperator::Ge => (lhs >= rhs) as DefinitionRange,
            BinaryOperator::BitShl => lhs << rhs,
            BinaryOperator::BitShr => lhs >> rhs,
            BinaryOperator::BitAnd => lhs & rhs,
            BinaryOperator::BitOr => lhs | rhs,
            BinaryOperator::LogAnd => ((lhs != 0) && (rhs != 0)) as DefinitionRange,
            BinaryOperator::LogOr => ((lhs != 0) || (rhs != 0)) as DefinitionRange,
        };

        trace!("apply_binary_operation:      = {}", result);

        Ok(result)
    }

    fn process_calc(&mut self, calc: &Calc) -> Result<()> {
        calc.calc_list_0.iter().fold(Ok(()), |res, elem| {
            res?;
            self.process_calc_list(elem)
        })
    }

    fn process_calc_list(&mut self, elem: &CalcList) -> Result<()> {
        self.process_instruction(&elem.instruction_0)
    }

    fn process_instruction(&mut self, insn: &Instruction) -> Result<()> {
        match insn {
            Instruction::Instruction15(ins) => self.process_assignment(&ins.assignment_0),
            Instruction::Instruction16(ins) => self
                .process_logical_or(&ins.logical_or_0)
                .map(|r| self.calc_results.push(r)),
        }
    }

    fn process_assignment(&mut self, assignment: &Assignment) -> Result<()> {
        let context = "process_assignment";
        let mut result = self.process_logical_or(&assignment.logical_or_2)?;
        let mut assignment_list = assignment.assignment_list_1.clone();
        // Prepend the left most (mandatory) assign item
        assignment_list.insert(
            0,
            AssignmentList {
                assign_item_0: assignment.assign_item_0.clone(),
            },
        );
        // Assign from right to left (right associative)
        for assign_item in assignment_list.iter().rev() {
            let id = &assign_item.assign_item_0.id_0.id_0.symbol;
            let op = assign_item
                .assign_item_0
                .assign_op_1
                .assign_op_0
                .symbol
                .as_str()
                .try_into()?;
            self.declare(id, context);
            if let Some(var) = self.env.get_mut(id) {
                trace!("assign: to variable {}", id);
                result = Self::apply_assign_operation(var, &op, result, context)?;
            } else {
                Err(miette!(CalcError::UndeclaredVariable {
                    context: "value".to_owned(),
                    input: FileSource::try_new(self.file_name.clone())?.into(),
                    token: (&assign_item.assign_item_0.id_0.id_0).into()
                }))?
            }
        }
        Ok(())
    }

    fn process_logical_or(&mut self, logical_or: &LogicalOr) -> Result<DefinitionRange> {
        let context = "process_logical_or";
        let mut result = self.process_logical_and(&logical_or.logical_and_0)?;
        for item in &logical_or.logical_or_list_1 {
            let op: BinaryOperator = item
                .logical_or_op_0
                .logical_or_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_logical_and(&item.logical_and_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_logical_and(&mut self, logical_and: &LogicalAnd) -> Result<DefinitionRange> {
        let context = "process_logical_and";
        let mut result = self.process_bitwise_or(&logical_and.bitwise_or_0)?;
        for item in &logical_and.logical_and_list_1 {
            let op: BinaryOperator = item
                .logical_and_op_0
                .logical_and_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_bitwise_or(&item.bitwise_or_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_bitwise_or(&mut self, bitwise_or: &BitwiseOr) -> Result<DefinitionRange> {
        let context = "process_bitwise_or";
        let mut result = self.process_bitwise_and(&bitwise_or.bitwise_and_0)?;
        for item in &bitwise_or.bitwise_or_list_1 {
            let op: BinaryOperator = item
                .bitwise_or_op_0
                .bitwise_or_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_bitwise_and(&item.bitwise_and_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_bitwise_and(&mut self, bitwise_and: &BitwiseAnd) -> Result<DefinitionRange> {
        let context = "process_bitwise_and";
        let mut result = self.process_equality(&bitwise_and.equality_0)?;
        for item in &bitwise_and.bitwise_and_list_1 {
            let op: BinaryOperator = item
                .bitwise_and_op_0
                .bitwise_and_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_equality(&item.equality_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_equality(&mut self, equality: &Equality) -> Result<DefinitionRange> {
        let context = "process_equality";
        let mut result = self.process_relational(&equality.relational_0)?;
        for item in &equality.equality_list_1 {
            let op: BinaryOperator = item
                .equality_op_0
                .equality_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_relational(&item.relational_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_relational(&mut self, relational: &Relational) -> Result<DefinitionRange> {
        let context = "process_relational";
        let mut result = self.process_bitwise_shift(&relational.bitwise_shift_0)?;
        for item in &relational.relational_list_1 {
            let op: BinaryOperator = item
                .relational_op_0
                .relational_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_bitwise_shift(&item.bitwise_shift_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_bitwise_shift(&mut self, bitwise_shift: &BitwiseShift) -> Result<DefinitionRange> {
        let context = "process_bitwise_shift";
        let mut result = self.process_sum(&bitwise_shift.summ_0)?;
        for item in &bitwise_shift.bitwise_shift_list_1 {
            let op: BinaryOperator = item
                .bitwise_shift_op_0
                .bitwise_shift_op_0
                .symbol
                .as_str()
                .try_into()?;
            let next_operand = self.process_sum(&item.summ_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_sum(&mut self, summ: &Summ) -> Result<DefinitionRange> {
        let context = "process_sum";
        let mut result = self.process_mult(&summ.mult_0)?;
        for item in &summ.summ_list_1 {
            let op: BinaryOperator = match &*item.add_op_0 {
                AddOp::AddOp42(plus) => plus.plus_0.plus_0.symbol.as_str().try_into(),
                AddOp::AddOp43(minus) => minus.minus_0.minus_0.symbol.as_str().try_into(),
            }?;
            let next_operand = self.process_mult(&item.mult_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_mult(&mut self, mult: &Mult) -> Result<DefinitionRange> {
        let context = "process_mult";
        let mut result = self.process_power(&mult.power_0)?;
        for item in &mult.mult_list_1 {
            let op: BinaryOperator = item.mult_op_0.mult_op_0.symbol.as_str().try_into()?;
            let next_operand = self.process_power(&item.power_1)?;
            result = Self::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_power(&mut self, power: &Power) -> Result<DefinitionRange> {
        let context = "process_power";
        let op = BinaryOperator::Pow;
        // Calculate from right to left (right associative)
        let result = power.power_list_1.iter().rev().fold(Ok(1), |acc, f| {
            if acc.is_err() {
                return acc;
            }
            let val = self.process_factor(&f.factor_1)?;
            Self::apply_binary_operation(val, &op, acc.unwrap(), context)
        })?;
        Self::apply_binary_operation(self.process_factor(&power.factor_0)?, &op, result, context)
    }

    fn process_factor(&mut self, factor: &Factor) -> Result<DefinitionRange> {
        let context = "process_factor";
        match factor {
            Factor::Factor54(Factor54 { number_0 }) => {
                Ok(self.parse_number(context, &number_0.number_0)?)
            }
            Factor::Factor55(Factor55 { idref_0 }) => self.value(&idref_0.id_0.id_0),
            Factor::Factor56(Factor56 { factor_1, .. }) => Ok(-(self.process_factor(factor_1)?)),
            Factor::Factor57(Factor57 { logical_or_1, .. }) => {
                self.process_logical_or(logical_or_1)
            }
        }
    }
}

impl Display for CalcGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(
            f,
            "Unassigned results\n{}",
            self.calc_results
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
    fn init(&mut self, file_name: &Path) {
        self.file_name = file_name.into();
    }

    /// Semantic action for user production 0:
    ///
    /// calc: {instruction <0>";"};
    ///
    fn calc(&mut self, arg: &Calc) -> Result<()> {
        self.process_calc(arg)?;
        Ok(())
    }
}
