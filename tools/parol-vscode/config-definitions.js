const ROOT_SECTION = "parol-vscode";

const CONFIG_DEFINITIONS = Object.freeze([
    { name: "max_k", defaultValue: 3, valueType: "integer" },
    {
        name: "formatting.empty_line_after_prod",
        defaultValue: true,
        valueType: "boolean",
    },
    {
        name: "formatting.prod_semicolon_on_nl",
        defaultValue: true,
        valueType: "boolean",
    },
    { name: "formatting.max_line_length", defaultValue: 100, valueType: "integer" },
]);

module.exports = {
    ROOT_SECTION,
    CONFIG_DEFINITIONS,
};