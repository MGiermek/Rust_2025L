use std::collections::{BTreeMap, HashMap, HashSet};
use crate::db_errors::MyDatabaseError;
use crate::models::where_parsing::WhereClause;
pub trait DatabaseKey {
    fn equals(&self, other: &Self) -> bool;
    fn validate_value_type(s: &ValueType) -> bool;
    fn get_from_value(val: &Value) -> Option<Self> where Self: Sized;
    fn get_from_string(s: String) -> Result<Self, MyDatabaseError> where Self: Sized;
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}
impl Value {
    pub fn is_bigger_than(&self, other: &Value) -> Result<bool, MyDatabaseError> {
        match (self, other) {
            (Value::Int(i1), Value::Int(i2)) => Ok(i1 > i2),
            (Value::Float(f1), Value::Float(f2)) => Ok(f1 > f2),
            (Value::Int(i1), Value::Float(f2)) => Ok((*i1 as f64) > *f2),
            (Value::Float(f1), Value::Int(i2)) => Ok(*f1 > (*i2 as f64)),
            (Value::String(s1), Value::String(s2)) => Ok(s1 > s2),
            _ => Err(MyDatabaseError::CannotCompareValues),
        }
    }
    pub fn is_equal_to(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(i1), Value::Float(f2)) => (*i1 as f64) == *f2,
            (Value::Float(f1), Value::Int(i2)) => *f1 == (*i2 as f64),
            _ => self == other,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum ValueType {
    Bool,
    String,
    Int,
    Float,
}
impl ValueType {
    pub fn get_value(&self, s: &str) -> Result<Value, MyDatabaseError> {
        match *self {
                ValueType::Bool => {
                    match s.trim().to_ascii_lowercase().as_str() {
                        "true" => Ok(Value::Bool(true)),
                        "false" => Ok(Value::Bool(false)),
                        _ => Err(MyDatabaseError::InvalidFieldValue),
                    }
                },
                ValueType::Int => {
                    match s.trim().parse::<i64>() {
                        Ok(num) => Ok(Value::Int(num)),
                        Err(_) => Err(MyDatabaseError::InvalidFieldValue),
                    }
                },
                ValueType::Float => {
                    match s.trim().parse::<f64>() {
                        Ok(num) => Ok(Value::Float(num)),
                        Err(_) => Err(MyDatabaseError::InvalidFieldValue),
                    }
                },
                ValueType::String => {
                    Ok(Value::String(s.trim().to_string()))
                },
            }
    }
}

#[derive(Debug)]
pub struct Record {
    values: HashMap<String, Value>,   
}
impl Record {
    pub fn get_value_for_column(&self, column_name: &str) -> Option<&Value> {
        self.values.get(column_name)
    }
}

#[derive(Debug)]
pub struct Table<K: DatabaseKey + Ord> {
    key_name: String,
    structure: HashMap<String, ValueType>, // column name to type
    records: BTreeMap<K, Record>,
}
impl<K: DatabaseKey + Ord> Table<K> {
    fn insert_values(&mut self, values: HashMap<String, Value>) -> Result<(), MyDatabaseError> {
        let table_keys: HashSet<&String> = self.structure.keys().collect();
        let value_keys: HashSet<&String> = values.keys().collect();

        if table_keys != value_keys {
            return Err(MyDatabaseError::KeysMismatch);
        }

        let Some(key_value) = values.get(&self.key_name) else {
            return Err(MyDatabaseError::KeysMismatch); // shouldn't happen due to earlier check
        };
        let Some(key)= K::get_from_value(key_value) else {
            return Err(MyDatabaseError::KeysMismatch); // shouldn't happen due to earlier check
        };

        if self.records.contains_key(&key) {
            return Err(MyDatabaseError::RecordAlreadyExists);
        }
        let record = Record {
            values,
        };
        self.records.insert(key, record); // returns None because of earlier check
        Ok(())
    }
    fn delete_key(&mut self, key_as_string: String) -> Result<(), MyDatabaseError> {
        let key = K::get_from_string(key_as_string)?;
        match self.records.remove(&key) {
            Some(_) => Ok(()),
            None => Err(MyDatabaseError::KeyNotFound),
        }
    }
    fn select_and_display(&mut self, values_to_select: &Vec<String>, condition: &Option<WhereClause>) -> Result<(), MyDatabaseError> {
        for value_name in values_to_select {
            if !self.structure.contains_key(value_name) {
                return Err(MyDatabaseError::InvalidFieldName);
            }
        }
        for value_name in values_to_select {
            print!("{}\t", value_name);
        }
        println!();
        for record in self.records.values() {
            if let Some(cond) = condition {
                let condition_met = cond.evaluate_for_record(record)?;
                if !condition_met {
                    continue;
                }
            }
            for value_name in values_to_select {
                let Some(value) = record.values.get(value_name) else {
                    return Err(MyDatabaseError::InvalidFieldName); // shouldn't happen due to earlier check
                };
                match value {
                    Value::Bool(b) => print!("{}\t", b),
                    Value::Int(i) => print!("{}\t", i),
                    Value::Float(f) => print!("{}\t", f),
                    Value::String(s) => print!("{}\t", s),
                }
            }
            println!();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AnyTableRef<'a> {
    StringKeyTable(&'a mut Table<String>),
    IntKeyTable(&'a mut Table<i64>),
}
impl<'a> AnyTableRef<'a> {
    pub fn get_type_for_name(&self, name: &str) -> Option<&ValueType> {
        match self {
            AnyTableRef::StringKeyTable(table) => table.structure.get(name),
            AnyTableRef::IntKeyTable(table) => table.structure.get(name),
        }
    }
    pub fn get_all_columns(&self) -> Vec<String> {
        match self {
            AnyTableRef::StringKeyTable(table) => table.structure.keys().cloned().collect(),
            AnyTableRef::IntKeyTable(table) => table.structure.keys().cloned().collect(),
        }
    }
    pub fn insert_values(&mut self, values: HashMap<String, Value>) -> Result<(), MyDatabaseError> {
        match self {
            AnyTableRef::StringKeyTable(table) => table.insert_values(values),
            AnyTableRef::IntKeyTable(table) => table.insert_values(values),
        }
    }
    pub fn delete_key(&mut self, key_as_string: String) -> Result<(), MyDatabaseError> {
        match self {
            AnyTableRef::StringKeyTable(table) => table.delete_key(key_as_string),
            AnyTableRef::IntKeyTable(table) => table.delete_key(key_as_string),
        }
    }
    pub fn select_and_display(&mut self, values_to_select: &Vec<String>, condition: &Option<WhereClause>) -> Result<(), MyDatabaseError> {
        match self {
            AnyTableRef::StringKeyTable(table) => table.select_and_display(values_to_select, condition),
            AnyTableRef::IntKeyTable(table) => table.select_and_display(values_to_select, condition),
        }
    }
    pub fn get_structure(&self) -> &HashMap<String, ValueType> {
        match self {
            AnyTableRef::StringKeyTable(table) => &table.structure,
            AnyTableRef::IntKeyTable(table) => &table.structure,
        }
    }
}

#[derive(Debug)]
pub struct Database<K: DatabaseKey + Ord> {
    tables: HashMap<String, Table<K>>,
    // executed_commands: Vec<String>,
}
impl<K: DatabaseKey + Ord> Database<K> {
    pub fn new() -> Self {
        Database::<K> {
            tables: HashMap::new(),
            // executed_commands: Vec::new(),
        }
    }
    fn create_table(&mut self, name: &str, key_name: &str, fields: HashMap<String, ValueType>) -> Result<(), MyDatabaseError> {
        let Some(key_type) = fields.get(key_name) else {
            return Err(MyDatabaseError::InvalidCommandFormat("CREATE. Key was not in fields"));
        };
        if !K::validate_value_type(key_type) {
            return Err(MyDatabaseError::InvalidKeyType);
        }
        if self.tables.contains_key(name) {
            return Err(MyDatabaseError::TableAlreadyExists(name.to_string()));
        }
        let table = Table::<K> {
            key_name: key_name.to_string(),
            structure: fields,
            records: BTreeMap::new(),
        };
        self.tables.insert(name.to_string(), table); // checked earlier that it has to return Some, couldn't match, because insert changes found values
        Ok(())
    }
}

#[derive(Debug)]
pub enum AnyDatabase {
    StringDatabase(Database<String>),
    IntDatabase(Database<i64>),
}
impl AnyDatabase {
    pub fn get_table_by_name(&mut self, name: &str) -> Result<AnyTableRef<'_>, MyDatabaseError> {
        match self {
            AnyDatabase::StringDatabase(db) => {
                match db.tables.get_mut(name) {
                    Some(table) => Ok(AnyTableRef::StringKeyTable(table)),
                    None => Err(MyDatabaseError::TableNotFound(name.to_string())),
                }
            },
            AnyDatabase::IntDatabase(db) => {
                match db.tables.get_mut(name) {
                    Some(table) => Ok(AnyTableRef::IntKeyTable(table)),
                    None => Err(MyDatabaseError::TableNotFound(name.to_string())),
                }
            },
        }
    }
    pub fn create_table(&mut self, name: &str, key_name: &str, fields: HashMap<String, ValueType>) -> Result<(), MyDatabaseError> {
        match self {
            AnyDatabase::StringDatabase(db) => db.create_table(name, key_name, fields),
            AnyDatabase::IntDatabase(db) => db.create_table(name, key_name, fields),
        }
    }
}