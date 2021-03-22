use crate::card::Card;
use crate::card_abstraction::CardAbstraction;
use crate::combos::CardComboIter;
use crate::constants::*;
use crate::round::BettingRound;
use rust_poker::hand_range::HandRange;
use rust_poker::HandIndexer;
use std::collections::HashMap;

/// maximum single array size for a block
const BLOCK_SIZE: usize = 1000000;

/// Converts a sparse array into a dense array
/// used for mapping flop/turn/river subsets into arrays that save memory
/// cards -> cannonical index -> bucket index -> dense index
#[derive(Debug)]
pub struct SparseAndDense {
    sparse_to_dense: HashMap<usize, usize>,
    dense_to_sparse: Vec<usize>,
    num: usize,
}

impl Default for SparseAndDense {
    fn default() -> Self {
        SparseAndDense {
            sparse_to_dense: HashMap::default(),
            dense_to_sparse: Vec::with_capacity(BLOCK_SIZE),
            num: 0,
        }
    }
}

impl SparseAndDense {
    pub const fn len(&self) -> usize {
        self.num
    }
    pub const fn is_empty(&self) -> bool {
        self.num == 0
    }
    /// gets and inserts if needed a sparse index and returns a dense index
    pub fn sparse_to_dense_or_insert(&mut self, sparse: usize) -> usize {
        match self.sparse_to_dense.get(&sparse) {
            Some(s) => *s,
            None => {
                let dense = self.num;
                self.num += 1;
                if self.num >= self.dense_to_sparse.len() {
                    self.dense_to_sparse.reserve(BLOCK_SIZE);
                }
                self.dense_to_sparse.push(sparse);
                self.sparse_to_dense.insert(sparse, dense);
                dense
            }
        }
    }
    /// gets dense index given a sparse one
    pub fn sparse_to_dense(&self, sparse: usize) -> Option<usize> {
        self.sparse_to_dense.get(&sparse).copied()
    }
    /// get the sparse index from a dense one
    pub fn dense_to_sparse(&self, dense: usize) -> usize {
        self.dense_to_sparse[dense]
    }
}

pub fn generate_buckets(
    hand_range: &HandRange,
    card_abstraction: &CardAbstraction,
    round: BettingRound,
    board_mask: u64,
) -> SparseAndDense {
    let (indexer, total_board_cards) = match round {
        BettingRound::PREFLOP => (HandIndexer::init(1, vec![2]), 0),
        BettingRound::FLOP => (HandIndexer::init(2, vec![2, 3]), 3),
        BettingRound::TURN => (HandIndexer::init(2, vec![2, 4]), 4),
        BettingRound::RIVER => (HandIndexer::init(2, vec![2, 5]), 5),
    };
    let mut cards: [u8; 7] = [CARD_COUNT; 7];
    let mut num_board_cards = 0;
    // copy board cards
    for i in 0..CARD_COUNT {
        if ((1u64 << i) & board_mask) != 0 {
            cards[num_board_cards + 2] = i;
            num_board_cards += 1;
        }
    }
    let mut sd = SparseAndDense::default();

    let board_combos: Vec<Vec<Card>> = match total_board_cards - num_board_cards > 0 {
        true => CardComboIter::new(board_mask, total_board_cards - num_board_cards).collect(),
        false => Vec::new(),
    };

    for hole_cards in &hand_range.hands {
        let hand_mask = (1u64 << hole_cards.0) | (1u64 << hole_cards.1);
        if (hand_mask & board_mask) != 0 {
            continue;
        }
        cards[0] = hole_cards.0;
        cards[1] = hole_cards.1;
        let combo_mask = hand_mask | board_mask;
        // if no board combos we're probably dealing with a preflop combo
        if board_combos.is_empty() {
            let cannon_index = indexer.get_index(&cards[0..2]);
            let bucket = card_abstraction.get(cannon_index as usize);
            sd.sparse_to_dense_or_insert(bucket as usize);
            continue;
        }
        for board_combo in &board_combos {
            let board_combo_mask: u64 = board_combo.iter().map(|c| 1u64 << c).sum();
            if (board_combo_mask & combo_mask) != 0 {
                continue;
            }
            for i in 0..board_combo.len() {
                cards[i + num_board_cards + 2] = board_combo[i];
            }
            let cannon_index = indexer.get_index(&cards[0..total_board_cards + 2]);
            let bucket = card_abstraction.get(cannon_index as usize);
            sd.sparse_to_dense_or_insert(bucket as usize);
        }
    }

    sd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card_abstraction::CardAbstractionOptions;
    use rust_poker::hand_range::get_card_mask;
    use rust_poker::hand_range::HandRange;
    use test::bench::Bencher;

    #[test]
    fn test_default() {
        let sd = SparseAndDense::default();
        assert_eq!(sd.is_empty(), true);
        assert_eq!(sd.len(), 0);
    }

    #[bench]
    // 226,904 ns/iter (+/- 42,727)
    fn bench_generate_flop_subset(b: &mut Bencher) {
        let flop_mask = get_card_mask("AcAd2h");
        let hand_range = HandRange::from_string("random".to_string());
        let ca = CardAbstraction::load(CardAbstractionOptions {
            abs_type: "null".to_string(),
            round: BettingRound::FLOP,
            k: 0,
            d: 0,
        })
        .unwrap();
        b.iter(|| {
            let sd = generate_buckets(&hand_range, &ca, BettingRound::FLOP, flop_mask);
            assert_eq!(sd.len(), 744);
            assert_eq!(sd.sparse_to_dense(0), None);
        });
    }

    #[bench]
    // 8,821,887 ns/iter (+/- 1,806,510)
    fn bench_generate_turn_subset(b: &mut Bencher) {
        let flop_mask = get_card_mask("AcAd2h");
        let hand_range = HandRange::from_string("random".to_string());
        let ca = CardAbstraction::load(CardAbstractionOptions {
            abs_type: "emd".to_string(),
            round: BettingRound::TURN,
            k: 5000,
            d: 50,
        })
        .unwrap();
        b.iter(|| {
            let sd = generate_buckets(&hand_range, &ca, BettingRound::TURN, flop_mask);
            assert_eq!(sd.len(), 396);
            assert_eq!(sd.sparse_to_dense(0), None);
        });
    }

    #[bench]
    // 34,460,103 ns/iter (+/- 4,196,082)
    fn bench_generate_river_subset(b: &mut Bencher) {
        let flop_mask = get_card_mask("AcAd2h");
        let hand_range = HandRange::from_string("22+,AT+,KT+,QT+,JT+".to_string());
        let ca = CardAbstraction::load(CardAbstractionOptions {
            abs_type: "ochs".to_string(),
            round: BettingRound::RIVER,
            k: 5000,
            d: 8,
        })
        .unwrap();
        b.iter(|| {
            let sd = generate_buckets(&hand_range, &ca, BettingRound::RIVER, flop_mask);
            assert_eq!(sd.len(), 547);
            assert_eq!(sd.sparse_to_dense(0), None);
        });
    }
}
