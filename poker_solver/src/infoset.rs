//! Contains code for storing information sets
//!
//! An information set contains all nessesary information for storing, generating, and querying polices
//!
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::result::Result;

macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

/// Applys regret matching to regrets and returns a normalized strategy profile
fn regret_matching(regrets: &[f64]) -> Vec<f64> {
    let n = regrets.len();
    let mut norm_sum = 0.0;
    let mut strategy = vec![0f64; n];
    for a in 0..n {
        strategy[a] = max!(regrets[a], 0.0);
        norm_sum += strategy[a];
    }
    if norm_sum > 0.0 {
        for s in &mut strategy {
            *s /= norm_sum;
        }
    } else {
        let q = 1.0 / n as f64;
        for s in &mut strategy {
            *s = q;
        }
    }
    strategy
}

/// Samples an action index given a normalized strategy profile
#[inline(always)]
pub fn sample_action_index<R: Rng>(strategy_profile: &[f64], rng: &mut R) -> usize {
    let mut sum = 0.0;
    let z = rng.gen_range(0.0, 1.0);
    for (action_index, prob) in strategy_profile.iter().enumerate() {
        if sum <= z && z < (sum + prob) {
            return action_index;
        }
        sum += prob;
    }
    panic!("invalid normalized pdf");
}
/// Stores the regrets and cummulative strategy for an infoset
/// An infoset is the set of states containing all-infomation known by the current acting player
#[derive(Debug)]
pub struct Infoset {
    pub cummulative_regrets: Vec<f64>,
    pub cummulative_strategy: Vec<f64>,
}

/// Stores all infosets
/// Maps serialized game states to information sets
#[derive(Debug)]
pub struct InfosetTable {
    pub table: HashMap<String, Infoset>,
}

impl Infoset {
    /// intializes an empty infoset
    pub fn init(n_actions: usize) -> Self {
        Infoset {
            cummulative_regrets: vec![0f64; n_actions],
            cummulative_strategy: vec![0f64; n_actions],
        }
    }
    /// Apply regret matching to generate a normalize strategy using regrets
    pub fn current_strategy(&self) -> Vec<f64> {
        regret_matching(&self.cummulative_regrets)
    }
    /// Apply regret matching to generate a normalized strategy using cummulative strategy
    pub fn average_strategy(&self) -> Vec<f64> {
        regret_matching(&self.cummulative_strategy)
    }
    /// Returns the number of actions
    pub fn action_count(&self) -> usize {
        self.cummulative_regrets.len()
    }
    /// writes an infoset as bytes to a file
    /// first 4 bytes are the infosets length (number of actions)
    /// remaining bytes are regrets and then strategy
    pub fn write_to_file(&self, file: &mut BufWriter<File>) -> Result<(), Box<dyn Error>> {
        let length = self.action_count() as u32;
        let mut buffer = vec![];
        buffer.write_u32::<LittleEndian>(length)?;
        for r in &self.cummulative_regrets {
            buffer.write_f64::<LittleEndian>(*r)?;
        }
        for s in &self.cummulative_strategy {
            buffer.write_f64::<LittleEndian>(*s)?;
        }
        file.write_all(&buffer)?;
        Ok(())
    }
    /// reads an infoset from a file
    pub fn read_from_file(file: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        let length = file.read_u32::<LittleEndian>()? as usize;
        let mut iset = Infoset::init(length);
        for i in 0..length {
            iset.cummulative_regrets[i] = file.read_f64::<LittleEndian>()?;
        }
        for i in 0..length {
            iset.cummulative_strategy[i] = file.read_f64::<LittleEndian>()?;
        }
        Ok(iset)
    }
}

impl Default for InfosetTable {
    fn default() -> Self {
        InfosetTable {
            table: HashMap::default(),
        }
    }
}

