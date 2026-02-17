using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using Parol.Runtime;

namespace CalcCsharp
{
    public sealed class CalcEvaluatorActions : CalcCsharpActions
    {
        private readonly List<long> _calcResults = [];
        private readonly SortedDictionary<string, long> _env = new(StringComparer.Ordinal);

        public override void OnCalc(Calc arg)
        {
            ProcessCalc(arg);
        }

        public override string ToString()
        {
            var builder = new StringBuilder();
            builder.AppendLine("Unassigned results");
            foreach (var value in _calcResults.AsEnumerable().Reverse())
            {
                builder.AppendLine(value.ToString(CultureInfo.InvariantCulture));
            }

            builder.AppendLine();
            builder.AppendLine("Env");
            foreach (var entry in _env)
            {
                builder.AppendLine($"{entry.Key} = {entry.Value}");
            }

            return builder.ToString().TrimEnd();
        }

        private void ProcessCalc(Calc calc)
        {
            foreach (var entry in calc.CalcList)
            {
                ProcessInstruction(entry.Instruction);
            }
        }

        private void ProcessInstruction(Instruction instruction)
        {
            switch (instruction)
            {
                case InstructionAssignmentVariant assignment:
                    ProcessAssignment(assignment.Value.Assignment);
                    break;
                case InstructionLogicalOrVariant logicalOr:
                    _calcResults.Add(EvaluateLogicalOr(logicalOr.Value.LogicalOr));
                    break;
                default:
                    throw new InvalidOperationException($"Unsupported instruction variant: {instruction.GetType().Name}");
            }
        }

        private void ProcessAssignment(Assignment assignment)
        {
            var result = EvaluateLogicalOr(assignment.LogicalOr);

            var assignmentItems = new List<AssignItem> { assignment.AssignItem };
            assignmentItems.AddRange(assignment.AssignmentList.Select(item => item.AssignItem));

            for (var index = assignmentItems.Count - 1; index >= 0; index--)
            {
                var assignItem = assignmentItems[index];
                var variableName = assignItem.Id.IdValue.Text;
                var op = assignItem.AssignOp.AssignOpValue.Text;

                if (!_env.TryGetValue(variableName, out var currentValue))
                {
                    currentValue = 0;
                    _env[variableName] = 0;
                }

                var updated = ApplyAssignOperation(currentValue, op, result);
                _env[variableName] = updated;
                result = updated;
            }
        }

        private long EvaluateLogicalOr(LogicalOr logicalOr)
        {
            var result = EvaluateLogicalAnd(logicalOr.LogicalAnd);
            foreach (var item in logicalOr.LogicalOrList)
            {
                result = ApplyBinaryOperation(result, item.LogicalOrOp.LogicalOrOpValue.Text, EvaluateLogicalAnd(item.LogicalAnd));
            }
            return result;
        }

        private long EvaluateLogicalAnd(LogicalAnd logicalAnd)
        {
            var result = EvaluateBitwiseOr(logicalAnd.BitwiseOr);
            foreach (var item in logicalAnd.LogicalAndList)
            {
                result = ApplyBinaryOperation(result, item.LogicalAndOp.LogicalAndOpValue.Text, EvaluateBitwiseOr(item.BitwiseOr));
            }
            return result;
        }

        private long EvaluateBitwiseOr(BitwiseOr bitwiseOr)
        {
            var result = EvaluateBitwiseAnd(bitwiseOr.BitwiseAnd);
            foreach (var item in bitwiseOr.BitwiseOrList)
            {
                result = ApplyBinaryOperation(result, item.BitwiseOrOp.BitwiseOrOpValue.Text, EvaluateBitwiseAnd(item.BitwiseAnd));
            }
            return result;
        }

        private long EvaluateBitwiseAnd(BitwiseAnd bitwiseAnd)
        {
            var result = EvaluateEquality(bitwiseAnd.Equality);
            foreach (var item in bitwiseAnd.BitwiseAndList)
            {
                result = ApplyBinaryOperation(result, item.BitwiseAndOp.BitwiseAndOpValue.Text, EvaluateEquality(item.Equality));
            }
            return result;
        }

        private long EvaluateEquality(Equality equality)
        {
            var result = EvaluateRelational(equality.Relational);
            foreach (var item in equality.EqualityList)
            {
                result = ApplyBinaryOperation(result, item.EqualityOp.EqualityOpValue.Text, EvaluateRelational(item.Relational));
            }
            return result;
        }

        private long EvaluateRelational(Relational relational)
        {
            var result = EvaluateBitwiseShift(relational.BitwiseShift);
            foreach (var item in relational.RelationalList)
            {
                result = ApplyBinaryOperation(result, item.RelationalOp.RelationalOpValue.Text, EvaluateBitwiseShift(item.BitwiseShift));
            }
            return result;
        }

