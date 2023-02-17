# LuaLint

LuaLint is a code checking tool. At this stage, its goal is to ensure that your code conforms to Kong's style specification.

Compatible with [OpenResty Lua Coding Style Guide](https://apache.googlesource.com/apisix/+/refs/tags/1.1/CODE_STYLE.md)

## Work in progress

The following are the rules and their corresponding constant names. The first column indicates whether development is complete.

| Progress | Rule Name                        | Description                                                         |
| -------- | -------------------------------- | ------------------------------------------------------------------- |
|          | `indent_with_spaces`             | Use 4 spaces for code indentation                                   |
|          | `operator_spacing`               | Keep one space on each side of the operator                         |
|          | `no_trailing_semicolon`          | No semicolons at the end of lines                                   |
| ✅       | `no_trailing_space`         | No spaces at the end of lines                                       |
| ✅       | `two_lines_between_functions`    | Keep two blank lines between functions                              |
| ✅       | `one_line_before_else`           | If-Else branching statement, one blank line before Else/ElseIf      |
| ✅       | `max_column_width`               | Up to N characters per line, alignment parameter is required.       |
|          | `str_concat_newline`             | String-aligned concatenation should be placed on a new line.        |
|          | `use_local_variables`            | Use local variables whenever possible                               |
|          | `uppercase_constants`            | Use upper case for constants                                        |
| ❓       | `pre_allocate_table`             | Pre-allocate the size of `table` using `table.new`                  |
|          | `no_nil_in_array`                | Do not use `nil` in arrays                                          |
|          | `no_string_concatenation`        | Do not use spliced strings in hot code paths                        |
|          | `snake_case_variable_names`      | Use `snake_case` to name variables                                  |
|          | `snake_case_function_names`      | Use `snake_case` to name functions                                  |
| ❓       | `early_function_return`          | function returns as early as possible                               |
| ❌       | `no_goto_statement`              | Don't use the `goto` statement                                      |
| ❓       | `localize_libraries`             | All required libraries should be localized                          |
| ❓       | `handle_error_messages`          | Handle error messages for all functions that return error messages  |
| ❓       | `error_message_string_parameter` | The error message is returned as a string as a second parameter     |
| ✅       | `table_ctor_comma`               | The last pair of `table` is followed by a comma                     |
| ✅       | `eof_blank_line`                 | The last line of the file is a blank line                           |

- [ ] require style - with or without parentheses

## Logging

Set the `LUALINT_LOG` environment variable to one of the following values to control the level of logging:

- `error`
- `warn`
- `info`
- `debug`
- `trace`

## Todo

- [x] Show filename
- [x] Exit code
- [ ] Preview error line

## Reference

- [How to Write a Code Formatter - Andreas Zwinkau](https://beza1e1.tuxen.de/articles/formatting_code.html)
- StyLua Project <https://github.com/JohnnyMorganz/StyLua>
- Full-Moon Project <https://github.com/Kampfkarren/full-moon>

## License

[Mozilla Public License Version 2.0](LICENSE)

Translated with www.DeepL.com/Translator (free version)

