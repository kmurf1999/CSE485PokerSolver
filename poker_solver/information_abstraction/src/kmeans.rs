use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};

use std::io::{self, Write};
use std::sync::Mutex;
use std::time::Instant;

pub trait Kmeans {
    fn init_random(
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> Self;
    fn init_pp(
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> Self;
    /// Main K-Means method
    /// Iterates until convergence or until maximum number of iterations
    fn run(
        &mut self,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        max_iterations: usize,
    ) -> f32;
    /// Initialize k cluster centers randomly with n restarts
    /// returns an initialized Kmeans object and the sum of the squared minimum intra center distance
    ///
    /// # Arguments
    /// * `k` number of cluster centers
    /// * `dataset` dataset to sample
    /// * `n_restarts` number of restarts (best one is selected)
    /// * `verbose` enable print progress messages
    fn init_centers_random(
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> (Array2<f32>, f32) {
        let start_time = Instant::now();
        if verbose {
            println!(
                "starting K-means random initialization with {} restarts",
                n_restarts
            );
        }

        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));

        let min_intra_center_dist = Mutex::new(f32::MAX);
        let final_cluster_centers = Mutex::new(Array2::zeros((k, dim)));

        (0..n_restarts).into_par_iter().for_each(|restart| {
            if verbose {
                print!("restart: {}\r", restart);
                io::stdout().flush().unwrap();
            }
            let mut rng = SmallRng::from_entropy();
            let mut centers = Array2::zeros((k, dim));
            for i in 0..k {
                let random_index = rng.gen_range(0, n_data);
                centers
                    .slice_mut(s![i, ..])
                    .assign(&dataset.slice(s![random_index, ..]));
            }
            let mut intra_center_dist = 0f32;
            for i in 0..k {
                let mut min_dist = f32::MAX;
                let center_i = &centers.slice(s![i, ..]);
                for j in 0..k {
                    if i == j {
                        continue;
                    }
                    let center_j = &centers.slice(s![j, ..]);
                    let dist = dist_func(center_i, center_j);
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                intra_center_dist += min_dist * min_dist;
            }
            let mut min_intra_center_dist = min_intra_center_dist.lock().unwrap();
            if intra_center_dist < *min_intra_center_dist {
                *min_intra_center_dist = intra_center_dist;
                *final_cluster_centers.lock().unwrap() = centers;
            }
        });

        if verbose {
            let duration = start_time.elapsed().as_millis();
            println!("done. took {}ms", duration);
        }

        (
            final_cluster_centers.into_inner().unwrap(),
            min_intra_center_dist.into_inner().unwrap(),
        )
    }
    /// Initializes k cluster centers using the Kmeans++ method.
    /// Returns a `Kmeans` object and the sum of the squared minimum intra-center distance
    ///
    /// # Arguments
    /// * `k` number of centers
    /// * `dataset` reference to histogram dataset
    /// * `n_restarts` number of restarts (best one is selected)
    /// * `verbose` enable to print progress messages
    fn init_centers_pp(
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> (Array2<f32>, f32) {
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
            let mut centers = Array2::zeros((k, dim));
            let mut min_sq_dists = vec![f32::MAX; n_data];
            let mut last_chosen = rng.gen_range(0, n_data);
            // assign first cluster randomly
            centers
                .slice_mut(s![0, ..])
                .assign(&dataset.slice(s![last_chosen, ..]));
            for i in 1..k {
                // update min sq dists
                min_sq_dists
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(j, min_sq_dist)| {
                        let dist = dist_func(
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
                centers
                    .slice_mut(s![i, ..])
                    .assign(&dataset.slice(s![last_chosen, ..]));
            }
            let intra_center_dist = min_sq_dists.iter().sum::<f32>();
            if intra_center_dist < best_intra_center_dist {
                best_intra_center_dist = intra_center_dist;
                final_cluster_centers = centers;
            }
        }

        if verbose {
            let duration = start_time.elapsed().as_millis();
            println!("done. took {}ms", duration);
        }

        (final_cluster_centers, best_intra_center_dist)
    }
}

/// A struct used for clustering histograms and vectors using the Kmeans algorithm
/// Does not leverge the triangle inequality to reduce distance function calls
pub struct VanillaKmeans {
    /// The number of centers
    k: usize,
    /// dimension of the data (how many bins in the histogram)
    dim: usize,
    /// how many datapoints we have
    n_data: usize,
    /// the actual cluster centers
    centers: Array2<f32>,
    /// which index each datapoint is assigned to
    pub assignments: Vec<usize>,
    /// how many datapoints are in each cluster
    center_counts: Vec<f32>,
    /// the sum of each datapoint in cluster i
    center_sums: Array2<f32>,
}

impl VanillaKmeans {
    /// Initializes each datapoint to its closest center and updates `self.center_sums` and `self.center_counts`
    fn initialize_assignments(
        &mut self,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
    ) {
        let mut assignments = vec![0usize; self.n_data];
        assignments
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, assignment)| {
                let datapoint = dataset.slice(s![i, ..]);
                let mut min_dist = f32::MAX;
                let mut min_idx = 0usize;
                for j in 0..self.k {
                    let dist = dist_func(&datapoint, &self.centers.slice(s![j, ..]));
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
            self.center_counts[assignments[i]] += 1.0;
            // update cluster sums
            self.center_sums
                .slice_mut(s![assignments[i], ..])
                .scaled_add(1.0, &datapoint);
        }
        self.assignments = assignments;
    }

    fn move_clusters(&mut self) {
        // parallel
        let center_sums = &self.center_sums;
        let center_counts = &self.center_counts;
        self.centers
            .axis_iter_mut(Axis(0))
            .into_par_iter()
            .enumerate()
            .for_each(|(i, mut center)| {
                center.assign(&center_sums.slice(s![i, ..]));
                center /= center_counts[i];
            });
    }
    /// Move datapoints to their nearest cluster
    /// Returns the number of datapoints that have changed and the updated **inertia**
    /// **inertia** is the sum of the squared distance between each datapoint and its assigned center.
    fn reassign_clusters(
        &mut self,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
    ) -> (usize, f32) {
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
                    let dist = dist_func(&datapoint, &self.centers.slice(s![j, ..]));
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
            if new_assignments[i] != self.assignments[i] {
                let datapoint = dataset.slice(s![i, ..]);
                // increment changed
                changed += 1;
                // remove old cluster and update new
                self.center_counts[self.assignments[i]] -= 1.0;
                self.center_counts[new_assignments[i]] += 1.0;
                // remove old cluster sum and update new
                self.center_sums
                    .slice_mut(s![self.assignments[i], ..])
                    .scaled_add(-1.0, &datapoint);
                self.center_sums
                    .slice_mut(s![new_assignments[i], ..])
                    .scaled_add(1.0, &datapoint);

                // assign new cluster
                self.assignments[i] = new_assignments[i];
            }
        }
        (changed, inertia)
    }
}

impl Kmeans for VanillaKmeans {
    /// Initialize Vanilla K-Means with random center initializations
    fn init_random(
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> Self {
        let dim = dataset.len_of(Axis(1));
        let n_data = dataset.len_of(Axis(0));
        let (centers, inertia) =
            Self::init_centers_random(k, dataset, dist_func, n_restarts, verbose);
        VanillaKmeans {
            k,
            dim,
            n_data,
            assignments: Vec::new(),
            centers,
            center_sums: Array2::zeros((k, dim)),
            center_counts: vec![0f32; k],
        }
    }
    /// Initialize Vanilla K-Means using the K-Means++ initialization procedure
    fn init_pp(
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> Self {
        let dim = dataset.len_of(Axis(1));
        let n_data = dataset.len_of(Axis(0));
        let (centers, inertia) = Self::init_centers_pp(k, dataset, dist_func, n_restarts, verbose);
        VanillaKmeans {
            k,
            dim,
            n_data,
            assignments: Vec::new(),
            centers,
            center_sums: Array2::zeros((k, dim)),
            center_counts: vec![0f32; k],
        }
    }
    /// Runs K-Means until convergence or the maximum number of iterations
    /// Returns the **inertia** of the final center positions
    ///
    /// # Arguments
    /// * `dataset` an array of vectors or histograms
    /// * `max_iterations`
    fn run(
        &mut self,
        dataset: &Array2<f32>,
        dist_func: &'static (dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        max_iterations: usize,
    ) -> f32 {
        self.n_data = dataset.len_of(Axis(0));
        self.dim = dataset.len_of(Axis(1));
        // initialize assignments
        self.initialize_assignments(dataset, dist_func);
        let mut final_inertia = 0f32;
        for _ in 0..max_iterations {
            self.move_clusters();
            let (changed, inertia) = self.reassign_clusters(dataset, dist_func);
            final_inertia = inertia;
            println!("{}, {}", inertia, changed);
            if changed == 0 {
                break;
            }
        }
        final_inertia
    }
}

/// A structure for implementing the Hammerly K-Means algorithm
/// uses distance bounds to greatly reduce the number of distance function calls
pub struct HammerlyKmeans {
    /// The number of centers
    k: usize,
    /// dimension of the data (how many bins in the histogram)
    dim: usize,
    /// how many datapoints we have
    n_data: usize,
    /// the actual cluster centers
    centers: Array2<f32>,
    /// which index each datapoint is assigned to
    pub assignments: Vec<usize>,
    /// how many datapoints are in each cluster
    center_counts: Vec<f32>,
    /// the sum of each datapoint in cluster i
    center_sums: Array2<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distance;
    use crate::histogram::read_ehs_histograms;
    use test::Bencher;

    #[bench]
    // test kmeans::tests::bench_run         ... bench:   1,959,121 ns/iter (+/- 696,657)
    fn bench_run(b: &mut Bencher) {
        let round = 0;
        let dim = 50;
        let n_samples = 10000;
        let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            let mut classifier = VanillaKmeans::init_random(k, &dataset, &distance::emd, 1, false);
            classifier.run(&dataset, &distance::emd, 100);
        });
    }

    #[bench]
    // test kmeans::tests::bench_init_random ... bench:     187,293 ns/iter (+/- 16,493)
    fn bench_init_random(b: &mut Bencher) {
        let round = 0;
        let dim = 50;
        let n_samples = 10000;
        let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            VanillaKmeans::init_random(k, &dataset, &distance::emd, 25, false);
        });
    }

    #[bench]
    // test kmeans::tests::bench_init_pp     ... bench:     417,163 ns/iter (+/- 222,699)
    fn bench_init_pp(b: &mut Bencher) {
        let round = 0;
        let dim = 50;
        let n_samples = 10000;
        let dataset = read_ehs_histograms(round, dim, n_samples).unwrap();
        let k = 8;
        b.iter(|| {
            VanillaKmeans::init_pp(k, &dataset, &distance::emd, 1, false);
        });
    }
}
