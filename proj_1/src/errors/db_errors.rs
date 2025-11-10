use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyDatabaseError {
    #[error("Given key is not valid in this type of database")]
    InvalidKeyType,

    #[error("Invalid command format for command {0}")]
    InvalidCommandFormat(&'static str),

    #[error("Invalid field type specified")]
    InvalidFieldType,

    #[error("Invalid field name specified")]
    InvalidFieldName,

    #[error("Invalid field value for the specified type")]
    InvalidFieldValue,

    #[error("Column name provided more thatn once")]
    DuplicateColumnName,

    #[error("Table '{0}' not found in database")]
    TableNotFound(String),

    #[error("Table with name '{0}' already exists in database")]
    TableAlreadyExists(String),
}