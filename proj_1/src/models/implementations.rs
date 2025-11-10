use crate::models::db_structure::*;
use crate::db_errors::MyDatabaseError;

impl DatabaseKey for i64{
    fn equals(&self, other: &Self) -> bool {
        self == other
    }
}
impl DatabaseKey for String {
    fn equals(&self, other: &Self) -> bool {
        self == other
    }
}


