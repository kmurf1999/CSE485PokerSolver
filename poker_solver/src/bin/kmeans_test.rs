#![feature(test)]
extern crate test;

use ndarray::prelude::*;
use ndarray::{Array2, Axis};
use poker_solver::card::cards_to_str;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};
use std::sync::Mutex;

// use rayon::prelude::*;
use ndarray::parallel::prelude::*;
use rust_poker::read_write::VecIO;
use rust_poker::HandIndexer;
use std::fs::File;

/// calculates a 1d emd in linear time
pub fn emd(p: &ArrayView1<f32>, q: &ArrayView1<f32>) -> f32 {
    let n = p.len();
    let mut dists = vec![0.0; n];
    for i in 0..n - 1 {
        dists[i + 1] = (p[i] + dists[i]) - q[i];
    }
    let dist = dists.iter().map(|d| d.abs()).sum::<f32>();
    return dist;
}

/// Return the l2 dist of two 1 dimension vectors
fn l2_dist(lhs: &ArrayView1<f32>, rhs: &ArrayView1<f32>) -> f32 {
    let mut sq_dist_sum = 0.0;
    for i in 0..lhs.len() {
        sq_dist_sum += (lhs[i] - rhs[i]).powf(2.0);
    }
    sq_dist_sum.sqrt()
}

// static ROUND_SIZE: &'static [usize] = &[169];

struct Kmeans {
    /// The number of centers
    k: usize,
    /// dimension of the data (how many bins in the histogram)
    dim: usize,
    /// how many datapoints we have
    n_data: usize,
    /// the actual cluster centers
    cluster_centers: Array2<f32>,
    /// which index each datapoint is assigned to
    pub cluster_assignments: Vec<usize>,
    // how many datapoints are in each cluster
    cluster_counts: Vec<usize>,
    // the sum of each datapoint in cluster i
    cluster_sums: Array2<f32>,
}

impl Kmeans {
    /// Initialize k cluster centers randomly with n restarts
    ///
    /// # Arguments
    /// * `k` number of cluster centers
    /// * `dataset` dataset to sample
    /// * `n_restarts`
    pub fn init_random(k: usize, dataset: &Array2<f32>, n_restarts: usize) -> (Self, f32) {
        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));
        let cluster_assignments: Vec<usize> = vec![0; n_data];
        let cluster_counts: Vec<usize> = vec![0; k];
        let cluster_sums = Array2::zeros((k, dim));

        let min_inertia = Mutex::new(f32::MAX);
        let final_cluster_centers = Mutex::new(Array2::zeros((k, dim)));

        (0..n_restarts).into_par_iter().for_each(|_| {
            let mut rng = SmallRng::from_entropy();
            let mut cluster_centers = Array2::zeros((k, dim));
            for i in 0..k {
                let random_index = rng.gen_range(0, n_data);
                cluster_centers
                    .slice_mut(s![i, ..])
                    .assign(&dataset.slice(s![random_index, ..]));
            }
            let mut inertia = 0f32;
            for i in 0..k {
                let mut min_dist = f32::MAX;
                let center_i = &cluster_centers.slice(s![i, ..]);
                for j in 0..k {
                    if i == j {
                        continue;
                    }
                    let center_j = &cluster_centers.slice(s![j, ..]);
                    let dist = emd(center_i, center_j);
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                inertia += min_dist;
            }
            let mut min_inertia = min_inertia.lock().unwrap();
            if inertia < *min_inertia {
                *min_inertia = inertia;
                *final_cluster_centers.lock().unwrap() = cluster_centers.to_owned();
            }
        });

        let classifier = Kmeans {
            k,
            cluster_centers: final_cluster_centers.into_inner().unwrap(),
            cluster_assignments,
            cluster_counts,
            cluster_sums,
            dim,
            n_data,
        };
        return (classifier, min_inertia.into_inner().unwrap());
    }
    /// Kmeans init plus plus
    pub fn init_pp(k: usize, dataset: &Array2<f32>) -> (Self, f32) {
        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));
        let cluster_assignments: Vec<usize> = vec![0; n_data];
        let cluster_counts: Vec<usize> = vec![0; k];
        let cluster_sums = Array2::zeros((k, dim));
        let mut cluster_centers = Array2::zeros((k, dim));
        let mut rng = thread_rng();

        let mut min_sq_dists = vec![f32::MAX; n_data];
        let mut last_chosen = rng.gen_range(0, n_data);
        // assign first cluster randomly
        cluster_centers
            .slice_mut(s![0, ..])
            .assign(&dataset.slice(s![last_chosen, ..]));

        for i in 1..k {
            // update min sq dists
            min_sq_dists
                .par_iter_mut()
                .enumerate()
                .for_each(|(j, min_sq_dist)| {
                    let dist = emd(
                        &dataset.slice(s![last_chosen, ..]),
                        &dataset.slice(s![j, ..]),
                    );
                    let dist_sq = dist * dist;
                    if dist_sq < *min_sq_dist {
                        *min_sq_dist = dist_sq;
                    }
                });
            let distribution = WeightedIndex::new(&min_sq_dists).unwrap();
            last_chosen = distribution.sample(&mut rng);
            cluster_centers
                .slice_mut(s![i, ..])
                .assign(&dataset.slice(s![last_chosen, ..]));
        }

        let inertia = min_sq_dists.iter().map(|d| d.sqrt()).sum::<f32>();

        let classifier = Kmeans {
            k,
            cluster_centers,
            cluster_assignments,
            cluster_counts,
            cluster_sums,
            dim,
            n_data,
        };
        return (classifier, inertia);
    }

    fn initialize_assignments(&mut self, dataset: &Array2<f32>) {
        for i in 0..self.n_data {
            let datapoint = dataset.slice(s![i, ..]);
            let mut min_dist = f32::MAX;
            let mut min_idx = 0usize;
            for j in 0..self.k {
                let dist = emd(&datapoint, &self.cluster_centers.slice(s![j, ..]));
                if dist < min_dist {
                    min_dist = dist;
                    min_idx = j;
                }
            }
            // update cluster assignments
            self.cluster_assignments[i] = min_idx;
            // update cluster counts
            self.cluster_counts[min_idx] += 1;
            // update cluster sums
            self.cluster_sums
                .slice_mut(s![min_idx, ..])
                .scaled_add(1.0, &datapoint);
        }
    }

    fn move_clusters(&mut self) {
        for i in 0..self.k {
            let mut new_center = self.cluster_sums.slice(s![i, ..]).to_owned();
            // divide cluster_sums[k] by the number of datapoints in it to get the average
            new_center /= self.cluster_counts[i] as f32;
            self.cluster_centers
                .slice_mut(s![i, ..])
                .assign(&new_center);
        }
    }
    /// Move datapoints to their nearest cluster
    /// return the number of datapoints that have changed
    /// and the sum of point to cluster
    ///
    fn reassign_clusters(&mut self, dataset: &Array2<f32>) -> (usize, f32) {
        let mut changed = 0;
        let mut dist_sum = 0f32;
        for i in 0..self.n_data {
            let old_assignment = self.cluster_assignments[i];
            let datapoint = dataset.slice(s![i, ..]);
            let mut min_dist = f32::MAX;
            let mut min_idx = 0usize;
            for j in 0..self.k {
                let dist = emd(&datapoint, &self.cluster_centers.slice(s![j, ..]));
                if dist < min_dist {
                    min_dist = dist;
                    min_idx = j;
                }
            }
            dist_sum += min_dist;
            if min_idx != old_assignment {
                // increment changed
                changed += 1;
                // assign new cluster
                self.cluster_assignments[i] = min_idx;
                // remove old cluster and update new
                self.cluster_counts[old_assignment] -= 1;
                self.cluster_counts[min_idx] += 1;
                // remove old cluster sum and update new
                self.cluster_sums
                    .slice_mut(s![old_assignment, ..])
                    .scaled_add(-1.0, &datapoint);
                self.cluster_sums
                    .slice_mut(s![min_idx, ..])
                    .scaled_add(1.0, &datapoint);
            }
        }
        return (changed, dist_sum);
    }

    pub fn run(&mut self, dataset: &Array2<f32>) -> f32 {
        self.n_data = dataset.len_of(Axis(0));
        self.dim = dataset.len_of(Axis(1));
        // initialize assignments
        self.initialize_assignments(dataset);

        let final_dist_sum;
        loop {
            self.move_clusters();
            let (changed, dist_sum) = self.reassign_clusters(dataset);
            if changed == 0 {
                final_dist_sum = dist_sum;
                break;
            }
        }
        return final_dist_sum;
    }
}

