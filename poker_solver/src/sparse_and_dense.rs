use std::collections::HashMap;

/// maximum single array size for a block
const BLOCK_SIZE: usize = 1000000;

/// Converts a sparse array into a dense array
/// used for mapping flop/turn/river subsets into arrays that save memory
/// cards -> cannonical index -> bucket index -> dense index
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
    pub fn sparse_to_dense(&mut self, sparse: usize) -> usize {
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
    /// get the sparse index from a dense one
    pub fn dense_to_sparse(&self, dense: usize) -> usize {
        self.dense_to_sparse[dense]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Card;
    use crate::combos::CardComboIter;
    use rust_poker::hand_range::HandRange;
    use rust_poker::HandIndexer;
    use test::bench::Bencher;

    #[test]
    fn test_default() {
        let sd = SparseAndDense::default();
        assert_eq!(sd.is_empty(), true);
        assert_eq!(sd.len(), 0);
    }

    #[bench]
    // 306,755 ns/iter (+/- 110,512)
    fn bench_generate_flop_subset(b: &mut Bencher) {
        let indexer = HandIndexer::init(2, vec![2, 3]);
        let flop = [0u8, 40, 2];
        let mut flop_mask = 0u64;
        for card in &flop {
            flop_mask |= 1u64 << card;
        }
        let hand_range = HandRange::from_string("random".to_string());
        let mut cards: [u8; 5] = [0, 0, flop[0], flop[1], flop[2]];
        b.iter(|| {
            let mut sd = SparseAndDense::default();
            for hole_cards in &hand_range.hands {
                let hand_mask = (1u64 << hole_cards.0) | (1u64 << hole_cards.1);
                if (hand_mask & flop_mask) != 0 {
                    continue;
                }
                cards[0] = hole_cards.0;
                cards[1] = hole_cards.1;
                let cannon_index = indexer.get_index(&cards);
                sd.sparse_to_dense(cannon_index as usize);
            }
            assert_eq!(sd.len(), 721);
        });
    }

    #[bench]
    // 5,360,807 ns/iter (+/- 1,573,741)
    // 9,281,069 ns/iter (+/- 1,096,527) if using random hand range
    fn bench_generate_turn_subset(b: &mut Bencher) {
        let indexer = HandIndexer::init(2, vec![2, 4]);
        let flop = [0u8, 40, 2];
        let mut flop_mask = 0u64;
        for card in &flop {
            flop_mask |= 1u64 << card;
        }
        let hand_range = HandRange::from_string("random".to_string());
        let mut cards: [u8; 6] = [0, 0, flop[0], flop[1], flop[2], 0];
        b.iter(|| {
            let mut sd = SparseAndDense::default();
            let board_combos: Vec<Vec<Card>> = CardComboIter::new(flop_mask, 1).collect();
            for hole_cards in &hand_range.hands {
                let hand_mask = (1u64 << hole_cards.0) | (1u64 << hole_cards.1);
                if (hand_mask & flop_mask) != 0 {
                    continue;
                }
                cards[0] = hole_cards.0;
                cards[1] = hole_cards.1;
                let combo_mask = hand_mask | flop_mask;
                for board_combo in &board_combos {
                    let board_combo_mask = 1u64 << board_combo[0];
                    if (board_combo_mask & combo_mask) != 0 {
                        continue;
                    }
                    cards[5] = board_combo[0];
                    let cannon_index = indexer.get_index(&cards);
                    sd.sparse_to_dense(cannon_index as usize);
                }
            }
            assert_eq!(sd.len(), 29769);
        });
    }

    #[bench]
    // 65,652,233 ns/iter (+/- 10,886,057)
    // 263,282,522 ns for river 0.26 seconds
    fn bench_generate_river_subset(b: &mut Bencher) {
        let indexer = HandIndexer::init(2, vec![2, 5]);
        let flop = [0u8, 40, 2];
        let mut flop_mask = 0u64;
        for card in &flop {
            flop_mask |= 1u64 << card;
        }
        let hand_range = HandRange::from_string("random".to_string());
        let mut cards: [u8; 7] = [0, 0, flop[0], flop[1], flop[2], 0, 0];
        b.iter(|| {
            let mut sd = SparseAndDense::default();
            let board_combos: Vec<Vec<Card>> = CardComboIter::new(flop_mask, 2).collect();
            for hole_cards in &hand_range.hands {
                let hand_mask = (1u64 << hole_cards.0) | (1u64 << hole_cards.1);
                if (hand_mask & flop_mask) != 0 {
                    continue;
                }
                cards[0] = hole_cards.0;
                cards[1] = hole_cards.1;
                let combo_mask = hand_mask | flop_mask;
                for board_combo in &board_combos {
                    let board_combo_mask = (1u64 << board_combo[0]) | (1u64 << board_combo[1]);
                    if (board_combo_mask & combo_mask) != 0 {
                        continue;
                    }
                    cards[5] = board_combo[0];
                    cards[6] = board_combo[1];
                    let cannon_index = indexer.get_index(&cards);
                    sd.sparse_to_dense(cannon_index as usize);
                }
            }
            assert_eq!(sd.len(), 630685);
        });
    }
}