impl InfosetTable {
    pub fn with_capacity(size: usize) -> InfosetTable {
        InfosetTable {
            table: HashMap::with_capacity(size),
        }
    }
    pub fn get_or_insert(&mut self, key: String, n_actions: usize) -> &mut Infoset {
        match self.table.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Infoset::init(n_actions)),
        }
    }
    pub fn get(&self, key: String) -> Option<&Infoset> {
        self.table.get(&key)
    }
    pub fn len(&self) -> usize {
        self.table.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// writes all information sets to a file
    /// currently uses a 10 byte buffer to store table keys
    pub fn write_to_file(&self, file: &mut BufWriter<File>) -> Result<(), Box<dyn Error>> {
        for (key, iset) in self.table.iter() {
            file.write_all(key.as_bytes())?;
            file.write_all(b"\n")?;
            iset.write_to_file(file)?;
        }
        file.flush()?;
        Ok(())
    }
    /// reads information sets from a file
    /// currently uses a 10 byte buffer to store table keys
    pub fn read_from_file(file: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        let mut infosets = InfosetTable::default();
        loop {
            let mut key = String::new();
            match file.read_line(&mut key) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    let iset = Infoset::read_from_file(file)?;
                    infosets.get_or_insert(key.trim_end().to_string(), iset.action_count());
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(infosets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::bench::Bencher;

    // UNIT TESTS
    #[test]
    fn test_regret_matching() {
        let regrets = vec![0.0, 0.0, 0.0];
        let strategy_profile = regret_matching(&regrets);
        for s in &strategy_profile {
            assert_eq!(*s, 0.3333333333333333);
        }
    }

    #[test]
    fn test_sample_action_index() {
        let strategy_profile = vec![1.0, 0.0, 0.0];
        let mut rng = rand::thread_rng();
        let ai = sample_action_index(&strategy_profile, &mut rng);
        assert_eq!(ai, 0);
        let strategy_profile = vec![0.0, 0.5, 0.5];
        let ai = sample_action_index(&strategy_profile, &mut rng);
        assert_ne!(ai, 0);
    }

    #[test]
    fn test_init_infoset() {
        let infoset = Infoset::init(5);
        assert_eq!(infoset.cummulative_regrets.len(), 5);
        assert_eq!(infoset.cummulative_strategy.len(), 5);
    }

    #[test]
    fn test_current_strategy() {
        let infoset = Infoset::init(3);
        let current_strategy = infoset.current_strategy();
        for a in &current_strategy {
            assert_eq!(*a, 0.3333333333333333);
        }
    }

    #[test]
    fn test_cummulative_strategy() {
        let infoset = Infoset::init(3);
        let average_strategy = infoset.average_strategy();
        for a in &average_strategy {
            assert_eq!(*a, 0.3333333333333333);
        }
    }

    #[test]
    fn test_rw_infoset() {
        let filename = "test_rw_infoset.dat";
        let file = File::create(filename).unwrap();
        let mut file = BufWriter::new(file);
        let mut rng = rand::thread_rng();
        let mut iset = Infoset::init(5);

        let regrets: Vec<f64> = (0..5)
            .into_iter()
            .map(|_| rng.gen_range(-10.0, 10.0))
            .collect();
        iset.cummulative_regrets[..5].clone_from_slice(&regrets[..5]);

        iset.write_to_file(&mut file).unwrap();
        file.flush().unwrap();

        let file = File::open(filename).unwrap();
        let mut file = BufReader::new(file);
        let iset = Infoset::read_from_file(&mut file).unwrap();

        for i in 0..5 {
            assert_eq!(regrets[i], iset.cummulative_regrets[i]);
        }

        std::fs::remove_file(filename).unwrap();
    }

    #[test]
    fn test_rw_infoset_table() {
        let filename = "test_rw_infoset_table.dat";
        let file = File::create(filename).unwrap();
        let mut file = BufWriter::new(file);
        let mut infosets = InfosetTable::default();

        let mut rng = rand::thread_rng();
        let regrets: Vec<f64> = (0..5)
            .into_iter()
            .map(|_| rng.gen_range(-10.0, 10.0))
            .collect();

        for i in 0..100 {
            let mut iset = Infoset::init(5);
            iset.cummulative_regrets[..5].clone_from_slice(&regrets[..5]);
            infosets.get_or_insert(i.to_string(), 5);
        }

        infosets.write_to_file(&mut file).unwrap();

        let file = File::open(filename).unwrap();
        let mut file = BufReader::new(file);
        let infosets = InfosetTable::read_from_file(&mut file).unwrap();

        assert_eq!(infosets.len(), 100);

        std::fs::remove_file(filename).unwrap();
    }

    #[test]
    fn test_get_or_insert() {
        let mut table = InfosetTable::default();
        table.get_or_insert("key".to_string(), 5);
        assert_eq!(table.len(), 1);
        table.get_or_insert("key".to_string(), 5);
        assert_eq!(table.len(), 1);
        table.get_or_insert("newkey".to_string(), 3);
        assert_eq!(table.len(), 2);
    }
    // BENCHMARKS
    #[bench]
    fn bench_regret_matching(b: &mut Bencher) {
        // 96 ns/iter (+/- 16)
        let regrets = vec![
            3.8901208570692436,
            8.03097953362121,
            -7.451353461846715,
            2.3215533257352217,
            7.020619986422162,
        ];
        b.iter(|| {
            regret_matching(&regrets);
        });
    }
    #[bench]
    fn bench_sample_action_index(b: &mut Bencher) {
        // 12 ns/iter (+/- 4)
        let mut rng = rand::thread_rng();
        let regrets: Vec<f64> = (0..5)
            .into_iter()
            .map(|_| rng.gen_range(-10.0, 10.0))
            .collect();
        let strategy_profile = regret_matching(&regrets);
        b.iter(|| {
            sample_action_index(&strategy_profile, &mut rng);
        });
    }

    #[bench]
    fn bench_write_infoset_table(b: &mut Bencher) {
        // 1,180,417 ns/iter (+/- 393,312)
        let filename = "bench_write_infoset_table.dat";
        let file = File::create(filename).unwrap();
        let mut file = BufWriter::new(file);
        let mut infosets = InfosetTable::default();

        for i in 0..1000 {
            infosets.get_or_insert(i.to_string(), 5);
        }

        b.iter(|| {
            infosets.write_to_file(&mut file).unwrap();
        });

        std::fs::remove_file(filename).unwrap();
    }

    #[bench]
    fn bench_read_infoset_table(b: &mut Bencher) {
        // 664,570 ns/iter (+/- 227,522)
        let filename = "bench_read_infoset_table.dat";
        let file = File::create(filename).unwrap();
        let mut file = BufWriter::new(file);
        let mut infosets = InfosetTable::default();

        for i in 0..1000 {
            infosets.get_or_insert(i.to_string(), 5);
        }

        infosets.write_to_file(&mut file).unwrap();

        b.iter(|| {
            let file = File::open(filename).unwrap();
            let mut file = BufReader::new(file);
            InfosetTable::read_from_file(&mut file).unwrap();
        });

        std::fs::remove_file(filename).unwrap();
    }
}
