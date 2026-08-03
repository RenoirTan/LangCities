/**
 * @file Parser for LangCities Domain Specific Language. Expressions in this DSL are used to generate values for LangCities Dictionaries.
 * @author RenoirTan <renoirtan2005@gmail.com>
 * @license MIT
 */

const identifier_int = /(?:\d+)/;
const identifier_alphanumeric = /(?:[a-z_][a-z0-9_]*)/;
const identifier_part = new RegExp(
  `(?:${identifier_int.source}|${identifier_alphanumeric.source})`
);
const user_namespaced_identifier = new RegExp(
  `(?:${identifier_part.source}@${identifier_part.source})`
);
const first_identifier_part = new RegExp(
  `(?:${identifier_part.source}(?:@${identifier_part.source})?)`
);
const identifier = new RegExp(
  `(?:\\$${first_identifier_part.source}(?:\\.${identifier_part.source})*)`
);

console.log(`Identifier Regex: ${identifier.source}`);

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "lcdcdsl",

  extras: $ => [
    /\s/,
  ],

  supertypes: $ => [
    $.expression,
    $.string_literal,
  ],

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => $.multi_expression,

    multi_expression: $ => seq(
      repeat($.expression_sep),
      field("first", $.expression),
      repeat(
        seq(
          repeat1($.expression_sep),
          field("others", $.expression),
        ),
      ),
      repeat($.expression_sep),
    ),
    expression_sep: $ => /\r\n|[\n\r]/,
    expression: $ => choice(
      $.function_call,
      $.identifier,
      $.string_literal,
      $.binary_expression,
      $.parenthesis_expression,
    ),

    parenthesis_expression: $ => seq(
      "(",
      $.expression,
      ")",
    ),

    binary_expression: $ => choice(
      $.add_expression,
    ),
    add_expression: $ => prec.left(1, seq($.expression, "+", $.expression)),

    function_call: $ => seq(
      $.identifier,
      "(",
      $.expression,
      repeat(seq(",", $.expression)),
      optional(","),
      ")",
    ),

    identifier: $ => identifier,

    /*
    identifier: $ => seq(
      "$",
      $.identifier_inner,
    ),
    identifier_inner: $ => seq(
      $.identifier,
      repeat(seq(".", $.identifier)),
    ),
    identifier: $ => choice(
      $.user_namespaced_identifier,
      $.pure_identifier,
    ),
    user_namespaced_identifier: $ => seq(
      $.identifier_part,
      "@",
      $.identifier_part,
    ),
    pure_identifier: $ => choice($.identifier_part),
    identifier_part: $ => choice($.identifier_int, $.identifier_alphanumeric),
    identifier_int: $ => /\d+/,
    */
    // identifier_alphanumeric: $ => /[a-z_][a-z0-9_]*/i,

    string_literal: $ => choice(
      $.unquoted_string_literal,
      $.dquoted_string_literal,
      $.squoted_string_literal,
    ),
    unquoted_string_literal: $ => /(?:[^\s$,(){}+\\"']|\\[$,(){}+\\"'])+/,
    dquoted_string_literal: $ => /"(?:\\.|[^"\\])*"/,
    squoted_string_literal: $ => /'(?:\\.|[^'\\])*'/,
  }
});
