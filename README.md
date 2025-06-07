# Blackjack Simulator

Play blackjack through the command line

To play normally and interact and place bets run

`cargo run`

## Playing Strategies

* Basic `cargo run -- -b`

* Counting `cargo run -- -c`

The running count and true count of the deck is track by the game

## Betting Strategies

Betting strategies only change for the counting strategy. Basic strategy uses a constant bet of $50

Counting strategy has 3 betting strategies. Each betting strategy leaves the table if the true count is negative.

 * Simple: Bet scales linearly as true count rises `cargo run -- -c`

 * Conservative: Bet scales linearly but never bets more than 2% of balance `cargo run -- -cc`

 * [Kelly Bet](https://en.wikipedia.org/wiki/Kelly_criterion): Probablistically optimally betting strategy. `cargo run -- -ca`

