use blackjack_sim::{
    config::GameConfig,
    game::Game,
    stages::{GameInPlay, InputNeeded}, strategy::PlayMode,
};

fn main() {
    clear_screen();
    let args: Vec<String> = std::env::args().collect();

    let play_mode = PlayMode::from_args(&args);

    let config = GameConfig::default();
    let game = Game::new(config);
    let balance = 10000;

    let mut game = GameInPlay::new(game, balance);
    let mut hands_played = 0;

    loop {
        match game.advance() {
            Some(InputNeeded::Bet) => {
                println!("{}", game);
                println!("Hands played: {}", hands_played);
                let mut bet = play_mode.bet(game.balance, game.game.true_count());
                println!("{}", game);
                println!("Hands played: {}", hands_played);
                if bet == 0 {
                    println!("Going to new table...");
                    game.new_table();
                    bet = play_mode.bet(game.balance, game.game.true_count());
                }
                game.bet(bet);
                println!("{}", game);
            }
            Some(InputNeeded::Choice) => {
                println!("{}", game);
                println!("Hands played: {}", hands_played);
                let choices = game.game.player_choices();
                let hand = game.game.player_current_hand();
                let dealer_card = game.game.dealer_up_card();
                let true_count = game.game.true_count();
                let choice = play_mode.choice(choices, &hand, &dealer_card, true_count);
                
                game.player_move(choice);
            }
            Some(InputNeeded::HandOver) => {
                hands_played += 1;
                println!("{}", game);
                println!("Hands played: {}", hands_played);
                play_mode.wait_for_player_input();
            }
            None => {
                println!("{}", game);
                println!("Hands played: {}", hands_played);
                play_mode.wait_for_player_input();
            }
        }
        clear_screen();
    }
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
