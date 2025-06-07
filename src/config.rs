use crate::types::Hand;


pub struct GameConfig {
    /// Number of decks in the shoe
    pub reserve_decks: usize,

    /// Dealer rules for hitting on soft 17
    pub dealer_rules: DealerRules,
    /// Dealer checks for blackjack
    pub dealer_checks_for_blackjack: bool,

    /// Payout odds for blackjack
    pub payout_odds: PayoutOdds,

    // Double Down

    /// Doubling down rules for the player
    pub doubling_down_rules: DoublingDownRules,
    /// Whether the player can double down after splitting
    pub player_can_double_after_split: bool,

    // Splitting

    /// Maximum number of splits a player can make
    pub player_splits: usize,
    /// Whether the player can resplit aces
    pub player_can_resplit_aces: bool,
    /// Whether the player can hit split aces
    pub player_can_hit_split_aces: bool,

    // Surrender
    /// Surrender rules for the player
    pub surrender_rules: SurrenderRules,
}

impl GameConfig {
    pub fn dealer_should_hit(&self, dealer_hand: &Hand) -> bool {
        let soft_17 = dealer_hand.is_soft() && dealer_hand.value() == 17;

        match self.dealer_rules {
            DealerRules::StandOnSoft17 => dealer_hand.value() < 17 || soft_17,
            DealerRules::HitOnSoft17 => dealer_hand.value() < 17,
        }
    }
    pub fn player_can_split(&self, hands: &[Hand], current_hand: usize) -> bool {
        if hands.len() >= self.player_splits {
            return false; // Cannot split more than allowed
        }
        if hands.is_empty() {
            return false; // No hands to split
        }
        let last_hand = &hands[current_hand];
        last_hand.can_split() // Check if the last hand can be split
    }

    pub fn player_can_double_down(&self, hands: &[Hand], current_hand: usize) -> bool {
        let hand = &hands[current_hand];
        if hand.cards.len() != 2 {
            return false; // Can only double down on two cards
        }
        if hands.len() > 1 && !self.player_can_double_after_split {
            return false; // Cannot double down after splitting if not allowed
        }
        match self.doubling_down_rules {
            DoublingDownRules::DoubleAny => true, // Can double down on any two cards
            DoublingDownRules::DoubleOnlyOn9To11 => {
                let value = hand.value();
                value == 9 || value == 10 || value == 11
            }
        }
    }
    pub fn player_can_surrender(&self, hands: &[Hand]) -> bool {
        if hands.is_empty() {
            return false; // No hands to surrender
        }
        match self.surrender_rules {
            SurrenderRules::NoSurrender => false, // Surrender not allowed
            SurrenderRules::EarlySurrender => true, // Can surrender at any time
            SurrenderRules::LateSurrender => true, // Can surrender after dealer checks for blackjack
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            // Standard 6-deck shoe
            reserve_decks: 6,
            
            // More common house rule (slightly favors house)
            dealer_rules: DealerRules::StandOnSoft17,
            
            // Standard peek rule for player protection
            dealer_checks_for_blackjack: true,
            
            // Traditional blackjack payout
            payout_odds: PayoutOdds::ThreeToTwo,
            
            // Liberal doubling rules
            doubling_down_rules: DoublingDownRules::DoubleAny,
            player_can_double_after_split: true,
            
            // Standard splitting rules
            player_splits: 3, // Can create up to 4 hands total
            player_can_resplit_aces: false, // Most casinos don't allow this
            player_can_hit_split_aces: false, // Very rare to allow this
            
            // Late surrender is fairly common
            surrender_rules: SurrenderRules::LateSurrender,
        }
    }
}

pub enum PayoutOdds {
    /// Pays $15 for a $10 bet
    ThreeToTwo,
    /// Pays $12 for a $10 bet
    SixToFive,
    /// Pays $10 for a $10 bet
    EvenMoney,
}

impl PayoutOdds {
    pub fn winning_amount(&self, bet: u32) -> u32 {
        match self {
            PayoutOdds::ThreeToTwo => (bet * 3) / 2,
            PayoutOdds::SixToFive => (bet * 6) / 5,
            PayoutOdds::EvenMoney => bet,
        }
    }
}

pub enum DealerRules {
    /// Dealer stands on soft 17
    StandOnSoft17,
    /// Dealer hits on soft 17
    HitOnSoft17,
}

pub enum DoublingDownRules {
    /// Player can double down on any two cards
    DoubleAny,
    /// Player can only double down on 9, 10, or 11
    DoubleOnlyOn9To11,
}

pub enum SurrenderRules {
    /// Player can surrender at any time
    EarlySurrender,
    /// Player can only surrender after the dealer checks for blackjack and does not have it
    LateSurrender,
    /// Player cannot surrender
    NoSurrender,
}
