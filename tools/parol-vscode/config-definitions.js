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
  {
    name: "formatting.max_line_length",
    defaultValue: 100,
    valueType: "integer",
  },
  {
    name: "formatting.indent_style",
    defaultValue: "4",
    valueType: "string",
  },
  {
    name: "formatting.space_after_colon",
    defaultValue: true,
    valueType: "boolean",
  },
  {
    name: "formatting.space_before_semicolon",
    defaultValue: false,
    valueType: "boolean",
  },
  {
    name: "formatting.space_around_pipe",
    defaultValue: true,
    valueType: "boolean",
  },
  {
    name: "formatting.space_around_equals",
    defaultValue: true,
    valueType: "boolean",
  },
  {
    name: "formatting.scanner_brace_newline",
    defaultValue: false,
    valueType: "boolean",
  },
  {
    name: "formatting.semicolon_end_of_line",
    defaultValue: false,
    valueType: "boolean",
  },
  {
    name: "formatting.place_top_level_alternatives_on_new_line_each",
    defaultValue: false,
    valueType: "boolean",
  },
  {
    name: "formatting.leading_line_comments_on_own_line",
    defaultValue: false,
    valueType: "boolean",
  },
]);

module.exports = {
  ROOT_SECTION,
  CONFIG_DEFINITIONS,
};
