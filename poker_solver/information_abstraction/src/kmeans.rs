use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};

use std::io::{self, Write};
use std::sync::Mutex;
use std::time::Instant;

use crate::distance::emd;

/// A struct used for clustering histograms and vectors using the Kmeans algorithm
/// Does not leverge the triangle inequality to reduce distance function calls
pub struct Kmeans {
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
    /// how many datapoints are in each cluster
    cluster_counts: Vec<f32>,
    /// the sum of each datapoint in cluster i
    cluster_sums: Array2<f32>,
}

impl Kmeans {
    /// Initialize k cluster centers randomly with n restarts
    /// returns an initialized Kmeans object and the sum of the squared minimum intra center distance
    ///
    /// # Arguments
    /// * `k` number of cluster centers
    /// * `dataset` dataset to sample
    /// * `n_restarts` number of restarts (best one is selected)
    /// * `verbose` enable print progress messages
    pub fn init_random(
        k: usize,
        dataset: &Array2<f32>,
        n_restarts: usize,
        verbose: bool,
    ) -> (Self, f32) {
        let start_time = Instant::now();
        if verbose {
            println!(
                "starting K-means random initialization with {} restarts",
                n_restarts
            );
        }

        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));
        let cluster_assignments: Vec<usize> = vec![0; n_data];
        let cluster_counts = vec![0.0; k];
        let cluster_sums = Array2::zeros((k, dim));

        let min_intra_center_dist = Mutex::new(f32::MAX);
        let final_cluster_centers = Mutex::new(Array2::zeros((k, dim)));

        (0..n_restarts).into_par_iter().for_each(|restart| {
            if verbose {
                print!("restart: {}\r", restart);
                io::stdout().flush().unwrap();
            }
            let mut rng = SmallRng::from_entropy();
            let mut cluster_centers = Array2::zeros((k, dim));
            for i in 0..k {
                let random_index = rng.gen_range(0, n_data);
                cluster_centers
                    .slice_mut(s![i, ..])
                    .assign(&dataset.slice(s![random_index, ..]));
            }
            let mut intra_center_dist = 0f32;
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
                intra_center_dist += min_dist * min_dist;
            }
            let mut min_intra_center_dist = min_intra_center_dist.lock().unwrap();
            if intra_center_dist < *min_intra_center_dist {
                *min_intra_center_dist = intra_center_dist;
                *final_cluster_centers.lock().unwrap() = cluster_centers;
            }
        });

        if verbose {
            let duration = start_time.elapsed().as_millis();
            println!("done. took {}ms", duration);
        }

        let classifier = Kmeans {
            k,
            cluster_centers: final_cluster_centers.into_inner().unwrap(),
            cluster_assignments,
            cluster_counts,
            cluster_sums,
            dim,
            n_data,
        };
        (classifier, min_intra_center_dist.into_inner().unwrap())
    }
    /// Initializes k cluster centers using the Kmeans++ method.
    /// Returns a `Kmeans` object and the sum of the squared minimum intra-center distance
    ///
    /// # Arguments
    /// * `k` number of centers
    /// * `dataset` reference to histogram dataset
    /// * `n_restarts` number of restarts (best one is selected)
    /// * `verbose` enable to print progress messages
    pub fn init_pp(
        k: usize,
        dataset: &Array2<f32>,
        n_restarts: usize,
        verbose: bool,
    ) -> (Self, f32) {
        let start_time = Instant::now();
        if verbose {
            println!(
                "starting K-means++ initialization with {} restarts",
                n_restarts
            );
        }

        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));

        let mut rng = thread_rng();

        let mut best_intra_center_dist = f32::MAX;
        let mut final_cluster_centers = Array2::zeros((k, dim));

        for restart in 0..n_restarts {
            if verbose {
                print!("restart: {}\r", restart);
                io::stdout().flush().unwrap();
            }
            let mut cluster_centers = Array2::zeros((k, dim));
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
            let intra_center_dist = min_sq_dists.iter().sum::<f32>();
            if intra_center_dist < best_intra_center_dist {
                best_intra_center_dist = intra_center_dist;
                final_cluster_centers = cluster_centers;
            }
        }

        if verbose {
            let duration = start_time.elapsed().as_millis();
            println!("done. took {}ms", duration);
        }

        let classifier = Kmeans {
            k,
            cluster_centers: final_cluster_centers,
            cluster_assignments: vec![0; n_data],
            cluster_counts: vec![0.0; k],
            cluster_sums: Array2::zeros((k, dim)),
            dim,
            n_data,
        };
        (classifier, best_intra_center_dist)
    }
    /// Initializes each datapoint to its closest center and updates `self.center_sums` and `self.center_counts`
    fn initialize_assignments(&mut self, dataset: &Array2<f32>) {
        let mut assignments = vec![0usize; self.n_data];
        assignments
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, assignment)| {
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
                *assignment = min_idx;
            });
        for i in 0..self.n_data {
            let datapoint = dataset.slice(s![i, ..]);
            // update cluster counts
            self.cluster_counts[assignments[i]] += 1.0;
            // update cluster sums
            self.cluster_sums
                .slice_mut(s![assignments[i], ..])
                .scaled_add(1.0, &datapoint);
        }
        self.cluster_assignments = assignments;
    }

    fn move_clusters(&mut self) {
        // parallel
        let cluster_sums = &self.cluster_sums;
        let cluster_counts = &self.cluster_counts;
        self.cluster_centers
            .axis_iter_mut(Axis(0))
            .into_par_iter()
            .enumerate()
            .for_each(|(i, mut center)| {
                center.assign(&cluster_sums.slice(s![i, ..]));
                center /= cluster_counts[i];
            });
    }
    /// Move datapoints to their nearest cluster
    /// Returns the number of datapoints that have changed and the updated **inertia**
    /// **inertia** is the sum of the squared distance between each datapoint and its assigned center.
    fn reassign_clusters(&mut self, dataset: &Array2<f32>) -> (usize, f32) {
        let mut changed = 0;
        let mut new_assignments = vec![0usize; self.n_data];

        let inertia = new_assignments
            .par_iter_mut()
            .enumerate()
            .map(|(i, assignment)| {
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
                *assignment = min_idx;
                min_dist * min_dist
            })
            .sum();

        for i in 0..self.n_data {
            if new_assignments[i] != self.cluster_assignments[i] {
                let datapoint = dataset.slice(s![i, ..]);
                // increment changed
                changed += 1;
                // remove old cluster and update new
                self.cluster_counts[self.cluster_assignments[i]] -= 1.0;
                self.cluster_counts[new_assignments[i]] += 1.0;
                // remove old cluster sum and update new
                self.cluster_sums
                    .slice_mut(s![self.cluster_assignments[i], ..])
                    .scaled_add(-1.0, &datapoint);
                self.cluster_sums
                    .slice_mut(s![new_assignments[i], ..])
                    .scaled_add(1.0, &datapoint);

                // assign new cluster
                self.cluster_assignments[i] = new_assignments[i];
            }
        }
        (changed, inertia)
    }
    /// Runs K-Means until convergence or the maximum number of iterations
    /// Returns the **inertia** of the final center positions
    ///
    /// # Arguments
    /// * `dataset` an array of vectors or histograms
    /// * `max_iterations`
    pub fn run(&mut self, dataset: &Array2<f32>, max_iterations: usize) -> f32 {
        self.n_data = dataset.len_of(Axis(0));
        self.dim = dataset.len_of(Axis(1));
        // initialize assignments
        self.initialize_assignments(dataset);
        let mut final_inertia = 0f32;
        for _ in 0..max_iterations {
            self.move_clusters();
            let (changed, inertia) = self.reassign_clusters(dataset);
            final_inertia = inertia;
            println!("{}, {}", inertia, changed);
            if changed == 0 {
                break;
            }
        }
        final_inertia
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::histogram::read_ehs_histograms;
    use test::Bencher;

    #[bench]
    fn bench_run(b: &mut Bencher) {
        let round = 0;
        let dim = 50;
        let n_samples = 10000;
        let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            let (mut classifier, _) = Kmeans::init_random(k, &dataset, 1, false);
            classifier.run(&dataset, 100);
        });
    }

    #[bench]
    fn bench_init_random(b: &mut Bencher) {
        let round = 0;
        let dim = 50;
        let n_samples = 10000;
        let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            Kmeans::init_random(k, &dataset, 25, false);
        });
    }

    #[bench]
    fn bench_init_pp(b: &mut Bencher) {
        let round = 0;
        let dim = 50;
        let n_samples = 10000;
        let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            Kmeans::init_pp(k, &dataset, 1, false);
        });
    }
}
