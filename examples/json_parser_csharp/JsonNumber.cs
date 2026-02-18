namespace JsonParserCsharp;

using Parol.Runtime;

public readonly struct JsonNumber
{
    public string Text { get; }

    public JsonNumber(string text)
    {
        Text = text;
    }

    public JsonNumber(Token token)
    {
        Text = token.Text;
    }

    public JsonNumber(Number value)
    {
        Text = value.NumberValue.Text;
    }

    public override string ToString() => Text;
}
