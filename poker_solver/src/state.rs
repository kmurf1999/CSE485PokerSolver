//!
//! `GameState` is used to represent the state of a HUNL Texas Holdem' Poker game
//! `GameState` is meant to be immutable.  To transition between states you must call the `apply_action` function with a valid action.  This will return a new state
//!
use crate::action::{Action, ACTIONS};
use crate::betting_abstraction::BettingAbstraction;
use crate::card::Card;
use std::convert::TryFrom;
// use crate::constants::*;
use crate::round::BettingRound;
// use rand::prelude::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
use rust_poker::hand_range::HandRange;
use std::result::Result;
use thiserror::Error as ThisError;

const CHANCE_PLAYER: i8 = -1;
const TERMINAL_PLAYER: i8 = -2;

/// Represents the state of a single player in a HUNL Texas Holdem Game
#[derive(Debug, Copy, Clone)]
pub struct PlayerState {
    /// Amount of chips player has
    stack: u32,
    /// Amount of chips player is currently betting
    wager: u32,
    /// Player hole cards
    /// card is n in 0..52 where n = 4 * rank + suit
    cards: [Card; 2],
    /// Has the player folded
    has_folded: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            stack: 0,
            wager: 0,
            cards: [52; 2],
            has_folded: false,
        }
    }
}

impl PlayerState {
    /// Creates a player state with an initial stack size
    ///
    /// # Arguements
    ///
    /// * `stack` - The stacking stack size in chips
    pub fn new(stack: u32) -> PlayerState {
        PlayerState {
            stack,
            wager: 0,
            // 52 since its the first invalid card index
            cards: [52; 2],
            has_folded: false,
        }
    }
    /// Return stack size
    pub const fn stack(&self) -> u32 {
        self.stack
    }
    /// Return wager
    pub const fn wager(&self) -> u32 {
        self.wager
    }
    /// Return cards
    pub const fn cards(&self) -> &[Card; 2] {
        &self.cards
    }
    /// Return has folded
    pub const fn folded(&self) -> bool {
        self.has_folded
    }
}

#[derive(Debug, Copy, Clone)]
/// Options for creating inital game state
///
/// # Preflop Example
/// ```
/// use poker_solver::state::{GameState, GameStateOptions};
/// let options = GameStateOptions {
///   stacks: [10000, 2],
///   blinds: [10, 5],
///   pot: 0,
///   initial_board: [52; 5]
/// };
/// let state = GameState::init(options);
/// assert_eq!(state.pot(), 15);
/// ```
///
/// # Postflop Example
/// ```
/// use poker_solver::state::{GameState, GameStateOptions};
/// let options = GameStateOptions {
///   stacks: [10000, 2],
///   blinds: [10, 5],
///   pot: 100,
///   initial_board: [0, 1, 2, 52, 52]
/// };
/// let state = GameState::init(options);
/// assert_eq!(state.pot(), 100);
/// ```
pub struct GameStateOptions {
    /// Initial stack sizes for each player
    /// - **Note:** stacks must be greater than zero
    pub stacks: [u32; 2],
    /// The blinds for each player (Big blind comes first).
    /// - If Betting round is preflop, blinds will be applied by subtracting for each players stack and added to the pot.
    /// - The first blind is used to determine the min bet.
    pub blinds: [u32; 2],
    /// Initial pot size
    pub pot: u32,
    /// Initial board cards
    /// - 52 is used as a default value to represent a card that has yet to be drawn
    pub initial_board: [Card; 5],
}

#[derive(Debug, ThisError)]
pub enum GameStateError {
    #[error("invalid board card count")]
    InvalidBoard,
    #[error("invalid pot")]
    InvalidPot,
    #[error("invalid stack")]
    InvalidStacks,
}

