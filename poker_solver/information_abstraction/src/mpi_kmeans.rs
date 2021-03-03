use mpi::collective::{self, SystemOperation};
use mpi::topology::Rank;
use mpi::traits::*;
use rand::distributions::{Distribution, WeightedIndex};

use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use rand::rngs::SmallRng;
use rand::{thread_rng, Rng, SeedableRng};
use std::io::{self, Write};
use std::sync::atomic::{AtomicU32, Ordering};

use std::sync::Mutex;
use std::time::Instant;

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

pub struct MPIKmeans {
    k: usize,
    dim: usize,
    n_data: usize,
    centers: Array2<f32>,
    center_sums: Array2<f32>,
    center_counts: Vec<usize>,
    center_movements: Vec<f32>,
    s: Vec<f32>,
    /// only the root process uses this field
    pub assignments: Vec<usize>,
}

struct MPIKmeansNode {
    assignments: Vec<usize>,
    lower_bounds: Vec<f32>,
    upper_bounds: Vec<f32>,
    indicies: Vec<usize>,
}

impl MPIKmeans {
    /// Initializes Kmeans using random initialization and return the `MPIKmeans` object to the root process
    pub fn init_random(
        world: mpi::topology::SystemCommunicator,
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> Self {
        let rank = world.rank();
        let size = world.size();
        let is_root = rank == 0;
        let root_process = world.process_at_rank(0);

        let start_time = Instant::now();
        if is_root && verbose {
            println!(
                "starting K-means random initialization with {} restarts",
                n_restarts
            );
        }

        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));
        let counter = AtomicU32::new(0);

        let max_intra_center_dist = Mutex::new(0f32);
        let final_cluster_centers = Mutex::new(Array2::zeros((k, dim)));

