use rand::Rng;

/// The Current Betting Round a Texas Holdem game is in
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BettingRound {
    PREFLOP,
    FLOP,
    TURN,
    RIVER
}

/// Represents a player action
#[derive(Debug, Copy, Clone)]
pub enum Action {
    /// Bet is described as a multiple of the pot
    /// BET(1.0) is a full-pot bet
    BET(f32),
    /// Raise is described as a multiple of the pot
    /// RAISE(1.0)
    RAISE(f32),
    /// Fold action
    FOLD,
    /// Call a bet or raise
    CALL,
    /// Check
    CHECK,
}

/// List of available actions
/// 
/// Note: Bet and Raise sizes are invalid
pub static ACTIONS: &'static [Action; 5] = &[
    Action::BET(0.0),
    Action::RAISE(0.0),
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
            cards: [0; 2],
            has_folded: false
        }
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
    /// assert_eq!(game_state.get_pot(), 100);
    /// ```
    pub fn new(pot: u32, stacks: [u32; 2]) -> GameState {
        GameState {
            round: BettingRound::PREFLOP,
            pot,
            players: stacks.map(|s| PlayerState::new(s)),
            current_player: 0,
            board: [0; 5],
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
    /// assert_eq!(game_state.get_current_player_idx(), 0);
    /// game_state.deal_cards(&mut rng);
    /// game_state = game_state.apply_action(&mut rng, Action::CHECK);
    /// assert_eq!(game_state.get_current_player_idx(), 1);
    /// ```
    pub fn apply_action<R: Rng>(&self, rng: &mut R, action: Action) -> GameState {
        let mut next_state = self.clone();
        match action {
            Action::BET(amt) => {
                let mut wager = (amt * (next_state.pot as f32)) as u32;

                // if player has less than wager, put player all-in
                if next_state.current_player().stack < wager {
                    wager = next_state.current_player().stack;
                }

                next_state.current_player_mut().wager = wager;
                next_state.current_player_mut().stack -= wager;
                next_state.pot += wager;
                next_state.current_player = 1 - next_state.current_player;
            },
            Action::RAISE(amt) => {
                let mut wager = (amt * (next_state.pot as f32)) as u32;
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
                // if current player is 0, next player
                // else bets settled
                if next_state.current_player == 0 {
                    next_state.current_player = 1;
                } else {
                    next_state.bets_settled = true;
                }
            }
        }

        // if betting is finished, advance to next round
        if next_state.bets_settled {
            next_state.next_round(rng);
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
                let mut used_card_mask = self.get_used_card_mask();
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
                let used_card_mask = self.get_used_card_mask();
                let mut card = rng.gen_range(0, 52);
                while ((1 << card) & used_card_mask) != 0 {
                    card = rng.gen_range(0, 52);
                }
                self.board[3] = card;
            },
            BettingRound::RIVER => {
                // deal 1 random community card
                let used_card_mask = self.get_used_card_mask();
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
    fn get_used_card_mask(&self) -> u64 {
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
                return self.other_player().wager == 0;
            },
            Action::RAISE(_) => {
                // only valid if other player has bet
                return self.other_player().wager != 0;
            },
            Action::CALL => {
                // only valid if other player has bet
                return self.other_player().wager != 0;
            },
            Action::FOLD => {
                // only valid if other player has bet
                return self.other_player().wager != 0;
            },
            Action::CHECK => {
                // only valid if other player has not bet
                return self.other_player().wager == 0;
            }
        }
    }
    /// Returns true if a player has folded
    fn has_player_folded(&self) -> bool {
        self.current_player().has_folded || self.other_player().has_folded
    }
    /// Returns true if a player is all in
    fn is_player_all_in(&self) -> bool {
        self.current_player().stack == 0 || self.other_player().stack == 0
    }
    /// Return true if game is over
    fn is_game_over(&self) -> bool {
        self.has_player_folded() || self.is_player_all_in() || (self.round == BettingRound::RIVER && self.bets_settled)
    }
    /// Return pot size (# of chips)
    pub const fn get_pot(&self) -> u32 {
        return self.pot;
    }
    /// Get stack size of player at index `player_idx`
    pub fn get_stack(&self, player_idx: usize) -> u32 {
        assert!(player_idx == 0 || player_idx == 1);
        return self.players[player_idx].stack;
    }
    /// Return index of current player
    pub const fn get_current_player_idx(&self) -> u8 {
        return self.current_player;
    }
    /// Return a reference to the acting player state
    fn current_player(&self) -> &PlayerState {
        return &self.players[usize::from(self.current_player)];
    }
    /// Return a reference to the not acting player state
    fn other_player(&self) -> &PlayerState {
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
    fn next_round<R: Rng>(&mut self, rng: &mut R) {
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