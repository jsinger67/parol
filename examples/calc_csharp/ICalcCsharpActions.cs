using System;
using System.Collections.Generic;
using System.Reflection;
using Parol.Runtime;
using Parol.Runtime.Scanner;

namespace CalcCsharp {
    // Deduced grammar types
    // Type derived for non-terminal AddOp
    public abstract record AddOp;
    public sealed record AddOpPlusVariant(AddOpPlus Value) : AddOp;
    public sealed record AddOpMinusVariant(AddOpMinus Value) : AddOp;

    // Type derived for non-terminal AssignItem
    public sealed record AssignItem(Id Id, AssignOp AssignOp);

    // Type derived for non-terminal AssignOp
    public sealed record AssignOp(Token AssignOpValue);

    // Type derived for non-terminal Assignment
    public sealed record Assignment(AssignItem AssignItem, List<AssignmentList> AssignmentList, LogicalOr LogicalOr);

    // Type derived for non-terminal AssignmentList
    public sealed record AssignmentList(AssignItem AssignItem);

    // Type derived for non-terminal BitwiseAnd
    public sealed record BitwiseAnd(Equality Equality, List<BitwiseAndList> BitwiseAndList);

    // Type derived for non-terminal BitwiseAndList
    public sealed record BitwiseAndList(BitwiseAndOp BitwiseAndOp, Equality Equality);

    // Type derived for non-terminal BitwiseAndOp
    public sealed record BitwiseAndOp(Token BitwiseAndOpValue);

    // Type derived for non-terminal BitwiseOr
    public sealed record BitwiseOr(BitwiseAnd BitwiseAnd, List<BitwiseOrList> BitwiseOrList);

    // Type derived for non-terminal BitwiseOrList
    public sealed record BitwiseOrList(BitwiseOrOp BitwiseOrOp, BitwiseAnd BitwiseAnd);

    // Type derived for non-terminal BitwiseOrOp
    public sealed record BitwiseOrOp(Token BitwiseOrOpValue);

    // Type derived for non-terminal BitwiseShift
    public sealed record BitwiseShift(Summ Summ, List<BitwiseShiftList> BitwiseShiftList);

    // Type derived for non-terminal BitwiseShiftList
    public sealed record BitwiseShiftList(BitwiseShiftOp BitwiseShiftOp, Summ Summ);

    // Type derived for non-terminal BitwiseShiftOp
    public sealed record BitwiseShiftOp(Token BitwiseShiftOpValue);

    // Type derived for non-terminal Calc
    public sealed record Calc(List<CalcList> CalcList);

    // Type derived for non-terminal CalcList
    public sealed record CalcList(Instruction Instruction);

    // Type derived for non-terminal Equality
    public sealed record Equality(Relational Relational, List<EqualityList> EqualityList);

    // Type derived for non-terminal EqualityList
    public sealed record EqualityList(EqualityOp EqualityOp, Relational Relational);

    // Type derived for non-terminal EqualityOp
    public sealed record EqualityOp(Token EqualityOpValue);

    // Type derived for non-terminal Factor
    public abstract record Factor;
    public sealed record FactorNumberVariant(FactorNumber Value) : Factor;
    public sealed record FactorIdRefVariant(FactorIdRef Value) : Factor;
    public sealed record FactorNegateFactorVariant(FactorNegateFactor Value) : Factor;
    public sealed record FactorLParenLogicalOrRParenVariant(FactorLParenLogicalOrRParen Value) : Factor;

    // Type derived for non-terminal Id
    public sealed record Id(Token IdValue);

    // Type derived for non-terminal IdRef
    public sealed record IdRef(Id Id);

    // Type derived for non-terminal Instruction
    public abstract record Instruction;
    public sealed record InstructionAssignmentVariant(InstructionAssignment Value) : Instruction;
    public sealed record InstructionLogicalOrVariant(InstructionLogicalOr Value) : Instruction;

    // Type derived for non-terminal LogicalAnd
    public sealed record LogicalAnd(BitwiseOr BitwiseOr, List<LogicalAndList> LogicalAndList);

    // Type derived for non-terminal LogicalAndList
    public sealed record LogicalAndList(LogicalAndOp LogicalAndOp, BitwiseOr BitwiseOr);

    // Type derived for non-terminal LogicalAndOp
    public sealed record LogicalAndOp(Token LogicalAndOpValue);

    // Type derived for non-terminal LogicalOr
    public sealed record LogicalOr(LogicalAnd LogicalAnd, List<LogicalOrList> LogicalOrList);

    // Type derived for non-terminal LogicalOrList
    public sealed record LogicalOrList(LogicalOrOp LogicalOrOp, LogicalAnd LogicalAnd);

    // Type derived for non-terminal LogicalOrOp
    public sealed record LogicalOrOp(Token LogicalOrOpValue);

    // Type derived for non-terminal Minus
    public sealed record Minus(Token MinusValue);

    // Type derived for non-terminal Mult
    public sealed record Mult(Power Power, List<MultList> MultList);

    // Type derived for non-terminal MultList
    public sealed record MultList(MultOp MultOp, Power Power);

    // Type derived for non-terminal MultOp
    public sealed record MultOp(Token MultOpValue);

    // Type derived for non-terminal Negate
    public sealed record Negate(Minus Minus);

    // Type derived for non-terminal Number
    public sealed record Number(CalcCsharp.CalcNumber NumberValue);

    // Type derived for non-terminal Plus
    public sealed record Plus(Token PlusValue);

    // Type derived for non-terminal PowOp
    public sealed record PowOp(Token PowOpValue);

    // Type derived for non-terminal Power
    public sealed record Power(Factor Factor, List<PowerList> PowerList);

    // Type derived for non-terminal PowerList
    public sealed record PowerList(PowOp PowOp, Factor Factor);

    // Type derived for non-terminal Relational
    public sealed record Relational(BitwiseShift BitwiseShift, List<RelationalList> RelationalList);

    // Type derived for non-terminal RelationalList
    public sealed record RelationalList(RelationalOp RelationalOp, BitwiseShift BitwiseShift);

    // Type derived for non-terminal RelationalOp
    public sealed record RelationalOp(Token RelationalOpValue);

    // Type derived for non-terminal Summ
    public sealed record Summ(Mult Mult, List<SummList> SummList);

    // Type derived for non-terminal SummList
    public sealed record SummList(AddOp AddOp, Mult Mult);

    // Type derived for production 15
    public sealed record InstructionAssignment(Assignment Assignment);

    // Type derived for production 16
    public sealed record InstructionLogicalOr(LogicalOr LogicalOr);

    // Type derived for production 42
    public sealed record AddOpPlus(Plus Plus);

    // Type derived for production 43
    public sealed record AddOpMinus(Minus Minus);

    // Type derived for production 54
    public sealed record FactorNumber(Number Number);

    // Type derived for production 55
    public sealed record FactorIdRef(IdRef IdRef);

    // Type derived for production 56
    public sealed record FactorNegateFactor(Negate Negate, Factor Factor);

    // Type derived for production 57
    public sealed record FactorLParenLogicalOrRParen(LogicalOr LogicalOr);

    /// <summary>
    /// User actions interface for the CalcCsharp grammar.
    /// </summary>
    public interface ICalcCsharpActions : IUserActions, IProvidesValueConverter {
        void OnCalc(Calc arg);

        void OnEqualityOp(EqualityOp arg);

        void OnAssignOp(AssignOp arg);

        void OnLogicalOrOp(LogicalOrOp arg);

