use rand::Rng;
use std::fmt;
use std::cmp::min;

/// The Current Betting Round a Texas Holdem game is in
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BettingRound {
    PREFLOP,
    FLOP,
    TURN,
    RIVER
}


impl fmt::Display for BettingRound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let round_str = match self {
            BettingRound::PREFLOP => "Preflop",
            BettingRound::FLOP => "Flop",
            BettingRound::TURN => "Turn",
            BettingRound::RIVER => "River",
        };
        write!(f, "{}", round_str)
    }
}

/// Represents a player action
#[derive(Debug, Copy, Clone)]
pub enum Action {
    /// Bet size in chips
    BET(u32),
    /// Raise is a "by value"
    /// meaning amount of chips past the call value
    RAISE(u32),
    /// Fold action
    FOLD,
    /// Call a bet or raise
    CALL,
    /// Check
    CHECK,
}

/// For printing actions to terminal
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Action::BET(size) => {
                write!(f, "Bet {}", size)
            },
            Action::RAISE(size) => {
                write!(f, "Raise {}", size)
            },
            Action::FOLD => {
                write!(f, "Fold")
            },
            Action::CALL => {
                write!(f, "Call")
            },
            Action::CHECK => {
                write!(f, "Check")
            }
        }
    }
}


/// List of available actions
/// 
/// Note: Bet and Raise sizes are invalid
pub static ACTIONS: &'static [Action; 5] = &[
    Action::BET(0),
    Action::RAISE(0),
    Action::FOLD,
    Action::CALL,
    Action::CHECK
];

/// Represents the state of a single player in a HUNL Texas Holdem Game
#[derive(Debug, Copy, Clone)]
pub struct PlayerState {
    /// Amount of chips player has
    stack: u32,
    /// Amount of chips player is currently betting
    wager: u32,
    /// Player hole cards
    /// card is n in 0..52 where n = 4 * rank + suit
    cards: [u8; 2],
    /// Has the player folded
    has_folded: bool
}

/// Represents the current state of a HUNL Texas Holdem Game
/// 
/// State is small and immutable
/// This means that functions which update state operate on a copy of itself
#[derive(Debug, Copy, Clone)]
pub struct GameState {
    /// The current betting round
    round: BettingRound,
    /// The size in chips of the pot
    pot: u32,
    /// The state of each player
    players: [PlayerState; 2],
    /// index of current player
    current_player: u8,
    /// Community Cards
    /// card is n in 0..52 where n = 4 * rank + suit
    board: [u8; 5],
    /// Has betting finished for the current round
    bets_settled: bool
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
            has_folded: false
        }
    }
    /// Return stack size
    pub const fn stack(&self) -> u32 {
        return self.stack;
    }
    /// Return wager
    pub const fn wager(&self) -> u32 {
        return self.wager;
    }
    /// Return cards
    pub const fn cards(&self) -> &[u8; 2] {
        return &self.cards;
    }
    /// Return has folded
    pub const fn folded(&self) -> bool {
        return self.has_folded;
    }
}

