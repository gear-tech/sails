//! Benchmarking utilities for Sails framework performance measurement.
//!
//! This module provides functionality to collect, store, and manage benchmark data
//! for various aspects of the Sails framework.
//! Benchmark data is persisted to a JSON file (`path_to_sails/benchmarks/bench_data.json`)
//! with file locking to ensure thread-safe concurrent access when running multiple benchmark tests.

#[cfg(all(test, not(debug_assertions)))]
mod benchmarks;
#[cfg(all(test, not(debug_assertions)))]
mod clients;

mod entities;
mod file;

use anyhow::{Context, Result};
pub use entities::{
    BenchCategory, BenchCategoryComparison, BenchCategoryComparisonReport, BenchData,
};
pub use file::BenchDataFile;
use std::{
    env,
    path::{Path, PathBuf},
};

pub fn store_bench_data(f: impl FnOnce(&mut BenchData)) -> Result<()> {
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bench_data.json");

    store_bench_data_to_file(path, f)
}

fn store_bench_data_to_file(path: impl AsRef<Path>, f: impl FnOnce(&mut BenchData)) -> Result<()> {
    let mut file = BenchDataFile::open(path).context("Failed to create `BenchDataFile`")?;

    file.lock_exclusive().unwrap_or_else(|e| {
        panic!("Failed to lock bench data file for writing: {e}");
    });

    let mut bench_data = file
        .read_bench_data()
        .context("Failed to read existing bench data")?;

    // Handle bench data
    f(&mut bench_data);

    // Write updated bench data.
    file.write_bench_data(bench_data)
        .context("Failed to update bench data")?;

    // Unlock the file
    file.unlock()
        .context("Failed to unlock bench data file after writing")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{BenchDataSerde, CounterBenchDataSerde};
    use std::{
        io::{Read, Seek, SeekFrom, Write},
        thread,
    };
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_not_overwritten() {
        // Create initial bench data.
        let initial_bench_data = BenchDataSerde {
            compute: 123,
            alloc: Default::default(),
            counter: CounterBenchDataSerde {
                async_call: 53,
                sync_call: 35,
            },
            cross_program: 42,
            redirect: 4242,
        };

        // Create a temporary file.
        let mut named_file = NamedTempFile::with_suffix(".json")
            .expect("Failed to create temporary file for testing");
        let path_h1 = named_file.path().to_path_buf();
        let path_h2 = named_file.path().to_path_buf();

        // Store initial bench data.
        {
            let mut file = named_file.as_file_mut();
            serde_json::to_writer_pretty(&mut file, &initial_bench_data)
                .expect("Failed to write serialized initial bench data to file");
            file.flush().expect("Failed to flush bench data to file");
            file.seek(SeekFrom::Start(0))
                .expect("Failed to seek to the start of the file");
        }

        // Spawn two threads to modify the bench data concurrently.
        let h1 = thread::spawn(move || {
            store_bench_data_to_file(path_h1, |bench_data| {
                bench_data.update_compute_bench(42);
                bench_data.update_cross_program_bench(0);
            })
            .unwrap();
        });

        let h2 = thread::spawn(move || {
            store_bench_data_to_file(path_h2, |bench_data| {
                bench_data.update_counter_bench(true, 84);
                bench_data.update_counter_bench(false, 126);
                bench_data.update_redirect_bench(4343);
            })
            .unwrap();
        });

        // Wait for both threads to finish.
        h1.join().expect("Thread 1 panicked");
        h2.join().expect("Thread 2 panicked");

        // Read the bench data from the file.
        let mut content = String::new();
        named_file
            .as_file_mut()
            .read_to_string(&mut content)
            .expect("Failed reading bench data bytes to string");
        let bench_data: BenchDataSerde =
            serde_json::from_str(&content).expect("Failed to deserialize bench data");

        // Check that the bench data was modified correctly.
        assert_eq!(
            bench_data,
            BenchDataSerde {
                compute: 42,
                alloc: Default::default(),
                counter: CounterBenchDataSerde {
                    async_call: 84,
                    sync_call: 126,
                },
                cross_program: 0,
                redirect: 4343,
            },
        )
    }
}
