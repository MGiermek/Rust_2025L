use std::collections::{BTreeMap, HashMap};
use crate::db_errors::MyDatabaseError;
pub trait DatabaseKey {
    fn equals(&self, other: &Self) -> bool;
}

enum Value {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}

struct Record {
    columns: HashMap<String, Value>,   
}

struct Table<K: DatabaseKey> {
    records: BTreeMap<K, Record>,
}

enum AnyTable {
    StringKeyTable(Table<String>),
    IntKeyTable(Table<i64>),
}

struct Database<K: DatabaseKey> {
    tables: HashMap<String, Table<K>>,
    executedCommands: Vec<String>,
}

enum AnyDatabase {
    StringDatabase(Database<String>),
    IntDatabase(Database<i64>),
}