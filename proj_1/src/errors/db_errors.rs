use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyDatabaseError {
    #[error("Given key is not valid in this type of database")]
    InvalidKeyType,
}