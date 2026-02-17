using System;
using System.Collections.Generic;
using System.Reflection;
using Parol.Runtime;
using Parol.Runtime.Scanner;

namespace JsonParserCsharp {
    // Deduced grammar types
    // Type derived for non-terminal Array
    public sealed record Array(ArraySuffix ArraySuffix);

    // Type derived for non-terminal ArrayList
    public sealed record ArrayList(Value Value);

    // Type derived for non-terminal ArraySuffix
    public abstract record ArraySuffix;
    public sealed record ArraySuffixValueArrayListRBracketVariant(ArraySuffixValueArrayListRBracket Value) : ArraySuffix;
    public sealed record ArraySuffixRBracketVariant(ArraySuffixRBracket Value) : ArraySuffix;

    // Type derived for non-terminal Json
    public sealed record Json(Value Value);

    // Type derived for non-terminal Number
    public sealed record Number(Token NumberValue);

    // Type derived for non-terminal Object
    public sealed record Object(ObjectSuffix ObjectSuffix);

    // Type derived for non-terminal ObjectList
    public sealed record ObjectList(Pair Pair);

    // Type derived for non-terminal ObjectSuffix
    public abstract record ObjectSuffix;
    public sealed record ObjectSuffixPairObjectListRBraceVariant(ObjectSuffixPairObjectListRBrace Value) : ObjectSuffix;
    public sealed record ObjectSuffixRBraceVariant(ObjectSuffixRBrace Value) : ObjectSuffix;

    // Type derived for non-terminal Pair
    public sealed record Pair(String String, Value Value);

    // Type derived for non-terminal String
    public sealed record String(Token StringValue);

    // Type derived for non-terminal Value
    public abstract record Value;
    public sealed record ValueStringVariant(ValueString Value) : Value;
    public sealed record ValueNumberVariant(ValueNumber Value) : Value;
    public sealed record ValueObjectVariant(ValueObject Value) : Value;
    public sealed record ValueArrayVariant(ValueArray Value) : Value;
    public sealed record ValueTrueVariant(ValueTrue Value) : Value;
    public sealed record ValueFalseVariant(ValueFalse Value) : Value;
    public sealed record ValueNullVariant(ValueNull Value) : Value;

    // Type derived for production 2
    public sealed record ObjectSuffixPairObjectListRBrace(Pair Pair, List<ObjectList> ObjectList);

    // Type derived for production 3
    public sealed record ObjectSuffixRBrace();

    // Type derived for production 8
    public sealed record ArraySuffixValueArrayListRBracket(Value Value, List<ArrayList> ArrayList);

    // Type derived for production 9
    public sealed record ArraySuffixRBracket();

    // Type derived for production 12
    public sealed record ValueString(String String);

    // Type derived for production 13
    public sealed record ValueNumber(Number Number);

    // Type derived for production 14
    public sealed record ValueObject(Object Object);

    // Type derived for production 15
    public sealed record ValueArray(Array Array);

    // Type derived for production 16
    public sealed record ValueTrue();

    // Type derived for production 17
    public sealed record ValueFalse();

    // Type derived for production 18
    public sealed record ValueNull();

    /// <summary>
    /// User actions interface for the JsonParserCsharp grammar.
    /// </summary>
    public interface IJsonParserCsharpActions : IUserActions, IProvidesValueConverter {
        void OnJson(Json arg);

        void OnObject(Object arg);

        void OnPair(Pair arg);

        void OnArray(Array arg);

        void OnValue(Value arg);

        void OnString(String arg);

        void OnNumber(Number arg);

        /// <summary>
        /// Semantic action for production 0:
        /// Json: Value; 
        /// </summary>
        void Json(object[] children);

        /// <summary>
        /// Semantic action for production 1:
        /// Object: '{'^ /* Clipped */ ObjectSuffix; 
        /// </summary>
        void Object(object[] children);

        /// <summary>
        /// Semantic action for production 2:
        /// ObjectSuffix: Pair ObjectList /* Vec */ '}'^ /* Clipped */; 
        /// </summary>
        void ObjectSuffix0(object[] children);

