use crate::{
    basic_grammar_trait::*,
    errors::BasicError,
    operators::{BinaryOperator, UnaryOperator},
};
#[allow(unused_imports)]
use anyhow::{anyhow, bail, Context};
use parol_runtime::{
    errors::FileSource,
    lexer::{Location, Token},
};
use parol_runtime::{log::trace, ParolError, Result};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Error, Formatter},
    marker::PhantomData,
};

///
/// The value range for the supported calculations
///
pub type DefinitionRange = f32;

///
/// The value range for line numbers
///
pub type LineNumberRange = u16;

const MAX_LINE_NUMBER: u16 = 63999;

#[derive(Clone, Debug, Default)]
pub struct BasicNumber(DefinitionRange);

impl<'t> TryFrom<&Token<'t>> for BasicNumber {
    type Error = anyhow::Error;

    fn try_from(basic_line_number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        let symbol = basic_line_number.text().replace(' ', "").replace('E', "e");
        Ok(Self(symbol.parse::<DefinitionRange>()?))
    }
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct BasicLineNumber(LineNumberRange, Location);

impl<'t> TryFrom<&Token<'t>> for BasicLineNumber {
    type Error = anyhow::Error;

    fn try_from(basic_line_number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        let symbol = basic_line_number.text().replace(' ', "");
        Ok(Self(
            symbol.parse::<LineNumberRange>()?,
            basic_line_number.location.clone(),
        ))
    }
}

#[derive(Debug)]
struct CompiledLine<'a, 't>
where
    't: 'a,
{
    statements: Vec<&'a Statement<'t>>,
    next_line: Option<LineNumberRange>,
}

#[derive(Debug, Default)]
pub struct BasicLines<'a, 't> {
    lines: BTreeMap<u16, (Location, CompiledLine<'a, 't>)>,
}

///
/// Data structure that implements the semantic actions for our Basic grammar
///
#[derive(Debug, Default)]
pub struct BasicGrammar<'t> {
    pub env: BTreeMap<String, DefinitionRange>,
    next_line: Option<LineNumberRange>,
    last_line: Option<LineNumberRange>,
    phantom: PhantomData<&'t str>, // Just to hold the lifetime generated by parol
}

impl<'t> BasicGrammar<'t> {
    pub fn new() -> Self {
        BasicGrammar::default()
    }

    fn value(&self, context: &str, id: &Token<'t>) -> DefinitionRange {
        let name: &str = if id.text().len() < 2 {
            id.text()
        } else {
            &id.text()[..2]
        };
        let value = self.env.get(name).cloned().unwrap_or_default();
        trace!("value @ {context}: {name} = {value}");
        value
    }

    fn set_value(&mut self, id: &str, context: &str, value: DefinitionRange) {
        let name: &str = if id.len() < 2 { id } else { &id[..2] };
        trace!("set_value @ {context}: {name} = {value}");
        self.env.insert(name.to_owned(), value);
    }

    fn process_basic(&mut self, basic: &Basic<'t>) -> Result<()> {
        self.process_lines(&basic.line, &basic.basic_list)
    }

