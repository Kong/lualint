use full_moon::ast::Expression;

pub trait HasTrailing {
    fn trailing(&self) -> Option<&str>;
}

impl HasTrailing for Expression {
    fn trailing(&self) -> Option<&str> {
        match self {
            Expression::BinaryOperator{lhs, binop,rhs } => {},
            Expression::Parentheses{contained, expression} => {},
            Expression::UnaryOperator{unop, expression} => {},
            Expression::Value {value} => {
                match value {
                    Value::Function {function} => {},
                    Value::Table {table} => {},
                    Value::String {string} => {},
                    Value::Number {number} => {},
                    Value::Nil {nil} => {},
                    Value::Boolean {boolean} => {},
                    Value::Vararg {vararg} => {},
                    Value::FunctionCall {function_call} => {},
                    Value::TableCall {table_call} => {},
                    Value::Field {field} => {},
                    Value::MethodCall {method_call} => {},
                    Value::Parentheses {parentheses} => {},
                    Value::Expression {expression} => {},
                }
            },
            _ => None,
        }
    }
}