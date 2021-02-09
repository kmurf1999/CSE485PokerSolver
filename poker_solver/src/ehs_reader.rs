use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::io::SeekFrom;
use std::mem::size_of;

use rust_poker::HandIndexer;

/// Structure to read f32 EHS values from an EHS.dat file
pub struct EHSReader {
    pub indexers: [HandIndexer; 4],
    offsets: [u64; 4],
    file: File,
}

impl EHSReader {
    /// Creates a reader
    pub fn new() -> Result<Self, Error> {
        let indexers = [
            HandIndexer::init(1, [2].to_vec()),
            HandIndexer::init(2, [2, 3].to_vec()),
            HandIndexer::init(2, [2, 4].to_vec()),
            HandIndexer::init(2, [2, 5].to_vec()),
        ];
        let mut offsets: [u64; 4] = [0; 4];
        for i in 1..4 {
            offsets[i] = offsets[i - 1] + indexers[i - 1].size(if i == 1 { 0 } else { 1 });
        }
        Ok(EHSReader {
            indexers,
            offsets,
            file: File::open("EHS.dat")?,
        })
    }

    /// Gets the EHS for a specific hand
    ///
    /// # Arguments
    ///
    /// * `cards` an array of 8 bit cards (first two indices are hole cards)
    /// * `round` round to evaluate (0 -> preflop, 4 -> river)
    pub fn get_ehs(&self, cards: &[u8], round: usize) -> Result<f32, Error> {
        assert!(round < 5);
        let mut reader = BufReader::with_capacity(size_of::<f32>(), &self.file);
        let index = self.indexers[round].get_index(cards);
        reader.seek(SeekFrom::Start(
            (index + self.offsets[round]) * size_of::<f32>() as u64,
        ))?;
        let buffer = reader.fill_buf()?;
        let ehs = f32::from_le_bytes(buffer.try_into().expect("slice length is incorrect"));
        Ok(ehs)
    }
}
