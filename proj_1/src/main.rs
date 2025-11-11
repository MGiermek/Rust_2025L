use std::io;

use proj_1::models::{commands::{AnyCommand, Command}, db_structure::*};
fn main() {
    let check = 28i64.equals(&28i64);
    println!("Equality check: {}", check);

    let string1 = String::from("abc");
    let check2 = string1.equals(&String::from("abc"));
    println!("String equality check: {}", check2);

    let a: &str = "xyz";
    println!("{} is a string", a.parse::<String>().unwrap());

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

        // match AnyCommand::parse_input(input.as_str(), &mut context_db) {
        //     Ok(cmd) => {
        //         match cmd.execute(&mut executed_commands) {
        //             Ok(_) => println!("Command executed successfully"),
        //             Err(e) => println!("Error executing command: {}", e),
        //         }
        //     }
        //     Err(e) => println!("Error parsing command: {}", e),
        // }
        // println!("{:?}", context_db);
        println!("-----------------------------------");
    }
}
