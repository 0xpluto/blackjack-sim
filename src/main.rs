use blackjack_sim::{
    config::GameConfig,
    game::Game,
    input::{get_player_bet, get_player_choice, wait_for_player_input},
    stages::{GameInPlay, InputNeeded}, strategy::BasicStrategy,
};

fn main() {
    clear_screen();
    let args: Vec<String> = std::env::args().collect();

    let use_basic = if args.contains(&String::from("-b")) {
        true
    } else {
        false
    };

    let config = GameConfig::default();
    let game = Game::new(config);
    let balance = 1000;

    let mut game = GameInPlay::new(game, balance);

    loop {
        match game.advance() {
            Some(InputNeeded::Bet) => {
                println!("{}", game);
                let bet = if use_basic {
                    100
                } else {
                    get_player_bet()
                };
                println!("{}", game);
                if bet == 0 {
                    game.terminate();
                }
                game.bet(bet);
                println!("{}", game);
            }
            Some(InputNeeded::Choice) => {
                println!("{}", game);
                let choices = game.game.player_choices();
                let choice = if use_basic {
                    let hand = game.game.player_current_hand();
                    let dealer_card = game.game.dealer_up_card();
                    let choice = BasicStrategy::choice(hand, &dealer_card, choices);
                    println!("Basic strategy suggests: {}", choice);
                    wait_for_player_input(use_basic);
                    choice
                } else {
                    get_player_choice(choices)
                };
                game.player_move(choice);
            }
            Some(InputNeeded::HandOver) => {
                println!("{}", game);
                wait_for_player_input(use_basic);
            }
            None => {
                println!("{}", game);
                wait_for_player_input(use_basic);
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
