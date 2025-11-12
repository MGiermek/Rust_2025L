use crate::db_errors::MyDatabaseError;
use crate::models::db_structure::*;
use crate::models::utilities::{split_once_skipping_outside_quotes, split_preserving_quote_insides};
use crate::models::where_parsing::WhereClause;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write, BufRead};
use std::path::Path;
pub trait Command<'b> {
    fn execute(self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError>;
    fn parse_input<'a>(input: &'a str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized;
}
#[derive(Debug)]
pub enum AnyCommand<'b> {
    CreateTable(CreateTableCmd<'b>),
    InsertRecord(InsertRecordCmd<'b>),
    DeleteRecord(DeleteRecordCmd<'b>),
    Select(SelectCmd<'b>),
    SaveAs(SaveAsCmd),
    ReadFrom(ReadFromCmd<'b>),
}
impl<'b> AnyCommand<'b> {
    pub fn create_and_execute(input: &str, context_db: &'b mut AnyDatabase, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        match AnyCommand::parse_input(input, context_db) {
            Ok(cmd) => {
                if let Err(e) = cmd.execute(executed_commands) {
                    return Err(MyDatabaseError::CommandExecuteError(Box::new(e)));
                }
            }
            Err(e) => return Err(MyDatabaseError::CommandParseError(Box::new(e))),
        }
        Ok(())
    }
}
impl<'b> Command<'b> for AnyCommand<'b> {
    fn execute(self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        match self {
            AnyCommand::CreateTable(cmd) => cmd.execute(executed_commands),
            AnyCommand::InsertRecord(cmd) => cmd.execute(executed_commands),
            AnyCommand::DeleteRecord(cmd) => cmd.execute(executed_commands),
            AnyCommand::Select(cmd) => cmd.execute(executed_commands),
            AnyCommand::SaveAs(cmd) => cmd.execute(executed_commands),
            AnyCommand::ReadFrom(cmd) => cmd.execute(executed_commands),
        }
    }
    fn parse_input<'a>(input: &'a str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
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
            "SAVE_AS" => {
                let cmd = SaveAsCmd::parse_input(rest, context_db)?;
                Ok(AnyCommand::SaveAs(cmd))
            },
            "READ_FROM" => {
                let cmd = ReadFromCmd::parse_input(rest, context_db)?;
                Ok(AnyCommand::ReadFrom(cmd))
            },
            _ => Err(MyDatabaseError::InvalidCommandFormat("UNKNOWN")),
        }
    }
}

#[derive(Debug)]
pub struct CreateTableCmd<'a> {
    original_string: String,
    db: &'a mut AnyDatabase,
    name: String,
    key_name: String,
    fields: HashMap<String, ValueType>
}
impl<'b> Command<'b> for CreateTableCmd<'b> {
    fn execute(self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        match self.db.create_table(&self.name, &self.key_name, self.fields) {
            Ok(_) => {
                executed_commands.push(self.original_string);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn parse_input<'a>(input: &'a str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
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
            original_string: format!("CREATE {}", input),
            db: context_db,
            name: name.trim().to_string(),
            key_name: key_name.trim().to_string(),
            fields,
        })
    }
}

