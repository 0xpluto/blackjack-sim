use std::collections::HashMap;

use crate::{config::GameConfig, types::{Card, Deck, Hand, HandResult, PlayerChoice, PlayerChoices}};

#[derive(Default)]
pub struct Game {
    /// Cards in the shoe
    reserves: Deck,
    /// Dealer's hand
    pub dealer_hand: Hand,

    /// Player's hands
    player_hands: Vec<Hand>,
    /// Index of the current player's hand being played
    /// This is used to track which hand the player is currently playing if the player has split their hand
    current_hand: usize,

    /// The initial wager
    initial_wager: u32,
    /// The amount the player has bet on each hand
    player_bet: HashMap<usize, u32>,

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

    pub fn has_started(&self) -> bool {
        self.initial_wager != 0
    }

    pub fn start_game(&mut self, player_wager: u32, balance: &mut u32) {
        self.reset_game_state();

        assert!(player_wager > 0, "Player wager must be greater than zero");

        self.player_bet.insert(0, player_wager); // Store the wager for the first hand
        self.initial_wager = player_wager;
        *balance -= player_wager; // Deduct the wager from the player's balance

        // Deal initial hands
        self.deal_starting_hands();
    }

    pub fn new_turn(&mut self, player_wager: u32, balance: &mut u32) {
        self.player_bet.insert(0, player_wager); // Store the wager for the first hand
        self.initial_wager = player_wager;
        *balance -= player_wager; // Deduct the wager from the player's balance
        self.current_hand = 0;

        self.deal_starting_hands();
    }

    pub fn player_payout(&self) -> u32 {
        let mut total_winnings = 0;
        for (i, _hand) in self.player_hands.iter().enumerate() {
            let player_bet = *self.player_bet.get(&i).unwrap();
            match self.player_wins(i) {
                HandResult::Blackjack => total_winnings += self.config.payout_odds.winning_amount(player_bet) + player_bet, // Blackjack pays out at the configured odds plus the original bet
                HandResult::Win => total_winnings += player_bet * 2,
                HandResult::Push => total_winnings += player_bet, // Push means no loss, return the bet
                HandResult::Lose => {}, // No winnings for a loss
                HandResult::NotFinished => {} // Game not finished, no winnings yet
            }
        }
        total_winnings
    }

    pub fn player_total_bet(&self) -> u32 {
        self.player_bet.values().sum()
    }

