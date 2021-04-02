#[derive(Debug)]
pub struct BettingAbstraction {
    /// An array of bet sizes for each round
    pub bet_sizes: [Vec<f64>; 4],
    /// An array of raise sizes for each round
    pub raise_sizes: [Vec<f64>; 4],
    /// If bet/raise size is greater than this fraction of our stack, go all in
    /// Ignored if value is zero
    pub all_in_threshold: f64,
}
