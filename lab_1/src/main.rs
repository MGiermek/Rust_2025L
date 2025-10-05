use std::io;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let ret_val = loop {
        println!("Podaj liczbę:");
        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Failed to readline");

        let mut x: u64 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => break true
        };

        if x == 0 {
            break false
        }

        x += rand::rng().random_range(0..=5);
        println!("{x}");

        let tab = fill_tab(x);
        println!("{:?}", tab);
        let collatz_tab = check_collatz(tab);
        println!("{:?}", collatz_tab);
        
        match write_to_file(collatz_tab) {
            Ok(_) => {}
            Err(_) => break true
        }
    };

    match ret_val {
        true => println!("Program ended with error"),
        false => println!("Program ended successfully")
    }

    let (number, takes) = match_two_random_numbers();
    println!("The guessed number is {number}! It took {takes} takes to guess it");
}

fn fill_tab(x: u64) -> [u64; 10] {
    let mut ret_tab: [u64; 10] = [0;10];
    let mut tmp = x;
    
    for power in &mut ret_tab {
        *power = tmp;
        tmp *= x;
    }
    ret_tab
}

fn check_collatz(tab: [u64; 10]) -> [bool; 10] {
    let mut ret_tab: [bool; 10] = [false; 10];
    for i in 0..tab.len() {
        let mut x: u64 = tab[i];
        for _j in 0..100 {
            if x == 1 {
                ret_tab[i] = true;
                break;
            }
            else if x.is_multiple_of(2) {
                x /= 2;
            }
            else {
                x = 3*x + 1;
            }
        }
    }
    ret_tab
}

fn write_to_file(tab: [bool; 10]) -> std::io::Result<()> {
    let mut file = File::create("xyz.txt").expect("Couldn't create file xyz.txt");
    write!(file, "{:?}", tab).expect("Couldn't write to file");
    Ok(())
}

fn match_two_random_numbers() -> (i8, u32) {
    let mut take_counter: u32 = 0;
    let mut first_number: i8;
    'outer_loop: loop {
        first_number = rand::rng().random_range(-10..=10);
        println!("New first number: {first_number}. Here are the guesses:");
        for _i in 0..5 {
            let guess: i8 = rand::rng().random_range(-10..=10);
            print!("{guess} ");
            take_counter += 1;
            if guess == first_number {
                println!();
                break 'outer_loop;
            }
        }
        println!();
    }
    (first_number, take_counter)
}