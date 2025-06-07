use blackjack_sim::{config::GameConfig, game::Game, input::{get_player_bet, get_player_choice}, types::PlayerChoice};

fn main() {
    clear_screen();
    
    let config = GameConfig::default();
    let mut game = Game::new(config);

    let mut balance = 1000;
    
    // Start a new game with a player wager
    let mut bet = get_player_bet();
    
    game.start_game(bet, &mut balance);

    loop {
        clear_screen();
        if bet == 0 {
            println!("Thank you for playing! Goodbye!");
            println!("Your final balance is: ${}", balance);
            break;
        }
        if game.dealer_has_blackjack() {
            println!("Dealer has a blackjack! You lose your bet of ${}", bet);
            game.reveal_dealer_hand();
            println!("{}", game);

            if new_turn(&mut game, &mut balance, &mut bet) {
                break;
            }
            continue; // Restart the loop for a new turn
        }
        
        if game.player_has_blackjack() {
            println!("You have a blackjack! You win ${}!", game.player_payout());
            game.reveal_dealer_hand();
            println!("{}", game);
            
            balance += game.player_payout();
            if new_turn(&mut game, &mut balance, &mut bet) {
                break; // Exit the loop if the player chooses to quit
            }
            continue; // Restart the loop for a new turn
        }
        
        println!("{}", game);
        
        if !game.player_can_play() {
            clear_screen();
            println!("Dealer's turn...");
            let dealer_revealed_card = game.dealer_down_card();
            println!("Dealer flips their down card: {}", dealer_revealed_card);
            game.reveal_dealer_hand();
            println!("{}", game);

            if !game.all_player_hands_busted() {
                clear_screen();
                let dealer_revealed_card = game.dealer_down_card();
                println!("Dealer flips their down card: {}", dealer_revealed_card);
                game.play_dealer_hand();
                println!("{}", game);
            }

            let payout = game.player_payout();
            if payout > bet {
                println!("You won ${}!", payout); // TODO: if the player doubled down then it will say you won even on a push
            } else if payout < bet {
                println!("You lost");
            } else {
                println!("It's a push, no winnings this round.");
            }
            balance += payout;
            
            // Check for end of game conditions, payouts, etc.
            if new_turn(&mut game, &mut balance, &mut bet) {
                break; // Exit the loop if the player chooses to quit
            }
            continue; // Restart the loop for a new turn
        };
        

        println!("Balance: ${}", balance);

        let player_choice = get_player_choice(game.player_choices());
        if matches!(player_choice, PlayerChoice::Double | PlayerChoice::Split) {
            bet *= 2; // Double the bet for double down or split
        }

        game.take_turn(player_choice, &mut balance);
    }
    
    // Game loop can be implemented here
    // For example, you can take player actions and update the game state accordingly.
    
    // Placeholder for game loop
    println!("{}", game);
    
}

fn clear_screen() {
    // This function clears the console screen.
    // It works on both Windows and Unix-like systems.
    if cfg!(target_os = "windows") {
        print!("{}[2J", 27 as char); // ANSI escape code for clearing the screen
    } else {
        print!("\x1B[2J\x1B[1;1H"); // ANSI escape code for clearing the screen and moving cursor to top left
    }
    std::io::Write::flush(&mut std::io::stdout()).unwrap(); // Ensure the command is executed immediately
}

fn new_turn(game: &mut Game, balance: &mut u32, orig_bet: &mut u32) -> bool {
    println!("Balance: ${}", balance);

    let bet = get_player_bet();
    *orig_bet = bet;
    if bet == 0 {
        println!("Thank you for playing! Goodbye!");
        println!("Your final balance is: ${}", balance);
        return true;
    }
    game.new_turn(bet, balance);
    false
}
