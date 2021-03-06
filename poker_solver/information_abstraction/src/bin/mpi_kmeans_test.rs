use information_abstraction::distance;
use information_abstraction::histogram::read_ehs_histograms;
use information_abstraction::mpi_kmeans::MPIKmeans;
use mpi::traits::*;
use ndarray::Axis;
use rust_poker::constants::{RANK_TO_CHAR, SUIT_TO_CHAR};
use rust_poker::HandIndexer;
use std::iter::FromIterator;

pub fn cards_to_str(cards: &[u8]) -> String {
    let mut chars: Vec<char> = Vec::new();
    cards.iter().filter(|c| **c < 52).for_each(|c| {
        chars.push(RANK_TO_CHAR[usize::from(*c >> 2)]);
        chars.push(SUIT_TO_CHAR[usize::from(*c & 3)]);
    });
    String::from_iter(chars)
}

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let round = 0;
    let dim = 50;
    let dataset = read_ehs_histograms(round, dim).unwrap();
    let k = 8;
    let indexer = HandIndexer::init(1, [2].to_vec());

    let mut classifier = MPIKmeans::init_pp(world, k, &dataset, &distance::emd, 100, true);
    classifier.run(&dataset, world, &distance::emd, 100, true);

    if world.rank() == 0 {
        let mut ranges = vec![String::new(); k];
        let mut cards = [0u8; 2];
        for i in 0usize..169 {
            indexer.get_hand(0, i as u64, &mut cards);
            ranges[classifier.assignments[i]] += cards_to_str(&cards).as_str();
            ranges[classifier.assignments[i]] += ",";
        }
        for i in 0..k {
            // println!("");
            println!("\"{}\",", ranges[i]);
        }
    }
}
