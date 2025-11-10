use proj_1::models::db_structure::*;
fn main() {
    let check = 28i64.equals(&28i64);
    println!("Equality check: {}", check);

    let string1 = String::from("abc");
    let check2 = string1.equals(&String::from("abc"));
    println!("String equality check: {}", check2);
}
