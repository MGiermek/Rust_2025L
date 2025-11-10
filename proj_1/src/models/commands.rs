use crate::db_errors::MyDatabaseError;
use crate::models::db_structure::*;
use std::collections::HashMap;
pub trait Command<'b> {
    fn execute(&self) -> Result<(), MyDatabaseError>;
    fn parse_input<'a>(input: &'a str, context_db: &'b AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized;
}
#[derive(Debug)]
pub enum AnyCommand<'b> {
    CreateTable(CreateTableCmd<'b>),
    InsertRecord(InsertRecordCmd<'b>),
    DeleteRecord(DeleteRecordCmd<'b>),
    Select(SelectCmd<'b>),
}
impl<'b> Command<'b> for AnyCommand<'b> {
    fn execute(&self) -> Result<(), MyDatabaseError> {
        match self {
            AnyCommand::CreateTable(cmd) => cmd.execute(),
            AnyCommand::InsertRecord(cmd) => cmd.execute(),
            AnyCommand::DeleteRecord(cmd) => cmd.execute(),
            AnyCommand::Select(cmd) => cmd.execute(),
        }
    }
    fn parse_input<'a>(input: &'a str, context_db: &'b AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((command_type, rest)) = input.trim().split_once(" ") else {
            return Err(MyDatabaseError::InvalidCommandFormat("UNKNOWN"));
        };
        match command_type {
            "CREATE" => {
                let cmd = CreateTableCmd::parse_input(rest, context_db)?;
                Ok(AnyCommand::CreateTable(cmd))
            },
            "INSERT" => {
                let cmd = InsertRecordCmd::parse_input(rest, context_db)?;
                Ok(AnyCommand::InsertRecord(cmd))
            },
            "DELETE" => {
                let cmd = DeleteRecordCmd::parse_input(rest, context_db)?;
                Ok(AnyCommand::DeleteRecord(cmd))
            },
            "SELECT" => {
                let cmd = SelectCmd::parse_input(rest, context_db)?;
                Ok(AnyCommand::Select(cmd))
            },
            _ => Err(MyDatabaseError::InvalidCommandFormat("UNKNOWN")),
        }
    }
}

#[derive(Debug)]
pub struct CreateTableCmd<'a> {
    db: &'a AnyDatabase,
    name: String,
    key_name: String,
    fields: HashMap<String, ValueType>
}
impl<'b> Command<'b> for CreateTableCmd<'b> {
    fn execute(&self) -> Result<(), MyDatabaseError> {
        todo!();
    }
    fn parse_input<'a>(input: &'a str, context_db: &'b AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((name, rest)) = input.split_once("KEY") else {
            return Err(MyDatabaseError::InvalidCommandFormat("CREATE"));
        };
        let Some((key_name, fields_str)) = rest.trim().split_once("FIELDS") else {
            return Err(MyDatabaseError::InvalidCommandFormat("CREATE"));
        };

        let mut fields: HashMap<String, ValueType> = HashMap::new();
        for field in fields_str.trim().split(",") {
            let Some((field_name, field_type_str)) = field.trim().split_once(":") else {
                return Err(MyDatabaseError::InvalidCommandFormat("CREATE"));
            };
            let field_type = match field_type_str.trim() {
                "Bool" => ValueType::Bool,
                "String" => ValueType::String,
                "Int" => ValueType::Int,
                "Float" => ValueType::Float,
                _ => return Err(MyDatabaseError::InvalidFieldType),
            };
            fields.insert(field_name.trim().to_string(), field_type);
        };

        Ok(CreateTableCmd::<'b> {
            db: context_db,
            name: name.trim().to_string(),
            key_name: key_name.trim().to_string(),
            fields,
        })
    }
}

#[derive(Debug)]
pub struct InsertRecordCmd<'a> {
    table: AnyTableRef<'a>,
    values: HashMap<String, Value>
}
impl<'b> Command<'b> for InsertRecordCmd<'b> {
    fn execute(&self) -> Result<(), MyDatabaseError> {
        todo!();
    }
    fn parse_input(input: &str, context_db: &'b AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((values, table_name)) = input.split_once("INTO") else {
            return Err(MyDatabaseError::InvalidCommandFormat("INSERT"));
        };
        let table = context_db.get_table_by_name(table_name.trim())?;
        let mut values_map: HashMap<String, Value> = HashMap::new();
        for value_pair in values.trim().split(",") {
            let Some((field_name, value_str)) = value_pair.trim().split_once(":") else {
                return Err(MyDatabaseError::InvalidCommandFormat("INSERT"));
            };
            let Some(field_type) = table.get_type_for_name(field_name.trim()) else {
                return Err(MyDatabaseError::InvalidFieldName);
            };
            let value = field_type.get_value(value_str.trim())?;
            if values_map.insert(field_name.trim().to_string(), value).is_some() {
                return Err(MyDatabaseError::DuplicateColumnName);
            }
        };
        Ok(InsertRecordCmd {
            table,
            values: values_map,
        })
    }
}

#[derive(Debug)]
pub struct DeleteRecordCmd<'a> {
    table: AnyTableRef<'a>,
    key_as_string: String,
}
impl<'b> Command<'b> for DeleteRecordCmd<'b> {
    fn execute(&self) -> Result<(), MyDatabaseError> {
        todo!();
    }
    fn parse_input(input: &str, context_db: &'b AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((key_str, table_name)) = input.split_once("FROM") else {
            return Err(MyDatabaseError::InvalidCommandFormat("DELETE"));
        };
        let table = context_db.get_table_by_name(table_name.trim())?;
        Ok(DeleteRecordCmd {
            table,
            key_as_string: key_str.trim().to_string(),
        })
    }
}

#[derive(Debug)]
pub struct SelectCmd<'a> {
    table: AnyTableRef<'a>,
    values_to_select: Vec<String>,
    condition: Option<String>,
}

impl<'b> Command<'b> for SelectCmd<'b> {
    fn execute(&self) -> Result<(), MyDatabaseError> {
        todo!();
    }
    fn parse_input(input: &str, context_db: &'b AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((fields, rest)) = input.split_once("FROM") else {
            return Err(MyDatabaseError::InvalidCommandFormat("SELECT"));
        };
        let (table_name, condition) = if let Some((table_part, cond_part)) = rest.trim().split_once("WHERE") {
            (table_part.trim(), Some(cond_part.trim().to_string()))
        } else {
            (rest.trim(), None)
        };
        let table = context_db.get_table_by_name(table_name)?;
        let mut values_to_select: Vec<String> = Vec::new();
        if (fields.trim() == "*") {
            values_to_select = table.get_all_columns();
            return Ok(SelectCmd {
                table,
                values_to_select,
                condition,
            });
        }
        for field in fields.trim().split(",") {
            values_to_select.push(field.trim().to_string());
        }
        Ok(SelectCmd {
            table,
            values_to_select,
            condition,
        })
    }
}