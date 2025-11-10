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

    let context_db = AnyDatabase::IntDatabase(Database::<i64>::new());

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let command = AnyCommand::parse_input(input.as_str(), &context_db);
        match command {
            Ok(cmd) => println!("Parsed command: {:?}", cmd),
            Err(e) => println!("Error parsing command: {:?}", e),
        }
        println!("-----------------------------------");
    }
}
