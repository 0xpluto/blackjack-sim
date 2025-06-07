use std::io::{self, Write};

use crate::types::{PlayerChoice, PlayerChoices};

pub fn get_player_bet() -> u32 {
    loop {
        print!("Enter your bet (or 0 to quit): $");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim().parse::<u32>() {
            Ok(bet) if bet > 0 => return bet,
            Ok(0) => return 0,
            _ => {
                println!("Please enter a valid bet amount (positive number).");
                continue;
            }
        }
    }
}

pub fn get_player_choice(choices: PlayerChoices) -> PlayerChoice {
    loop {
        println!();
        print_player_choices(choices);
        
        // io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match PlayerChoice::parse_choice(&input, choices) {
            Ok(choice) => {
                return choice;
            }
            Err(e) => {
                println!("{}", e);
                println!("Please try again.");
            }
        }
    }
}

fn print_player_choices(choices: PlayerChoices) {
    println!("Available actions:");
    if choices.contains(PlayerChoices::HIT) {
        println!("  [H] Hit");
    }
    if choices.contains(PlayerChoices::STAND) {
        println!("  [S] Stand");
    }
    if choices.contains(PlayerChoices::DOUBLE) {
        println!("  [D] Double Down");
    }
    if choices.contains(PlayerChoices::SPLIT) {
        println!("  [P] Split");
    }
    if choices.contains(PlayerChoices::SURRENDER) {
        println!("  [R] Surrender");
    }
}