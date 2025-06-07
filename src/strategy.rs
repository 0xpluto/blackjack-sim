use crate::{input::{get_player_bet, get_player_choice, wait_for_player_input}, types::{Card, CardFace, Hand, PlayerChoice, PlayerChoices}};

const BASE_BET: u32 = 50; // Base bet size for basic strategy

pub enum PlayMode {
    Interactive,
    Basic,
    Counting,
    CountingConservative,
    CountingAdvanced,
}

impl PlayMode {
    pub fn from_args(args: &[String]) -> Self {
        if args.contains(&String::from("-b")) {
            PlayMode::Basic
        } else if args.contains(&String::from("-c")) {
            PlayMode::Counting
        } else if args.contains(&String::from("-cc")) {
            PlayMode::CountingConservative
        } else if args.contains(&String::from("-ca")) {
            PlayMode::CountingAdvanced
        } else {
            PlayMode::Interactive
        }
    }
    pub fn bet(&self, balance: u32, true_count: isize) -> u32 {
        match self {
            PlayMode::Interactive => get_player_bet(),
            PlayMode::Basic => 100, // Fixed bet for basic strategy
            PlayMode::Counting => simple_counting_bet(true_count),
            PlayMode::CountingConservative => counting_bet_size(balance, true_count),
            PlayMode::CountingAdvanced => kelly_bet_size(balance, true_count),
        }
    }
    pub fn choice(&self, choices: PlayerChoices, hand: &Hand, dealer_card: &Card, true_count: isize) -> PlayerChoice {
        match self {
            PlayMode::Interactive => get_player_choice(choices),
            PlayMode::Basic => {
                let choice = BasicStrategy::choice(hand, dealer_card, choices);
                println!("Basic strategy suggests: {}", choice);
                wait_for_player_input(true);
                choice
            }
            PlayMode::Counting | PlayMode::CountingAdvanced | PlayMode::CountingConservative => {
                let choice= CountingStrategy::choice(hand, dealer_card, choices, true_count);
                println!("Counting strategy suggests: {}", choice);
                wait_for_player_input(true);
                choice
            }
        }
    }
    pub fn wait_for_player_input(&self) {
        if !matches!(self, PlayMode::Interactive) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            println!("\nPress Enter to continue...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
        }
    }
}

pub struct BasicStrategy;

impl BasicStrategy {

