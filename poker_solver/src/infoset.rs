//! Contains code for storing information sets
//!
//! An information set contains all nessesary information for storing, generating, and querying polices
//!
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
use std::collections::hash_map::{Entry, Iter};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::result::Result;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

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
    pub table: RwLock<HashMap<String, Mutex<HashMap<String, Infoset>>>>,
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
            table: Default::default(),
        }
    }
}

impl InfosetTable {
    /// writes all information sets to a file
    pub fn write_to_file(&self, file: &mut BufWriter<File>) -> Result<(), Box<dyn Error>> {
        let table = self.table.read().unwrap();
        for (private_key, child_table) in table.iter() {
            let child_table = child_table.lock().unwrap();
            for (public_key, infoset) in child_table.iter() {
                file.write_all(format!("{}-{}\n", private_key, public_key).as_bytes())?;
                infoset.write_to_file(file)?;
            }
        }
        file.flush()?;
        Ok(())
    }
    /// reads information sets from a file
    pub fn read_from_file(file: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        let infosets = InfosetTable::default();
        {
            let mut table = infosets.table.write().unwrap();
            loop {
                let mut key = String::new();
                match file.read_line(&mut key) {
                    Ok(size) => {
                        if size == 0 {
                            break;
                        }
                        let split_key: Vec<&str> = key.trim_end().split('-').collect();
                        let private_key = split_key[0];
                        let public_key = split_key[1];
                        let iset = Infoset::read_from_file(file)?;
                        let child_table = match table.entry(private_key.to_string()) {
                            Entry::Occupied(o) => o.into_mut(),
                            Entry::Vacant(v) => v.insert(Default::default()),
                        };
                        child_table
                            .get_mut()
                            .unwrap()
                            .insert(public_key.to_string(), iset);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }
        Ok(infosets)
    }

    fn private_key_exists(&self, key: &str) -> bool {
        self.table.read().unwrap().contains_key(key)
    }

    pub fn insert_private_key(&self, key: String) {
        if !self.private_key_exists(&key) {
            self.table.write().unwrap().insert(key, Default::default());
        }
    }

    pub fn get_strategy(
        &self,
        private_key: &str,
        public_key: String,
        n_actions: usize,
    ) -> Vec<f64> {
        match self.table.read().unwrap().get(private_key) {
            Some(table) => {
                let mut table = table.lock().unwrap();
                let infoset = table
                    .entry(public_key)
                    .or_insert_with(|| Infoset::init(n_actions));

                infoset.current_strategy()
            }
            None => panic!("history key should exist in table"),
        }
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
        let infosets = InfosetTable::default();

        let mut rng = rand::thread_rng();

        for i in 0..100 {
            infosets.insert_private_key(format!("{}", i));
            let table = infosets.table.read().unwrap();
            let mut child_table = table.get(&format!("{}", i)).unwrap().lock().unwrap();
            child_table.insert("D".to_string(), Infoset::init(3));
        }
        infosets.write_to_file(&mut file).unwrap();

        let file = File::open(filename).unwrap();
        let mut file = BufReader::new(file);
        let infosets = InfosetTable::read_from_file(&mut file).unwrap();

        let table = infosets.table.read().unwrap();
        for i in 0..100 {
            let child_table = table.get(&format!("{}", i));
            assert!(child_table.is_some());
            let ct = child_table.unwrap().lock().unwrap();
            assert!(ct.get("D").is_some());
        }

        std::fs::remove_file(filename).unwrap();
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
        // 747,745 ns/iter (+/- 145,955)
        let filename = "bench_write_infoset_table.dat";
        let file = File::create(filename).unwrap();
        let mut file = BufWriter::new(file);
        let infosets = Arc::new(InfosetTable::default());

        for i in 0..1000 {
            infosets.insert_private_key(format!("{}", i));
            let table = infosets.table.read().unwrap();
            let mut child_table = table.get(&format!("{}", i)).unwrap().lock().unwrap();
            child_table.insert("D".to_string(), Infoset::init(3));
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
        let infosets = InfosetTable::default();

        for i in 0..1000 {
            infosets.insert_private_key(format!("{}", i));
            let table = infosets.table.read().unwrap();
            let mut child_table = table.get(&format!("{}", i)).unwrap().lock().unwrap();
            child_table.insert("D".to_string(), Infoset::init(3));
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