impl GameState {
    /// Create a game state object with pot set to initial pot size
    /// and player stacks set to stack size
    /// 
    /// # Arguments
    /// 
    /// * `pot` - Initial pot size
    /// * `stack` - Initial stack size for each player
    /// 
    /// # Examples
    /// 
    /// ```
    /// use poker_solver::state::GameState;
    /// let game_state = GameState::new(100, [10000, 10000]);
    /// assert_eq!(game_state.pot(), 100);
    /// ```
    pub fn new(pot: u32, stacks: [u32; 2]) -> GameState {
        GameState {
            round: BettingRound::PREFLOP,
            pot,
            players: stacks.map(|s| PlayerState::new(s)),
            current_player: 0,
            // 52 since its the first invalid card
            board: [52; 5],
            bets_settled: false
        }
    }
    /// Creates a game state object but with initial wager equal to blinds
    /// 
    /// Both players post their blinds
    /// First to act is switched preflop
    /// 
    /// # Arguments
    /// 
    /// * `stacks` The stack size of each player
    /// * `blidds` Big and small blind size, Big should come before small
    pub fn init_with_blinds(stacks: [u32; 2], blinds: [u32; 2]) -> GameState {
        let mut players = stacks.map(|s| PlayerState::new(s));
        let big_blind = min(stacks[0], blinds[0]);
        players[0].stack -= big_blind;
        players[0].wager = big_blind;
        let small_blind = min(stacks[1], blinds[1]);
        players[1].stack -= small_blind;
        players[1].wager = small_blind;
        GameState {
            round: BettingRound::PREFLOP,
            pot: big_blind + small_blind,
            players,
            current_player: 1,
            // 52 since its the first invalid card
            board: [52; 5],
            bets_settled: false
        }
    }
    /// Return a list of valid actions a player can take
    /// 
    /// Does not account for Bet or Raise sizes
    /// 
    /// # Examples
    /// 
    /// ```
    /// use poker_solver::state::GameState;
    /// 
    /// let mut state = GameState::new(0, [1000, 1000]);
    /// assert_eq!(state.valid_actions().len(), 2); // CHECK or BET
    pub fn valid_actions(&self) -> Vec<Action> {
        return ACTIONS.into_iter().filter(|&&action| {
            self.is_action_valid(action)
        }).cloned().collect();
    }
    /// Apply an action and return a new updated state object
    /// 
    /// # Arguments
    /// 
    /// * `rng` A random number generator
    /// * `action` A valid action
    /// 
    /// # Examples
    /// 
    /// ```
    /// use poker_solver::state::GameState;
    /// use poker_solver::state::Action;
    /// let mut rng = rand::thread_rng();
    /// let mut game_state = GameState::new(100, [10000, 10000]);
    /// assert_eq!(game_state.current_player_idx(), 0);
    /// game_state.deal_cards(&mut rng);
    /// game_state = game_state.apply_action(&mut rng, Action::CHECK);
    /// assert_eq!(game_state.current_player_idx(), 1);
    /// ```
    pub fn apply_action(&self, action: Action) -> GameState {
        let mut next_state = self.clone();
        match action {
            Action::BET(bet_size) => {
                // if player has less than wager, put player all-in
                let mut wager = bet_size;
                if next_state.current_player().stack < bet_size {
                    wager = next_state.current_player().stack;
                }

                next_state.current_player_mut().wager = wager;
                next_state.current_player_mut().stack -= wager;
                next_state.pot += wager;
                next_state.current_player = 1 - next_state.current_player;
            },
            Action::RAISE(raise_size) => {
                // call amount plus our raise
                let diff = next_state.other_player().wager - next_state.current_player().wager;
                let mut wager = diff + raise_size;

                // if player has less than wager, put player all in
                if next_state.current_player().stack < wager {
                    wager = next_state.current_player().stack;
                }
                next_state.current_player_mut().wager += wager;
                next_state.current_player_mut().stack -= wager;
                next_state.pot += wager;
                next_state.current_player = 1 - next_state.current_player;
            },
            Action::CALL => {
                // get difference in bets
                let mut wager = next_state.other_player().wager - next_state.current_player().wager;
                // if player does not have enough to call bet
                // put player all in
                // and return chips to other player
                if next_state.current_player().stack < wager {
                    let diff = wager - next_state.current_player().stack;
                    next_state.other_player_mut().wager -= diff;
                    next_state.other_player_mut().stack += diff;
                    next_state.pot -= diff;
                    wager = next_state.current_player().stack;
                }
                next_state.current_player_mut().wager += wager;
                next_state.current_player_mut().stack -= wager;
                next_state.pot += wager;
                next_state.bets_settled = true;
            },
            Action::FOLD => {
                // return difference in wager to other player
                let diff = next_state.other_player().wager - next_state.current_player().wager;
                next_state.other_player_mut().wager -= diff;
                next_state.other_player_mut().stack += diff;
                // remove from pot
                next_state.pot -= diff;
                // set player folded
                next_state.current_player_mut().has_folded = true;
                next_state.bets_settled = true;
            },
            Action::CHECK => {
                // last to act switchs pre flop
                if next_state.round == BettingRound::PREFLOP && next_state.current_player == 0 {
                    next_state.bets_settled = true;
                }
                // if current player is last to act bets are even
                if next_state.round != BettingRound::PREFLOP && next_state.current_player == 1 {
                    next_state.bets_settled = true;
                }
                next_state.current_player = 1 - next_state.current_player;
            }
        }
        return next_state;
    }
    /// Deals cards in the current round
    /// and update game state
    /// 
    /// If self.round == PREFLOP, it will deal cards to both players
    /// If self.round == FLOP, it will deal 3 public cards
    /// If self.round == TURN, it will deal 1 public card
    /// If self.round == RIVER, it will deal 1 public card
    /// 
    /// # Arguments
    /// 
    /// * `rng` a mutable reference to a random number generator
    pub fn deal_cards<R: Rng>(&mut self, rng: &mut R) {
        match self.round {
            BettingRound::PREFLOP => {
                // deal 2 random cards to each player
                // generate 4 unique random cards
                let mut used_card_mask = 0u64;
                let cards: Vec<u8> = (0..4).map(|_| {
                    let mut card = rng.gen_range(0, 52);
                    while ((1 << card) & used_card_mask) != 0 {
                        card = rng.gen_range(0, 52);
                    }
                    used_card_mask |= 1 << card;
                    card
                }).collect();
                // assign to players
                self.players[0].cards[0] = cards[0];
                self.players[0].cards[1] = cards[1];
                self.players[1].cards[0] = cards[2];
                self.players[1].cards[1] = cards[3];
            },
            BettingRound::FLOP => {
                // deal 3 random community cards
                let mut used_card_mask = self.used_card_mask();
                // generate 3 unique cards
                let cards: Vec<u8> = (0..3).map(|_| {
                    let mut card = rng.gen_range(0, 52);
                    while ((1 << card) & used_card_mask) != 0 {
                        card = rng.gen_range(0, 52);
                    }
                    used_card_mask |= 1 << card;
                    card
                }).collect();
                // assign to board
                self.board[0] = cards[0];
                self.board[1] = cards[1];
                self.board[2] = cards[2];
            },
            BettingRound::TURN => {
                // deal 1 random community card
                let used_card_mask = self.used_card_mask();
                let mut card = rng.gen_range(0, 52);
                while ((1 << card) & used_card_mask) != 0 {
                    card = rng.gen_range(0, 52);
                }
                self.board[3] = card;
            },
            BettingRound::RIVER => {
                // deal 1 random community card
                let used_card_mask = self.used_card_mask();
                let mut card = rng.gen_range(0, 52);
                while ((1 << card) & used_card_mask) != 0 {
                    card = rng.gen_range(0, 52);
                }
                self.board[4] = card;
            }
        }
    }
    /// Return a bitmask representing which cards have already been dealt
    /// 
    /// Used for rejection sampling
    fn used_card_mask(&self) -> u64 {
        let mut used_card_mask = 0u64;

        used_card_mask |= 1 << self.players[0].cards[0];
        used_card_mask |= 1 << self.players[0].cards[1];
        used_card_mask |= 1 << self.players[1].cards[0];
        used_card_mask |= 1 << self.players[1].cards[1];

        if self.round == BettingRound::PREFLOP {
            return used_card_mask;
        }

        for i in 0..3 {
            used_card_mask |= 1 << self.board[i];
        }

        if self.round == BettingRound::TURN {
            return used_card_mask;
        }

        used_card_mask |= 1 << self.board[3];

        if self.round == BettingRound::RIVER {
            return used_card_mask;
        }

        used_card_mask |= 1 << self.board[4];

        return used_card_mask;
    }
    /// Checks whether an action is valid in the current context
    /// 
    /// # Arguments
    /// 
    /// * `action` A poker action
    fn is_action_valid(&self, action: Action) -> bool {
        match action {
            Action::BET(_) => {
                // only valid if other player has not bet
                // and we have chips to bet
                let valid = (self.other_player().wager == 0)
                && (self.current_player().stack > 0);
                return valid;
            },
            Action::RAISE(_) => {
                // only valid if other player has bet
                // and we have more money than their wager
                let valid = (self.other_player().wager != 0)
                && (self.current_player().stack > self.other_player().wager);
                return valid;
            },
            Action::CALL => {
                // only valid if other player has bet more than us
                return self.other_player().wager > self.current_player().wager;
            },
            Action::FOLD => {
                // only valid if other player has bet more than us
                return self.other_player().wager > self.current_player().wager;
            },
            Action::CHECK => {
                // only valid if other player has not bet
                return self.current_player().wager >= self.other_player().wager;
            }
        }
    }
    /// Returns index of folded player or None
    pub fn player_folded(&self) -> Option<u8> {
        if self.current_player().has_folded {
            return Some(self.current_player);
        }
        if self.other_player().has_folded {
            return Some(1 - self.current_player);
        }
        return None;
    }
    /// Return index of all in player or None
    pub fn player_all_in(&self) -> Option<u8> {
        if self.current_player().stack == 0 {
            return Some(self.current_player);
        }
        if self.other_player().stack == 0 {
            return Some(1 - self.current_player);
        }
        return None;
    }
    /// Return true if game is over
    pub fn is_game_over(&self) -> bool {
        self.player_folded().is_some()
        || (self.player_all_in().is_some() && self.bets_settled)
        || (self.round == BettingRound::RIVER && self.bets_settled)
    }
    /// Return pot size (# of chips)
    pub const fn pot(&self) -> u32 {
        return self.pot;
    }
    /// Return round
    pub const fn round(&self) -> BettingRound {
        return self.round;
    }
    /// Return index of current player
    pub const fn current_player_idx(&self) -> u8 {
        return self.current_player;
    }
    /// Return reference to player at index
    pub const fn player(&self, player_index: usize) -> &PlayerState {
        return &self.players[player_index];
    }
    /// Return reference to public cards
    pub const fn board(&self) -> &[u8; 5] {
        return &self.board;
    }
    /// Return bets settled
    pub fn bets_settled(&self) -> bool {
        return self.bets_settled;
    }
    /// Return a reference to the acting player state
    pub fn current_player(&self) -> &PlayerState {
        return &self.players[usize::from(self.current_player)];
    }
    /// Return a reference to the not acting player state
    pub fn other_player(&self) -> &PlayerState {
        return &self.players[usize::from(1-self.current_player)];
    }
    /// Return a mutable reference to the acting player state
    fn current_player_mut(&mut self) -> &mut PlayerState {
        return &mut self.players[usize::from(self.current_player)];
    }
    /// Return a reference to the not acting player state
    fn other_player_mut(&mut self) -> &mut PlayerState {
        return &mut self.players[usize::from(1 - self.current_player)];
    }
    /// Advance the game state to the next betting round
    pub fn next_round<R: Rng>(&mut self, rng: &mut R) {
        // advance round or do nothing if round is river
        match self.round {
            BettingRound::PREFLOP => {
                self.round = BettingRound::FLOP;
            },
            BettingRound::FLOP => {
                self.round = BettingRound::TURN;
            },
            BettingRound::TURN => {
                self.round = BettingRound::RIVER;
            },
            BettingRound::RIVER => {
                // game is over, do nothing
                return;
            }
        }
        self.bets_settled = false;
        self.current_player = 0;
        self.current_player_mut().wager = 0;
        self.other_player_mut().wager = 0;
        self.deal_cards(rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}