use blackjack_sim::{config::GameConfig, game::Game, input::{get_player_bet, get_player_choice}, types::HandResult};

fn main() {
    // This is the entry point of the application.
    // You can initialize your game here, set up configurations, and start the game loop.
    
    // flush stdout to ensure prompt messages are displayed immediately
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    println!("Welcome to the Blackjack Game!");
    
    // Example of initializing a game with default configuration
    let config = GameConfig::default();
    let mut game = Game::new(config);

    let mut balance = 1000;
    
    // Start a new game with a player wager
    let bet = get_player_bet();
    
    game.start_game(bet);

    loop {
        if bet == 0 {
            println!("Thank you for playing! Goodbye!");
            println!("Your final balance is: ${}", balance);
            break;
        }
        println!("{}", game);
        
        if !game.player_can_play() {
            println!("Dealer's turn...");
            let dealer_revealed_card = game.dealer_down_card();
            println!("Dealer flips their down card: {}", dealer_revealed_card);

            if !game.all_player_hands_busted() {
                game.play_dealer_hand();
                println!("{}", game);
            } else {
                game.reveal_dealer_hand();
            }

            let payout = game.player_balance_change();
            if payout > 0 {
                println!("You won ${}!", payout);
            } else if payout < 0 {
                println!("You lost ${}.", -payout);
            } else {
                println!("It's a push, no winnings this round.");
            }
            balance = (balance as i32 + payout) as u32;

            println!("Your balance is: ${}", balance);

            println!("{}", game);
            
            // Check for end of game conditions, payouts, etc.
            let bet = get_player_bet();
            if bet == 0 {
                println!("Thank you for playing! Goodbye!");
                println!("Your final balance is: ${}", balance);
                break;
            }
            game.new_turn(bet);
            continue; // Restart the loop for a new turn
        };

        let player_choice = get_player_choice(game.player_choices());

        if let Err(e) = game.take_turn(player_choice) {
            println!("Error: {}", e);
            continue; // Skip to the next iteration if there was an error
        }
    }
    
    // Game loop can be implemented here
    // For example, you can take player actions and update the game state accordingly.
    
    // Placeholder for game loop
    println!("{}", game);
    
}
