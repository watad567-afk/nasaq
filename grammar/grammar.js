/// Tree-sitter grammar for Nasaq (.nasaq)
/// Install: tree-sitter generate && tree-sitter build

module.exports = grammar({
  name: "nasaq",

  extras: ($) => [/\s/, $.comment],

  rules: {
    source_file: ($) => repeat($._item),

    _item: ($) =>
      choice(
        $.module_decl,
        $.function_decl,
        $.component_decl,
        $.import_decl,
        $.export_decl
      ),

    module_decl: ($) => seq("module", $.identifier),
    import_decl: ($) => seq("import", $.string),
    export_decl: ($) => seq("export", $._item),

    function_decl: ($) =>
      seq(
        optional("export"),
        "fn",
        $.identifier,
        "(",
        optional($.param_list),
        ")",
        optional(seq("->", $.type_name)),
        $.block
      ),

    component_decl: ($) =>
      seq(
        optional("export"),
        "component",
        $.identifier,
        "(",
        optional($.param_list),
        ")",
        "{",
        repeat(choice($.state_decl, $.view_block, $.style_block)),
        "}"
      ),

    state_decl: ($) =>
      seq("state", $.identifier, ":", $.type_name, "=", $._expr, optional(";")),

    view_block: ($) => seq("view", "{", repeat($._view_node), "}"),

    style_block: ($) =>
      seq(optional("style"), optional("scoped"), "{", $.css_text, "}"),

    param_list: ($) => seq($.param, repeat(seq(",", $.param))),
    param: ($) =>
      seq($.identifier, optional(seq(":", $.type_name)), optional(seq("=", $._expr))),

    block: ($) => seq("{", repeat($.statement), "}"),
    statement: ($) =>
      choice($.let_stmt, $.return_stmt, $.expr_stmt),

    let_stmt: ($) =>
      seq(optional("mut"), "let", $.identifier, optional(seq(":", $.type_name)), "=", $._expr, optional(";")),
    return_stmt: ($) => seq("return", optional($._expr), optional(";")),
    expr_stmt: ($) => seq($._expr, optional(";")),

    _expr: ($) =>
      choice(
        $.int_literal,
        $.string_literal,
        $.identifier,
        $.binary_expr,
        $.call_expr,
        $.assign_expr
      ),

    binary_expr: ($) => prec.left(1, seq($._expr, choice("+", "-", "*", "/"), $._expr)),
    call_expr: ($) => seq($.identifier, "(", optional($.arg_list), ")"),
    assign_expr: ($) => prec.right(1, seq($.identifier, "=", $._expr)),
    arg_list: ($) => seq($._expr, repeat(seq(",", $._expr))),

    _view_node: ($) =>
      choice($.html_element, $.interpolation, $.text),

    html_element: ($) =>
      seq(
        "<",
        $.tag_name,
        repeat($.attribute),
        optional("/"),
        ">",
        repeat($._view_node),
        optional(seq("</", $.tag_name, ">"))
      ),

    attribute: ($) =>
      choice(
        seq($.attr_name, "=", $.attr_value),
        seq("on:", $.identifier, "=", "{", $._expr, "}")
      ),

    attr_value: ($) => choice($.string_literal, seq("{", $._expr, "}")),
    interpolation: ($) => seq("{", $._expr, "}"),
    text: ($) => /[^<{]+/,

    type_name: ($) => choice("Int", "String", "Bool", $.identifier),
    identifier: ($) => /[A-Za-z_][A-Za-z0-9_]*/,
    tag_name: ($) => /[a-z][a-z0-9-]*/,
    attr_name: ($) => /[a-zA-Z_:][a-zA-Z0-9_:-]*/,
    int_literal: ($) => /\d+/,
    string_literal: ($) => /"([^"\\]|\\.)*"/,
    css_text: ($) => /[^{}]+/,
    comment: ($) => token(choice(seq("//", /.*/), seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"))),
  },
});
