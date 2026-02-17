# json_parser_csharp

This example ports `examples/json_parser` to C#.
It demonstrates how `parol` generates C# parser/types and how semantic actions can consume the typed model.

It also demonstrates the typed-token conversion approach:

- Grammar annotation: `String: /.../ : JsonParserCsharp::JsonString;`
- Grammar annotation: `Number: /.../ : JsonParserCsharp::JsonNumber;`
- Generated `String` and `Number` models now contain `JsonString`/`JsonNumber` instead of raw `Token`.
- `JsonString` and `JsonNumber` convert from `Token` via constructor.

## Run

From the repository root, execute:

```powershell
dotnet run --project .\examples\json_parser_csharp\json_parser_csharp.csproj -- examples/json/JsonParserTest.json
```

You can also use smaller inputs in `examples/json` (for example `object.json`, `array.json`, `number.json`).
