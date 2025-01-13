pub struct GameConfig {
    pub reserve_decks: usize,
    pub dealer_hits_soft_17: bool,
    pub player_can_double_after_split: bool,
    pub player_can_hit_after_double: bool,
    pub player_splits: usize,
}