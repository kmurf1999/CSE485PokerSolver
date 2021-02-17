use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use rust_poker::equity_calculator::calc_equity;
use rust_poker::hand_range::{get_card_mask, HandRange};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use rust_poker::{
    constants::{RANK_TO_CHAR, SUIT_TO_CHAR},
    HandIndexer,
};
use std::io::{self, Write};

static OCHS_CLUSTERS: &[&str; 8] = &[
    "88+",
    "A9o,ATo,AJo,AQo,AKo,KTo,KJo,KQo,A7s,A8s,A9s,ATs,AJs,AQs,AKs,K9s,KTs,KJs,KQs,QTs,QJs,66,77",
    "A2o,A3o,A4o,A5o,A6o,A7o,A8o,K5o,K6o,K7o,K8o,K9o,33,44,55,A2s,A3s,A4s,A5s,A6s,K3s,K4s,K5s,K6s,K7s,K8s",
    "Q8o,Q9o,QTo,QJo,J8o,J9o,JTo,T9o,Q6s,Q7s,Q8s,Q9s,J7s,J8s,J9s,JTs,T7s,T8s,T9s",
    "22,K2o,K3o,K4o,Q2o,Q3o,Q4o,Q5o,Q6o,Q7o,J4o,J5o,J6o,J7o,K2s,Q2s,Q3s,Q4s,Q5s,J2,J3s,J4s,J5s,J6s",
    "T3s,T4s,T5s,T6s,95s,96s,97s,98s,85s,86s,75s,76s,65s,76o,86o,96o,T6o,87o,97o,T7o,98o,T8o",
    "T2s,92s,93s,94s,82s,83s,84s,74s,75o,85o,95o,T5o,84o,94o,T4o,93o,T3o,J3o,92o,T2o,J2o",
    "32s,42s,52s,62s,72s,43s,53s,63s,73s,54s,64s,32o,42o,43o,52o,53o,54o,62o,63o,64o,65o,72o,73o,74o,82o,83o"
];

#[inline(always)]
fn cards_to_str(cards: &[u8]) -> String {
    let mut out = String::new();
    for i in 0..cards.len() {
        let rank = cards[i] / 4;
        let suit = cards[i] % 4;
        out.push(RANK_TO_CHAR[usize::from(rank)]);
        out.push(SUIT_TO_CHAR[usize::from(suit)]);
    }
    out
}

pub fn gen_ochs_features(round: u8, sim_count: u64) -> Array2<f32> {
    let start_time = Instant::now();
    println!(
        "Generating ochs vectors for round: {}, sim_count: {}",
        3, sim_count
    );

    let indexer = match round {
        1 => HandIndexer::init(2, [2, 3].to_vec()),
        2 => HandIndexer::init(2, [2, 4].to_vec()),
        3 => HandIndexer::init(2, [2, 5].to_vec()),
        _ => panic!("invalid round"),
    };
    let n_board_cards = match round {
        1 => 3,
        2 => 4,
        3 => 5,
        _ => panic!("invalid round"),
    };

    let round_size = if round == 0 {
        indexer.size(0)
    } else {
        indexer.size(1)
    } as usize;

    // 123156254 * 32 bits * 8 = 3.941000128 gigabytes
    let mut ochs_vectors: Array2<f32> = Array2::zeros((round_size, OCHS_CLUSTERS.len()));
    let counter = AtomicU32::new(0);

    ochs_vectors
        .axis_iter_mut(Axis(0))
        .into_par_iter()
        .enumerate()
        .for_each(|(i, mut ochs_vec)| {
            // print progress
            let c = counter.fetch_add(1, Ordering::SeqCst);
            if i.trailing_zeros() >= 12 {
                print!("{:.3}% \r", c as f32 / round_size as f32);
                io::stdout().flush().unwrap();
            }
            let mut cards = [0u8; 7];
            indexer.get_hand(1, i as u64, &mut cards);
            let hole_cards = cards_to_str(&cards[0..2]);
            let board_cards = cards_to_str(&cards[2..(n_board_cards + 2)]);
            let board_mask = get_card_mask(board_cards.as_str());
            for i in 0..OCHS_CLUSTERS.len() {
                let ranges = HandRange::from_strings(
                    [hole_cards.to_string(), OCHS_CLUSTERS[i].to_string()].to_vec(),
                );
                let equity = calc_equity(&ranges, board_mask, 1, sim_count);
                ochs_vec[i] = equity[0] as f32;
            }
        });

    let duration = start_time.elapsed().as_millis();
    println!("done. took {}ms", duration);

    ochs_vectors
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_poker::HandIndexer;

    #[test]
    fn test_get_hole_cards() {
        let indexer = HandIndexer::init(2, [2, 5].to_vec());
        let mut cards = [0u8; 7];
        indexer.get_hand(1, 5000, &mut cards);
        assert_eq!(cards_to_str(&cards[0..2]), "AsTh");
        assert_eq!(cards_to_str(&cards[2..7]), "6d8d9d2c3c");
    }
}