/// Represents the current state of a HUNL Texas Holdem Game
///
/// State is *immutable*.  To transition between states, call `state.apply_action(action)`.  This will return a new, updated state.
///
#[derive(Debug)]
pub struct GameState {
    /// The current betting round
    round: BettingRound,
    /// The size of the pot in chips
    pot: u32,
    /// The state of each player
    players: [PlayerState; 2],
    /// The blinds for each player
    /// - big blind comes first
    blinds: [u32; 2],
    /// Index of current player
    /// - An index of `-1` represents the Chance Player.
    /// - An index of `-2` represents a Terminal State.
    current_player: i8,
    /// Community Cards
    /// `card = n = [0, 52)` where `n = 4 * rank + suit`
    board: [Card; 5],
    /// 64bit mask of used cards that is used for sampling
    used_cards_mask: u64,
    /// The game history represented as a string
    history: String,
}

impl Clone for GameState {
    fn clone(&self) -> GameState {
        GameState {
            round: self.round,
            pot: self.pot,
            players: self.players,
            blinds: self.blinds,
            current_player: self.current_player,
            board: self.board,
            used_cards_mask: self.used_cards_mask,
            history: self.history.clone(),
        }
    }
}

impl GameState {
    /// Create a game state object
    /// **Note:** Will panic if wagers are specified and game is not in preflop
    pub fn new(options: GameStateOptions) -> Result<GameState, GameStateError> {
        let mut board = [52; 5];
        let mut board_card_count = 0;
        let mut used_cards_mask = 0u64;
        let mut pot = options.pot;
        // copy board cards
        for i in 0..5 {
            if options.initial_board[i] >= 52 {
                break;
            }
            board_card_count += 1;
            board[i] = options.initial_board[i];
            used_cards_mask |= 1u64 << board[i];
        }
        let round = match board_card_count {
            0 => BettingRound::Preflop,
            3 => BettingRound::Flop,
            4 => BettingRound::Turn,
            5 => BettingRound::River,
            _ => {
                return Err(GameStateError::InvalidBoard);
            }
        };
        let mut players = [PlayerState::default(); 2];
        for i in 0..2 {
            players[i].stack = options.stacks[i];
            if players[i].stack == 0 {
                return Err(GameStateError::InvalidStacks);
            }
        }
        if round == BettingRound::Preflop {
            for i in 0..2 {
                players[i].wager += std::cmp::min(options.blinds[i], players[i].stack);
                players[i].stack -= players[i].wager;
                pot += players[i].wager;
            }
        }
        Ok(GameState {
            round,
            pot,
            blinds: options.blinds,
            players,
            current_player: CHANCE_PLAYER,
            board,
            used_cards_mask,
            history: String::new(),
        })
    }
    /// Return true is this is a terminal state
    pub const fn is_terminal(&self) -> bool {
        self.current_player() == TERMINAL_PLAYER
    }
    /// Return true if this is an action node
    pub const fn is_action(&self) -> bool {
        self.current_player > CHANCE_PLAYER
    }
    /// Return true if this is a private chance node
    pub const fn is_chance(&self) -> bool {
        self.current_player == CHANCE_PLAYER
    }
    /// Is this the initial game state
    pub fn is_initial_state(&self) -> bool {
        self.history.is_empty()
    }
    /// Has a player folded
    pub fn player_folded(&self) -> bool {
        self.players[0].has_folded || self.players[1].has_folded
    }
    /// Is a player all in
    pub fn player_allin(&self) -> bool {
        (self.players[0].stack == 0) || (self.players[1].stack == 0)
    }
    /// Returns a list of valid actions given a player's betting abstraction
    pub fn valid_actions(&self, betting_abstraction: &BettingAbstraction) -> Vec<Action> {
        let mut actions = vec![];
        let our_stack = self.acting_player().stack;
        let our_stack_double = our_stack as f64;
        let pot_double = self.pot as f64;

        ACTIONS.iter().for_each(|action| {
            match action {
                Action::BetRaise(_) => {
                    // iterate over all bet sizes in this abstraction for this round
                    let is_bet = self.other_player().wager() == 0;
                    if is_bet {
                        for bet_size in &betting_abstraction.bet_sizes[usize::from(self.round)] {
                            let amt = (bet_size * pot_double) as u32;
                            let over_threshold = (betting_abstraction.all_in_threshold > 0f64)
                                && (amt as f64
                                    > (our_stack_double * betting_abstraction.all_in_threshold));
                            if over_threshold {
                                let action = Action::BetRaise(our_stack);
                                if self.is_action_valid(action) {
                                    actions.push(action);
                                }
                                break;
                            }
                            let action = Action::BetRaise(amt);
                            if self.is_action_valid(action) {
                                actions.push(action);
                            }
                        }
                    } else {
                        for raise_size in &betting_abstraction.raise_sizes[usize::from(self.round)]
                        {
                            let amt = (raise_size * pot_double) as u32;
                            let over_threshold = (betting_abstraction.all_in_threshold > 0f64)
                                && (amt as f64
                                    > (our_stack_double * betting_abstraction.all_in_threshold));
                            if over_threshold {
                                let action = Action::BetRaise(our_stack);
                                if self.is_action_valid(action) {
                                    actions.push(action);
                                }
                                break;
                            }
                            let action = Action::BetRaise(amt);
                            if self.is_action_valid(action) {
                                actions.push(action);
                            }
                        }
                    }
                }
                _ => {
                    if self.is_action_valid(*action) {
                        actions.push(*action);
                    }
                }
            }
        });

        actions
    }
    /// Returns true if the specified action is valid
    /// - **Note:** For chance actions, it does not consider conflicting cards or number of cards specified
    pub fn is_action_valid(&self, action: Action) -> bool {
        match action {
            Action::Fold => self.other_player().wager > self.acting_player().wager,
            Action::CheckCall => !self.is_chance(),
            Action::BetRaise(amt) => {
                if self.is_chance() {
                    return false;
                }
                let is_bet = self.other_player().wager == 0;
                if is_bet {
                    let min_bet = std::cmp::min(self.blinds[0], self.acting_player().stack);
                    let max_bet = self.acting_player().stack;
                    if amt >= min_bet && amt <= max_bet {
                        return true;
                    }
                } else {
                    let min_raise =
                        std::cmp::min(2 * self.other_player().wager, self.acting_player().stack);
                    let max_raise = self.acting_player().stack;
                    if amt >= min_raise && amt <= max_raise {
                        return true;
                    }
                }
                false
            }
            Action::Chance(_) => self.is_chance(),
        }
    }
    /// Applys an action an returns a new, updated game state
    /// - Assumes that the specified action is valid
    pub fn apply_action(&self, action: Action) -> GameState {
        let mut next_state = self.clone();
        match action {
            Action::BetRaise(amt) => {
                if next_state.other_player().wager > next_state.acting_player().wager {
                    next_state.history.push_str(&format!("R({})", amt));
                } else {
                    next_state.history.push_str(&format!("B({})", amt));
                }
                next_state.acting_player_mut().wager += amt;
                next_state.acting_player_mut().stack -= amt;
                next_state.pot += amt;
                next_state.current_player = 1 - next_state.current_player;
            }
            Action::Fold => {
                let wager_diff = next_state.other_player().wager - next_state.acting_player().wager;
                next_state.acting_player_mut().stack += wager_diff;
                next_state.acting_player_mut().has_folded = true;
                next_state.pot -= wager_diff;
                next_state.current_player = TERMINAL_PLAYER;
                next_state.history.push('F');
            }
            Action::CheckCall => {
                let is_call = next_state.other_player().wager > next_state.acting_player().wager;
                if is_call {
                    let to_call =
                        next_state.other_player().wager - next_state.acting_player().wager;
                    if to_call > next_state.acting_player().stack {
                        let diff = to_call - next_state.acting_player().stack;
                        next_state.pot += next_state.acting_player().stack;
                        next_state.pot -= diff;
                        next_state.acting_player_mut().wager +=
                            next_state.acting_player_mut().stack;
                        next_state.acting_player_mut().stack = 0;
                        next_state.other_player_mut().wager -= diff;
                        next_state.other_player_mut().stack += diff;
                    } else {
                        next_state.acting_player_mut().wager += to_call;
                        next_state.acting_player_mut().stack -= to_call;
                        next_state.pot += to_call;
                    }
                    // allow the big blind to bet preflop if other player just calls
                    if next_state.round == BettingRound::Preflop
                        && next_state.current_player > 0
                        && next_state.history == "D"
                    {
                        next_state.current_player = 0;
                    } else if next_state.round == BettingRound::River {
                        next_state.current_player = TERMINAL_PLAYER;
                    } else {
                        next_state.current_player = CHANCE_PLAYER;
                    }
                    next_state.players[0].wager = 0;
                    next_state.players[1].wager = 0;
                    next_state.history.push('C');
                } else {
                    // if first player checked, then the action is on the next player
                    // if the second player checked, then the action is on the chance player and the round is incremented
                    // preflop, the big blind is last to act
                    if next_state.current_player == 0 && next_state.round != BettingRound::Preflop {
                        next_state.current_player = 1 - next_state.current_player;
                    } else if next_state.round == BettingRound::River {
                        next_state.current_player = TERMINAL_PLAYER;
                    } else {
                        next_state.current_player = CHANCE_PLAYER;
                    }
                    next_state.history.push('X');
                }
            }
            Action::Chance(cards) => {
                // deal private cards to players
                if next_state.is_initial_state() {
                    let mut i = 0;
                    for player in 0..2 {
                        next_state.players[player].cards[0] = cards[i];
                        next_state.players[player].cards[1] = cards[i + 1];
                        next_state.used_cards_mask |= (1u64 << cards[i]) | (1u64 << cards[i + 1]);
                        i += 2;
                    }
                } else {
                    match next_state.round {
                        BettingRound::Preflop => {
                            next_state.board[..3].clone_from_slice(&cards[..3]);
                            for card in &next_state.board[..3] {
                                next_state.used_cards_mask |= 1u64 << card;
                            }
                        }
                        BettingRound::Flop => {
                            next_state.board[3] = cards[0];
                            next_state.used_cards_mask |= 1u64 << cards[0];
                        }
                        BettingRound::Turn => {
                            next_state.board[4] = cards[0];
                            next_state.used_cards_mask |= 1u64 << cards[0];
                        }
                        BettingRound::River => panic!("invalid round"),
                    };
                    next_state.round = BettingRound::try_from(usize::from(next_state.round) + 1)
                        .unwrap_or(BettingRound::River);
                }
                if next_state.player_allin() {
                    next_state.current_player = if next_state.round == BettingRound::River {
                        TERMINAL_PLAYER
                    } else {
                        CHANCE_PLAYER
                    };
                } else if next_state.players[0].wager > next_state.players[1].wager {
                    next_state.current_player = 1;
                } else {
                    next_state.current_player = 0;
                };
                next_state.history.push('D');
            }
        };
        next_state
    }
    /// Returns the default player action (i.e.) Check/Fold
    pub fn default_action(&self) -> Action {
        assert_ne!(self.current_player, CHANCE_PLAYER);
        if self.other_player().wager > 0 {
            Action::Fold
        } else {
            Action::CheckCall
        }
    }
    /// Returns the rewards for each player
    /// - Rewards are zero sum
    /// - **Note:** Can only be called on a terminal state
    pub fn rewards(&self) -> [f64; 2] {
        let mut payouts = [0f64; 2];
        // TODO unimplemented
        if self.player_folded() {
            for (player, payout) in payouts.iter_mut().enumerate() {
                if self.players[player].has_folded {
                    *payout = -0.5 * self.pot as f64;
                } else {
                    *payout = 0.5 * self.pot as f64;
                }
            }
            return payouts;
        }
        // evaluate player hands
        let mut scores: [u16; 2] = [0; 2];
        let mut board: Hand = Hand::default();
        for card in &self.board {
            board += CARDS[usize::from(*card)];
        }
        for (player, score) in scores.iter_mut().enumerate() {
            let mut player_hand = board;
            for card in &self.players[player].cards {
                player_hand += CARDS[usize::from(*card)];
            }
            *score = evaluate(&player_hand);
        }
        // calc payouts
        if scores[0] > scores[1] {
            payouts[0] = 0.5 * self.pot as f64;
            payouts[1] = -0.5 * self.pot as f64;
        }
        if scores[0] < scores[1] {
            payouts[0] = -0.5 * self.pot as f64;
            payouts[1] = 0.5 * self.pot as f64;
        }
        if scores[0] == scores[1] {
            payouts[0] = 0.0;
            payouts[1] = 0.0;
        }

        payouts
    }
    /// Returns the reward for a single player
    pub fn player_reward(&self, player: usize) -> f64 {
        self.rewards()[player]
    }
    /// Samples a private card dealing from 2 hand ranges and returns new state
    pub fn sample_private_chance_from_ranges<R: Rng>(
        &self,
        rng: &mut R,
        hand_ranges: &[HandRange],
    ) -> Action {
        assert_eq!(self.current_player, CHANCE_PLAYER);
        assert_eq!(hand_ranges.len(), 2);
        let mut used_cards_mask = self.used_cards_mask;
        let mut cards: [Card; 4] = [52; 4];
        // give players their cards
        for (player, hr) in hand_ranges.iter().enumerate() {
            loop {
                let combo = hr.hands.choose(rng).unwrap();
                let combo_mask = (1u64 << combo.0) | (1u64 << combo.1);
                if (combo_mask & used_cards_mask) == 0 {
                    used_cards_mask |= combo_mask;
                    cards[2 * player] = combo.0;
                    cards[2 * player + 1] = combo.1;
                    break;
                }
            }
        }
        Action::Chance(cards)
    }
    /// Samples a private card dealing
    pub fn sample_private_chance<R: Rng>(&self, rng: &mut R) -> Action {
        // give players their cards randomly
        let mut cards: [Card; 4] = [52; 4];
        let mut used_cards_mask = self.used_cards_mask;
        for player in 0..2 {
            for card in 0..2 {
                loop {
                    let c: Card = rng.gen_range(0, 52);
                    if ((1u64 << c) & used_cards_mask) == 0 {
                        cards[2 * player + card] = c;
                        used_cards_mask |= 1u64 << c;
                        break;
                    }
                }
            }
        }
        Action::Chance(cards)
    }
    /// Samples a public card dealing
    pub fn sample_public_chance<R: Rng>(&self, rng: &mut R) -> Action {
        assert_eq!(self.current_player, CHANCE_PLAYER);
        let mut cards: [Card; 4] = [52; 4];
        let mut used_cards_mask = self.used_cards_mask;
        match self.round {
            BettingRound::River => panic!("invalid round for public chance"),
            BettingRound::Preflop => {
                for card in &mut cards[0..3] {
                    loop {
                        let c: Card = rng.gen_range(0, 52);
                        if ((1u64 << c) & used_cards_mask) == 0 {
                            *card = c;
                            used_cards_mask |= 1u64 << c;
                            break;
                        }
                    }
                }
            }
            BettingRound::Flop => loop {
                let c: Card = rng.gen_range(0, 52);
                if ((1u64 << c) & used_cards_mask) == 0 {
                    cards[0] = c;
                    used_cards_mask |= 1u64 << c;
                    break;
                }
            },
            BettingRound::Turn => loop {
                let c: Card = rng.gen_range(0, 52);
                if ((1u64 << c) & used_cards_mask) == 0 {
                    cards[0] = c;
                    used_cards_mask |= 1u64 << c;
                    break;
                }
            },
        }
        Action::Chance(cards)
    }
    /// Returns the value of the pot
    pub const fn pot(&self) -> u32 {
        self.pot
    }
    /// Returns the history as a string for use as a key
    pub fn history_string(&self) -> &str {
        &self.history
    }
    /// Returns board cards
    pub const fn board(&self) -> &[Card; 5] {
        &self.board
    }
    /// Returns the current player index
    pub const fn current_player(&self) -> i8 {
        self.current_player
    }
    /// Returns the current round
    pub const fn round(&self) -> BettingRound {
        self.round
    }
    /// Returns reference to player at index
    pub fn player(&self, player: usize) -> &PlayerState {
        &self.players[player]
    }
    /// Returns a reference to the acting player
    /// - **Note:** should only be called on a player action node
    pub fn acting_player(&self) -> &PlayerState {
        assert_ne!(self.current_player, CHANCE_PLAYER);
        &self.players[self.current_player as usize]
    }
    fn acting_player_mut(&mut self) -> &mut PlayerState {
        assert_ne!(self.current_player, CHANCE_PLAYER);
        &mut self.players[self.current_player as usize]
    }
    /// Returns a reference to the player not acting
    /// - **Note:** should only be called on a player action node
    pub fn other_player(&self) -> &PlayerState {
        assert_ne!(self.current_player, CHANCE_PLAYER);
        &self.players[1 - self.current_player as usize]
    }
    fn other_player_mut(&mut self) -> &mut PlayerState {
        assert_ne!(self.current_player, CHANCE_PLAYER);
        &mut self.players[1 - self.current_player as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let options = GameStateOptions {
            stacks: [10000, 10000],
            blinds: [10, 5],
            pot: 100,
            initial_board: [0, 1, 2, 3, 4],
        };
        let game_state = GameState::new(options);
        let mut rng = rand::thread_rng();
        assert_eq!(game_state.is_ok(), true);
        let mut game_state = game_state.unwrap();
        assert_eq!(game_state.current_player, CHANCE_PLAYER);
        assert_eq!(game_state.is_initial_state(), true);
        assert_eq!(game_state.round, BettingRound::River);
        game_state = game_state.apply_action(game_state.sample_private_chance(&mut rng));
        assert_eq!(game_state.current_player, 0);
    }

    #[test]
    fn test_sample_private_chance() {
        let options = GameStateOptions {
            stacks: [10000, 10000],
            blinds: [10, 5],
            pot: 100,
            initial_board: [0, 1, 2, 52, 52],
        };
        let mut rng = rand::thread_rng();
        let mut game_state = GameState::new(options).unwrap();
        let private_chance_action = game_state.sample_private_chance(&mut rng);
        assert!(game_state.is_action_valid(private_chance_action));
        game_state = game_state.apply_action(private_chance_action);
        assert_eq!(game_state.current_player, 0);
        assert_ne!(game_state.used_cards_mask, 0);
    }

    #[test]
    fn test_sample_private_chance_from_ranges() {
        let options = GameStateOptions {
            stacks: [10000, 10000],
            blinds: [10, 5],
            pot: 0,
            initial_board: [52; 5],
        };
        let mut game_state = GameState::new(options).unwrap();
        let mut rng = rand::thread_rng();
        let hand_ranges = HandRange::from_strings(vec!["22+".to_string(), "66+".to_string()]);
        let private_chance_action =
            game_state.sample_private_chance_from_ranges(&mut rng, &hand_ranges[0..2]);
        assert!(game_state.is_action_valid(private_chance_action));
        game_state = game_state.apply_action(private_chance_action);
        assert_ne!(game_state.used_cards_mask, 0);
        assert_eq!(game_state.current_player, 1);
    }

    #[test]
    fn test_sample_public_chance() {
        let options = GameStateOptions {
            stacks: [10000, 10000],
            blinds: [10, 5],
            pot: 100,
            initial_board: [0, 1, 2, 52, 52],
        };
        let mut rng = rand::thread_rng();
        let mut game_state = GameState::new(options).unwrap();
        let private_chance_action = game_state.sample_private_chance(&mut rng);
        game_state = game_state.apply_action(private_chance_action);
        game_state = game_state.apply_action(Action::CheckCall);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player, CHANCE_PLAYER);
        let public_chance_action = game_state.sample_public_chance(&mut rng);
        assert!(game_state.is_action_valid(public_chance_action));
        if let Action::Chance(cards) = public_chance_action {
            assert_ne!(cards[0], 52);
            assert_eq!(cards[1], 52);
        }
        game_state = game_state.apply_action(public_chance_action);
        assert_eq!(game_state.round, BettingRound::Turn);
    }

