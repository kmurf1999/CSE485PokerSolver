use poker_solver::card_abstraction::{CardAbstraction, CardAbstractionOptions};
use poker_solver::round::BettingRound;
use rust_poker::HandIndexer;

fn main() {
    // create public flop hands 0->1755
    let flop_indexer = HandIndexer::init(1, vec![3]);
    let mut public_flop_hands = [[0u8; 3]; 1755];
    let mut cards = [0u8; 3];
    for i in 0..1755 {
        flop_indexer.get_hand(0, i as u64, &mut cards);
        public_flop_hands[i][..3].clone_from_slice(&cards[..3]);
    }
    // create transition table
    // T[i][j], 0 <= i < 1755, 0 <= j < 5000
    // number of of private card combinations for where a hand with public flop i, transitions into bucket j
    let ca = CardAbstraction::load(CardAbstractionOptions {
        abs_type: String::from("emd"),
        d: 50,
        k: 5000,
        round: BettingRound::Flop,
    })
    .unwrap();
    let hand_indexer = HandIndexer::init(2, vec![2, 3]);
    let mut transition_table = vec![vec![0usize; 5000]; 1755];
    // iterate over each flop hand
    for i in 0..1755 {
        // iterate of each private hand
        for c1 in 0..52 {
            if public_flop_hands[i].contains(&c1) {
                continue;
            }
            for c2 in c1 + 1..52 {
                if public_flop_hands[i].contains(&c2) {
                    continue;
                }

                let mut cards = [c1, c2, 0, 0, 0];
                for j in 0..3 {
                    cards[j + 2] = public_flop_hands[i][j];
                }

                let cannon_idx = hand_indexer.get_index(&cards) as usize;
                let bucket_idx = ca.get(cannon_idx) as usize;
                transition_table[i][bucket_idx] += 1;
            }
        }
    }
    // calculate distances
    let mut distances = vec![vec![0f64; 1755]; 1755];
    const V: f64 = 1176.0;
    for i in 0..1755 {
        for j in i + 1..1755 {
            let mut s = 0;
            for b in 0..5000 {
                let ci = transition_table[i][b];
                let cj = transition_table[j][b];
                s += std::cmp::min(ci, cj);
            }
            distances[i][j] = (V - s as f64) / V;
        }
    }
}