        (0..(max!(1, n_restarts / size as usize)))
            .into_par_iter()
            .for_each(|_| {
                let c = counter.fetch_add(1, Ordering::SeqCst);
                if is_root && verbose {
                    print!("restart: {}\r", c * size as u32);
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
                let mut max_intra_center_dist = max_intra_center_dist.lock().unwrap();
                if intra_center_dist > *max_intra_center_dist {
                    *max_intra_center_dist = intra_center_dist;
                    *final_cluster_centers.lock().unwrap() = centers;
                }
            });

        let max_intra_center_dist = max_intra_center_dist.into_inner().unwrap();
        let mut final_cluster_centers = final_cluster_centers.into_inner().unwrap();

        let mut world_max_dist: f32 = 0.0;
        world.all_reduce_into(
            &max_intra_center_dist,
            &mut world_max_dist,
            SystemOperation::max(),
        );

        // are we the node with the best seeding?
        let is_best_node = (max_intra_center_dist - world_max_dist).abs() < 1.0e-9;
        // if root is best node we don't have to send anything
        if is_best_node && is_root {
            // do nothing
        } else {
            // send our clusters to root
            if is_best_node {
                // send to root
                world
                    .process_at_rank(0)
                    .send(&final_cluster_centers.as_slice().unwrap()[..]);
            }
            // receive best centers as root
            if is_root {
                let (centers, status) = world.any_process().receive_vec::<f32>();
                final_cluster_centers = Array2::from_shape_vec((k, dim), centers).unwrap();
            }
        }

        if is_root && verbose {
            let duration = start_time.elapsed().as_millis();
            println!(
                "done. took {}ms. sum(intra-center dist^2) = {}",
                duration, world_max_dist,
            );
        }

        root_process.broadcast_into(&mut final_cluster_centers.as_slice_mut().unwrap()[..]);

        MPIKmeans {
            k,
            dim,
            n_data,
            assignments: Vec::new(),
            centers: final_cluster_centers,
            center_sums: Array2::zeros((k, dim)),
            center_counts: vec![0usize; k],
            center_movements: vec![0f32; k],
            s: vec![f32::MAX; k],
        }
    }
    /// Initializes Kmeans using k++ initialization and return the `MPIKmeans` object to the root process
    pub fn init_pp(
        world: mpi::topology::SystemCommunicator,
        k: usize,
        dataset: &Array2<f32>,
        dist_func: &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        n_restarts: usize,
        verbose: bool,
    ) -> Self {
        // for i in 0..n_restarts
        //  create a min sq dists array
        //  scatter array and indexes
        //  calculate new min sq dists
        //  root decides on next
        let rank = world.rank();
        let size = world.size() as usize;
        let is_root = rank == 0;
        let root_process = world.process_at_rank(0);
        let start_time = Instant::now();
        let n_data = dataset.len_of(Axis(0));
        let dim = dataset.len_of(Axis(1));
        let mut rng = thread_rng();

        if is_root && verbose {
            println!(
                "starting K-means++ initialization with {} restarts",
                n_restarts
            );
        }

        // send indicies to each node
        let (total_size, batch_size) = crate::split_into_batches(n_data, size);
        let all_indicies: Vec<usize> = (0..total_size).into_iter().collect();
        let mut batch_indicies: Vec<usize> = vec![0usize; batch_size];
        if is_root {
            root_process.scatter_into_root(&all_indicies[..], &mut batch_indicies[..]);
        } else {
            root_process.scatter_into(&mut batch_indicies[..]);
        }

        // only root needs to keep track of these
        let mut best_intra_center_dist = 0f32;
        let mut final_cluster_centers = Array2::zeros((k, dim));

        for restart in 0..n_restarts {
            if is_root && verbose {
                print!("restart: {}\r", restart);
                io::stdout().flush().unwrap();
            }
            let mut centers = Array2::zeros((0, 0));
            let mut all_min_sq_dists: Vec<f32> = Vec::new();
            let mut batch_min_sq_dists = vec![f32::MAX; batch_size];
            if is_root {
                centers = Array2::zeros((k, dim));
                all_min_sq_dists = vec![f32::MAX; total_size];
            }
            let mut last_chosen: usize = 0;
            // send the chosen index to all nodes
            if is_root {
                last_chosen = rng.gen_range(0, n_data);
            }
            root_process.broadcast_into(&mut last_chosen);

            // assign first cluster randomly
            if is_root {
                centers
                    .slice_mut(s![0, ..])
                    .assign(&dataset.slice(s![last_chosen, ..]));
            }

            for i in 1..k {
                // update min sq dists
                batch_min_sq_dists
                    .par_iter_mut()
                    .zip(&batch_indicies)
                    .for_each(|(min_sq_dist, j)| {
                        if *j >= n_data {
                            return;
                        }
                        let dist = dist_func(
                            &dataset.slice(s![last_chosen, ..]),
                            &dataset.slice(s![*j, ..]),
                        );
                        let dist_sq = dist * dist;
                        if dist_sq < *min_sq_dist {
                            *min_sq_dist = dist_sq;
                        }
                    });

                // gather min square dists into root
                if is_root {
                    root_process.gather_into_root(
                        &batch_min_sq_dists.as_slice()[..],
                        &mut all_min_sq_dists.as_mut_slice()[..],
                    );
                } else {
                    root_process.gather_into(&batch_min_sq_dists.as_slice()[..]);
                }
                // sample
                if is_root {
                    let distribution = WeightedIndex::new(&all_min_sq_dists[0..n_data]).unwrap();
                    last_chosen = distribution.sample(&mut rng);
                    centers
                        .slice_mut(s![i, ..])
                        .assign(&dataset.slice(s![last_chosen, ..]));
                }
                root_process.broadcast_into(&mut last_chosen);
            }

            if is_root {
                let intra_center_dist: f32 = (0..k)
                    .into_par_iter()
                    .map(|i| {
                        let mut min_dist = f32::MAX;
                        for j in 0..k {
                            if i == j {
                                continue;
                            }
                            let d = dist_func(&centers.slice(s![i, ..]), &centers.slice(s![j, ..]));
                            if d < min_dist {
                                min_dist = d;
                            }
                        }
                        min_dist * min_dist
                    })
                    .sum();
                if intra_center_dist > best_intra_center_dist {
                    best_intra_center_dist = intra_center_dist;
                    final_cluster_centers = centers;
                }
            }
        }

        if is_root && verbose {
            let duration = start_time.elapsed().as_millis();
            println!(
                "done. took {}ms. sum(intra-center dist^2) = {}",
                duration, best_intra_center_dist,
            );
        }

        root_process.broadcast_into(&mut final_cluster_centers.as_slice_mut().unwrap()[..]);

        MPIKmeans {
            k,
            dim,
            n_data,
            assignments: Vec::new(),
            centers: final_cluster_centers,
            center_sums: Array2::zeros((k, dim)),
            center_counts: vec![0usize; k],
            center_movements: vec![0f32; k],
            s: vec![f32::MAX; k],
        }
    }
    pub fn run(
        &mut self,
        dataset: &Array2<f32>,
        world: mpi::topology::SystemCommunicator,
        dist_func: &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
        max_iterations: usize,
        verbose: bool,
    ) {
        let size = world.size() as usize;
        let rank = world.rank();
        let is_root = rank == 0;
        let root_process = world.process_at_rank(0);
        let start_time = Instant::now();
        if is_root && verbose {
            println!(
                "starting Hammery K-means. max iterations: {}",
                max_iterations
            );
        }
        self.n_data = dataset.len_of(Axis(0));
        self.dim = dataset.len_of(Axis(1));

        let (total_size, batch_size) = crate::split_into_batches(self.n_data, size);
        let all_indicies: Vec<usize> = (0..total_size).into_iter().collect();
        let mut node = MPIKmeansNode {
            lower_bounds: vec![0f32; batch_size],
            upper_bounds: vec![f32::MAX; batch_size],
            assignments: vec![0usize; batch_size],
            indicies: vec![0usize; batch_size],
        };
        if is_root {
            root_process.scatter_into_root(&all_indicies[..], &mut node.indicies[..]);
        } else {
            root_process.scatter_into(&mut node.indicies[..]);
        }

        // assign data to closest centers
        // move centers
        for iteration in 0..max_iterations {
            self.update_s(world, dist_func);
            let (changed, inertia) = self.update_assignments(dataset, world, &mut node, dist_func);
            if is_root && verbose {
                println!(
                    "iteration: {}, inertia: {}, changed: {}",
                    iteration, inertia, changed
                );
            }
            self.move_centers(world, dist_func);
            let furthest_moving_center = self.update_bounds(world, &mut node);
            if furthest_moving_center == 0.0 {
                break;
            }
        }

        if is_root && verbose {
            let duration = start_time.elapsed().as_millis();
            println!("done. took {}ms", duration);
        }

        if is_root {
            self.assignments = vec![0usize; total_size];
            root_process.gather_into_root(&node.assignments[..], &mut self.assignments[..]);
        } else {
            root_process.gather_into(&node.assignments[..]);
        }
    }
    fn update_bounds(
        &mut self,
        world: mpi::topology::SystemCommunicator,
        node: &mut MPIKmeansNode,
    ) -> f32 {
        // let assignments = &self.assignments;
        let center_movements = &self.center_movements;
        let mut furthest_moving_center = 0;
        let mut longest = center_movements[0];
        let mut second_longest = if 1 < self.k {
            center_movements[1]
        } else {
            center_movements[0]
        };
        if longest < second_longest {
            furthest_moving_center = 1;
            std::mem::swap(&mut longest, &mut second_longest);
        }
        // we could parlellize this, but it probably wouldn't result in a practical difference
        for i in 2..self.k {
            if longest < center_movements[i] {
                second_longest = longest;
                longest = center_movements[i];
                furthest_moving_center = i;
            } else if second_longest < center_movements[i] {
                second_longest = center_movements[i];
            }
        }
        node.upper_bounds
            .par_iter_mut()
            .zip(node.lower_bounds.par_iter_mut())
            .zip(node.assignments.par_iter())
            .zip(node.indicies.par_iter())
            .for_each(|(((ub, lb), assignment), index)| {
                if *index >= self.n_data {
                    return;
                }
                *ub += center_movements[*assignment];
                *lb -= if *assignment == furthest_moving_center {
                    second_longest
                } else {
                    longest
                }
            });
        longest
    }