        private long EvaluateBitwiseShift(BitwiseShift bitwiseShift)
        {
            var result = EvaluateSumm(bitwiseShift.Summ);
            foreach (var item in bitwiseShift.BitwiseShiftList)
            {
                result = ApplyBinaryOperation(result, item.BitwiseShiftOp.BitwiseShiftOpValue.Text, EvaluateSumm(item.Summ));
            }
            return result;
        }

        private long EvaluateSumm(Summ summ)
        {
            var result = EvaluateMult(summ.Mult);
            foreach (var item in summ.SummList)
            {
                var op = item.AddOp switch
                {
                    AddOpPlusVariant => "+",
                    AddOpMinusVariant => "-",
                    _ => throw new InvalidOperationException($"Unsupported add op variant: {item.AddOp.GetType().Name}")
                };
                result = ApplyBinaryOperation(result, op, EvaluateMult(item.Mult));
            }
            return result;
        }

        private long EvaluateMult(Mult mult)
        {
            var result = EvaluatePower(mult.Power);
            foreach (var item in mult.MultList)
            {
                result = ApplyBinaryOperation(result, item.MultOp.MultOpValue.Text, EvaluatePower(item.Power));
            }
            return result;
        }

        private long EvaluatePower(Power power)
        {
            var factors = new List<long> { EvaluateFactor(power.Factor) };
            factors.AddRange(power.PowerList.Select(item => EvaluateFactor(item.Factor)));

            var result = factors[^1];
            for (var index = factors.Count - 2; index >= 0; index--)
            {
                result = ApplyBinaryOperation(factors[index], "**", result);
            }

            return result;
        }

        private long EvaluateFactor(Factor factor)
        {
            return factor switch
            {
                FactorNumberVariant number => ParseNumber(number.Value.Number),
                FactorIdRefVariant idRef => LookupVariable(idRef.Value.IdRef.Id.IdValue.Text),
                FactorNegateFactorVariant negated => -EvaluateFactor(negated.Value.Factor),
                FactorLParenLogicalOrRParenVariant grouped => EvaluateLogicalOr(grouped.Value.LogicalOr),
                _ => throw new InvalidOperationException($"Unsupported factor variant: {factor.GetType().Name}")
            };
        }

        private static long ParseNumber(Number number)
        {
            return long.Parse(number.NumberValue.Text, CultureInfo.InvariantCulture);
        }

        private long LookupVariable(string name)
        {
            if (_env.TryGetValue(name, out var value))
            {
                return value;
            }

            throw new InvalidOperationException($"Undeclared variable: {name}");
        }

        private static long ApplyAssignOperation(long lhs, string op, long rhs)
        {
            return op switch
            {
                "=" => rhs,
                "+=" => lhs + rhs,
                "-=" => lhs - rhs,
                "*=" => lhs * rhs,
                "/=" => rhs == 0 ? throw new DivideByZeroException("Division by zero detected") : lhs / rhs,
                "%=" => rhs == 0 ? throw new DivideByZeroException("Division by zero detected") : lhs % rhs,
                "<<=" => lhs << checked((int)rhs),
                ">>=" => lhs >> checked((int)rhs),
                "&=" => lhs & rhs,
                "^=" => lhs ^ rhs,
                "|=" => lhs | rhs,
                _ => throw new InvalidOperationException($"Unsupported assignment operator: {op}")
            };
        }

        private static long ApplyBinaryOperation(long lhs, string op, long rhs)
        {
            return op switch
            {
                "+" => lhs + rhs,
                "-" => lhs - rhs,
                "*" => lhs * rhs,
                "/" => rhs == 0 ? throw new DivideByZeroException("Division by zero detected") : lhs / rhs,
                "%" => rhs == 0 ? throw new DivideByZeroException("Division by zero detected") : lhs % rhs,
                "**" => Pow(lhs, rhs),
                "==" => lhs == rhs ? 1 : 0,
                "!=" => lhs != rhs ? 1 : 0,
                "<" => lhs < rhs ? 1 : 0,
                "<=" => lhs <= rhs ? 1 : 0,
                ">" => lhs > rhs ? 1 : 0,
                ">=" => lhs >= rhs ? 1 : 0,
                "<<" => lhs << checked((int)rhs),
                ">>" => lhs >> checked((int)rhs),
                "&" => lhs & rhs,
                "|" => lhs | rhs,
                "&&" => (lhs != 0 && rhs != 0) ? 1 : 0,
                "||" => (lhs != 0 || rhs != 0) ? 1 : 0,
                _ => throw new InvalidOperationException($"Unsupported binary operator: {op}")
            };
        }

        private static long Pow(long lhs, long rhs)
        {
            if (rhs < 0)
            {
                throw new InvalidOperationException($"Exponent {rhs} cannot be negative");
            }

            if (rhs > int.MaxValue)
            {
                throw new InvalidOperationException($"Exponent {rhs} is too large");
            }

            return checked((long)Math.Pow(lhs, rhs));
        }
    }
}
