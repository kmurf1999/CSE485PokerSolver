/// starting stack size
pub const STACK_SIZE: u32 = 10000;
/// maximum players in a game
pub const MAX_PLAYERS: usize = 2;
/// min players allowed in a game
pub const MIN_PLAYERS: usize = 2;
/// blinds [big, small]
pub const BLINDS: [u32; 2] = [10, 5];
/// timeout in seconds for a client taking an action
pub const ACTION_TIMEOUT: u64 = 30;
/// total number of cards in deck
pub const CARD_COUNT: u8 = 52;
