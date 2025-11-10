use std::collections::{BTreeMap, HashMap};
use crate::db_errors::MyDatabaseError;
pub trait DatabaseKey {
    fn equals(&self, other: &Self) -> bool;
}

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}
#[derive(Debug)]
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

#[derive(Debug)]
pub struct Table<K: DatabaseKey> {
    structure: HashMap<String, ValueType>, // column name to type
    records: BTreeMap<K, Record>,
}

#[derive(Debug)]
pub enum AnyTableRef<'a> {
    StringKeyTable(&'a Table<String>),
    IntKeyTable(&'a Table<i64>),
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
}

#[derive(Debug)]
pub struct Database<K: DatabaseKey> {
    tables: HashMap<String, Table<K>>,
    executed_commands: Vec<String>,
}
impl<K: DatabaseKey> Database<K> {
    pub fn new() -> Self {
        Database::<K> {
            tables: HashMap::new(),
            executed_commands: Vec::new(),
        }
    }
    pub fn create_table(&mut self, name: &str, key_name: &str, fields: HashMap<String, ValueType>) -> Result<(), MyDatabaseError> {
        let table = Table::<K> {
            structure: fields,
            records: BTreeMap::new(),
        };
        if self.tables.contains_key(name) {
            return Err(MyDatabaseError::TableAlreadyExists(name.to_string()));
        }
        self.tables.insert(name.to_string(), table);
        Ok(())
    }
}

#[derive(Debug)]
pub enum AnyDatabase {
    StringDatabase(Database<String>),
    IntDatabase(Database<i64>),
}
impl AnyDatabase {
    pub fn get_table_by_name(&self, name: &str) -> Result<AnyTableRef<'_>, MyDatabaseError> {
        match self {
            AnyDatabase::StringDatabase(db) => {
                match db.tables.get(name) {
                    Some(table) => Ok(AnyTableRef::StringKeyTable(table)),
                    None => Err(MyDatabaseError::TableNotFound(name.to_string())),
                }
            },
            AnyDatabase::IntDatabase(db) => {
                match db.tables.get(name) {
                    Some(table) => Ok(AnyTableRef::IntKeyTable(table)),
                    None => Err(MyDatabaseError::TableNotFound(name.to_string())),
                }
            },
        }
    }
}