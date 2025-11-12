use crate::{db_errors::MyDatabaseError, models::utilities::split_preserving_quote_insides};
use std::collections::HashMap;
use crate::models::db_structure::{ValueType, Record, Value};

#[derive(PartialEq, Debug)]
enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual
}
impl ComparisonOperator {
    fn evaluate(&self, v1: &Value, v2: &Value) -> Result<Value, MyDatabaseError> {
        let result = match self {
            ComparisonOperator::Equal => v1.is_equal_to(v2),
            ComparisonOperator::NotEqual => !v1.is_equal_to(v2),
            ComparisonOperator::GreaterThan => v1.is_bigger_than(v2)?,
            ComparisonOperator::LessThan => v2.is_bigger_than(v1)?,
            ComparisonOperator::GreaterThanOrEqual => {
                v1.is_bigger_than(v2)? || v1.is_equal_to(v2)
            },
            ComparisonOperator::LessThanOrEqual => {
                v2.is_bigger_than(v1)? || v2.is_equal_to(v1)
            }
        };
        Ok(Value::Bool(result))
    }
}

#[derive(PartialEq, Debug)]
enum LogicalOperator {
    And,
    Or,
}
impl LogicalOperator {
    fn evaluate(&self, v1: &Value, v2: &Value) -> Result<Value, MyDatabaseError> {
        match (v1, v2) {
            (Value::Bool(b1), Value::Bool(b2)) => {
                let result = match self {
                    LogicalOperator::And => *b1 && *b2,
                    LogicalOperator::Or => *b1 || *b2,
                };
                Ok(Value::Bool(result))
            },
            _ => Err(MyDatabaseError::InvalidLogicalOperation),
        }
    }
}

#[derive(PartialEq, Debug)]
enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
impl MathOperator {
    fn evaluate(&self, v1: &Value, v2: &Value) -> Result<Value, MyDatabaseError> {
        match (v1, v2) {
            (Value::Float(f1), Value::Float(f2)) => {
                let result = match self {
                    MathOperator::Add => f1 + f2,
                    MathOperator::Subtract => f1 - f2,
                    MathOperator::Multiply => f1 * f2,
                    MathOperator::Divide => {
                        if *f2 == 0.0 {
                            return Err(MyDatabaseError::DivisionByZero);
                        }
                        f1 / f2
                    },
                };
                Ok(Value::Float(result))
            },
            (Value::Int(i1), Value::Int(i2)) => {
                let result = match self {
                    MathOperator::Add => i1 + i2,
                    MathOperator::Subtract => i1 - i2,
                    MathOperator::Multiply => i1 * i2,
                    MathOperator::Divide => {
                        if *i2 == 0 {
                            return Err(MyDatabaseError::DivisionByZero);
                        }
                        i1 / i2
                    },
                };
                Ok(Value::Int(result))
            },
            (Value::Int(i1), Value::Float(f2)) => {
                let f1 = *i1 as f64;
                let result = match self {
                    MathOperator::Add => f1 + f2,
                    MathOperator::Subtract => f1 - f2,
                    MathOperator::Multiply => f1 * f2,
                    MathOperator::Divide => {
                        if *f2 == 0.0 {
                            return Err(MyDatabaseError::DivisionByZero);
                        }
                        f1 / f2
                    },
                };
                Ok(Value::Float(result))
            },
            (Value::Float(f1), Value::Int(i2)) => {
                let f2 = *i2 as f64;
                let result = match self {
                    MathOperator::Add => f1 + f2,
                    MathOperator::Subtract => f1 - f2,
                    MathOperator::Multiply => f1 * f2,
                    MathOperator::Divide => {
                        if f2 == 0.0 {
                            return Err(MyDatabaseError::DivisionByZero);
                        }
                        f1 / f2
                    },
                };
                Ok(Value::Float(result))
            }
            _ => Err(MyDatabaseError::InvalidMathOperation),
        }
    }
}

#[derive(PartialEq, Debug)]
enum AnyOperator {
    Comparison(ComparisonOperator),
    Logical(LogicalOperator),
    Math(MathOperator),
}
impl AnyOperator {
    fn evaluate(&self, v1: &Value, v2: &Value) -> Result<Value, MyDatabaseError> {
        match self {
            AnyOperator::Comparison(op) => op.evaluate(v1, v2),
            AnyOperator::Logical(op) => op.evaluate(v1, v2),
            AnyOperator::Math(op) => op.evaluate(v1, v2),
        }
    }
}