    pub fn choice(hand: &Hand, dealer_card: &Card, choices: PlayerChoices) -> PlayerChoice {
        let value = hand.value();
        let splitable_card = if choices.contains(PlayerChoices::SPLIT) {
            hand.splitable_card()
        } else {
            None
        };
        let dealer_value = dealer_card.face_value();
        let is_soft = hand.is_soft();

        use CardFace::*;
        match (dealer_value, value, is_soft, splitable_card) {

            // Splits

            (_, _, _, Some(Ace)) => {
                PlayerChoice::Split // Always split aces
            }
            (_, _, _, Some(Face(_))) => {
                PlayerChoice::Stand // Never split face cards
            }
            (_, _, _, Some(Number(10))) => {
                PlayerChoice::Stand // Never split 10s
            }
            (d, _, _, Some(Number(9))) => {
                if d == 7 || d >= 10 {
                    PlayerChoice::Stand // Stand on 9s against dealer 7, 10, or Ace
                } else {
                    PlayerChoice::Split // Split 9s against dealer 2-6, 8-9
                }
            }
            (d, _, _, Some(Number(8))) => {
                if d >= 10 {
                    PlayerChoice::Hit // Hit on 8s against dealer 10, or Ace
                } else {
                    PlayerChoice::Split // Split 8s against dealer 2-9
                }
            }
            (d, _, _, Some(Number(7))) => {
                if d >= 8 {
                    PlayerChoice::Hit // Stand on 7s against dealer 8, 9, 10, or Ace
                } else {
                    PlayerChoice::Split // Split 7s against dealer 2-6
                }
            }
            (d, _, _, Some(Number(6))) => {
                if d >= 7 {
                    PlayerChoice::Hit // Stand on 6s against dealer 7, 8, 9, 10, or Ace
                } else {
                    PlayerChoice::Split // Split 6s against dealer 2-6
                }
            }
            (d, _, _, Some(Number(5))) => {
                if d <= 9 {
                    PlayerChoice::Double // Double on 5s against dealer 2-9
                } else {
                    PlayerChoice::Hit // Hit on 5s against dealer 10 or Ace
                }
            }
            (d, _, _, Some(Number(4))) => {
                if d == 5 || d == 6 {
                    PlayerChoice::Split // Double on 4s against dealer 5-6
                } else {
                    PlayerChoice::Hit // Hit on 4s against dealer 2-4, 7-10, or Ace
                }
            }
            (d, _, _, Some(Number(x))) if x <= 3 => {
                if d >= 8 {
                    PlayerChoice::Hit // Hit 3s/2s against dealer 8-10, or Ace
                } else {
                    PlayerChoice::Split // Split 3s/2s against dealer 2-7
                }
            }
            // Unreachable splits
            (_, _, _, Some(Number(x))) if !(2..=10).contains(&x) => {
                unreachable!("Splitable card not in range 2-10: {}", x)
            }


            // Soft totals

            (_, p, true, _) if p >= 19 => {
                PlayerChoice::Stand // Stand on soft 19 or higher
            }
            (d, 18, true, _) => {
                if d >= 9 {
                    PlayerChoice::Hit
                } else if d == 7 || d == 8 || d == 2 {
                    PlayerChoice::Stand // Stand on soft 18 against dealer 7 or 8
                } else {
                    PlayerChoice::Double // Double on soft 18 against dealer 3-6
                }
            }
            (d, 17, true, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (3..=6).contains(&d) {
                    PlayerChoice::Double // Double on soft 17 against dealer 3-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, p, true, _) if (15..=16).contains(&p) => {
                if choices.contains(PlayerChoices::DOUBLE) && (4..=6).contains(&d){
                    PlayerChoice::Double // Double on soft 16 against dealer 3-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, p, true, _) if (13..=14).contains(&p) => {
                if choices.contains(PlayerChoices::DOUBLE) && (5..=6).contains(&d) {
                    PlayerChoice::Double // Double on soft 14 against dealer 4-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            // Unreachable soft totals
            (_, p, true, _) if p <= 12 => {
                unreachable!("Soft total below 13")
            }

            // Hard totals

            (_, 17..=u8::MAX, _, _) => {
                PlayerChoice::Stand // Stand on hard 17 or higher
            }
            (d, 13..=16, _, _) => {
                if d <= 6 {
                    PlayerChoice::Stand // Stand on hard 13-16 against dealer 2-6
                } else {
                    PlayerChoice::Hit // Hit on hard 13-16 against dealer 7-10, or Ace
                }
            }
            (d, 12, _, _) => {
                if (4..=6).contains(&d) {
                    PlayerChoice::Stand // Stand on hard 12 against dealer 4-6
                } else {
                    PlayerChoice::Hit // Hit on hard 12 against dealer 2-3, 7-10, or Ace
                }
            }
            (d, 11, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (2..=10).contains(&d) {
                    PlayerChoice::Double // Double on hard 11 against dealer 2-10
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, 10, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (2..=9).contains(&d) {
                    PlayerChoice::Double // Double on hard 10 against dealer 2-9
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (d, 9, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (3..=6).contains(&d) {
                    PlayerChoice::Double // Double on hard 9 against dealer 3-6
                } else {
                    PlayerChoice::Hit // Hit if double not allowed
                }
            }
            (_, 0_u8..=8_u8, _, _) => {
                PlayerChoice::Hit // Hit on hard 8 or lower
            }
        }
    }
}

pub struct CountingStrategy;

impl CountingStrategy {
    pub fn choice(hand: &Hand, dealer_card: &Card, choices: PlayerChoices, true_count: isize) -> PlayerChoice {
        let value = hand.value();
        let splitable_card = if choices.contains(PlayerChoices::SPLIT) {
            hand.splitable_card()
        } else {
            None
        };
        let dealer_value = dealer_card.face_value();
        let is_soft = hand.is_soft();

        use CardFace::*;
        match (dealer_value, value, is_soft, splitable_card.clone(), true_count) {

            // COUNT-ADJUSTED SPLITS

            (_, _, _, Some(Ace), _) => {
                PlayerChoice::Split // Always split aces
            }
            (_, _, _, Some(Face(_)), _) => {
                PlayerChoice::Stand // Never split face cards
            }
            // COUNT ADJUSTMENT: Split 10s in very high counts
            (6, _, _, Some(Number(10)), tc) if tc >= 4 => {
                PlayerChoice::Split // Split 10s vs 6 when TC >= +4
            }
            (5, _, _, Some(Number(10)), tc) if tc >= 5 => {
                PlayerChoice::Split // Split 10s vs 5 when TC >= +5
            }
            (_, _, _, Some(Number(10)), _) => {
                PlayerChoice::Stand // Otherwise never split 10s
            }
            (d, _, _, Some(Number(9)), _) => {
                if d == 7 || d >= 10 {
                    PlayerChoice::Stand
                } else {
                    PlayerChoice::Split
                }
            }
            (d, _, _, Some(Number(8)), _) => {
                if d >= 10 {
                    PlayerChoice::Hit
                } else {
                    PlayerChoice::Split
                }
            }
            (d, _, _, Some(Number(7)), _) => {
                if d >= 8 {
                    PlayerChoice::Hit
                } else {
                    PlayerChoice::Split
                }
            }
            (d, _, _, Some(Number(6)), _) => {
                if d >= 7 {
                    PlayerChoice::Hit
                } else {
                    PlayerChoice::Split
                }
            }
            (d, _, _, Some(Number(5)), _) => {
                if d <= 9 {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, _, _, Some(Number(4)), _) => {
                if d == 5 || d == 6 {
                    PlayerChoice::Split
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, _, _, Some(Number(x)), _) if x <= 3 => {
                if d >= 8 {
                    PlayerChoice::Hit
                } else {
                    PlayerChoice::Split
                }
            }
            (_, _, _, Some(Number(x)), _) if !(2..=10).contains(&x) => {
                unreachable!("Splitable card not in range 2-10: {}", x)
            }

            // COUNT-ADJUSTED SOFT TOTALS

            (_, p, true, _, _) if p >= 19 => {
                PlayerChoice::Stand
            }
            // COUNT ADJUSTMENT: A,8 vs 6 - double in positive counts
            (6, 18, true, _, tc) if tc >= 1 && choices.contains(PlayerChoices::DOUBLE) => {
                PlayerChoice::Double // Double A,8 vs 6 when TC >= +1
            }
            (d, 18, true, _, _) => {
                if d >= 9 {
                    PlayerChoice::Hit
                } else if d == 7 || d == 8 || d == 2 {
                    PlayerChoice::Stand
                } else {
                    PlayerChoice::Double
                }
            }
            (d, 17, true, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (3..=6).contains(&d) {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, p, true, _, _) if (15..=16).contains(&p) => {
                if choices.contains(PlayerChoices::DOUBLE) && (4..=6).contains(&d) {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, p, true, _, _) if (13..=14).contains(&p) => {
                if choices.contains(PlayerChoices::DOUBLE) && (5..=6).contains(&d) {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (_, p, true, _, _) if p <= 12 => {
                unreachable!("Soft total below 13")
            }

            // COUNT-ADJUSTED HARD TOTALS

            (_, 17..=u8::MAX, _, _, _) => {
                PlayerChoice::Stand
            }
            // COUNT ADJUSTMENTS: Stand on stiff hands in positive counts
            (10, 16, false, _, tc) if tc >= 0 => {
                PlayerChoice::Stand // Stand 16 vs 10 when TC >= 0
            }
            (9, 16, false, _, tc) if tc >= 1 => {
                PlayerChoice::Stand // Stand 16 vs 9 when TC >= +1
            }
            (2, 13, false, _, tc) if tc >= 1 => {
                PlayerChoice::Stand // Stand 13 vs 2 when TC >= +1
            }
            (3, 12, false, _, tc) if tc >= 2 => {
                PlayerChoice::Stand // Stand 12 vs 3 when TC >= +2
            }
            (2, 12, false, _, tc) if tc >= 3 => {
                PlayerChoice::Stand // Stand 12 vs 2 when TC >= +3
            }
            (d, 13..=16, false, _, _) => {
                if d <= 6 {
                    PlayerChoice::Stand
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, 12, false, _, _) => {
                if (4..=6).contains(&d) {
                    PlayerChoice::Stand
                } else {
                    PlayerChoice::Hit
                }
            }
            // COUNT ADJUSTMENTS: More aggressive doubling in positive counts
            (11, 11, false, _, tc) if tc >= 1 && choices.contains(PlayerChoices::DOUBLE) => {
                PlayerChoice::Double // Double 11 vs A when TC >= +1
            }
            (2, 9, false, _, tc) if tc >= 1 && choices.contains(PlayerChoices::DOUBLE) => {
                PlayerChoice::Double // Double 9 vs 2 when TC >= +1
            }
            (7, 9, false, _, tc) if tc >= 3 && choices.contains(PlayerChoices::DOUBLE) => {
                PlayerChoice::Double // Double 9 vs 7 when TC >= +3
            }
            (10, 10, false, _, tc) if tc >= 4 && choices.contains(PlayerChoices::DOUBLE) => {
                PlayerChoice::Double // Double 10 vs 10 when TC >= +4
            }
            (11, 10, false, _, tc) if tc >= 4 && choices.contains(PlayerChoices::DOUBLE) => {
                PlayerChoice::Double // Double 10 vs A when TC >= +4
            }
            (d, 11, false, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (2..=10).contains(&d) {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, 10, false, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (2..=9).contains(&d) {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (d, 9, false, _, _) => {
                if choices.contains(PlayerChoices::DOUBLE) && (3..=6).contains(&d) {
                    PlayerChoice::Double
                } else {
                    PlayerChoice::Hit
                }
            }
            (_, 0_u8..=8_u8, _, _, _) => {
                PlayerChoice::Hit
            }
            // Unreachable hard totals
            _ => {
                unreachable!("Unexpected values: dealer={}, value={}, soft={}, splitable_card={:?}, true_count={}", 
                    dealer_value, value, is_soft, splitable_card, true_count)
            }
        }
    }
}

fn counting_bet_size(balance: u32, true_count: isize) -> u32 {
    // Conservative bankroll management: never bet more than 2% of balance
    let max_bet = (balance as f32 * 0.02) as u32;
    
    // Base betting unit (minimum bet)
    let base_unit = BASE_BET; // $25 base unit
    
    // Don't bet more than we can afford
    let max_affordable = max_bet.max(base_unit);
    
    let bet_size = match true_count {
        // Negative counts: leave the table
        tc if tc <= -1 => 0, // Leave when count is negative
        
        // Neutral count: minimum bet
        0 => base_unit, // Minimum bet when neutral
        
        // Positive counts: scale bet with advantage
        1 => base_unit * 2,     // 2 units at TC +1
        2 => base_unit * 3,     // 3 units at TC +2  
        3 => base_unit * 4,     // 4 units at TC +3
        4 => base_unit * 6,     // 6 units at TC +4
        5 => base_unit * 8,     // 8 units at TC +5
        
        // Very high counts: maximum aggression but capped
        tc if tc >= 6 => base_unit * 10, // 10 units at TC +6 and above
        
        // This shouldn't happen but handle it
        _ => base_unit,
    };
    
    // Never bet more than we can afford (unless leaving table)
    if bet_size == 0 { 0 } else { bet_size.min(max_affordable) }
}

// Alternative: More aggressive Kelly Criterion approach
fn kelly_bet_size(balance: u32, true_count: isize) -> u32 {
    let base_unit = 50; // Larger base unit for Kelly
    
    // Leave table when count is negative
    if true_count <= -1 {
        return 0;
    }
    
    // Kelly formula approximation for blackjack
    let player_advantage = match true_count {
        0 => -0.005, // House edge at neutral
        1 => 0.005,  // +0.5% edge
        2 => 0.010,  // +1.0% edge
        3 => 0.015,  // +1.5% edge
        4 => 0.020,  // +2.0% edge
        5 => 0.025,  // +2.5% edge
        tc if tc >= 6 => 0.030, // +3.0% edge cap
        _ => 0.0,
    };
    
    if player_advantage <= 0.0 {
        return base_unit; // Minimum bet when no advantage
    }
    
    // Kelly fraction
    let kelly_fraction = player_advantage / 2.0;
    let kelly_bet = (balance as f32 * kelly_fraction) as u32;
    
    // Cap between base unit and reasonable maximum
    let max_bet = balance / 20; // Never more than 5% of bankroll
    kelly_bet.clamp(base_unit, max_bet)
}

// Simplified approach for beginners
fn simple_counting_bet(true_count: isize) -> u32 {
    let base_bet = BASE_BET;
    
    match true_count {
        tc if tc <= -1 => 0,                 // Leave table when negative
        0 => base_bet,                       // 1 unit at neutral
        1 => base_bet * 2,                   // 2 units  
        2 => base_bet * 4,                   // 4 units
        3 => base_bet * 6,                   // 6 units
        tc if tc >= 4 => base_bet * 8,       // 8 units max
        _ => base_bet,
    }
}