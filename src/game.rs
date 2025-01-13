use crate::{config::GameConfig, types::{Deck, Hand, PlayerChoice}};

pub struct Game {
    reserves: Deck,
    dealer_hand: Hand,
    player_hands: Vec<Hand>,
    current_hand: usize,
    player_bet: u32,

    config: GameConfig,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let mut reserves = Deck::multiple(config.reserve_decks as usize);

        let hands = reserves.deal_hands(2);
        let dealer_hand = hands[0].clone();
        let player_hands = vec![hands[1].clone()];

        Self {
            reserves,
            dealer_hand,
            player_hands,
            current_hand: 0,
            player_bet: 0,
            config,
        }
    }

    pub fn take_turn(&mut self, choice: PlayerChoice) {
        match choice {
            PlayerChoice::Stand => {
                // Do nothing

            }
            PlayerChoice::Hit => {
                // Draw a card
                let hand = &mut self.player_hands[self.current_hand];
                hand.push(self.reserves.cards.pop().unwrap());

            }
            PlayerChoice::Double => {
                // Double the bet, draw a card, and stand
            }
            PlayerChoice::Split => {
                // Split the hand into two hands
                let hand = self.player_hands.pop().unwrap();
                let (new_hand1, new_hand2) = hand.split();

                self.player_hands.push(new_hand1);
                self.player_hands.push(new_hand2);
            }
        }
    }

}