#[derive(PartialEq, Debug)]
enum ClauseElement {
    OpeningBracket,
    ClosingBracket,
    Operator(AnyOperator),
    ColumnIdentifier(String),
    Constant(Value)
}
impl ClauseElement {
    fn get_importance(&self) -> i8 {
        match self {
            ClauseElement::OpeningBracket | ClauseElement::ClosingBracket => -1, // doesn't matter, it's a special case anyway
            ClauseElement::Operator(AnyOperator::Logical(op)) => match op {
                LogicalOperator::And => 1,
                LogicalOperator::Or => 0,
            },
            ClauseElement::Operator(AnyOperator::Comparison(_)) => 2,
            ClauseElement::Operator(AnyOperator::Math(op)) => match op {
                MathOperator::Add | MathOperator::Subtract => 3,
                MathOperator::Multiply | MathOperator::Divide => 4,
            },
            ClauseElement::ColumnIdentifier(_) | ClauseElement::Constant(_) => 10, // again, doesn't matter
        }
    }
}

#[derive(Debug)]
enum BoxedOrReferencedElement<'a> {
    Boxed(Box<ClauseElement>),
    Referenced(&'a ClauseElement),
}
impl<'a> BoxedOrReferencedElement<'a> {
    fn get_element_ref(&self) -> &ClauseElement {
        match self {
            BoxedOrReferencedElement::Boxed(b) => b.as_ref(),
            BoxedOrReferencedElement::Referenced(r) => r,
        }
    }
}

#[derive(Debug)]
pub struct WhereClause {
    onp_elements: Vec<ClauseElement>
}
impl WhereClause {
    pub fn create_from_string(mut clause: String, columns: &HashMap<String, ValueType>) -> Result<WhereClause, MyDatabaseError> {
        let operators = vec!["=", "!=", ">", "<", ">=", "<=", "AND", "OR", "+", "-", "*", "/", "(", ")"];
        for op in operators {
            clause = clause.replace(op, &format!(" {} ", op));
        }
        
        let mut elements: Vec<ClauseElement> = Vec::new();
        for token in split_preserving_quote_insides(&clause, ' ') {
            match token {
                "(" => elements.push(ClauseElement::OpeningBracket),
                ")" => elements.push(ClauseElement::ClosingBracket),
                "=" => elements.push(ClauseElement::Operator(AnyOperator::Comparison(ComparisonOperator::Equal))),
                "!=" => elements.push(ClauseElement::Operator(AnyOperator::Comparison(ComparisonOperator::NotEqual))),
                ">" => elements.push(ClauseElement::Operator(AnyOperator::Comparison(ComparisonOperator::GreaterThan))),
                "<" => elements.push(ClauseElement::Operator(AnyOperator::Comparison(ComparisonOperator::LessThan))),
                ">=" => elements.push(ClauseElement::Operator(AnyOperator::Comparison(ComparisonOperator::GreaterThanOrEqual))),
                "<=" => elements.push(ClauseElement::Operator(AnyOperator::Comparison(ComparisonOperator::LessThanOrEqual))),
                "AND" => elements.push(ClauseElement::Operator(AnyOperator::Logical(LogicalOperator::And))),
                "OR" => elements.push(ClauseElement::Operator(AnyOperator::Logical(LogicalOperator::Or))),
                "+" => elements.push(ClauseElement::Operator(AnyOperator::Math(MathOperator::Add))),
                "-" => elements.push(ClauseElement::Operator(AnyOperator::Math(MathOperator::Subtract))),
                "*" => elements.push(ClauseElement::Operator(AnyOperator::Math(MathOperator::Multiply))),
                "/" => elements.push(ClauseElement::Operator(AnyOperator::Math(MathOperator::Divide))),
                _ => {
                    if columns.contains_key(token) {
                        elements.push(ClauseElement::ColumnIdentifier(token.to_string()));
                    } else if let Ok(constant) = token.parse::<f64>() {
                        elements.push(ClauseElement::Constant(Value::Float(constant)));
                    } else if token.eq_ignore_ascii_case("true") {
                        elements.push(ClauseElement::Constant(Value::Bool(true)));
                    } else if token.eq_ignore_ascii_case("false") {
                        elements.push(ClauseElement::Constant(Value::Bool(false)));
                    } else if token.starts_with("\"") && token.ends_with("\"") && token.len() >= 2 {
                        let str_content = &token[1..token.len()-1];
                        elements.push(ClauseElement::Constant(Value::String(str_content.to_string())));
                    } else {
                        return Err(MyDatabaseError::InvalidWhereClauseFormat(format!("Unknown token in WHERE clause: {}", token)));
                    }
                }
            }
        }

        let mut help_stack = Vec::new();
        let mut onp_elements = Vec::new();

        for element in elements {
            match element {
                ClauseElement::ColumnIdentifier(_) | ClauseElement::Constant(_) => onp_elements.push(element),
                ClauseElement::ClosingBracket => {
                    let mut opening_bracket_found = false;
                    while let Some(operator) = help_stack.pop() {
                        if operator != ClauseElement::OpeningBracket {
                            onp_elements.push(operator);
                        }
                        else {
                            opening_bracket_found = true;
                            break;
                        }
                    }
                    if opening_bracket_found != true {
                        return Err(MyDatabaseError::InvalidWhereClauseFormat("Opening bracket missing".to_string()));
                    }
                }
                ClauseElement::OpeningBracket => help_stack.push(element),
                _ => {
                    let importance = element.get_importance();
                    while let Some(operator) = help_stack.last() {
                        if operator.get_importance() >= importance {
                            let Some(popped) = help_stack.pop() else {
                                return Err(MyDatabaseError::InvalidWhereClauseFormat("This error shouldn't happend".to_string()));
                            };
                            onp_elements.push(popped);
                        }
                        else {
                            break;
                        }
                    }
                    help_stack.push(element);
                }
            }
        }
        while let Some(operator) = help_stack.pop() {
            if operator == ClauseElement::OpeningBracket {
                return Err(MyDatabaseError::InvalidWhereClauseFormat("Closing bracket missing".to_string()));
            }
            onp_elements.push(operator);
        }
        Ok(WhereClause { onp_elements })
    }

