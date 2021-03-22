use crate::round::BettingRound;
use rust_poker::read_write::VecIO;
use rust_poker::HandIndexer;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::result::Result;

/// Options to load Card abstraction
/// abstraction should be stored in `data` folder
#[derive(Debug)]
pub struct CardAbstractionOptions {
    /// abstraction type
    /// null, emd, ochs, pa (potential aware)
    pub abs_type: String,
    /// which round this abstraction if for
    pub round: BettingRound,
    /// buckets per abstraction
    pub k: usize,
    /// dimension of data used to generate abstraction
    pub d: usize,
}

/// Structure for loading card (information) abstraction from a file and into memory
#[derive(Debug)]
pub struct CardAbstraction {
    /// number of cannonical cards in this round
    round_size: usize,
    /// number of buckets in this abstraction
    n_buckets: usize,
    /// actual abstraction buckets
    use_short: bool,
    pub short_buckets: Vec<u16>,
    pub long_buckets: Vec<u32>,
}

impl CardAbstraction {
    /// Loads an abstraction from a file
    ///
    /// # Example
    /// ```
    /// let options = CardAbstractionOptions {
    ///   abs_type: "emd".to_string(),
    ///   k: 5000,
    ///   d: 50,
    ///   round: BettingRound::FLOP,
    /// };
    /// let card_abs = CardAbstraction::load(options).unwrap();
    /// ```
    pub fn load(options: CardAbstractionOptions) -> Result<Self, Box<dyn Error>> {
        let indexer = match options.round {
            BettingRound::PREFLOP => HandIndexer::init(1, vec![2]),
            BettingRound::FLOP => HandIndexer::init(2, vec![2, 3]),
            BettingRound::TURN => HandIndexer::init(2, vec![2, 4]),
            BettingRound::RIVER => HandIndexer::init(2, vec![2, 5]),
        };
        let round = usize::from(options.round);
        let round_size = indexer.size(if round == 0 { 0 } else { 1 }) as usize;

        if options.abs_type == "null" {
            let n_buckets = round_size;
            let long_buckets = (0..n_buckets).map(|x| x as u32).collect();
            return Ok(CardAbstraction {
                round_size,
                n_buckets,
                long_buckets,
                short_buckets: Vec::new(),
                use_short: false,
            });
        }

        let n_buckets = options.k;
        let filename = format!(
            "data/{}-abs-r{}-k{}-d{}.dat",
            options.abs_type, round, options.k, options.d
        );
        let mut file = OpenOptions::new().read(true).open(filename)?;
        let short_buckets = file.read_vec_from_file::<u16>()?;
        let abs = CardAbstraction {
            round_size,
            n_buckets,
            use_short: true,
            short_buckets,
            long_buckets: Vec::new(),
        };
        Ok(abs)
    }
    #[inline(always)]
    /// gets the number of buckets in this abstraction
    pub const fn n_buckets(&self) -> usize {
        self.n_buckets
    }
    #[inline(always)]
    /// gets the number of cannonical hands in this round
    pub const fn round_size(&self) -> usize {
        self.round_size
    }
    #[inline(always)]
    pub fn get(&self, index: usize) -> u32 {
        if self.use_short {
            return self.short_buckets[index].into();
        }
        self.long_buckets[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::bench::Bencher;

    #[bench]
    // 780,262 ns/iter (+/- 95,274)
    fn bench_load_emd_flop(b: &mut Bencher) {
        b.iter(|| {
            let options = CardAbstractionOptions {
                abs_type: "emd".to_string(),
                k: 5000,
                d: 50,
                round: BettingRound::FLOP,
            };
            let card_abs = CardAbstraction::load(options).unwrap();
            assert_eq!(card_abs.get(0), 412);
            assert_eq!(5000, card_abs.n_buckets());
            assert_eq!(1286792, card_abs.round_size());
        });
    }

    #[bench]
    // 21,380,269 ns/iter (+/- 6,738,865)
    fn bench_load_emd_turn(b: &mut Bencher) {
        b.iter(|| {
            let options = CardAbstractionOptions {
                abs_type: "emd".to_string(),
                k: 5000,
                d: 50,
                round: BettingRound::TURN,
            };
            let card_abs = CardAbstraction::load(options).unwrap();
            assert_eq!(card_abs.get(0), 3200);
            assert_eq!(5000, card_abs.n_buckets());
            assert_eq!(13960050, card_abs.round_size());
        });
    }

    #[bench]
    // 142,376,793 ns/iter (+/- 51,651,815)
    fn bench_load_ochs_river(b: &mut Bencher) {
        b.iter(|| {
            let options = CardAbstractionOptions {
                abs_type: "ochs".to_string(),
                k: 5000,
                d: 8,
                round: BettingRound::RIVER,
            };
            let card_abs = CardAbstraction::load(options).unwrap();
            assert_eq!(card_abs.get(0), 4233);
            assert_eq!(5000, card_abs.n_buckets());
            assert_eq!(123156254, card_abs.round_size());
        });
    }
}