        void OnLogicalAndOp(LogicalAndOp arg);

        void OnBitwiseOrOp(BitwiseOrOp arg);

        void OnBitwiseAndOp(BitwiseAndOp arg);

        void OnBitwiseShiftOp(BitwiseShiftOp arg);

        void OnRelationalOp(RelationalOp arg);

        void OnPlus(Plus arg);

        void OnMinus(Minus arg);

        void OnPowOp(PowOp arg);

        void OnMultOp(MultOp arg);

        void OnInstruction(Instruction arg);

        void OnAssignItem(AssignItem arg);

        void OnAssignment(Assignment arg);

        void OnLogicalOr(LogicalOr arg);

        void OnLogicalAnd(LogicalAnd arg);

        void OnBitwiseOr(BitwiseOr arg);

        void OnBitwiseAnd(BitwiseAnd arg);

        void OnEquality(Equality arg);

        void OnRelational(Relational arg);

        void OnBitwiseShift(BitwiseShift arg);

        void OnAddOp(AddOp arg);

        void OnSumm(Summ arg);

        void OnMult(Mult arg);

        void OnPower(Power arg);

        void OnNegate(Negate arg);

        void OnFactor(Factor arg);

        void OnNumber(Number arg);

        void OnIdRef(IdRef arg);

        void OnId(Id arg);

        /// <summary>
        /// Semantic action for production 0:
        /// Calc: CalcList /* Vec */; 
        /// </summary>
        void Calc(object[] children);

        /// <summary>
        /// Semantic action for production 1:
        /// CalcList: Instruction ";"^ /* Clipped */ CalcList; 
        /// </summary>
        void CalcList0(object[] children);

        /// <summary>
        /// Semantic action for production 2:
        /// CalcList: ; 
        /// </summary>
        void CalcList1(object[] children);

        /// <summary>
        /// Semantic action for production 3:
        /// EqualityOp: "==|!="; 
        /// </summary>
        void EqualityOp(object[] children);

        /// <summary>
        /// Semantic action for production 4:
        /// AssignOp: "(\+|-|\*|/|%|<<|>>|&|\^|\|)?="; 
        /// </summary>
        void AssignOp(object[] children);

        /// <summary>
        /// Semantic action for production 5:
        /// LogicalOrOp: "\|\|"; 
        /// </summary>
        void LogicalOrOp(object[] children);

        /// <summary>
        /// Semantic action for production 6:
        /// LogicalAndOp: "&&"; 
        /// </summary>
        void LogicalAndOp(object[] children);

        /// <summary>
        /// Semantic action for production 7:
        /// BitwiseOrOp: "\|"; 
        /// </summary>
        void BitwiseOrOp(object[] children);

        /// <summary>
        /// Semantic action for production 8:
        /// BitwiseAndOp: "&"; 
        /// </summary>
        void BitwiseAndOp(object[] children);

        /// <summary>
        /// Semantic action for production 9:
        /// BitwiseShiftOp: "<<|>>"; 
        /// </summary>
        void BitwiseShiftOp(object[] children);

        /// <summary>
        /// Semantic action for production 10:
        /// RelationalOp: "<=|<|>=|>"; 
        /// </summary>
        void RelationalOp(object[] children);

        /// <summary>
        /// Semantic action for production 11:
        /// Plus: "\+"; 
        /// </summary>
        void Plus(object[] children);

        /// <summary>
        /// Semantic action for production 12:
        /// Minus: "-"; 
        /// </summary>
        void Minus(object[] children);

        /// <summary>
        /// Semantic action for production 13:
        /// PowOp: "\*\*"; 
        /// </summary>
        void PowOp(object[] children);

        /// <summary>
        /// Semantic action for production 14:
        /// MultOp: "\*|/|%"; 
        /// </summary>
        void MultOp(object[] children);

        /// <summary>
        /// Semantic action for production 15:
        /// Instruction: Assignment; 
        /// </summary>
        void Instruction0(object[] children);

        /// <summary>
        /// Semantic action for production 16:
        /// Instruction: LogicalOr; 
        /// </summary>
        void Instruction1(object[] children);

        /// <summary>
        /// Semantic action for production 17:
        /// AssignItem: Id AssignOp; 
        /// </summary>
        void AssignItem(object[] children);

        /// <summary>
        /// Semantic action for production 18:
        /// Assignment: AssignItem AssignmentList /* Vec */ LogicalOr; 
        /// </summary>
        void Assignment(object[] children);

        /// <summary>
        /// Semantic action for production 19:
        /// AssignmentList: AssignItem AssignmentList; 
        /// </summary>
        void AssignmentList0(object[] children);

        /// <summary>
        /// Semantic action for production 20:
        /// AssignmentList: ; 
        /// </summary>
        void AssignmentList1(object[] children);

        /// <summary>
        /// Semantic action for production 21:
        /// LogicalOr: LogicalAnd LogicalOrList /* Vec */; 
        /// </summary>
        void LogicalOr(object[] children);

        /// <summary>
        /// Semantic action for production 22:
        /// LogicalOrList: LogicalOrOp LogicalAnd LogicalOrList; 
        /// </summary>
        void LogicalOrList0(object[] children);

        /// <summary>
        /// Semantic action for production 23:
        /// LogicalOrList: ; 
        /// </summary>
        void LogicalOrList1(object[] children);

        /// <summary>
        /// Semantic action for production 24:
        /// LogicalAnd: BitwiseOr LogicalAndList /* Vec */; 
        /// </summary>
        void LogicalAnd(object[] children);

        /// <summary>
        /// Semantic action for production 25:
        /// LogicalAndList: LogicalAndOp BitwiseOr LogicalAndList; 
        /// </summary>
        void LogicalAndList0(object[] children);

        /// <summary>
        /// Semantic action for production 26:
        /// LogicalAndList: ; 
        /// </summary>
        void LogicalAndList1(object[] children);

        /// <summary>
        /// Semantic action for production 27:
        /// BitwiseOr: BitwiseAnd BitwiseOrList /* Vec */; 
        /// </summary>
        void BitwiseOr(object[] children);

        /// <summary>
        /// Semantic action for production 28:
        /// BitwiseOrList: BitwiseOrOp BitwiseAnd BitwiseOrList; 
        /// </summary>
        void BitwiseOrList0(object[] children);

        /// <summary>
        /// Semantic action for production 29:
        /// BitwiseOrList: ; 
        /// </summary>
        void BitwiseOrList1(object[] children);

        /// <summary>
        /// Semantic action for production 30:
        /// BitwiseAnd: Equality BitwiseAndList /* Vec */; 
        /// </summary>
        void BitwiseAnd(object[] children);

        /// <summary>
        /// Semantic action for production 31:
        /// BitwiseAndList: BitwiseAndOp Equality BitwiseAndList; 
        /// </summary>
        void BitwiseAndList0(object[] children);

        /// <summary>
        /// Semantic action for production 32:
        /// BitwiseAndList: ; 
        /// </summary>
        void BitwiseAndList1(object[] children);

        /// <summary>
        /// Semantic action for production 33:
        /// Equality: Relational EqualityList /* Vec */; 
        /// </summary>
        void Equality(object[] children);

        /// <summary>
        /// Semantic action for production 34:
        /// EqualityList: EqualityOp Relational EqualityList; 
        /// </summary>
        void EqualityList0(object[] children);

        /// <summary>
        /// Semantic action for production 35:
        /// EqualityList: ; 
        /// </summary>
        void EqualityList1(object[] children);