    pub fn evaluate_for_record(&self, record: &Record) -> Result<bool, MyDatabaseError> {
        let mut eval_stack: Vec<BoxedOrReferencedElement> = Vec::new();
        for element in &self.onp_elements {
            match element {
                ClauseElement::ClosingBracket | ClauseElement::OpeningBracket => return Err(MyDatabaseError::WronglyParsedClause("Brackets shouldn't be in ONP".to_string())),
                ClauseElement::ColumnIdentifier(_) | ClauseElement::Constant(_) => eval_stack.push(BoxedOrReferencedElement::Referenced(element)),
                ClauseElement::Operator(op) => {
                    let Some(right) = eval_stack.pop() else {
                        return Err(MyDatabaseError::WronglyParsedClause("Not enough elements on stack for operation".to_string()));
                    };
                    let Some(left) = eval_stack.pop() else {
                        return Err(MyDatabaseError::WronglyParsedClause("Not enough elements on stack for operation".to_string()));
                    };

                    let left_value = match left.get_element_ref() {
                        ClauseElement::ColumnIdentifier(col_name) => {
                            let Some(ret) = record.get_value_for_column(col_name) else {
                                return Err(MyDatabaseError::WronglyParsedClause(format!("Column {} not found in record", col_name)));
                            };
                            ret
                        },
                        ClauseElement::Constant(c) => {
                            c
                        },
                        _ => return Err(MyDatabaseError::WronglyParsedClause("Expected column identifier or constant".to_string())),
                    };

                    let right_value = match right.get_element_ref() {
                        ClauseElement::ColumnIdentifier(col_name) => {
                            let Some(ret) = record.get_value_for_column(col_name) else {
                                return Err(MyDatabaseError::WronglyParsedClause(format!("Column {} not found in record", col_name)));
                            };
                            ret
                        },
                        ClauseElement::Constant(c) => {
                            c
                        },
                        _ => return Err(MyDatabaseError::WronglyParsedClause("Expected column identifier or constant".to_string())),
                    };

                    let result = ClauseElement::Constant(op.evaluate(left_value, right_value)?);
                    eval_stack.push(BoxedOrReferencedElement::Boxed(Box::new(result)));
                }
            }
        }
        if eval_stack.len() != 1 {
            // println!("Eval stack: {:?}", eval_stack);
            return Err(MyDatabaseError::WronglyParsedClause("More than one element left on stack after evaluation".to_string()));
        }
        let Some(final_element) = eval_stack.pop() else {
            return Err(MyDatabaseError::WronglyParsedClause("This error shouldn't happen".to_string()));
        };
        match final_element.get_element_ref() {
            ClauseElement::Constant(Value::Bool(b)) => Ok(*b),
            _ => Err(MyDatabaseError::WronglyParsedClause("Final element is not a boolean constant".to_string())),
        }
    }
}