        /// <summary>
        /// Semantic action for production 3:
        /// ObjectSuffix: '}'^ /* Clipped */; 
        /// </summary>
        void ObjectSuffix1(object[] children);

        /// <summary>
        /// Semantic action for production 4:
        /// ObjectList: ','^ /* Clipped */ Pair ObjectList; 
        /// </summary>
        void ObjectList0(object[] children);

        /// <summary>
        /// Semantic action for production 5:
        /// ObjectList: ; 
        /// </summary>
        void ObjectList1(object[] children);

        /// <summary>
        /// Semantic action for production 6:
        /// Pair: String : JsonParserCsharp::JsonString  ':'^ /* Clipped */ Value; 
        /// </summary>
        void Pair(object[] children);

        /// <summary>
        /// Semantic action for production 7:
        /// Array: '['^ /* Clipped */ ArraySuffix; 
        /// </summary>
        void Array(object[] children);

        /// <summary>
        /// Semantic action for production 8:
        /// ArraySuffix: Value ArrayList /* Vec */ ']'^ /* Clipped */; 
        /// </summary>
        void ArraySuffix0(object[] children);

        /// <summary>
        /// Semantic action for production 9:
        /// ArraySuffix: ']'^ /* Clipped */; 
        /// </summary>
        void ArraySuffix1(object[] children);

        /// <summary>
        /// Semantic action for production 10:
        /// ArrayList: ','^ /* Clipped */ Value ArrayList; 
        /// </summary>
        void ArrayList0(object[] children);

        /// <summary>
        /// Semantic action for production 11:
        /// ArrayList: ; 
        /// </summary>
        void ArrayList1(object[] children);

        /// <summary>
        /// Semantic action for production 12:
        /// Value: String : JsonParserCsharp::JsonString ; 
        /// </summary>
        void Value0(object[] children);

        /// <summary>
        /// Semantic action for production 13:
        /// Value: Number : JsonParserCsharp::JsonNumber ; 
        /// </summary>
        void Value1(object[] children);

        /// <summary>
        /// Semantic action for production 14:
        /// Value: Object; 
        /// </summary>
        void Value2(object[] children);

        /// <summary>
        /// Semantic action for production 15:
        /// Value: Array; 
        /// </summary>
        void Value3(object[] children);

        /// <summary>
        /// Semantic action for production 16:
        /// Value: 'true'^ /* Clipped */; 
        /// </summary>
        void Value4(object[] children);

        /// <summary>
        /// Semantic action for production 17:
        /// Value: 'false'^ /* Clipped */; 
        /// </summary>
        void Value5(object[] children);

        /// <summary>
        /// Semantic action for production 18:
        /// Value: 'null'^ /* Clipped */; 
        /// </summary>
        void Value6(object[] children);

        /// <summary>
        /// Semantic action for production 19:
        /// String: /"(\\.|[^"\\])*"/; 
        /// </summary>
        void String(object[] children);

        /// <summary>
        /// Semantic action for production 20:
        /// Number: /-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?[0-9]+)?/; 
        /// </summary>
        void Number(object[] children);

    }

