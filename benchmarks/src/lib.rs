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

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchData {
    pub compute: u64,
    pub alloc: BTreeMap<usize, u64>,
    pub counter: CounterBenchData,
    pub cross_program: u64,
    pub redirect: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterBenchData {
    pub async_call: u64,
    pub sync_call: u64,
}

pub fn store_bench_data(f: impl FnOnce(&mut BenchData)) -> Result<()> {
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bench_data.json");

    store_bench_data_to_file(path, f)
}

fn store_bench_data_to_file(path: impl AsRef<Path>, f: impl FnOnce(&mut BenchData)) -> Result<()> {
    // Open file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("Failed to open or create bench data file")?;

    // Lock file
    file.lock_exclusive().unwrap_or_else(|e| {
        panic!("Failed to lock bench data file for writing: {e}");
    });

    // Read bench data
    let mut content = String::new();
    file.read_to_string(&mut content)
        .context("Failed reading bench data bytes to string")?;
    let mut bench_data =
        serde_json::from_str(&content).context("Failed to deserialize bench data")?;

    // Handle bench data
    f(&mut bench_data);

    // Serialize back
    let bench_data_string = serde_json::to_string_pretty(&bench_data)?;

    // Write updated bench data
    file.set_len(0).context("Failed to erase file content")?;
    file.seek(SeekFrom::Start(0))
        .context("Failed to seek to the start of the file")?;
    file.write_all(bench_data_string.as_bytes())
        .context("Failed to write serialized bench data to file")?;
    file.flush().context("Failed to flush bench data to file")?;

    // Unlock file
    <File as FileExt>::unlock(&file).context("Failed to unlock bench data file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_not_overwritten() {
        // Create initial bench data.
        let initial_bench_data = BenchData {
            compute: 123,
            alloc: BTreeMap::new(),
            counter: CounterBenchData {
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
                bench_data.compute = 42;
                bench_data.cross_program = 0;
            })
            .unwrap();
        });

        let h2 = thread::spawn(move || {
            store_bench_data_to_file(path_h2, |bench_data| {
                bench_data.counter.async_call = 84;
                bench_data.counter.sync_call = 126;
                bench_data.redirect = 4343;
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
        let bench_data: BenchData =
            serde_json::from_str(&content).expect("Failed to deserialize bench data");

        // Check that the bench data was modified correctly.
        assert_eq!(
            bench_data,
            BenchData {
                compute: 42,
                alloc: BTreeMap::new(),
                counter: CounterBenchData {
                    async_call: 84,
                    sync_call: 126,
                },
                cross_program: 0,
                redirect: 4343,
            },
        )
    }
}
