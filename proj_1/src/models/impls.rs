use crate::models::db_structure::*;

impl DatabaseKey for i64{
    fn equals(&self, other: &Self) -> bool {
        self == other
    }
    fn validate_value_type(val_type: &ValueType) -> bool {
        *val_type == ValueType::Int
    }
    fn get_from_value(val: &Value) -> Option<Self> where Self: Sized {
        if let Value::Int(num) = val {
            Some(*num)
        } else {
            None
        }
    }
    fn get_from_string(s: String) -> Result<Self, crate::db_errors::MyDatabaseError> where Self: Sized {
        match s.parse::<i64>() {
            Ok(num) => Ok(num),
            Err(_) => Err(crate::db_errors::MyDatabaseError::InvalidFieldValue),
        }
    }
}
impl DatabaseKey for String {
    fn equals(&self, other: &Self) -> bool {
        self == other
    }
    fn validate_value_type(val_type: &ValueType) -> bool {
        *val_type == ValueType::String
    }
    fn get_from_value(val: &Value) -> Option<Self> where Self: Sized {
        if let Value::String(s) = val {
            Some(s.clone())
        } else {
            None
        }
    }
    fn get_from_string(s: String) -> Result<Self, crate::db_errors::MyDatabaseError> where Self: Sized {
        Ok(s)
    }
}

