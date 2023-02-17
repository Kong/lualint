# LuaLint

LuaLint 是一个代码检查工具。目前阶段，它的目标是确保你的代码符合 Kong 的风格规范。

兼容 [OpenResty Lua Coding Style Guide](https://apache.googlesource.com/apisix/+/refs/tags/1.1/CODE_STYLE.md)

以下是各规则及其对应的常量名。第一列表示是否开发完毕。

|开发进度| 规则                             | 说明 |
|----| -------------------------------- | ------- |
|    | `INDENT_WITH_SPACES`             | 代码缩进使用 4 个空格 |
|    | `OPERATOR_SPACING`               | 运算符两边各保留一个空格 |
|    | `NO_TRAILING_SEMICOLON`          | 不在行尾添加分号 |
|    | `no_trailing_space`         | 不在行尾添加空格 |
| ✅ | `TWO_LINES_BETWEEN_FUNCTIONS`    | 函数之间保留两空行 |
|    | `ONE_LINE_BEFORE_ELSE`           | If-Else 分支语句中，Else、ElseIf 行前保留一空行 |
|    | `MAX_LINE_LENGTH_N`              | 每行最多 N 个字符，如果超过则需要对齐参数。 |
|    | `STR_CONCAT_NEWLINE`             | 字符串对齐的连接符应放到新行。 |
|    | `USE_LOCAL_VARIABLES`            | 尽可能使用局部变量 |
|    | `UPPERCASE_CONSTANTS`            | 常量使用大写 |
| ❓ | `PRE_ALLOCATE_TABLE`             | 使用 `table.new` 预先分配 `table` 的大小 |
|    | `NO_NIL_IN_ARRAY`                | 不在数组中使用 `nil` |
|    | `NO_STRING_CONCATENATION`        | 不在热代码路径使用拼接字符串 |
|    | `SNAKE_CASE_VARIABLE_NAMES`      | 使用 `snake_case` 命名变量 |
|    | `SNAKE_CASE_FUNCTION_NAMES`      | 使用 `snake_case` 命名函数 |
| ❓ | `EARLY_FUNCTION_RETURN`          | 函数尽早返回 |
|    | `NO_GOTO_STATEMENTS`             | 不要使用 `goto` 语句 |
|    | `LOCALIZE_LIBRARIES`             | 所有需要的库都应局部化 |
|    | `HANDLE_ERROR_MESSAGES`          | 对于所有返回错误信息的函数，都要处理错误信息 |
|    | `ERROR_MESSAGE_STRING_PARAMETER` | 错误信息以字符串的形式作为第二参数返回 |
| ✅ | `COMMA_AFTER_LAST_TABLE_ELEMENT` | `table` 的最后一个元素后面要加逗号 |

## 参考资料

- [How to Write a Code Formatter ― Andreas Zwinkau](https://beza1e1.tuxen.de/articles/formatting_code.html)
- https://github.com/JohnnyMorganz/StyLua

## License

[Mozilla Public License Version 2.0](LICENSE)