#[derive(Debug)]
pub struct InsertRecordCmd<'a> {
    original_string: String,
    table: AnyTableRef<'a>,
    values: HashMap<String, Value>
}
impl<'b> Command<'b> for InsertRecordCmd<'b> {
    fn execute(mut self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        match self.table.insert_values(self.values) {
            Ok(_) => {
                executed_commands.push(self.original_string);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn parse_input(input: &str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((values, table_name)) = input.split_once("INTO") else {
            return Err(MyDatabaseError::InvalidCommandFormat("INSERT"));
        };
        let table = context_db.get_table_by_name(table_name.trim())?;
        let mut values_map: HashMap<String, Value> = HashMap::new();
        for value_pair in split_preserving_quote_insides(values, ',') {
            let Some((field_name, value_str)) = split_once_skipping_outside_quotes(value_pair, '=') else {
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
            original_string: format!("INSERT {}", input),
            table,
            values: values_map,
        })
    }
}

#[derive(Debug)]
pub struct DeleteRecordCmd<'a> {
    original_string: String,
    table: AnyTableRef<'a>,
    key_as_string: String,
}
impl<'b> Command<'b> for DeleteRecordCmd<'b> {
    fn execute(mut self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        match self.table.delete_key(self.key_as_string) {
            Ok(_) => {
                executed_commands.push(self.original_string);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn parse_input(input: &str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        let Some((key_str, table_name)) = input.split_once("FROM") else {
            return Err(MyDatabaseError::InvalidCommandFormat("DELETE"));
        };
        let table = context_db.get_table_by_name(table_name.trim())?;
        Ok(DeleteRecordCmd {
            original_string: format!("DELETE {}", input),
            table,
            key_as_string: key_str.trim().to_string(),
        })
    }
}

#[derive(Debug)]
pub struct SelectCmd<'a> {
    original_string: String,
    table: AnyTableRef<'a>,
    values_to_select: Vec<String>,
    condition: Option<WhereClause>,
}

impl<'b> Command<'b> for SelectCmd<'b> {
    fn execute(mut self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        match self.table.select_and_display(&self.values_to_select, &self.condition) {
            Ok(_) => {
                executed_commands.push(self.original_string);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn parse_input(input: &str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
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
        if fields.trim() == "*" {
            values_to_select = table.get_all_columns();
        }
        else {
            for field in split_preserving_quote_insides(fields, ',') {
                values_to_select.push(field.trim().to_string());
            }
        }

        let condition = if let Some(cond_str) = condition {
            Some(WhereClause::create_from_string(cond_str, table.get_structure())?)
        } else {
            None
        };

        // println!("Where condition: {:?}", condition);
        
        Ok(SelectCmd {
            original_string: format!("SELECT {}", input),
            table,
            values_to_select,
            condition,
        })
    }
}

#[derive(Debug)]
pub struct SaveAsCmd {
    filename: String,
}
impl<'b> Command<'b> for SaveAsCmd {
    fn execute(self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        let path = Path::new(&self.filename);
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(MyDatabaseError::IoError(e));
            }
        }
        let mut file: fs::File = match fs::File::create(&self.filename) {
            Ok(f) => f,
            Err(e) => return Err(MyDatabaseError::IoError(e)),
        };
        for command in executed_commands {
            if let Err(e) = writeln!(file, "{}", command) {
                return Err(MyDatabaseError::IoError(e));
            }
        }
        println!("Commands saved to {}", self.filename);
        Ok(())
    }
    fn parse_input(input: &str, _context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        Ok(SaveAsCmd {
            filename: input.trim().to_string(),
        })
    }
}

#[derive(Debug)]
pub struct ReadFromCmd<'a> {
    db: &'a mut AnyDatabase,
    filename: String,
}
impl<'b> Command<'b> for ReadFromCmd<'b> {
    fn execute(self, executed_commands: &mut Vec<String>) -> Result<(), MyDatabaseError> {
        let path = Path::new(&self.filename);
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(MyDatabaseError::IoError(e)),
        };
        let reader = io::BufReader::new(file);
        println!("Reading and executing commands below:\n");
        for line_result in reader.lines() {
            match line_result {
                Ok(l) => {
                    println!("{}", l);
                    AnyCommand::create_and_execute(l.as_str(), self.db, executed_commands)?;
                }
                Err(e) => return Err(MyDatabaseError::IoError(e)),
            };
        }
        Ok(())
    }
    fn parse_input(input: &str, context_db: &'b mut AnyDatabase) -> Result<Self, MyDatabaseError> where Self: Sized {
        Ok(ReadFromCmd {
            db: context_db,
            filename: input.trim().to_string(),
        })
    }
}