    pub fn player_wins(&self, hand: usize) -> HandResult {
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

    pub fn player_can_play(&self) -> bool {
        if self.player_hands.len() == 1 && self.player_hands[0].is_blackjack() {
            return false; // Player has blackjack, no further actions needed
        }
        !self.player_choices().is_empty()
    }

    pub fn player_current_hand(&self) -> &Hand {
        self.player_hands.get(self.current_hand).unwrap()
    }

    /// Returns true if the player's turn is over
    pub fn take_turn(&mut self, choice: PlayerChoice, balance: &mut u32) {
        match choice {
            PlayerChoice::Stand => {
                self.current_hand += 1; // Move to the next hand
                if self.player_hands.get(self.current_hand).is_some() {
                    // We split and haven't dealt the next card yet
                    let card = self.pop_card();
                    let hand = &mut self.player_hands[self.current_hand];
                    hand.push(card);
                }
            }
            PlayerChoice::Hit => {
                // Draw a card
                let card = self.pop_card();
                let hand = &mut self.player_hands[self.current_hand];
                hand.push(card);

                if hand.is_bust() {
                    println!("Hand is bust: {}", hand);
                    self.current_hand += 1; // Move to the next hand
                }
            }
            PlayerChoice::Double => {
                // Double the bet, draw a card, and stand
                self.player_bet.insert(self.current_hand, self.initial_wager * 2); // Update the bet for the current hand
                *balance -= self.initial_wager; // Deduct the doubled bet from balance
                
                let card = self.pop_card();
                let hand = &mut self.player_hands[self.current_hand];
                hand.push(card);

                if hand.is_bust() {
                    println!("Hand is bust after doubling down: {}", hand);
                }
                self.current_hand += 1; // Move to the next hand after doubling down
                if self.player_hands.get(self.current_hand).is_some() {
                    // We split and haven't dealt the next card yet
                    let card = self.pop_card();
                    let hand = &mut self.player_hands[self.current_hand];
                    hand.push(card);
                }
            }
            PlayerChoice::Split => {
                self.player_bet.insert(self.current_hand + 1, self.initial_wager);
                *balance -= self.initial_wager; // Deduct the bet for the split hands
                
                // Split the hand into two hands
                let hand = self.player_hands.pop().unwrap();
                let (mut new_hand1, new_hand2) = hand.split();
                // Deal the next card to the first new hand
                let card1 = self.pop_card();
                new_hand1.push(card1);

                self.player_hands.push(new_hand1);
                self.player_hands.push(new_hand2);
            }
            PlayerChoice::Surrender => {
                // Handle surrender logic
                // This could mean the player loses half their bet
                self.player_bet.insert(self.current_hand, self.initial_wager / 2);

                self.current_hand += 1; // Move to the next hand
            }
        }
    }

    pub fn all_player_hands_busted(&self) -> bool {
        for hand in &self.player_hands {
            if !hand.is_bust() {
                return false; // At least one hand is not bust
            }
        }
        true // All hands are bust
    }

    pub fn play_dealer_hand(&mut self) -> Vec<Card> {
        let mut dealer_cards = Vec::new();
        while self.config.dealer_should_hit(&self.dealer_hand) && self.dealer_can_hit() {
            let card = self.pop_card();
            self.dealer_hand.push(card.clone());
            dealer_cards.push(card);
        }
        self.dealer_hand.hide_card = false; // Dealer reveals all cards after playing
        dealer_cards
    }
    pub fn reveal_dealer_hand(&mut self) {
        self.dealer_hand.hide_card = false; // Reveal dealer's hand
    }
    fn dealer_can_hit(&self) -> bool {
        !self.dealer_hand.is_bust()
    }
    /// Only reveals if the dealer has blackjack if config allows it
    pub fn dealer_has_blackjack(&self) -> bool {
        if !self.config.dealer_checks_for_blackjack && self.player_can_play() {
            return false;
        }
        self.dealer_hand.is_blackjack()
    }
    pub fn player_has_blackjack(&self) -> bool {
        if self.player_hands.is_empty() {
            return false; // No player hands to check
        }
        let current_hand = self.player_hands.get(self.current_hand);
        if current_hand.is_none() {
            return false; // No current hand to check
        }
        let current_hand = current_hand.unwrap();
        current_hand.is_blackjack()
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
        self.player_bet.clear();
        self.initial_wager = 0;
        self.shoe_needs_shuffling = false;
    }

    fn deal_starting_hands(&mut self) {
        if self.shoe_needs_shuffling {
            self.reserves = Deck::create_shoe(self.config.reserve_decks as usize);
            self.shoe_needs_shuffling = false; // Reset the flag after shuffling
        }
        let (dealer_hand, reshuffle) = self.reserves.deal_hand(2, true);
        self.shoe_needs_shuffling |= reshuffle;
        self.dealer_hand = dealer_hand;

        let (player_hand, reshuffle) = self.reserves.deal_hand(2, false);
        self.shoe_needs_shuffling |= reshuffle;
        self.player_hands.clear();
        self.current_hand = 0; // Reset current hand index
        self.player_hands.push(player_hand);
    }

    pub fn player_choices(&self) -> PlayerChoices {
        let mut choices = PlayerChoices::empty();

        if self.player_hands.get(self.current_hand).is_none() {
            return choices; // No current hand to play
        }

        // Always allow hit and stand
        choices.insert(PlayerChoices::HIT);
        choices.insert(PlayerChoices::STAND);

        if self.config.player_can_split(&self.player_hands, self.current_hand) {
            choices.insert(PlayerChoices::SPLIT);
        }
        if self.config.player_can_double_down(&self.player_hands, self.current_hand) {
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
        writeln!(f, "=== BLACKJACK ===")?;
        writeln!(f, "Wager: ${}", self.initial_wager)?;
        writeln!(f)?;
        
        // Dealer's hand
        writeln!(f, "DEALER:")?;
        if self.dealer_hand.dealer_show_card().is_blackjack_card() && !self.dealer_hand.is_blackjack() {
            writeln!(f, "  {} (Not blackjack)", self.dealer_hand)?;
        } else {
            writeln!(f, "  {}", self.dealer_hand)?;
        }
        writeln!(f)?;
        
        // Player's hands
        if self.player_hands.len() == 1 {
            writeln!(f, "PLAYER")?;
            writeln!(f, "  {}", self.player_hands[0])?;
        } else {
            writeln!(f, "PLAYER HANDS:")?;
            for (i, hand) in self.player_hands.iter().enumerate() {
                let wager = self.player_bet.get(&i).unwrap();
                let marker = if i == self.current_hand { " <- CURRENT" } else { "" };
                writeln!(f, "  ${} Hand {}: {}{}", wager, i + 1, hand, marker)?;
            }
        }
        writeln!(f)?;
        
        // Game status
        let cards_left = self.reserves.cards.len();
        let decks_left = (cards_left / 52) as isize;
        let running_count = self.reserves.count;
        let true_count = running_count / decks_left;
        writeln!(f, "Cards remaining: {}, Running Count: {}, True Count: {}, Decks left {}", cards_left, running_count, true_count, decks_left)?;
        if self.shoe_needs_shuffling {
            writeln!(f, "⚠️  SHUFFLE NEEDED")?;
        }
        
        Ok(())
    }
}
