/// Contains functions for calculating distances between vectors and histograms
use ndarray::prelude::*;

/// calculates the Earth Mover's Distance between two one-dimensional histograms in linear time
/// Both `p` and `q` must be **normalized** histograms
///
/// # Arguments
/// * `p` the first normalized histogram
/// * `q` the second normalize histogram
pub fn emd(p: &ArrayView1<f32>, q: &ArrayView1<f32>) -> f32 {
    let n = p.len();
    let mut dists = vec![0.0; n];
    for i in 0..n - 1 {
        dists[i + 1] = (p[i] + dists[i]) - q[i];
    }
    dists.iter().map(|d| d.abs()).sum::<f32>()
}

/// Calculates the L2 distance between two vectors
fn l2(lhs: &ArrayView1<f32>, rhs: &ArrayView1<f32>) -> f32 {
    let mut sq_dist_sum = 0.0;
    for i in 0..lhs.len() {
        sq_dist_sum += (lhs[i] - rhs[i]).powf(2.0);
    }
    sq_dist_sum.sqrt()
}
