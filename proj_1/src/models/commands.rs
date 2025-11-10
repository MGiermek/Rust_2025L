trait Command {
    fn execute(&self);
}

struct CreateTableCmd {
    db: AnyDatabase,
    keyName: String,
    fieldsString: String
}

struct InsertRecordCmd {
    table: AnyTable,
    valuesString: String
}

struct DeleteRecordCmd {
    table: AnyTable,
    keyAsString: String,
}

struct SelectCmd {
    table: AnyTable,
    valuesToSelect: String,
    condition: Option<String>,
}