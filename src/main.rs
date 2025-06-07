use blackjack_sim::{
    config::GameConfig,
    game::Game,
    input::{get_player_bet, get_player_choice, wait_for_player_input},
    stages::{GameInPlay, InputNeeded},
};

fn main() {
    clear_screen();

    let config = GameConfig::default();
    let game = Game::new(config);
    let balance = 1000;

    let mut game = GameInPlay::new(game, balance);

    loop {
        match game.advance() {
            Some(InputNeeded::Bet) => {
                println!("{}", game);
                let bet = get_player_bet();
                println!("{}", game);
                if bet == 0 {
                    game.terminate();
                }
                game.bet(bet);
                println!("{}", game);
            }
            Some(InputNeeded::Choice) => {
                println!("{}", game);
                let choice = get_player_choice(game.game.player_choices());
                game.player_move(choice);
            }
            None => {
                println!("{}", game);
                wait_for_player_input();
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
