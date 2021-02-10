use information_abstraction::histogram::read_ehs_histograms;
use information_abstraction::kmeans::Kmeans;
use ndarray::prelude::*;
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
    let round = 0;
    let dim = 50;
    let n_samples = 30000;
    let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
    let k = 8;
    let indexer = HandIndexer::init(2, [2, 4].to_vec());

    let (mut classifier, _) = Kmeans::init_pp(k, &dataset, 200, true);
    // println!("intra_cluster_dist: {}", intra_cluster_dist);
    let _ = classifier.run(&dataset, 100);
    // println!("inertia: {},", inertia);

    let mut ranges = vec![String::new(); k];
    let mut cards = [0u8; 2];
    for i in 0usize..169 {
        indexer.get_hand(0, i as u64, &mut cards);
        ranges[classifier.cluster_assignments[i]] += cards_to_str(&cards).as_str();
        ranges[classifier.cluster_assignments[i]] += ",";
    }

    for i in 0..k {
        // println!("");
        println!("\"{}\",", ranges[i]);
    }
}