    fn update_assignments(
        &mut self,
        dataset: &Array2<f32>,
        world: mpi::topology::SystemCommunicator,
        node: &mut MPIKmeansNode,
        dist_func: &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
    ) -> (usize, f32) {
        // get world paramaters
        let rank = world.rank();
        let size = world.size() as usize;
        let is_root = rank == 0;
        let root_process = world.process_at_rank(0);
        // get batch indicies and scatter them
        let mut center_counts = vec![0usize; self.k];
        let mut center_sums: Array2<f32> = Array2::zeros((self.k, self.dim));
        self.center_sums = Array2::zeros((self.k, self.dim));
        self.center_counts = vec![0usize; self.k];
        //
        let batch_changed = AtomicU32::new(0);
        let mut total_changed = 0;
        let batch_inertia: f32;
        let mut total_inertia = 0f32;
        // create a copy of initial assignments
        let mut batch_assignments = node.assignments.to_owned();
        batch_assignments
            .par_iter_mut()
            .zip(node.indicies.par_iter())
            .zip(node.lower_bounds.par_iter_mut())
            .zip(node.upper_bounds.par_iter_mut())
            .for_each(|(((ba, bi), lb), ub)| {
                if *bi >= self.n_data {
                    return;
                }
                let mut closest = *ba;
                let upper_comp_bound = max!(self.s[closest], *lb);
                if *ub <= upper_comp_bound {
                    return;
                }
                *ub = dist_func(
                    &dataset.slice(s![*bi, ..]),
                    &self.centers.slice(s![closest, ..]),
                );
                if *ub <= upper_comp_bound {
                    return;
                }
                let mut lower = f32::MAX;
                let mut upper = *ub;
                for j in 0..self.k {
                    if j == closest {
                        continue;
                    }
                    let dist =
                        dist_func(&dataset.slice(s![*bi, ..]), &self.centers.slice(s![j, ..]));
                    if dist < upper {
                        lower = upper;
                        upper = dist;
                        closest = j;
                    } else if dist < lower {
                        lower = dist;
                    }
                }
                *lb = lower;
                if *ba != closest {
                    batch_changed.fetch_add(1, Ordering::SeqCst);
                    *ba = closest;
                    *ub = upper;
                }
            });
        node.assignments = batch_assignments;

        node.assignments
            .iter()
            .zip(&node.indicies)
            .for_each(|(ba, bi)| {
                if *bi >= self.n_data {
                    return;
                }
                center_counts[*ba] += 1;
                center_sums
                    .slice_mut(s![*ba, ..])
                    .scaled_add(1.0, &dataset.slice(s![*bi, ..]));
            });
        // sum center counts
        world.all_reduce_into(
            &center_counts.as_slice()[..],
            &mut self.center_counts.as_mut_slice()[..],
            SystemOperation::sum(),
        );
        // sum center sums
        world.all_reduce_into(
            &center_sums.as_slice().unwrap()[..],
            &mut self.center_sums.as_slice_mut().unwrap()[..],
            SystemOperation::sum(),
        );
        world.all_reduce_into(
            &batch_changed.into_inner(),
            &mut total_changed,
            SystemOperation::sum(),
        );
        // get batch inertia
        batch_inertia = node
            .upper_bounds
            .par_iter()
            .zip(node.indicies.par_iter())
            .map(|(ub, bi)| {
                if *bi >= self.n_data {
                    return 0.0;
                }
                *ub * *ub
            })
            .sum();
        // sum inertia
        world.all_reduce_into(&batch_inertia, &mut total_inertia, SystemOperation::sum());
        (total_changed, total_inertia)
    }
    fn move_centers(
        &mut self,
        world: mpi::topology::SystemCommunicator,
        dist_func: &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
    ) {
        let rank = world.rank();
        let root_process = world.process_at_rank(0);
        let is_root = rank == 0;

        if is_root {
            let center_sums = &self.center_sums;
            let center_counts = &self.center_counts;
            self.center_movements = self
                .centers
                .axis_iter_mut(Axis(0))
                .into_par_iter()
                .enumerate()
                .map(|(i, mut center)| {
                    let mut new_center = center_sums.slice(s![i, ..]).to_owned();
                    new_center /= center_counts[i] as f32;
                    let dist = dist_func(&center.view(), &new_center.view());
                    center.assign(&new_center);
                    dist
                })
                .collect();
        }
        root_process.broadcast_into(&mut self.center_movements.as_mut_slice()[..]);
        root_process.broadcast_into(&mut self.centers.as_slice_mut().unwrap()[..]);
    }
    fn update_s(
        &mut self,
        world: mpi::topology::SystemCommunicator,
        dist_func: &(dyn Fn(&ArrayView1<f32>, &ArrayView1<f32>) -> f32 + Sync),
    ) {
        let rank = world.rank();
        let is_root = rank == 0;
        let root_process = world.process_at_rank(0);
        let mut s = vec![f32::MAX; self.k];
        if is_root {
            s.par_iter_mut().enumerate().for_each(|(i, si)| {
                *si = f32::MAX;
                for j in 0..self.k {
                    if i == j {
                        continue;
                    }
                    let dist = dist_func(
                        &self.centers.slice(s![i, ..]),
                        &self.centers.slice(s![j, ..]),
                    );
                    if dist < *si {
                        *si = dist;
                    }
                }
                *si /= 2.0;
            });
        }
        root_process.broadcast_into(&mut s.as_mut_slice()[..]);
        self.s = s;
    }
}

impl MPIKmeansNode {}