        /// <summary>
        /// Semantic action for production 36:
        /// Relational: BitwiseShift RelationalList /* Vec */; 
        /// </summary>
        void Relational(object[] children);

        /// <summary>
        /// Semantic action for production 37:
        /// RelationalList: RelationalOp BitwiseShift RelationalList; 
        /// </summary>
        void RelationalList0(object[] children);

        /// <summary>
        /// Semantic action for production 38:
        /// RelationalList: ; 
        /// </summary>
        void RelationalList1(object[] children);

        /// <summary>
        /// Semantic action for production 39:
        /// BitwiseShift: Summ BitwiseShiftList /* Vec */; 
        /// </summary>
        void BitwiseShift(object[] children);

        /// <summary>
        /// Semantic action for production 40:
        /// BitwiseShiftList: BitwiseShiftOp Summ BitwiseShiftList; 
        /// </summary>
        void BitwiseShiftList0(object[] children);

        /// <summary>
        /// Semantic action for production 41:
        /// BitwiseShiftList: ; 
        /// </summary>
        void BitwiseShiftList1(object[] children);

        /// <summary>
        /// Semantic action for production 42:
        /// AddOp: Plus; 
        /// </summary>
        void AddOp0(object[] children);

        /// <summary>
        /// Semantic action for production 43:
        /// AddOp: Minus; 
        /// </summary>
        void AddOp1(object[] children);

        /// <summary>
        /// Semantic action for production 44:
        /// Summ: Mult SummList /* Vec */; 
        /// </summary>
        void Summ(object[] children);

        /// <summary>
        /// Semantic action for production 45:
        /// SummList: AddOp Mult SummList; 
        /// </summary>
        void SummList0(object[] children);

        /// <summary>
        /// Semantic action for production 46:
        /// SummList: ; 
        /// </summary>
        void SummList1(object[] children);

        /// <summary>
        /// Semantic action for production 47:
        /// Mult: Power MultList /* Vec */; 
        /// </summary>
        void Mult(object[] children);

        /// <summary>
        /// Semantic action for production 48:
        /// MultList: MultOp Power MultList; 
        /// </summary>
        void MultList0(object[] children);

        /// <summary>
        /// Semantic action for production 49:
        /// MultList: ; 
        /// </summary>
        void MultList1(object[] children);

        /// <summary>
        /// Semantic action for production 50:
        /// Power: Factor PowerList /* Vec */; 
        /// </summary>
        void Power(object[] children);

        /// <summary>
        /// Semantic action for production 51:
        /// PowerList: PowOp Factor PowerList; 
        /// </summary>
        void PowerList0(object[] children);

        /// <summary>
        /// Semantic action for production 52:
        /// PowerList: ; 
        /// </summary>
        void PowerList1(object[] children);

        /// <summary>
        /// Semantic action for production 53:
        /// Negate: Minus; 
        /// </summary>
        void Negate(object[] children);

        /// <summary>
        /// Semantic action for production 54:
        /// Factor: Number; 
        /// </summary>
        void Factor0(object[] children);

        /// <summary>
        /// Semantic action for production 55:
        /// Factor: IdRef; 
        /// </summary>
        void Factor1(object[] children);

        /// <summary>
        /// Semantic action for production 56:
        /// Factor: Negate Factor; 
        /// </summary>
        void Factor2(object[] children);

        /// <summary>
        /// Semantic action for production 57:
        /// Factor: "\("^ /* Clipped */ LogicalOr "\)"^ /* Clipped */; 
        /// </summary>
        void Factor3(object[] children);

        /// <summary>
        /// Semantic action for production 58:
        /// Number: "0|[1-9][0-9]*"; 
        /// </summary>
        void Number(object[] children);

        /// <summary>
        /// Semantic action for production 59:
        /// IdRef: Id; 
        /// </summary>
        void IdRef(object[] children);

        /// <summary>
        /// Semantic action for production 60:
        /// Id: "[a-zA-Z_][a-zA-Z0-9_]*"; 
        /// </summary>
        void Id(object[] children);

    }

    /// <summary>
    /// Base class for user actions for the CalcCsharp grammar.
    /// </summary>
    public partial class CalcCsharpActions : ICalcCsharpActions {
        /// <inheritdoc/>
        public virtual object CallSemanticActionForProductionNumber(int productionNumber, object[] children) {
            switch (productionNumber) {
                case 0: { var value = MapCalc(children); OnCalc(value); return value; }
                case 1: return MapCalcList0(children);
                case 2: return MapCalcList1(children);
                case 3: { var value = MapEqualityOp(children); OnEqualityOp(value); return value; }
                case 4: { var value = MapAssignOp(children); OnAssignOp(value); return value; }
                case 5: { var value = MapLogicalOrOp(children); OnLogicalOrOp(value); return value; }
                case 6: { var value = MapLogicalAndOp(children); OnLogicalAndOp(value); return value; }
                case 7: { var value = MapBitwiseOrOp(children); OnBitwiseOrOp(value); return value; }
                case 8: { var value = MapBitwiseAndOp(children); OnBitwiseAndOp(value); return value; }
                case 9: { var value = MapBitwiseShiftOp(children); OnBitwiseShiftOp(value); return value; }
                case 10: { var value = MapRelationalOp(children); OnRelationalOp(value); return value; }
                case 11: { var value = MapPlus(children); OnPlus(value); return value; }
                case 12: { var value = MapMinus(children); OnMinus(value); return value; }
                case 13: { var value = MapPowOp(children); OnPowOp(value); return value; }
                case 14: { var value = MapMultOp(children); OnMultOp(value); return value; }
                case 15: { var value = MapInstruction0(children); OnInstruction(value); return value; }
                case 16: { var value = MapInstruction1(children); OnInstruction(value); return value; }
                case 17: { var value = MapAssignItem(children); OnAssignItem(value); return value; }
                case 18: { var value = MapAssignment(children); OnAssignment(value); return value; }
                case 19: return MapAssignmentList0(children);
                case 20: return MapAssignmentList1(children);
                case 21: { var value = MapLogicalOr(children); OnLogicalOr(value); return value; }
                case 22: return MapLogicalOrList0(children);
                case 23: return MapLogicalOrList1(children);
                case 24: { var value = MapLogicalAnd(children); OnLogicalAnd(value); return value; }
                case 25: return MapLogicalAndList0(children);
                case 26: return MapLogicalAndList1(children);
                case 27: { var value = MapBitwiseOr(children); OnBitwiseOr(value); return value; }
                case 28: return MapBitwiseOrList0(children);
                case 29: return MapBitwiseOrList1(children);
                case 30: { var value = MapBitwiseAnd(children); OnBitwiseAnd(value); return value; }
                case 31: return MapBitwiseAndList0(children);
                case 32: return MapBitwiseAndList1(children);
                case 33: { var value = MapEquality(children); OnEquality(value); return value; }
                case 34: return MapEqualityList0(children);
                case 35: return MapEqualityList1(children);
                case 36: { var value = MapRelational(children); OnRelational(value); return value; }
                case 37: return MapRelationalList0(children);
                case 38: return MapRelationalList1(children);
                case 39: { var value = MapBitwiseShift(children); OnBitwiseShift(value); return value; }
                case 40: return MapBitwiseShiftList0(children);
                case 41: return MapBitwiseShiftList1(children);
                case 42: { var value = MapAddOp0(children); OnAddOp(value); return value; }
                case 43: { var value = MapAddOp1(children); OnAddOp(value); return value; }
                case 44: { var value = MapSumm(children); OnSumm(value); return value; }
                case 45: return MapSummList0(children);
                case 46: return MapSummList1(children);
                case 47: { var value = MapMult(children); OnMult(value); return value; }
                case 48: return MapMultList0(children);
                case 49: return MapMultList1(children);
                case 50: { var value = MapPower(children); OnPower(value); return value; }
                case 51: return MapPowerList0(children);
                case 52: return MapPowerList1(children);
                case 53: { var value = MapNegate(children); OnNegate(value); return value; }
                case 54: { var value = MapFactor0(children); OnFactor(value); return value; }
                case 55: { var value = MapFactor1(children); OnFactor(value); return value; }
                case 56: { var value = MapFactor2(children); OnFactor(value); return value; }
                case 57: { var value = MapFactor3(children); OnFactor(value); return value; }
                case 58: { var value = MapNumber(children); OnNumber(value); return value; }
                case 59: { var value = MapIdRef(children); OnIdRef(value); return value; }
                case 60: { var value = MapId(children); OnId(value); return value; }
                default: throw new ArgumentException($"Invalid production number {productionNumber}");
            }
        }

