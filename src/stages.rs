use crate::{
    game::Game,
    types::{Card, PlayerChoice},
};

pub struct GameInPlay {
    pub game: Game,
    stage: Stage,
    pub balance: u32,
    original_bet: Option<u32>,
}

impl GameInPlay {
    pub fn new(game: Game, balance: u32) -> Self {
        Self {
            game,
            stage: Stage::Start,
            balance,
            original_bet: None,
        }
    }

    pub fn advance(&mut self) -> Option<InputNeeded> {
        if !self.game.has_started() {
            self.stage = Stage::Start;
            return Some(InputNeeded::Bet);
        }

        if matches!(self.stage, Stage::HandOver) {
            self.stage = Stage::AwaitBet;
            return Some(InputNeeded::Bet);
        }

        if (self.game.dealer_has_blackjack() || self.game.player_has_blackjack())
            && !matches!(self.stage, Stage::Payout(_))
        {
            self.game.reveal_dealer_hand();
            self.stage = Stage::Payout(vec![]);
            return None; // No input needed, just show results
        }

        if matches!(self.stage, Stage::DealerTurn) {
            let cards = if !self.game.all_player_hands_busted() {
                self.game.play_dealer_hand()
            } else {
                vec![]
            };
            self.stage = Stage::Payout(cards);
            return None; // No input needed, dealer has played
        }

        if matches!(self.stage, Stage::Payout(_)) {
            let payout = self.game.player_payout();
            self.balance += payout;
            self.stage = Stage::HandOver;
            return Some(InputNeeded::HandOver);
        }

        // Player has made every input they can for this round
        if !self.game.player_can_play() {
            // Reveal the dealer's cards
            self.game.reveal_dealer_hand();

            self.stage = Stage::DealerTurn;
            return None; // No input needed, dealer's turn
        }

        // Player can't play and the dealer has not yet played and we're not ready for payouts
        if !self.game.player_can_play()
            && !self.game.all_player_hands_busted()
            && !matches!(self.stage, Stage::Payout(_))
        {
            println!("Second if statement dealer play");
            let cards = self.game.play_dealer_hand();
            // Dealer has played and the player has no more actions
            self.stage = Stage::Payout(cards);
            return None;
        }

        Some(InputNeeded::Choice)
    }

    pub fn player_move(&mut self, choice: PlayerChoice) {
        if matches!(choice, PlayerChoice::Double | PlayerChoice::Split) {
            self.original_bet = Some(self.original_bet.unwrap() * 2);
        }
        self.game.take_turn(choice, &mut self.balance);

        self.stage = Stage::CheckWinConditions; // Check if the player has blackjack or if the dealer needs to play
    }

    pub fn bet(&mut self, bet: u32) {
        assert!(
            self.stage.bet_needed(),
            "Bet is not needed in the current stage."
        );
        self.original_bet = Some(bet);

        match self.stage {
            Stage::Start => {
                self.game.start_game(bet, &mut self.balance);
            }
            Stage::AwaitBet => {
                self.game.new_turn(bet, &mut self.balance);
            }
            _ => unreachable!(),
        }
        self.stage = Stage::CheckWinConditions; // Dealer or player could have blackjack
    }

    pub fn new_table(&mut self) {
        self.game = Game::new(self.game.config.clone());
        self.balance = self.balance;
        self.stage = Stage::Start;
        self.original_bet = None;
    }

    pub fn terminate(&mut self) {
        self.stage = Stage::Exiting;
    }
}

impl std::fmt::Display for GameInPlay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.stage {
            Stage::Start => {
                writeln!(f, "Welcome to Blackjack! Please place your initial bet.")
            }
            Stage::AwaitBet => {
                writeln!(f, "{}", self.game)?;
                writeln!(f, "Balance: ${}", self.balance)?;
                writeln!(f, "Please place your bet for the next hand.")
            }
            Stage::Exiting => {
                writeln!(
                    f,
                    "Thank you for playing! Your final balance is: ${}",
                    self.balance
                )
            }
            Stage::Payout(cards) => {
                writeln!(f, "{}", self.game)?;
                writeln!(f, "Balance: ${}", self.balance)?;
                for card in cards {
                    writeln!(f, "Dealer draws: {}", card)?;
                }
                if self.game.dealer_hand.is_bust() {
                    writeln!(f, "Dealer busts! You win this round.")?;
                }
                Ok(())
            }
            Stage::DealerTurn => {
                writeln!(f, "{}", self.game)?;
                writeln!(f, "Balance: ${}", self.balance)?;
                writeln!(f, "Dealer's turn is in progress. Please wait...")
            }
            Stage::CheckWinConditions => {
                writeln!(f, "{}", self.game)?;
                writeln!(f, "Balance: ${}", self.balance)
            }
            Stage::HandOver => {
                writeln!(f, "{}", self.game)?;
                writeln!(f, "Balance: ${}", self.balance)?;
                let payout = self.game.player_payout();
                let total_bet = self.game.player_total_bet();
                if payout > total_bet {
                    writeln!(
                        f,
                        "You won ${}! Your current balance is: ${}",
                        payout - total_bet,
                        self.balance
                    )?;
                } else if payout < total_bet {
                    writeln!(
                        f,
                        "You lost your bet of ${}. Your current balance is: ${}",
                        total_bet, self.balance
                    )?;
                } else {
                    writeln!(
                        f,
                        "It's a push! Your bet of ${} is returned to you. Current balance: ${}",
                        total_bet, self.balance
                    )?;
                }

                writeln!(f, "Hand over. Please wait for the next round.")
            }
        }
    }
}

pub enum Stage {
    /// The game hasn't made the initial bet yet
    Start,
    /// The game is waiting for the player to place a bet after a hand has already been played
    AwaitBet,
    /// The player requests to exit the game
    Exiting,
    /// Game has concluded and payouts are next
    ///
    /// Cards are what the dealer has drawn
    Payout(Vec<Card>),
    /// The player cannot play anymore, dealer's turn
    DealerTurn,

    CheckWinConditions,
    HandOver,
}

impl Stage {
    pub fn bet_needed(&self) -> bool {
        matches!(self, Stage::AwaitBet | Stage::Start)
    }
}

pub enum InputNeeded {
    Bet,
    Choice,
    HandOver,
}
