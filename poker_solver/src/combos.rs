use crate::card::{Card, CARD_COUNT};

/// generates card combinations to iterate over
pub struct CardComboIter {
    /// combo where each index maps to an index in cards
    combo: Vec<usize>,
    /// all valid cards that we can use
    cards: Vec<Card>,
    /// number of valid cards
    n: usize,
    /// number of cards in the combination
    k: usize,
}

impl CardComboIter {
    pub fn new(fixed_board: u64, k: usize) -> Self {
        let mut cards: Vec<Card> = Vec::new();
        // get all valid cards
        for i in 0..CARD_COUNT {
            if ((1u64 << i) & fixed_board) == 0 {
                cards.push(i);
            }
        }
        let n = cards.len();
        let combo = vec![0usize; k];
        CardComboIter { combo, cards, n, k }
    }
}

impl Iterator for CardComboIter {
    type Item = Vec<Card>;
    fn next(&mut self) -> Option<Vec<Card>> {
        let mut pivot = self.k - 1;
        while self.combo[pivot] == self.n - self.k + pivot {
            if pivot == 0 {
                return None;
            }
            pivot -= 1;
        }
        self.combo[pivot] += 1;
        for i in pivot + 1..self.k {
            self.combo[i] = self.combo[pivot] + i - pivot;
        }
        // transform combo into card vector
        let cards: Vec<Card> = self.combo.iter().map(|i| self.cards[*i]).collect();
        Some(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::bench::Bencher;

    #[test]
    fn test_new() {
        let iter = CardComboIter::new(0b111, 2);
        assert_eq!(iter.combo, vec![3, 4]);
    }

    #[bench]
    // 102,820 ns/iter (+/- 18,469)
    fn bench_generate_2(b: &mut Bencher) {
        // simulates generating a flop subset
        let flop_mask: u64 = 0b111;
        b.iter(|| {
            for c in CardComboIter::new(flop_mask, 2) {
                assert!(c.len() == 2);
            }
        });
    }

    #[bench]
    // 1,859,732 ns/iter (+/- 264,907)
    fn bench_generate_3(b: &mut Bencher) {
        // simulates generating a turn subset
        let flop_mask: u64 = 0b111;
        b.iter(|| {
            for c in CardComboIter::new(flop_mask, 3) {
                assert!(c.len() == 3);
            }
        });
    }

    #[bench]
    // 21,169,474 ns/iter (+/- 2,658,146)
    fn bench_generate_4(b: &mut Bencher) {
        // simulates generating a river subset
        let flop_mask: u64 = 0b111;
        b.iter(|| {
            for c in CardComboIter::new(flop_mask, 4) {
                assert!(c.len() == 4);
            }
        });
    }
}
