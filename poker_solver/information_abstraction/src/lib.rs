#![feature(test)]
extern crate test;

pub fn split_into_batches(real_size: usize, size: usize) -> (usize, usize) {
    if (real_size % size) == 0 {
        return (real_size, size);
    }
    let q = real_size / size;
    let total_size = size * (q + 1);
    (total_size, total_size / size)
}

pub mod distance;
pub mod ehs;
pub mod histogram;
pub mod kmeans;
pub mod mpi_kmeans;
pub mod ochs;
