/// Stores regrets for each action not
pub struct MCCFRNode {
    /// vector of regrets [u32; n_buckets * n_actions]
    regrets: Vec<u32>,
    /// vector of sum probs [u32; n_buckets * n_actions]
    sum_probs: Vec<u32>,
    /// is a terminal node
    terminal: bool,
    /// node index
    index: usize,
    /// player
    player: usize,
}

impl MCCFRNode {}

pub struct MCCFRTree {
    arena: Vec<MCCFRNode>,
}

impl MCCFRTree {}

pub struct MCCFR {
    strategy_interval: u32,
    prune_threshhold: u32,
    discount_interval: u32,
}

impl MCCFR {
    // fn run(&mut self, iterations: usize) {
    //     for t in 0..iterations {
    //         for p in 0..self.players() {
    //             if t % self.strategy_interval == 0 {
    //                 // sample an action for every action node and update the cummulative strategy
    //                 self.update_strategy(p);
    //             }
    //             if t > self.prune_threshhold {
    //                 // let q = sample normal [0, 1)
    //                 // if q < 0.05 traverse no prune
    //                 // else traverse prune
    //             } else {
    //                 // traverse no prune
    //             }
    //         }
    //         if t < self.lccfr_threshold && t % self.discount_interval == 0 {
    //             let d = (t / self.discount_interval) / (t / (self.discount_interval + 1));
    //             // discount all regrets and cummulative strategy by d
    //             self.discount(d);
    //         }
    //     }
    // }
    // fn traverse_prune(&mut self, node: &mut MCCFRNode, player: u8) -> f32 {}
}
