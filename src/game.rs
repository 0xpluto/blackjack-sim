use crate::{config::GameConfig, error::Error, types::{Card, Deck, Hand, HandResult, PlayerChoice, PlayerChoices}};

#[derive(Default)]
pub struct Game {
    /// Cards in the shoe
    reserves: Deck,
    /// Dealer's hand
    dealer_hand: Hand,
    /// Dealer is showing all their cards
    dealer_showing_all: bool,

    /// Player's hands
    player_hands: Vec<Hand>,
    /// Index of the current player's hand being played
    /// This is used to track which hand the player is currently playing if the player has split their hand
    current_hand: usize,
    /// The amount the player has bet for the current game
    player_bet: u32,

    /// Indicates if the shoe needs to be shuffled
    shoe_needs_shuffling: bool,

    config: GameConfig,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let mut this = Self {
            config,
            ..Default::default()
        };
        this.reset_game_state();

        this
    }

    pub fn start_game(&mut self, player_wager: u32) {
        self.reset_game_state();

        self.player_bet = player_wager;

        // Deal initial hands
        self.deal_starting_hands();
    }

    pub fn new_turn(&mut self, player_wager: u32) {
        self.player_bet = player_wager;
        self.current_hand = 0;

        self.deal_starting_hands();
    }

    pub fn player_balance_change(&self) -> i32 {
        let mut total_winnings = 0i32;
        for (i, _hand) in self.player_hands.iter().enumerate() {
            match self.player_wins(i) {
                HandResult::Blackjack => total_winnings += self.config.payout_odds.winning_amount(self.player_bet) as i32,
                HandResult::Win => total_winnings += self.player_bet as i32,
                HandResult::Push => {},
                HandResult::Lose => total_winnings -= self.player_bet as i32, // No winnings for a loss
                HandResult::NotFinished => {} // Game not finished, no winnings yet
            }
        }
        total_winnings
    }

    fn player_wins(&self, hand: usize) -> HandResult {
        let player_hand = &self.player_hands[hand];
        let dealer_hand = &self.dealer_hand;
        let player_has_blackjack = player_hand.is_blackjack();
        let dealer_has_blackjack = dealer_hand.is_blackjack();
        let player_bust = player_hand.is_bust();
        let dealer_bust = dealer_hand.is_bust();

        match (player_has_blackjack, dealer_has_blackjack, player_bust, dealer_bust) {
            // Both player and dealer have blackjack
            (true, true, _, _) => HandResult::Push,
            // Player has blackjack, dealer does not
            (true, false, _, _) => HandResult::Blackjack,
            // Dealer has blackjack, player does not
            (false, true, _, _) => HandResult::Lose,
            // Player busts
            (_, _, true, _) => HandResult::Lose,
            // Dealer busts
            (_, _, _, true) => HandResult::Win,
            // Neither busts or has blackjack
            _ => {
                let player_value = player_hand.value();
                let dealer_value = dealer_hand.value();
                if player_value > dealer_value {
                    HandResult::Win
                } else if player_value < dealer_value {
                    HandResult::Lose
                } else {
                    HandResult::Push
                }
            },
        }
    }

    // pub fn post_turn_advancement(&mut self) {
    //     // Check if the player has any hands left to play
    //     if self.player_hands.is_empty() {
    //         return false;
    //     }

    //     // If the current hand is bust, move to the next hand
    //     if self.player_hands[self.current_hand].is_bust() {
    //         if self.current_hand + 1 < self.player_hands.len() {
    //             self.current_hand += 1;
    //             return true; // Player has another hand to play
    //         } else {
    //             return false; // No more hands left
    //         }
    //     }

    //     true // Player can continue with the current hand
    // }

    pub fn player_can_play(&self) -> bool {
        !self.player_choices().is_empty() || (self.player_hands.len() == 1 && self.player_hands[0].is_blackjack())
    }

    /// Returns true if the player's turn is over
    pub fn take_turn(&mut self, choice: PlayerChoice) -> Result<(), Error> {
        match choice {
            PlayerChoice::Stand => {
                self.current_hand += 1; // Move to the next hand
            }
            PlayerChoice::Hit => {
                // Draw a card
                let hand = &mut self.player_hands[self.current_hand];
                hand.push(self.reserves.cards.pop().unwrap());

                if hand.is_bust() {
                    println!("Hand is bust: {}", hand);
                    self.current_hand += 1; // Move to the next hand
                }
            }
            PlayerChoice::Double => {
                // Double the bet, draw a card, and stand
            }
            PlayerChoice::Split => {
                // Split the hand into two hands
                if !self.config.player_can_split(&self.player_hands) {
                    return Err(Error::CannotSplit);
                }
                let hand = self.player_hands.pop().unwrap();
                let (new_hand1, new_hand2) = hand.split();

                self.player_hands.push(new_hand1);
                self.player_hands.push(new_hand2);
            }
            PlayerChoice::Surrender => {
                // Handle surrender logic
                // This could mean the player loses half their bet
                self.player_bet /= 2;
                self.current_hand += 1; // Move to the next hand
            }
        }

        Ok(())
    }

    pub fn all_player_hands_busted(&self) -> bool {
        for hand in &self.player_hands {
            if !hand.is_bust() {
                return false; // At least one hand is not bust
            }
        }
        true // All hands are bust
    }

    pub fn play_dealer_hand(&mut self) {
        while self.config.dealer_should_hit(&self.dealer_hand) && self.dealer_can_hit() {
            let card = self.pop_card();
            println!("Dealer draws: {}", card);
            self.dealer_hand.push(card);
            if self.dealer_hand.is_bust() {
                println!("Dealer busts!");
                break; // Dealer busts, no more actions needed
            }
        }
        self.dealer_hand.hide_card = false; // Dealer reveals all cards after playing
    }
    pub fn reveal_dealer_hand(&mut self) {
        self.dealer_hand.hide_card = false; // Reveal dealer's hand
    }
    fn dealer_can_hit(&self) -> bool {
        !self.dealer_hand.is_bust()
    }

    pub fn dealer_up_card(&self) -> Card {
        self.dealer_hand.cards.get(1).cloned().unwrap()
    }
    pub fn dealer_down_card(&self) -> Card {
        self.dealer_hand.cards.first().cloned().unwrap()
    }

    fn pop_card(&mut self) -> Card {
        let (card, reshuffle) = self.reserves.draw();
        self.shoe_needs_shuffling |= reshuffle;

        card
    }

    /// Set the game back to default with no hands drawn and a fresh shoe
    fn reset_game_state(&mut self) {
        self.reserves = Deck::create_shoe(self.config.reserve_decks as usize);
        self.dealer_hand = Hand::new(true);
        self.player_hands.clear();
        self.current_hand = 0;
        self.player_bet = 0;
        self.shoe_needs_shuffling = false;
    }

    fn deal_starting_hands(&mut self) {
        (self.dealer_hand, self.shoe_needs_shuffling) = self.reserves.deal_hand(2, true);
        let (player_hand, reshuffle) = self.reserves.deal_hand(2, false);
        self.shoe_needs_shuffling |= reshuffle;
        self.player_hands.clear();
        self.current_hand = 0; // Reset current hand index
        self.player_hands.push(player_hand);
    }

    pub fn player_choices(&self) -> PlayerChoices {
        let mut choices = PlayerChoices::empty();

        let current_hand = &self.player_hands.get(self.current_hand);
        if current_hand.is_none() {
            return choices; // No current hand to play
        }
        let current_hand = current_hand.unwrap();

        // Always allow hit and stand
        choices.insert(PlayerChoices::HIT);
        choices.insert(PlayerChoices::STAND);

        if self.config.player_can_split(&self.player_hands) {
            choices.insert(PlayerChoices::SPLIT);
        }
        if self.config.player_can_double_down(&current_hand) {
            choices.insert(PlayerChoices::DOUBLE);
        }
        if self.config.player_can_surrender(&self.player_hands) {
            choices.insert(PlayerChoices::SURRENDER);
        }

        choices
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== BLACKJACK GAME ===")?;
        writeln!(f)?;
        
        // Dealer's hand
        writeln!(f, "DEALER:")?;
        writeln!(f, "  {}", self.dealer_hand)?;
        writeln!(f)?;
        
        // Player's hands
        if self.player_hands.len() == 1 {
            writeln!(f, "PLAYER (Bet: ${}):", self.player_bet)?;
            writeln!(f, "  {}", self.player_hands[0])?;
        } else {
            writeln!(f, "PLAYER HANDS (Bet: ${} each):", self.player_bet)?;
            for (i, hand) in self.player_hands.iter().enumerate() {
                let marker = if i == self.current_hand { " <- CURRENT" } else { "" };
                writeln!(f, "  Hand {}: {}{}", i + 1, hand, marker)?;
            }
        }
        writeln!(f)?;
        
        // Game status
        writeln!(f, "Cards remaining: {}", self.reserves.cards.len())?;
        if self.shoe_needs_shuffling {
            writeln!(f, "⚠️  SHUFFLE NEEDED")?;
        }
        
        Ok(())
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game {{ dealer_hand: {:?}, player_hands: {:?}, current_hand: {}, player_bet: {}, shoe_needs_shuffling: {} }}",
               self.dealer_hand, self.player_hands, self.current_hand, self.player_bet, self.shoe_needs_shuffling)
    }
}