        /// <inheritdoc/>
        public virtual void OnComment(Token token) { }

        /// <inheritdoc/>
        public virtual IValueConverter ValueConverter { get; } = new GeneratedValueConverter();

        private sealed class GeneratedValueConverter : IValueConverter {
            public bool TryConvert(object value, Type targetType, out object? convertedValue) {
                convertedValue = null;
                if (value == null) return false;
                var sourceType = value.GetType();
                foreach (var owner in new[] { sourceType, targetType }) {
                    foreach (var method in owner.GetMethods(BindingFlags.Public | BindingFlags.Static)) {
                        if ((method.Name == "op_Implicit" || method.Name == "op_Explicit")
                            && method.ReturnType == targetType) {
                            var parameters = method.GetParameters();
                            if (parameters.Length == 1 && parameters[0].ParameterType.IsAssignableFrom(sourceType)) {
                                convertedValue = method.Invoke(null, new[] { value });
                                return true;
                            }
                        }
                    }
                }
                var ctor = targetType.GetConstructor(new[] { sourceType });
                if (ctor != null) {
                    convertedValue = ctor.Invoke(new[] { value });
                    return true;
                }
                return false;
            }
        }

        private static TTarget ConvertValue<TTarget>(object value) {
            return RuntimeValueConverter.Convert<TTarget>(value);
        }

        /// <summary>
        /// User-facing action for non-terminal Calc.
        /// </summary>
        public virtual void OnCalc(Calc arg) { }