    #[test]
    fn test_preflop() {
        let options = GameStateOptions {
            stacks: [10000, 10000],
            blinds: [10, 5],
            pot: 0,
            initial_board: [52; 5],
        };
        let mut rng = rand::thread_rng();
        // test checking through preflop
        let mut game_state = GameState::new(options).unwrap();
        game_state = game_state.apply_action(game_state.sample_private_chance(&mut rng));
        assert_eq!(game_state.current_player(), 1);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), 0);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        // test with bb raise
        let mut game_state = GameState::new(options).unwrap();
        game_state = game_state.apply_action(game_state.sample_private_chance(&mut rng));
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.pot(), 20);
        game_state = game_state.apply_action(Action::BetRaise(15));
        assert_eq!(game_state.current_player(), 1);
        assert_eq!(game_state.pot(), 35);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        // test with all in
        let mut game_state = GameState::new(options).unwrap();
        game_state = game_state.apply_action(game_state.sample_private_chance(&mut rng));
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.pot(), 20);
        game_state = game_state.apply_action(Action::BetRaise(game_state.acting_player().stack()));
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.pot(), 20000);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        game_state = game_state.apply_action(game_state.sample_public_chance(&mut rng));
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        // test with fold
        let mut game_state = GameState::new(options).unwrap();
        game_state = game_state.apply_action(game_state.sample_private_chance(&mut rng));
        game_state = game_state.apply_action(Action::Fold);
        assert_eq!(game_state.is_terminal(), true);
    }

    #[test]
    fn test_postflop() {
        let options = GameStateOptions {
            stacks: [10000, 10000],
            blinds: [10, 5],
            pot: 20,
            initial_board: [0, 1, 2, 52, 52],
        };
        let mut rng = rand::thread_rng();
        let mut initial_state = GameState::new(options).unwrap();
        initial_state = initial_state.apply_action(initial_state.sample_private_chance(&mut rng));
        // test check check
        let mut game_state = initial_state.clone();
        assert_eq!(game_state.current_player(), 0);
        game_state = game_state.apply_action(Action::CheckCall);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        assert_eq!(game_state.pot(), 20);
        // test bet call
        let mut game_state = initial_state.clone();
        game_state = game_state.apply_action(Action::BetRaise(10));
        assert_eq!(game_state.pot(), 30);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        // test check bet call
        let mut game_state = initial_state.clone();
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), 1);
        game_state = game_state.apply_action(Action::BetRaise(30));
        assert_eq!(game_state.current_player(), 0);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        assert_eq!(game_state.pot(), 80);
        // test bet raise call
        let mut game_state = initial_state.clone();
        game_state = game_state.apply_action(Action::BetRaise(10));
        game_state = game_state.apply_action(Action::BetRaise(30));
        assert_eq!(game_state.current_player(), 0);
        game_state = game_state.apply_action(Action::CheckCall);
        assert_eq!(game_state.current_player(), CHANCE_PLAYER);
        assert_eq!(game_state.pot(), 80);
        // test bet raise fold
        let mut game_state = initial_state.clone();
        game_state = game_state.apply_action(Action::BetRaise(10));
        assert_eq!(game_state.pot(), 30);
        game_state = game_state.apply_action(Action::BetRaise(30));
        assert_eq!(game_state.pot(), 60);
        game_state = game_state.apply_action(Action::Fold);
        assert_eq!(game_state.pot(), 40);
    }
}