    fn process_lines(
        &mut self,
        first_line: &Line<'t>,
        other_lines: &[BasicList<'t>],
    ) -> Result<()> {
        let lines = self.pre_process_lines(first_line, other_lines)?;
        self.interpret(&lines)
    }

    fn pre_process_lines<'a>(
        &mut self,
        first_line: &'a Line<'t>,
        other_lines: &'a [BasicList<'t>],
    ) -> Result<BasicLines<'a, 't>> {
        let context = "pre_process_lines";

        let mut lines = BasicLines::default();
        let (k, v) = self.pre_process_line(first_line)?;
        lines.lines.insert(k.0, (k.1, v));

        for line in other_lines {
            let (k, v) = self.pre_process_line(&line.line)?;
            if lines.lines.insert(k.0, (k.1.clone(), v)).is_some() {
                return Err(ParolError::UserError(
                    BasicError::LineNumberDefinedTwice {
                        context: context.to_owned(),
                        input: FileSource::try_new(k.1.file_name.clone())
                            .map_err(anyhow::Error::from)?,
                        token: k.1,
                    }
                    .into(),
                ));
            }
        }

        // Add the follow relation
        self.next_line = None;
        lines.lines.iter_mut().rev().for_each(|(k, v)| {
            v.1.next_line = self.next_line;
            self.next_line = Some(*k);
        });

        // We need the last line only to be able to abort search for undefine destination in GOTO
        self.last_line = lines.lines.iter().rev().next().map(|l| *l.0);

        Ok(lines)
    }

    fn pre_process_line<'a>(
        &mut self,
        line: &'a Line<'t>,
    ) -> Result<(BasicLineNumber, CompiledLine<'a, 't>)> {
        let context = "pre_process_line";
        let basic_line_number = &line.line_number.line_number;
        if basic_line_number.0 > MAX_LINE_NUMBER {
            return Err(BasicError::LineNumberTooLarge {
                context: context.to_owned(),
                input: FileSource::try_new(basic_line_number.1.file_name.clone())
                    .map_err(anyhow::Error::from)?,
                token: basic_line_number.1.clone(),
            }
            .into());
        }

        // On each line there can exist multiple statements separated by colons!
        let mut statements = vec![line.statement.as_ref()];

        line.line_list.iter().for_each(|statement| {
            statements.push(statement.statement.as_ref());
        });

        let compiled_line = CompiledLine {
            statements,
            next_line: None,
        };

        Ok((basic_line_number.clone(), compiled_line))
    }

    fn interpret<'a>(&mut self, lines: &BasicLines<'a, 't>) -> Result<()> {
        while self.next_line.is_some() {
            self.interpret_line(lines)?;
        }
        Ok(())
    }

    fn interpret_line<'a>(&mut self, lines: &BasicLines<'a, 't>) -> Result<()> {
        if let Some(current_line) = lines.lines.get(&self.next_line.unwrap()) {
            self.next_line = current_line.1.next_line;
            let mut continue_statements = true;
            for statement in &current_line.1.statements {
                self.interpret_statement(statement, lines, &mut continue_statements)?;
                if !continue_statements {
                    break;
                }
            }
        } else {
        }
        Ok(())
    }

    fn interpret_statement<'a>(
        &mut self,
        statement: &'a Statement<'t>,
        lines: &BasicLines<'a, 't>,
        continue_statements: &mut bool,
    ) -> Result<()> {
        *continue_statements = true;
        match statement {
            Statement::Remark(remark) => self.process_remark(remark),
            Statement::GotoStatement(goto) => {
                *continue_statements = false;
                self.process_goto(&goto.goto_statement.line_number.line_number, lines)
            }
            Statement::IfStatement(if_statement) => {
                self.process_if_statement(if_statement, continue_statements, lines)
            }
            Statement::Assignment(assign) => self.process_assign(assign),
            Statement::PrintStatement(print_statement) => {
                self.process_print_statement(print_statement)
            }
            Statement::EndStatement(end_statement) => {
                *continue_statements = false;
                self.process_end_statement(end_statement)
            }
        }
    }

    fn process_remark(&self, _remark: &StatementRemark) -> Result<()> {
        Ok(())
    }

    fn process_goto<'a>(
        &mut self,
        basic_line_number: &BasicLineNumber,
        lines: &BasicLines<'a, 't>,
    ) -> Result<()> {
        let context = "process_goto";
        let mut line_number = basic_line_number.0;

        let last_line = self.last_line.unwrap();
        while !lines.lines.contains_key(&line_number) && line_number < last_line {
            // A GOTO statement targets a line that doesn't exist.
            // The Commodore BASIC defines that the execution is proceeded at the first
            // executable statement encountered after the given line number.
            line_number += 1;
            trace!("Trying next line: {line_number}");
        }

        if !lines.lines.contains_key(&line_number) {
            return Err(BasicError::LineNumberBeyondLastLine {
                context: context.to_owned(),
                input: FileSource::try_new(basic_line_number.1.file_name.clone())
                    .map_err(anyhow::Error::from)?,
                token: basic_line_number.1.clone(),
            }
            .into());
        }

        trace!("{context}: setting next line to {line_number}");
        self.next_line = Some(line_number);
        Ok(())
    }

    fn process_if_statement<'a>(
        &mut self,
        if_statement: &'a StatementIfStatement<'t>,
        continue_statements: &mut bool,
        lines: &BasicLines<'a, 't>,
    ) -> Result<()> {
        let context = "process_if_statement";
        *continue_statements = true;
        let condition = self.process_expression(&if_statement.if_statement.expression)?;
        trace!("{context}: condition: {condition}");
        if condition != 0.0 {
            match &*if_statement.if_statement.if_body {
                IfBody::ThenStatement(then) => {
                    self.interpret_statement(&then.statement, lines, continue_statements)
                }
                IfBody::GotoLineNumber(goto) => {
                    self.process_goto(&goto.line_number.line_number, lines)
                }
            }
        } else {
            Ok(())
        }
    }

    fn process_assign(&mut self, assign: &StatementAssignment) -> Result<()> {
        let context = "process_assign";
        let value = self.process_expression(&assign.assignment.expression)?;
        let symbol = assign.assignment.variable.variable.text();
        trace!("{context}: {symbol} = {value}");
        self.set_value(symbol, context, value);
        Ok(())
    }

    fn process_print_statement(&mut self, print_statement: &StatementPrintStatement) -> Result<()> {
        let value = self.process_expression(&print_statement.print_statement.expression)?;
        print!("{value} ");
        for elem in &print_statement.print_statement.print_statement_list {
            let value = self.process_expression(&elem.expression)?;
            print!("{value} ");
        }
        Ok(())
    }

    fn process_end_statement(&mut self, _end_statement: &StatementEndStatement) -> Result<()> {
        let context = "process_end_statement";
        trace!("{context}: setting next line to None");
        self.next_line = None;
        Ok(())
    }

    fn process_expression(&mut self, expression: &Expression) -> Result<DefinitionRange> {
        self.process_logical_or(&expression.logical_or)
    }

    fn process_logical_or(&mut self, logical_or: &LogicalOr) -> Result<DefinitionRange> {
        let context = "process_logical_or";
        let mut result = self.process_logical_and(&logical_or.logical_and)?;
        for item in &logical_or.logical_or_list {
            let op: BinaryOperator = item.logical_or_op.logical_or_op.text().try_into()?;
            let next_operand = self.process_logical_and(&item.logical_and)?;
            result = BinaryOperator::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_logical_and(&mut self, logical_and: &LogicalAnd) -> Result<DefinitionRange> {
        let context = "process_logical_and";
        let mut result = self.process_logical_not(&logical_and.logical_not)?;
        for item in &logical_and.logical_and_list {
            let op: BinaryOperator = item.logical_and_op.logical_and_op.text().try_into()?;
            let next_operand = self.process_logical_not(&item.logical_not)?;
            result = BinaryOperator::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_logical_not(&mut self, logical_not: &LogicalNot) -> Result<DefinitionRange> {
        let context = "process_logical_not";
        if let Some(not) = &logical_not.logical_not_opt {
            let result = self.process_relational(&logical_not.relational)?;
            let op: UnaryOperator = not.logical_not_op.logical_not_op.text().try_into()?;
            Ok(UnaryOperator::apply_unary_operation(&op, result, context))
        } else {
            self.process_relational(&logical_not.relational)
        }
    }

    fn process_relational(&mut self, relational: &Relational) -> Result<DefinitionRange> {
        let context = "process_relational";
        let mut result = self.process_summation(&relational.summation)?;
        for item in &relational.relational_list {
            let op: BinaryOperator = item.relational_op.relational_op.text().try_into()?;
            let next_operand = self.process_summation(&item.summation)?;
            result = BinaryOperator::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_summation(&mut self, summation: &Summation) -> Result<DefinitionRange> {
        let context = "process_summation";
        let mut result = self.process_multiplication(&summation.multiplication)?;
        for item in &summation.summation_list {
            let op: BinaryOperator = match &*item.summation_list_group {
                SummationListGroup::Plus(plus) => plus.plus.plus.text().try_into(),
                SummationListGroup::Minus(minus) => minus.minus.minus.text().try_into(),
            }?;
            let next_operand = self.process_multiplication(&item.multiplication)?;
            result = BinaryOperator::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_multiplication(
        &mut self,
        multiplication: &Multiplication,
    ) -> Result<DefinitionRange> {
        let context = "process_multiplication";
        let mut result = self.process_factor(&multiplication.factor)?;
        for item in &*multiplication.multiplication_list {
            let op: BinaryOperator = item.mul_op.mul_op.text().try_into()?;
            let next_operand = self.process_factor(&item.factor)?;
            result = BinaryOperator::apply_binary_operation(result, &op, next_operand, context)?;
        }
        Ok(result)
    }

    fn process_factor(&mut self, factor: &Factor) -> Result<DefinitionRange> {
        let context = "process_factor";
        match factor {
            Factor::Literal(FactorLiteral { literal }) => match &*literal.number {
                Number::Float(flt) => match &*flt.float {
                    Float::Float1(float) => Ok(float.float1.float1.0),
                    Float::Float2(float) => Ok(float.float2.float2.0),
                },
                Number::Integer(int) => Ok(int.integer.integer.0),
            },
            Factor::Variable(FactorVariable { variable }) => {
                Ok(self.value(context, &variable.variable))
            }
            Factor::MinusFactor(FactorMinusFactor { factor, .. }) => {
                Ok(-(self.process_factor(factor)?))
            }
            Factor::LParenExpressionRParen(FactorLParenExpressionRParen { expression, .. }) => {
                self.process_expression(expression)
            }
        }
    }
}

impl Display for Basic<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, ":-)")
    }
}

impl Display for BasicGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "{}",
            self.env
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<'t> BasicGrammarTrait<'t> for BasicGrammar<'t> {
    /// Semantic action for non-terminal 'Basic'
    fn basic(&mut self, basic: &Basic<'t>) -> Result<()> {
        self.process_basic(basic)
    }
}

impl From<BasicError> for ParolError {
    fn from(error: BasicError) -> Self {
        ParolError::UserError(error.into())
    }
}
