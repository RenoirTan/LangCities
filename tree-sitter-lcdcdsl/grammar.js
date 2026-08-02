/**
 * @file Parser for LangCities' Domain Specific Language. Expressions in this DSL are used to generate values for LangCities Dictionaries.
 * @author RenoirTan <renoirtan2005@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "lcdcdsl",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello",
  }
});
