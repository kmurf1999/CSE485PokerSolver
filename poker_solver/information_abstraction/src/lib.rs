#![feature(test)]
extern crate test;

pub fn split_into_batches(real_size: usize, size: usize) -> (usize, usize) {
    let q = real_size / size;
    let batch_size = size * (q + 1);
    (batch_size, batch_size / size)
}

pub mod distance;
pub mod ehs;
pub mod histogram;
pub mod kmeans;
pub mod ochs;