/// Reads histogram data from file and returns a 2D array
///
/// # Arguments
/// * `round` the round of data to be read (0 is preflop, 4 is river)
/// * `dim` the dimension of the historgram (number of bins)
/// * `n_samples` the number of samples per histogram
pub fn read_histogram_data(
    round: usize,
    dim: usize,
    n_samples: usize,
) -> Result<Array2<f32>, Box<dyn std::error::Error>> {
    let mut file = File::open(format!("hist-r{}-d{}-s{}.dat", round, dim, n_samples))?;
    let flat_data = file.read_vec_from_file::<f32>()?;
    // TODO handle shape error instead
    let data = Array2::from_shape_vec((flat_data.len() / dim, dim), flat_data)?;
    Ok(data)
}

fn main() {
    let round = 0;
    let dim = 20;
    let n_samples = 5000;
    let dataset = read_histogram_data(round, dim, n_samples).unwrap();
    let k = 8;

    let indexer = HandIndexer::init(1, [2].to_vec());

    let (mut classifier, inertia) = Kmeans::init_pp(k, &dataset);
    // println!("inertia: {}", inertia);
    let dist_sum = classifier.run(&dataset);
    println!("{},", dist_sum);

    let mut ranges = vec![String::new(); k];
    let mut cards = [0u8; 2];
    for i in 0usize..169 {
        indexer.get_hand(0, i as u64, &mut cards);
        ranges[classifier.cluster_assignments[i]] += cards_to_str(&cards).as_str();
        ranges[classifier.cluster_assignments[i]] += ",";
    }

    for i in 0..k {
        println!("");
        println!("\"{}\",", ranges[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_init_random(b: &mut Bencher) {
        let round = 0;
        let dim = 10;
        let n_samples = 1000;
        let dataset = read_histogram_data(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            Kmeans::init_random(k, &dataset, 8);
        });
    }

    #[bench]
    fn bench_init_pp(b: &mut Bencher) {
        let round = 0;
        let dim = 10;
        let n_samples = 1000;
        let dataset = read_histogram_data(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            Kmeans::init_pp(k, &dataset);
        });
    }
}