        /// <summary>
        /// User-facing action for non-terminal EqualityOp.
        /// </summary>
        public virtual void OnEqualityOp(EqualityOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal AssignOp.
        /// </summary>
        public virtual void OnAssignOp(AssignOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal LogicalOrOp.
        /// </summary>
        public virtual void OnLogicalOrOp(LogicalOrOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal LogicalAndOp.
        /// </summary>
        public virtual void OnLogicalAndOp(LogicalAndOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal BitwiseOrOp.
        /// </summary>
        public virtual void OnBitwiseOrOp(BitwiseOrOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal BitwiseAndOp.
        /// </summary>
        public virtual void OnBitwiseAndOp(BitwiseAndOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal BitwiseShiftOp.
        /// </summary>
        public virtual void OnBitwiseShiftOp(BitwiseShiftOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal RelationalOp.
        /// </summary>
        public virtual void OnRelationalOp(RelationalOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal Plus.
        /// </summary>
        public virtual void OnPlus(Plus arg) { }

        /// <summary>
        /// User-facing action for non-terminal Minus.
        /// </summary>
        public virtual void OnMinus(Minus arg) { }

        /// <summary>
        /// User-facing action for non-terminal PowOp.
        /// </summary>
        public virtual void OnPowOp(PowOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal MultOp.
        /// </summary>
        public virtual void OnMultOp(MultOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal Instruction.
        /// </summary>
        public virtual void OnInstruction(Instruction arg) { }

        /// <summary>
        /// User-facing action for non-terminal AssignItem.
        /// </summary>
        public virtual void OnAssignItem(AssignItem arg) { }

        /// <summary>
        /// User-facing action for non-terminal Assignment.
        /// </summary>
        public virtual void OnAssignment(Assignment arg) { }

        /// <summary>
        /// User-facing action for non-terminal LogicalOr.
        /// </summary>
        public virtual void OnLogicalOr(LogicalOr arg) { }

        /// <summary>
        /// User-facing action for non-terminal LogicalAnd.
        /// </summary>
        public virtual void OnLogicalAnd(LogicalAnd arg) { }

        /// <summary>
        /// User-facing action for non-terminal BitwiseOr.
        /// </summary>
        public virtual void OnBitwiseOr(BitwiseOr arg) { }

        /// <summary>
        /// User-facing action for non-terminal BitwiseAnd.
        /// </summary>
        public virtual void OnBitwiseAnd(BitwiseAnd arg) { }

        /// <summary>
        /// User-facing action for non-terminal Equality.
        /// </summary>
        public virtual void OnEquality(Equality arg) { }

        /// <summary>
        /// User-facing action for non-terminal Relational.
        /// </summary>
        public virtual void OnRelational(Relational arg) { }

        /// <summary>
        /// User-facing action for non-terminal BitwiseShift.
        /// </summary>
        public virtual void OnBitwiseShift(BitwiseShift arg) { }

        /// <summary>
        /// User-facing action for non-terminal AddOp.
        /// </summary>
        public virtual void OnAddOp(AddOp arg) { }

        /// <summary>
        /// User-facing action for non-terminal Summ.
        /// </summary>
        public virtual void OnSumm(Summ arg) { }

        /// <summary>
        /// User-facing action for non-terminal Mult.
        /// </summary>
        public virtual void OnMult(Mult arg) { }

        /// <summary>
        /// User-facing action for non-terminal Power.
        /// </summary>
        public virtual void OnPower(Power arg) { }

        /// <summary>
        /// User-facing action for non-terminal Negate.
        /// </summary>
        public virtual void OnNegate(Negate arg) { }

        /// <summary>
        /// User-facing action for non-terminal Factor.
        /// </summary>
        public virtual void OnFactor(Factor arg) { }

        /// <summary>
        /// User-facing action for non-terminal Number.
        /// </summary>
        public virtual void OnNumber(Number arg) { }

        /// <summary>
        /// User-facing action for non-terminal IdRef.
        /// </summary>
        public virtual void OnIdRef(IdRef arg) { }

        /// <summary>
        /// User-facing action for non-terminal Id.
        /// </summary>
        public virtual void OnId(Id arg) { }

        /// <summary>
        /// Semantic action for production 0:
        /// Calc: CalcList /* Vec */; 
        /// </summary>
        public virtual void Calc(object[] children) {
            var value = MapCalc(children);
            OnCalc(value);
        }

        private static Calc MapCalc(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Calc((List<CalcList>)children[0 + 0]);
            if (children.Length == 1 && children[0] is Calc directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 0 (Calc: CalcList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 1:
        /// CalcList: Instruction ";"^ /* Clipped */ CalcList; 
        /// </summary>
        public virtual void CalcList0(object[] children) {
            var value = MapCalcList0(children);
        }

        private static List<CalcList> MapCalcList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<CalcList>();
            if (children.Length == 1 && children[0] is List<CalcList> directValue) return directValue;
            if (children.Length == 1) {
                var item = new CalcList((Instruction)children[0 + 0]);
                return new List<CalcList> { item };
            }
            if (children.Length == 1 + 1 && children[1] is List<CalcList> previous) {
                var item = new CalcList((Instruction)children[0 + 0]);
                var items = new List<CalcList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 1 (CalcList: Instruction \";\"^ /* Clipped */ CalcList;)");
        }

        /// <summary>
        /// Semantic action for production 2:
        /// CalcList: ; 
        /// </summary>
        public virtual void CalcList1(object[] children) {
            var value = MapCalcList1(children);
        }

        private static List<CalcList> MapCalcList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<CalcList>();
            if (children.Length == 1 && children[0] is List<CalcList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 2 (CalcList: ;)");
        }

        /// <summary>
        /// Semantic action for production 3:
        /// EqualityOp: "==|!="; 
        /// </summary>
        public virtual void EqualityOp(object[] children) {
            var value = MapEqualityOp(children);
            OnEqualityOp(value);
        }

        private static EqualityOp MapEqualityOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new EqualityOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is EqualityOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 3 (EqualityOp: \"==|!=\";)");
        }

        /// <summary>
        /// Semantic action for production 4:
        /// AssignOp: "(\+|-|\*|/|%|<<|>>|&|\^|\|)?="; 
        /// </summary>
        public virtual void AssignOp(object[] children) {
            var value = MapAssignOp(children);
            OnAssignOp(value);
        }

        private static AssignOp MapAssignOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new AssignOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is AssignOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 4 (AssignOp: \"(\\+|-|\\*|/|%|<<|>>|&|\\^|\\|)?=\";)");
        }

        /// <summary>
        /// Semantic action for production 5:
        /// LogicalOrOp: "\|\|"; 
        /// </summary>
        public virtual void LogicalOrOp(object[] children) {
            var value = MapLogicalOrOp(children);
            OnLogicalOrOp(value);
        }

        private static LogicalOrOp MapLogicalOrOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new LogicalOrOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is LogicalOrOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 5 (LogicalOrOp: \"\\|\\|\";)");
        }

        /// <summary>
        /// Semantic action for production 6:
        /// LogicalAndOp: "&&"; 
        /// </summary>
        public virtual void LogicalAndOp(object[] children) {
            var value = MapLogicalAndOp(children);
            OnLogicalAndOp(value);
        }

        private static LogicalAndOp MapLogicalAndOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new LogicalAndOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is LogicalAndOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 6 (LogicalAndOp: \"&&\";)");
        }

        /// <summary>
        /// Semantic action for production 7:
        /// BitwiseOrOp: "\|"; 
        /// </summary>
        public virtual void BitwiseOrOp(object[] children) {
            var value = MapBitwiseOrOp(children);
            OnBitwiseOrOp(value);
        }

        private static BitwiseOrOp MapBitwiseOrOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new BitwiseOrOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is BitwiseOrOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 7 (BitwiseOrOp: \"\\|\";)");
        }

        /// <summary>
        /// Semantic action for production 8:
        /// BitwiseAndOp: "&"; 
        /// </summary>
        public virtual void BitwiseAndOp(object[] children) {
            var value = MapBitwiseAndOp(children);
            OnBitwiseAndOp(value);
        }

        private static BitwiseAndOp MapBitwiseAndOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new BitwiseAndOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is BitwiseAndOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 8 (BitwiseAndOp: \"&\";)");
        }

        /// <summary>
        /// Semantic action for production 9:
        /// BitwiseShiftOp: "<<|>>"; 
        /// </summary>
        public virtual void BitwiseShiftOp(object[] children) {
            var value = MapBitwiseShiftOp(children);
            OnBitwiseShiftOp(value);
        }

        private static BitwiseShiftOp MapBitwiseShiftOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new BitwiseShiftOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is BitwiseShiftOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 9 (BitwiseShiftOp: \"<<|>>\";)");
        }

        /// <summary>
        /// Semantic action for production 10:
        /// RelationalOp: "<=|<|>=|>"; 
        /// </summary>
        public virtual void RelationalOp(object[] children) {
            var value = MapRelationalOp(children);
            OnRelationalOp(value);
        }

        private static RelationalOp MapRelationalOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new RelationalOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is RelationalOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 10 (RelationalOp: \"<=|<|>=|>\";)");
        }

        /// <summary>
        /// Semantic action for production 11:
        /// Plus: "\+"; 
        /// </summary>
        public virtual void Plus(object[] children) {
            var value = MapPlus(children);
            OnPlus(value);
        }

        private static Plus MapPlus(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Plus((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is Plus directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 11 (Plus: \"\\+\";)");
        }

        /// <summary>
        /// Semantic action for production 12:
        /// Minus: "-"; 
        /// </summary>
        public virtual void Minus(object[] children) {
            var value = MapMinus(children);
            OnMinus(value);
        }

        private static Minus MapMinus(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Minus((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is Minus directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 12 (Minus: \"-\";)");
        }

        /// <summary>
        /// Semantic action for production 13:
        /// PowOp: "\*\*"; 
        /// </summary>
        public virtual void PowOp(object[] children) {
            var value = MapPowOp(children);
            OnPowOp(value);
        }

        private static PowOp MapPowOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new PowOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is PowOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 13 (PowOp: \"\\*\\*\";)");
        }

        /// <summary>
        /// Semantic action for production 14:
        /// MultOp: "\*|/|%"; 
        /// </summary>
        public virtual void MultOp(object[] children) {
            var value = MapMultOp(children);
            OnMultOp(value);
        }

        private static MultOp MapMultOp(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new MultOp((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is MultOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 14 (MultOp: \"\\*|/|%\";)");
        }

        /// <summary>
        /// Semantic action for production 15:
        /// Instruction: Assignment; 
        /// </summary>
        public virtual void Instruction0(object[] children) {
            var value = MapInstruction0(children);
            OnInstruction(value);
        }

        private static Instruction MapInstruction0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new InstructionAssignment((Assignment)children[0 + 0]);
                return new InstructionAssignmentVariant(value);
            }
            if (children.Length == 1 && children[0] is Instruction directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 15 (Instruction: Assignment;)");
        }

        /// <summary>
        /// Semantic action for production 16:
        /// Instruction: LogicalOr; 
        /// </summary>
        public virtual void Instruction1(object[] children) {
            var value = MapInstruction1(children);
            OnInstruction(value);
        }

        private static Instruction MapInstruction1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new InstructionLogicalOr((LogicalOr)children[0 + 0]);
                return new InstructionLogicalOrVariant(value);
            }
            if (children.Length == 1 && children[0] is Instruction directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 16 (Instruction: LogicalOr;)");
        }

        /// <summary>
        /// Semantic action for production 17:
        /// AssignItem: Id AssignOp; 
        /// </summary>
        public virtual void AssignItem(object[] children) {
            var value = MapAssignItem(children);
            OnAssignItem(value);
        }

        private static AssignItem MapAssignItem(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new AssignItem((Id)children[0 + 0], (AssignOp)children[0 + 1]);
            if (children.Length == 1 && children[0] is AssignItem directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 17 (AssignItem: Id AssignOp;)");
        }

        /// <summary>
        /// Semantic action for production 18:
        /// Assignment: AssignItem AssignmentList /* Vec */ LogicalOr; 
        /// </summary>
        public virtual void Assignment(object[] children) {
            var value = MapAssignment(children);
            OnAssignment(value);
        }

        private static Assignment MapAssignment(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 3 ) return new Assignment((AssignItem)children[0 + 0], (List<AssignmentList>)children[0 + 1], (LogicalOr)children[0 + 2]);
            if (children.Length == 1 && children[0] is Assignment directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 18 (Assignment: AssignItem AssignmentList /* Vec */ LogicalOr;)");
        }

        /// <summary>
        /// Semantic action for production 19:
        /// AssignmentList: AssignItem AssignmentList; 
        /// </summary>
        public virtual void AssignmentList0(object[] children) {
            var value = MapAssignmentList0(children);
        }

        private static List<AssignmentList> MapAssignmentList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<AssignmentList>();
            if (children.Length == 1 && children[0] is List<AssignmentList> directValue) return directValue;
            if (children.Length == 1) {
                var item = new AssignmentList((AssignItem)children[0 + 0]);
                return new List<AssignmentList> { item };
            }
            if (children.Length == 1 + 1 && children[1] is List<AssignmentList> previous) {
                var item = new AssignmentList((AssignItem)children[0 + 0]);
                var items = new List<AssignmentList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 19 (AssignmentList: AssignItem AssignmentList;)");
        }

        /// <summary>
        /// Semantic action for production 20:
        /// AssignmentList: ; 
        /// </summary>
        public virtual void AssignmentList1(object[] children) {
            var value = MapAssignmentList1(children);
        }

        private static List<AssignmentList> MapAssignmentList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<AssignmentList>();
            if (children.Length == 1 && children[0] is List<AssignmentList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 20 (AssignmentList: ;)");
        }

        /// <summary>
        /// Semantic action for production 21:
        /// LogicalOr: LogicalAnd LogicalOrList /* Vec */; 
        /// </summary>
        public virtual void LogicalOr(object[] children) {
            var value = MapLogicalOr(children);
            OnLogicalOr(value);
        }

        private static LogicalOr MapLogicalOr(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new LogicalOr((LogicalAnd)children[0 + 0], (List<LogicalOrList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is LogicalOr directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 21 (LogicalOr: LogicalAnd LogicalOrList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 22:
        /// LogicalOrList: LogicalOrOp LogicalAnd LogicalOrList; 
        /// </summary>
        public virtual void LogicalOrList0(object[] children) {
            var value = MapLogicalOrList0(children);
        }

        private static List<LogicalOrList> MapLogicalOrList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<LogicalOrList>();
            if (children.Length == 1 && children[0] is List<LogicalOrList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new LogicalOrList((LogicalOrOp)children[0 + 0], (LogicalAnd)children[0 + 1]);
                return new List<LogicalOrList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<LogicalOrList> previous) {
                var item = new LogicalOrList((LogicalOrOp)children[0 + 0], (LogicalAnd)children[0 + 1]);
                var items = new List<LogicalOrList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 22 (LogicalOrList: LogicalOrOp LogicalAnd LogicalOrList;)");
        }

        /// <summary>
        /// Semantic action for production 23:
        /// LogicalOrList: ; 
        /// </summary>
        public virtual void LogicalOrList1(object[] children) {
            var value = MapLogicalOrList1(children);
        }

        private static List<LogicalOrList> MapLogicalOrList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<LogicalOrList>();
            if (children.Length == 1 && children[0] is List<LogicalOrList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 23 (LogicalOrList: ;)");
        }

        /// <summary>
        /// Semantic action for production 24:
        /// LogicalAnd: BitwiseOr LogicalAndList /* Vec */; 
        /// </summary>
        public virtual void LogicalAnd(object[] children) {
            var value = MapLogicalAnd(children);
            OnLogicalAnd(value);
        }

        private static LogicalAnd MapLogicalAnd(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new LogicalAnd((BitwiseOr)children[0 + 0], (List<LogicalAndList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is LogicalAnd directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 24 (LogicalAnd: BitwiseOr LogicalAndList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 25:
        /// LogicalAndList: LogicalAndOp BitwiseOr LogicalAndList; 
        /// </summary>
        public virtual void LogicalAndList0(object[] children) {
            var value = MapLogicalAndList0(children);
        }

        private static List<LogicalAndList> MapLogicalAndList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<LogicalAndList>();
            if (children.Length == 1 && children[0] is List<LogicalAndList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new LogicalAndList((LogicalAndOp)children[0 + 0], (BitwiseOr)children[0 + 1]);
                return new List<LogicalAndList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<LogicalAndList> previous) {
                var item = new LogicalAndList((LogicalAndOp)children[0 + 0], (BitwiseOr)children[0 + 1]);
                var items = new List<LogicalAndList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 25 (LogicalAndList: LogicalAndOp BitwiseOr LogicalAndList;)");
        }

        /// <summary>
        /// Semantic action for production 26:
        /// LogicalAndList: ; 
        /// </summary>
        public virtual void LogicalAndList1(object[] children) {
            var value = MapLogicalAndList1(children);
        }

        private static List<LogicalAndList> MapLogicalAndList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<LogicalAndList>();
            if (children.Length == 1 && children[0] is List<LogicalAndList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 26 (LogicalAndList: ;)");
        }

        /// <summary>
        /// Semantic action for production 27:
        /// BitwiseOr: BitwiseAnd BitwiseOrList /* Vec */; 
        /// </summary>
        public virtual void BitwiseOr(object[] children) {
            var value = MapBitwiseOr(children);
            OnBitwiseOr(value);
        }

        private static BitwiseOr MapBitwiseOr(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new BitwiseOr((BitwiseAnd)children[0 + 0], (List<BitwiseOrList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is BitwiseOr directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 27 (BitwiseOr: BitwiseAnd BitwiseOrList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 28:
        /// BitwiseOrList: BitwiseOrOp BitwiseAnd BitwiseOrList; 
        /// </summary>
        public virtual void BitwiseOrList0(object[] children) {
            var value = MapBitwiseOrList0(children);
        }

        private static List<BitwiseOrList> MapBitwiseOrList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<BitwiseOrList>();
            if (children.Length == 1 && children[0] is List<BitwiseOrList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new BitwiseOrList((BitwiseOrOp)children[0 + 0], (BitwiseAnd)children[0 + 1]);
                return new List<BitwiseOrList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<BitwiseOrList> previous) {
                var item = new BitwiseOrList((BitwiseOrOp)children[0 + 0], (BitwiseAnd)children[0 + 1]);
                var items = new List<BitwiseOrList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 28 (BitwiseOrList: BitwiseOrOp BitwiseAnd BitwiseOrList;)");
        }

        /// <summary>
        /// Semantic action for production 29:
        /// BitwiseOrList: ; 
        /// </summary>
        public virtual void BitwiseOrList1(object[] children) {
            var value = MapBitwiseOrList1(children);
        }

        private static List<BitwiseOrList> MapBitwiseOrList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<BitwiseOrList>();
            if (children.Length == 1 && children[0] is List<BitwiseOrList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 29 (BitwiseOrList: ;)");
        }

        /// <summary>
        /// Semantic action for production 30:
        /// BitwiseAnd: Equality BitwiseAndList /* Vec */; 
        /// </summary>
        public virtual void BitwiseAnd(object[] children) {
            var value = MapBitwiseAnd(children);
            OnBitwiseAnd(value);
        }

        private static BitwiseAnd MapBitwiseAnd(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new BitwiseAnd((Equality)children[0 + 0], (List<BitwiseAndList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is BitwiseAnd directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 30 (BitwiseAnd: Equality BitwiseAndList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 31:
        /// BitwiseAndList: BitwiseAndOp Equality BitwiseAndList; 
        /// </summary>
        public virtual void BitwiseAndList0(object[] children) {
            var value = MapBitwiseAndList0(children);
        }

        private static List<BitwiseAndList> MapBitwiseAndList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<BitwiseAndList>();
            if (children.Length == 1 && children[0] is List<BitwiseAndList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new BitwiseAndList((BitwiseAndOp)children[0 + 0], (Equality)children[0 + 1]);
                return new List<BitwiseAndList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<BitwiseAndList> previous) {
                var item = new BitwiseAndList((BitwiseAndOp)children[0 + 0], (Equality)children[0 + 1]);
                var items = new List<BitwiseAndList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 31 (BitwiseAndList: BitwiseAndOp Equality BitwiseAndList;)");
        }

        /// <summary>
        /// Semantic action for production 32:
        /// BitwiseAndList: ; 
        /// </summary>
        public virtual void BitwiseAndList1(object[] children) {
            var value = MapBitwiseAndList1(children);
        }

        private static List<BitwiseAndList> MapBitwiseAndList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<BitwiseAndList>();
            if (children.Length == 1 && children[0] is List<BitwiseAndList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 32 (BitwiseAndList: ;)");
        }

        /// <summary>
        /// Semantic action for production 33:
        /// Equality: Relational EqualityList /* Vec */; 
        /// </summary>
        public virtual void Equality(object[] children) {
            var value = MapEquality(children);
            OnEquality(value);
        }

        private static Equality MapEquality(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new Equality((Relational)children[0 + 0], (List<EqualityList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is Equality directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 33 (Equality: Relational EqualityList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 34:
        /// EqualityList: EqualityOp Relational EqualityList; 
        /// </summary>
        public virtual void EqualityList0(object[] children) {
            var value = MapEqualityList0(children);
        }

        private static List<EqualityList> MapEqualityList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<EqualityList>();
            if (children.Length == 1 && children[0] is List<EqualityList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new EqualityList((EqualityOp)children[0 + 0], (Relational)children[0 + 1]);
                return new List<EqualityList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<EqualityList> previous) {
                var item = new EqualityList((EqualityOp)children[0 + 0], (Relational)children[0 + 1]);
                var items = new List<EqualityList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 34 (EqualityList: EqualityOp Relational EqualityList;)");
        }

        /// <summary>
        /// Semantic action for production 35:
        /// EqualityList: ; 
        /// </summary>
        public virtual void EqualityList1(object[] children) {
            var value = MapEqualityList1(children);
        }

        private static List<EqualityList> MapEqualityList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<EqualityList>();
            if (children.Length == 1 && children[0] is List<EqualityList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 35 (EqualityList: ;)");
        }

        /// <summary>
        /// Semantic action for production 36:
        /// Relational: BitwiseShift RelationalList /* Vec */; 
        /// </summary>
        public virtual void Relational(object[] children) {
            var value = MapRelational(children);
            OnRelational(value);
        }

        private static Relational MapRelational(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new Relational((BitwiseShift)children[0 + 0], (List<RelationalList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is Relational directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 36 (Relational: BitwiseShift RelationalList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 37:
        /// RelationalList: RelationalOp BitwiseShift RelationalList; 
        /// </summary>
        public virtual void RelationalList0(object[] children) {
            var value = MapRelationalList0(children);
        }

        private static List<RelationalList> MapRelationalList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<RelationalList>();
            if (children.Length == 1 && children[0] is List<RelationalList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new RelationalList((RelationalOp)children[0 + 0], (BitwiseShift)children[0 + 1]);
                return new List<RelationalList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<RelationalList> previous) {
                var item = new RelationalList((RelationalOp)children[0 + 0], (BitwiseShift)children[0 + 1]);
                var items = new List<RelationalList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 37 (RelationalList: RelationalOp BitwiseShift RelationalList;)");
        }

        /// <summary>
        /// Semantic action for production 38:
        /// RelationalList: ; 
        /// </summary>
        public virtual void RelationalList1(object[] children) {
            var value = MapRelationalList1(children);
        }

        private static List<RelationalList> MapRelationalList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<RelationalList>();
            if (children.Length == 1 && children[0] is List<RelationalList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 38 (RelationalList: ;)");
        }

        /// <summary>
        /// Semantic action for production 39:
        /// BitwiseShift: Summ BitwiseShiftList /* Vec */; 
        /// </summary>
        public virtual void BitwiseShift(object[] children) {
            var value = MapBitwiseShift(children);
            OnBitwiseShift(value);
        }

        private static BitwiseShift MapBitwiseShift(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new BitwiseShift((Summ)children[0 + 0], (List<BitwiseShiftList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is BitwiseShift directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 39 (BitwiseShift: Summ BitwiseShiftList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 40:
        /// BitwiseShiftList: BitwiseShiftOp Summ BitwiseShiftList; 
        /// </summary>
        public virtual void BitwiseShiftList0(object[] children) {
            var value = MapBitwiseShiftList0(children);
        }

        private static List<BitwiseShiftList> MapBitwiseShiftList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<BitwiseShiftList>();
            if (children.Length == 1 && children[0] is List<BitwiseShiftList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new BitwiseShiftList((BitwiseShiftOp)children[0 + 0], (Summ)children[0 + 1]);
                return new List<BitwiseShiftList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<BitwiseShiftList> previous) {
                var item = new BitwiseShiftList((BitwiseShiftOp)children[0 + 0], (Summ)children[0 + 1]);
                var items = new List<BitwiseShiftList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 40 (BitwiseShiftList: BitwiseShiftOp Summ BitwiseShiftList;)");
        }

        /// <summary>
        /// Semantic action for production 41:
        /// BitwiseShiftList: ; 
        /// </summary>
        public virtual void BitwiseShiftList1(object[] children) {
            var value = MapBitwiseShiftList1(children);
        }

        private static List<BitwiseShiftList> MapBitwiseShiftList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<BitwiseShiftList>();
            if (children.Length == 1 && children[0] is List<BitwiseShiftList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 41 (BitwiseShiftList: ;)");
        }

        /// <summary>
        /// Semantic action for production 42:
        /// AddOp: Plus; 
        /// </summary>
        public virtual void AddOp0(object[] children) {
            var value = MapAddOp0(children);
            OnAddOp(value);
        }

        private static AddOp MapAddOp0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new AddOpPlus((Plus)children[0 + 0]);
                return new AddOpPlusVariant(value);
            }
            if (children.Length == 1 && children[0] is AddOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 42 (AddOp: Plus;)");
        }

        /// <summary>
        /// Semantic action for production 43:
        /// AddOp: Minus; 
        /// </summary>
        public virtual void AddOp1(object[] children) {
            var value = MapAddOp1(children);
            OnAddOp(value);
        }

        private static AddOp MapAddOp1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new AddOpMinus((Minus)children[0 + 0]);
                return new AddOpMinusVariant(value);
            }
            if (children.Length == 1 && children[0] is AddOp directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 43 (AddOp: Minus;)");
        }

        /// <summary>
        /// Semantic action for production 44:
        /// Summ: Mult SummList /* Vec */; 
        /// </summary>
        public virtual void Summ(object[] children) {
            var value = MapSumm(children);
            OnSumm(value);
        }

        private static Summ MapSumm(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new Summ((Mult)children[0 + 0], (List<SummList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is Summ directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 44 (Summ: Mult SummList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 45:
        /// SummList: AddOp Mult SummList; 
        /// </summary>
        public virtual void SummList0(object[] children) {
            var value = MapSummList0(children);
        }

        private static List<SummList> MapSummList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<SummList>();
            if (children.Length == 1 && children[0] is List<SummList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new SummList((AddOp)children[0 + 0], (Mult)children[0 + 1]);
                return new List<SummList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<SummList> previous) {
                var item = new SummList((AddOp)children[0 + 0], (Mult)children[0 + 1]);
                var items = new List<SummList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 45 (SummList: AddOp Mult SummList;)");
        }

        /// <summary>
        /// Semantic action for production 46:
        /// SummList: ; 
        /// </summary>
        public virtual void SummList1(object[] children) {
            var value = MapSummList1(children);
        }

        private static List<SummList> MapSummList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<SummList>();
            if (children.Length == 1 && children[0] is List<SummList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 46 (SummList: ;)");
        }

        /// <summary>
        /// Semantic action for production 47:
        /// Mult: Power MultList /* Vec */; 
        /// </summary>
        public virtual void Mult(object[] children) {
            var value = MapMult(children);
            OnMult(value);
        }

        private static Mult MapMult(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new Mult((Power)children[0 + 0], (List<MultList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is Mult directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 47 (Mult: Power MultList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 48:
        /// MultList: MultOp Power MultList; 
        /// </summary>
        public virtual void MultList0(object[] children) {
            var value = MapMultList0(children);
        }

        private static List<MultList> MapMultList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<MultList>();
            if (children.Length == 1 && children[0] is List<MultList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new MultList((MultOp)children[0 + 0], (Power)children[0 + 1]);
                return new List<MultList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<MultList> previous) {
                var item = new MultList((MultOp)children[0 + 0], (Power)children[0 + 1]);
                var items = new List<MultList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 48 (MultList: MultOp Power MultList;)");
        }

        /// <summary>
        /// Semantic action for production 49:
        /// MultList: ; 
        /// </summary>
        public virtual void MultList1(object[] children) {
            var value = MapMultList1(children);
        }

        private static List<MultList> MapMultList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<MultList>();
            if (children.Length == 1 && children[0] is List<MultList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 49 (MultList: ;)");
        }

        /// <summary>
        /// Semantic action for production 50:
        /// Power: Factor PowerList /* Vec */; 
        /// </summary>
        public virtual void Power(object[] children) {
            var value = MapPower(children);
            OnPower(value);
        }

        private static Power MapPower(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new Power((Factor)children[0 + 0], (List<PowerList>)children[0 + 1]);
            if (children.Length == 1 && children[0] is Power directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 50 (Power: Factor PowerList /* Vec */;)");
        }

        /// <summary>
        /// Semantic action for production 51:
        /// PowerList: PowOp Factor PowerList; 
        /// </summary>
        public virtual void PowerList0(object[] children) {
            var value = MapPowerList0(children);
        }

        private static List<PowerList> MapPowerList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<PowerList>();
            if (children.Length == 1 && children[0] is List<PowerList> directValue) return directValue;
            if (children.Length == 2) {
                var item = new PowerList((PowOp)children[0 + 0], (Factor)children[0 + 1]);
                return new List<PowerList> { item };
            }
            if (children.Length == 2 + 1 && children[2] is List<PowerList> previous) {
                var item = new PowerList((PowOp)children[0 + 0], (Factor)children[0 + 1]);
                var items = new List<PowerList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 51 (PowerList: PowOp Factor PowerList;)");
        }

        /// <summary>
        /// Semantic action for production 52:
        /// PowerList: ; 
        /// </summary>
        public virtual void PowerList1(object[] children) {
            var value = MapPowerList1(children);
        }

        private static List<PowerList> MapPowerList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<PowerList>();
            if (children.Length == 1 && children[0] is List<PowerList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 52 (PowerList: ;)");
        }

        /// <summary>
        /// Semantic action for production 53:
        /// Negate: Minus; 
        /// </summary>
        public virtual void Negate(object[] children) {
            var value = MapNegate(children);
            OnNegate(value);
        }

        private static Negate MapNegate(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Negate((Minus)children[0 + 0]);
            if (children.Length == 1 && children[0] is Negate directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 53 (Negate: Minus;)");
        }

        /// <summary>
        /// Semantic action for production 54:
        /// Factor: Number; 
        /// </summary>
        public virtual void Factor0(object[] children) {
            var value = MapFactor0(children);
            OnFactor(value);
        }

        private static Factor MapFactor0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new FactorNumber((Number)children[0 + 0]);
                return new FactorNumberVariant(value);
            }
            if (children.Length == 1 && children[0] is Factor directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 54 (Factor: Number;)");
        }

        /// <summary>
        /// Semantic action for production 55:
        /// Factor: IdRef; 
        /// </summary>
        public virtual void Factor1(object[] children) {
            var value = MapFactor1(children);
            OnFactor(value);
        }

        private static Factor MapFactor1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new FactorIdRef((IdRef)children[0 + 0]);
                return new FactorIdRefVariant(value);
            }
            if (children.Length == 1 && children[0] is Factor directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 55 (Factor: IdRef;)");
        }

        /// <summary>
        /// Semantic action for production 56:
        /// Factor: Negate Factor; 
        /// </summary>
        public virtual void Factor2(object[] children) {
            var value = MapFactor2(children);
            OnFactor(value);
        }

        private static Factor MapFactor2(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2) {
                var value = new FactorNegateFactor((Negate)children[0 + 0], (Factor)children[0 + 1]);
                return new FactorNegateFactorVariant(value);
            }
            if (children.Length == 1 && children[0] is Factor directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 56 (Factor: Negate Factor;)");
        }

        /// <summary>
        /// Semantic action for production 57:
        /// Factor: "\("^ /* Clipped */ LogicalOr "\)"^ /* Clipped */; 
        /// </summary>
        public virtual void Factor3(object[] children) {
            var value = MapFactor3(children);
            OnFactor(value);
        }

        private static Factor MapFactor3(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new FactorLParenLogicalOrRParen((LogicalOr)children[0 + 0]);
                return new FactorLParenLogicalOrRParenVariant(value);
            }
            if (children.Length == 1 && children[0] is Factor directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 57 (Factor: \"\\(\"^ /* Clipped */ LogicalOr \"\\)\"^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 58:
        /// Number: "0|[1-9][0-9]*"; 
        /// </summary>
        public virtual void Number(object[] children) {
            var value = MapNumber(children);
            OnNumber(value);
        }

        private static Number MapNumber(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Number(ConvertValue<CalcCsharp.CalcNumber>(children[0 + 0]));
            if (children.Length == 1 && children[0] is Number directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 58 (Number: \"0|[1-9][0-9]*\";)");
        }

        /// <summary>
        /// Semantic action for production 59:
        /// IdRef: Id; 
        /// </summary>
        public virtual void IdRef(object[] children) {
            var value = MapIdRef(children);
            OnIdRef(value);
        }

        private static IdRef MapIdRef(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new IdRef((Id)children[0 + 0]);
            if (children.Length == 1 && children[0] is IdRef directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 59 (IdRef: Id;)");
        }

        /// <summary>
        /// Semantic action for production 60:
        /// Id: "[a-zA-Z_][a-zA-Z0-9_]*"; 
        /// </summary>
        public virtual void Id(object[] children) {
            var value = MapId(children);
            OnId(value);
        }

        private static Id MapId(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Id((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is Id directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 60 (Id: \"[a-zA-Z_][a-zA-Z0-9_]*\";)");
        }

    }
}
