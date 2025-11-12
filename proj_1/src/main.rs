use std::io;

use proj_1::models::{commands::AnyCommand, db_structure::*};
fn main() {
    let check_str = "INSERT k1 = 5, k2 = \"Rybki i akwarium\", k3 = false, k4 = 77.99 INTO abc";
    println!("Spilitted: {:?}", proj_1::models::utilities::split_preserving_quote_insides(check_str, ' '));

    let mut context_db = AnyDatabase::IntDatabase(Database::<i64>::new());
    let mut executed_commands = Vec::<String>::new();
    println!("Give me COMMMAAAAAANDS");

    loop {
        let mut input = String::new();
        let Ok(_) = io::stdin().read_line(&mut input) else {
            println!("Failed to read line");
            continue;
        };

        match AnyCommand::create_and_execute(input.as_str(), &mut context_db, &mut executed_commands) {
            Ok(_) => println!("Command executed successfully"),
            Err(e) => println!("{}", e),
        }
        println!("-----------------------------------");
    }
}
