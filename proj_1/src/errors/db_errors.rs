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

    #[error("Keys do not match the table structure")]
    KeysMismatch,

    #[error("Record with the given key already exists")]
    RecordAlreadyExists,

    #[error("Specified key not found in table")]
    KeyNotFound,

    #[error("IO Error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error parsing command: {0}")]
    CommandParseError(Box<MyDatabaseError>),

    #[error("Error executing command: {0}")]
    CommandExecuteError(Box<MyDatabaseError>),

    #[error("Error parsing where clause: {0}")]
    InvalidWhereClauseFormat(String),

    #[error("Wrongly parsed clause: {0}")]
    WronglyParsedClause(String),

    #[error("Cannot determine which value is bigger for bools or strings with numbers")]
    CannotCompareValues,

    #[error("Cannot perform AND/OR operation on non-boolean values")]
    InvalidLogicalOperation,

    #[error("Cannot perform mathematical operation on non-numeric values")]
    InvalidMathOperation,

    #[error("Cannot divide by zero")]
    DivisionByZero,
}