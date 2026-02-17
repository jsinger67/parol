namespace CalcCsharp;

using Parol.Runtime.Scanner;
using Parol.Runtime;

public readonly struct CalcNumber
{
    public long Value { get; }

    public CalcNumber(long value)
    {
        Value = value;
    }

    public CalcNumber(Token token)
    {
        Value = long.Parse(token.Text);
    }

    public static implicit operator long(CalcNumber number) => number.Value;

    public override string ToString() => Value.ToString();
}
