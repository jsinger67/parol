namespace JsonParserCsharp;

using Parol.Runtime;

public readonly struct JsonString
{
    public string Text { get; }

    public JsonString(string text)
    {
        Text = text;
    }

    public JsonString(Token token)
    {
        Text = token.Text;
    }

    public override string ToString() => Text;
}
