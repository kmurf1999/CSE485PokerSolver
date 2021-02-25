extern crate mpi;

// use mpi::topology::Rank;
use mpi::traits::*;

fn get_batch_size(real_size: usize, size: usize) -> (usize, usize) {
    let q = real_size / size;
    let batch_size = size * (q + 1);
    (batch_size, batch_size / size)
}

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank() as usize;
    let size = world.size() as usize;
    let root_process = world.process_at_rank(0);

    let n = 169;

    let (total_size, batch_size) = get_batch_size(n, size);

    let mut total = Vec::new();
    if rank == 0 {
        let mut t = vec![0usize; total_size];
        for i in 0..n {
            t[i] = i;
        }
        total = t;
    }
    let mut batch = vec![0usize; batch_size];

    if rank == 0 {
        root_process.scatter_into_root(&total[..], &mut batch[..]);
    } else {
        root_process.scatter_into(&mut batch[..]);
    }

    for i in 0..batch.len() {
        batch[i] *= 2;
    }

    if rank == 0 {
        root_process.gather_into_root(&batch[..], &mut total[..]);
    } else {
        root_process.gather_into(&batch[..]);
    }

    if rank == 0 {
        println!("{:?}", total);
    }
}