    /// <summary>
    /// Base class for user actions for the JsonParserCsharp grammar.
    /// </summary>
    public partial class JsonParserCsharpActions : IJsonParserCsharpActions {
        /// <inheritdoc/>
        public virtual object CallSemanticActionForProductionNumber(int productionNumber, object[] children) {
            switch (productionNumber) {
                case 0: { var value = MapJson(children); OnJson(value); return value; }
                case 1: { var value = MapObject(children); OnObject(value); return value; }
                case 2: return MapObjectSuffix0(children);
                case 3: return MapObjectSuffix1(children);
                case 4: return MapObjectList0(children);
                case 5: return MapObjectList1(children);
                case 6: { var value = MapPair(children); OnPair(value); return value; }
                case 7: { var value = MapArray(children); OnArray(value); return value; }
                case 8: return MapArraySuffix0(children);
                case 9: return MapArraySuffix1(children);
                case 10: return MapArrayList0(children);
                case 11: return MapArrayList1(children);
                case 12: { var value = MapValue0(children); OnValue(value); return value; }
                case 13: { var value = MapValue1(children); OnValue(value); return value; }
                case 14: { var value = MapValue2(children); OnValue(value); return value; }
                case 15: { var value = MapValue3(children); OnValue(value); return value; }
                case 16: { var value = MapValue4(children); OnValue(value); return value; }
                case 17: { var value = MapValue5(children); OnValue(value); return value; }
                case 18: { var value = MapValue6(children); OnValue(value); return value; }
                case 19: { var value = MapString(children); OnString(value); return value; }
                case 20: { var value = MapNumber(children); OnNumber(value); return value; }
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
        /// User-facing action for non-terminal Json.
        /// </summary>
        public virtual void OnJson(Json arg) { }

        /// <summary>
        /// User-facing action for non-terminal Object.
        /// </summary>
        public virtual void OnObject(Object arg) { }

        /// <summary>
        /// User-facing action for non-terminal Pair.
        /// </summary>
        public virtual void OnPair(Pair arg) { }

        /// <summary>
        /// User-facing action for non-terminal Array.
        /// </summary>
        public virtual void OnArray(Array arg) { }

        /// <summary>
        /// User-facing action for non-terminal Value.
        /// </summary>
        public virtual void OnValue(Value arg) { }

        /// <summary>
        /// User-facing action for non-terminal String.
        /// </summary>
        public virtual void OnString(String arg) { }

        /// <summary>
        /// User-facing action for non-terminal Number.
        /// </summary>
        public virtual void OnNumber(Number arg) { }

        /// <summary>
        /// Semantic action for production 0:
        /// Json: Value; 
        /// </summary>
        public virtual void Json(object[] children) {
            var value = MapJson(children);
            OnJson(value);
        }

        private static Json MapJson(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Json((Value)children[0 + 0]);
            if (children.Length == 1 && children[0] is Json directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 0 (Json: Value;)");
        }

        /// <summary>
        /// Semantic action for production 1:
        /// Object: '{'^ /* Clipped */ ObjectSuffix; 
        /// </summary>
        public virtual void Object(object[] children) {
            var value = MapObject(children);
            OnObject(value);
        }

        private static Object MapObject(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Object((ObjectSuffix)children[0 + 0]);
            if (children.Length == 1 && children[0] is Object directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 1 (Object: '{'^ /* Clipped */ ObjectSuffix;)");
        }

        /// <summary>
        /// Semantic action for production 2:
        /// ObjectSuffix: Pair ObjectList /* Vec */ '}'^ /* Clipped */; 
        /// </summary>
        public virtual void ObjectSuffix0(object[] children) {
            var value = MapObjectSuffix0(children);
        }

        private static ObjectSuffix MapObjectSuffix0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2) {
                var value = new ObjectSuffixPairObjectListRBrace((Pair)children[0 + 0], (List<ObjectList>)children[0 + 1]);
                return new ObjectSuffixPairObjectListRBraceVariant(value);
            }
            if (children.Length == 1 && children[0] is ObjectSuffix directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 2 (ObjectSuffix: Pair ObjectList /* Vec */ '}'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 3:
        /// ObjectSuffix: '}'^ /* Clipped */; 
        /// </summary>
        public virtual void ObjectSuffix1(object[] children) {
            var value = MapObjectSuffix1(children);
        }

        private static ObjectSuffix MapObjectSuffix1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) {
                var value = new ObjectSuffixRBrace();
                return new ObjectSuffixRBraceVariant(value);
            }
            if (children.Length == 1 && children[0] is ObjectSuffix directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 3 (ObjectSuffix: '}'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 4:
        /// ObjectList: ','^ /* Clipped */ Pair ObjectList; 
        /// </summary>
        public virtual void ObjectList0(object[] children) {
            var value = MapObjectList0(children);
        }

        private static List<ObjectList> MapObjectList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<ObjectList>();
            if (children.Length == 1 && children[0] is List<ObjectList> directValue) return directValue;
            if (children.Length == 1) {
                var item = new ObjectList((Pair)children[0 + 0]);
                return new List<ObjectList> { item };
            }
            if (children.Length == 1 + 1 && children[1] is List<ObjectList> previous) {
                var item = new ObjectList((Pair)children[0 + 0]);
                var items = new List<ObjectList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 4 (ObjectList: ','^ /* Clipped */ Pair ObjectList;)");
        }

        /// <summary>
        /// Semantic action for production 5:
        /// ObjectList: ; 
        /// </summary>
        public virtual void ObjectList1(object[] children) {
            var value = MapObjectList1(children);
        }

        private static List<ObjectList> MapObjectList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<ObjectList>();
            if (children.Length == 1 && children[0] is List<ObjectList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 5 (ObjectList: ;)");
        }

        /// <summary>
        /// Semantic action for production 6:
        /// Pair: String : JsonParserCsharp::JsonString  ':'^ /* Clipped */ Value; 
        /// </summary>
        public virtual void Pair(object[] children) {
            var value = MapPair(children);
            OnPair(value);
        }

        private static Pair MapPair(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2 ) return new Pair(ConvertValue<String>(children[0 + 0]), (Value)children[0 + 1]);
            if (children.Length == 1 && children[0] is Pair directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 6 (Pair: String : JsonParserCsharp::JsonString  ':'^ /* Clipped */ Value;)");
        }

        /// <summary>
        /// Semantic action for production 7:
        /// Array: '['^ /* Clipped */ ArraySuffix; 
        /// </summary>
        public virtual void Array(object[] children) {
            var value = MapArray(children);
            OnArray(value);
        }

        private static Array MapArray(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Array((ArraySuffix)children[0 + 0]);
            if (children.Length == 1 && children[0] is Array directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 7 (Array: '['^ /* Clipped */ ArraySuffix;)");
        }

        /// <summary>
        /// Semantic action for production 8:
        /// ArraySuffix: Value ArrayList /* Vec */ ']'^ /* Clipped */; 
        /// </summary>
        public virtual void ArraySuffix0(object[] children) {
            var value = MapArraySuffix0(children);
        }

        private static ArraySuffix MapArraySuffix0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 2) {
                var value = new ArraySuffixValueArrayListRBracket((Value)children[0 + 0], (List<ArrayList>)children[0 + 1]);
                return new ArraySuffixValueArrayListRBracketVariant(value);
            }
            if (children.Length == 1 && children[0] is ArraySuffix directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 8 (ArraySuffix: Value ArrayList /* Vec */ ']'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 9:
        /// ArraySuffix: ']'^ /* Clipped */; 
        /// </summary>
        public virtual void ArraySuffix1(object[] children) {
            var value = MapArraySuffix1(children);
        }

        private static ArraySuffix MapArraySuffix1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) {
                var value = new ArraySuffixRBracket();
                return new ArraySuffixRBracketVariant(value);
            }
            if (children.Length == 1 && children[0] is ArraySuffix directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 9 (ArraySuffix: ']'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 10:
        /// ArrayList: ','^ /* Clipped */ Value ArrayList; 
        /// </summary>
        public virtual void ArrayList0(object[] children) {
            var value = MapArrayList0(children);
        }

        private static List<ArrayList> MapArrayList0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<ArrayList>();
            if (children.Length == 1 && children[0] is List<ArrayList> directValue) return directValue;
            if (children.Length == 1) {
                var item = new ArrayList((Value)children[0 + 0]);
                return new List<ArrayList> { item };
            }
            if (children.Length == 1 + 1 && children[1] is List<ArrayList> previous) {
                var item = new ArrayList((Value)children[0 + 0]);
                var items = new List<ArrayList>();
                items.Add(item);
                items.AddRange(previous);
                return items;
            }
            throw new InvalidOperationException("Unsupported C# mapping for production 10 (ArrayList: ','^ /* Clipped */ Value ArrayList;)");
        }

        /// <summary>
        /// Semantic action for production 11:
        /// ArrayList: ; 
        /// </summary>
        public virtual void ArrayList1(object[] children) {
            var value = MapArrayList1(children);
        }

        private static List<ArrayList> MapArrayList1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) return new List<ArrayList>();
            if (children.Length == 1 && children[0] is List<ArrayList> directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 11 (ArrayList: ;)");
        }

        /// <summary>
        /// Semantic action for production 12:
        /// Value: String : JsonParserCsharp::JsonString ; 
        /// </summary>
        public virtual void Value0(object[] children) {
            var value = MapValue0(children);
            OnValue(value);
        }

        private static Value MapValue0(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new ValueString(ConvertValue<String>(children[0 + 0]));
                return new ValueStringVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 12 (Value: String : JsonParserCsharp::JsonString ;)");
        }

        /// <summary>
        /// Semantic action for production 13:
        /// Value: Number : JsonParserCsharp::JsonNumber ; 
        /// </summary>
        public virtual void Value1(object[] children) {
            var value = MapValue1(children);
            OnValue(value);
        }

        private static Value MapValue1(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new ValueNumber(ConvertValue<Number>(children[0 + 0]));
                return new ValueNumberVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 13 (Value: Number : JsonParserCsharp::JsonNumber ;)");
        }

        /// <summary>
        /// Semantic action for production 14:
        /// Value: Object; 
        /// </summary>
        public virtual void Value2(object[] children) {
            var value = MapValue2(children);
            OnValue(value);
        }

        private static Value MapValue2(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new ValueObject((Object)children[0 + 0]);
                return new ValueObjectVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 14 (Value: Object;)");
        }

        /// <summary>
        /// Semantic action for production 15:
        /// Value: Array; 
        /// </summary>
        public virtual void Value3(object[] children) {
            var value = MapValue3(children);
            OnValue(value);
        }

        private static Value MapValue3(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1) {
                var value = new ValueArray((Array)children[0 + 0]);
                return new ValueArrayVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 15 (Value: Array;)");
        }

        /// <summary>
        /// Semantic action for production 16:
        /// Value: 'true'^ /* Clipped */; 
        /// </summary>
        public virtual void Value4(object[] children) {
            var value = MapValue4(children);
            OnValue(value);
        }

        private static Value MapValue4(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) {
                var value = new ValueTrue();
                return new ValueTrueVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 16 (Value: 'true'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 17:
        /// Value: 'false'^ /* Clipped */; 
        /// </summary>
        public virtual void Value5(object[] children) {
            var value = MapValue5(children);
            OnValue(value);
        }

        private static Value MapValue5(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) {
                var value = new ValueFalse();
                return new ValueFalseVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 17 (Value: 'false'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 18:
        /// Value: 'null'^ /* Clipped */; 
        /// </summary>
        public virtual void Value6(object[] children) {
            var value = MapValue6(children);
            OnValue(value);
        }

        private static Value MapValue6(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 0) {
                var value = new ValueNull();
                return new ValueNullVariant(value);
            }
            if (children.Length == 1 && children[0] is Value directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 18 (Value: 'null'^ /* Clipped */;)");
        }

        /// <summary>
        /// Semantic action for production 19:
        /// String: /"(\\.|[^"\\])*"/; 
        /// </summary>
        public virtual void String(object[] children) {
            var value = MapString(children);
            OnString(value);
        }

        private static String MapString(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new String((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is String directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 19 (String: /\"(\\\\.|[^\"\\\\])*\"/;)");
        }

        /// <summary>
        /// Semantic action for production 20:
        /// Number: /-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?[0-9]+)?/; 
        /// </summary>
        public virtual void Number(object[] children) {
            var value = MapNumber(children);
            OnNumber(value);
        }

        private static Number MapNumber(object[] children) {
            if (children == null) throw new ArgumentNullException(nameof(children));
            if (children.Length == 1 ) return new Number((Token)children[0 + 0]);
            if (children.Length == 1 && children[0] is Number directValue) return directValue;
            throw new InvalidOperationException("Unsupported C# mapping for production 20 (Number: /-?(0|[1-9][0-9]*)(\\.[0-9]+)?([eE][-+]?[0-9]+)?/;)");
        }

    }
}
