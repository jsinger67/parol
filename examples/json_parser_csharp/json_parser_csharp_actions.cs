using System;
using System.Linq;

namespace JsonParserCsharp
{
    public sealed class JsonRenderActions : JsonParserCsharpActions
    {
        private Json? _json;

        public override void OnJson(Json arg)
        {
            _json = arg;
        }

        public override string ToString()
        {
            if (_json is null)
            {
                return "No parse result";
            }

            return RenderValue(_json.Value);
        }

        private static string RenderValue(Value value)
        {
            return value switch
            {
                ValueStringVariant valueString => RenderString(valueString.Value.String),
                ValueNumberVariant valueNumber => RenderNumber(valueNumber.Value.Number),
                ValueObjectVariant valueObject => RenderObject(valueObject.Value.Object),
                ValueArrayVariant valueArray => RenderArray(valueArray.Value.Array),
                ValueTrueVariant => "true",
                ValueFalseVariant => "false",
                ValueNullVariant => "null",
                _ => throw new InvalidOperationException($"Unsupported value variant: {value.GetType().Name}")
            };
        }

        private static string RenderString(JsonString value)
        {
            return value.Text;
        }

        private static string RenderNumber(JsonNumber value)
        {
            return value.Text;
        }

        private static string RenderObject(Object @object)
        {
            return $"{{{RenderObjectSuffix(@object.ObjectSuffix)}}}";
        }

        private static string RenderObjectSuffix(ObjectSuffix objectSuffix)
        {
            return objectSuffix switch
            {
                ObjectSuffixPairObjectListRBraceVariant fullObject =>
                    RenderPair(fullObject.Value.Pair) +
                    string.Concat(fullObject.Value.ObjectList.Select(RenderObjectListItem)),
                ObjectSuffixRBraceVariant => string.Empty,
                _ => throw new InvalidOperationException($"Unsupported object suffix variant: {objectSuffix.GetType().Name}")
            };
        }

        private static string RenderObjectListItem(ObjectList objectList)
        {
            return $", {RenderPair(objectList.Pair)}";
        }

        private static string RenderPair(Pair pair)
        {
            return $"{RenderString(pair.String)}: {RenderValue(pair.Value)}";
        }

        private static string RenderArray(Array array)
        {
            return $"[{RenderArraySuffix(array.ArraySuffix)}]";
        }

        private static string RenderArraySuffix(ArraySuffix arraySuffix)
        {
            return arraySuffix switch
            {
                ArraySuffixValueArrayListRBracketVariant populatedArray =>
                    RenderValue(populatedArray.Value.Value) +
                    string.Concat(populatedArray.Value.ArrayList.Select(RenderArrayListItem)),
                ArraySuffixRBracketVariant => string.Empty,
                _ => throw new InvalidOperationException($"Unsupported array suffix variant: {arraySuffix.GetType().Name}")
            };
        }

        private static string RenderArrayListItem(ArrayList arrayList)
        {
            return $", {RenderValue(arrayList.Value)}";
        }
    }
}
