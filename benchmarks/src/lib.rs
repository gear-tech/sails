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
    collections::{BTreeMap, btree_map::IntoIter as BTreeMapIntoIter},
    env,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

pub struct BenchDataOuter(BTreeMap<BenchCategory, u64>);

impl BenchDataOuter {
    pub fn from_json_str(str: &str) -> Result<Self> {
        let data: BenchData = serde_json::from_str(str)
            .context("Failed to deserialize `BenchData` from JSON string")?;

        let mut map = BTreeMap::new();
        map.insert(BenchCategory::Compute, data.compute);
        for (key, value) in data.alloc {
            map.insert(BenchCategory::Alloc(key), value);
        }
        map.insert(BenchCategory::CounterSync, data.counter.sync_call);
        map.insert(BenchCategory::CounterAsync, data.counter.async_call);
        map.insert(BenchCategory::CrossProgram, data.cross_program);
        map.insert(BenchCategory::Redirect, data.redirect);

        Ok(Self(map))
    }

    pub fn update_compute_bench(&mut self, value: u64) {
        self.0.insert(BenchCategory::Compute, value);
    }

    pub fn update_alloc_bench(&mut self, size: usize, value: u64) {
        self.0.insert(BenchCategory::Alloc(size), value);
    }

    pub fn update_counter_bench(&mut self, is_async: bool, value: u64) {
        if is_async {
            self.0.insert(BenchCategory::CounterAsync, value);
        } else {
            self.0.insert(BenchCategory::CounterSync, value);
        }
    }

    pub fn update_cross_program_bench(&mut self, value: u64) {
        self.0.insert(BenchCategory::CrossProgram, value);
    }

    pub fn update_redirect_bench(&mut self, value: u64) {
        self.0.insert(BenchCategory::Redirect, value);
    }

    pub fn into_json_string(self) -> Result<String> {
        let mut bench_data = BenchData::default();
        for (key, value) in self.0 {
            // match statement is crucial for not missing any new added category
            match key {
                BenchCategory::Compute => bench_data.compute = value,
                BenchCategory::Alloc(size) => {
                    bench_data.alloc.insert(size, value);
                }
                BenchCategory::CounterSync => bench_data.counter.sync_call = value,
                BenchCategory::CounterAsync => bench_data.counter.async_call = value,
                BenchCategory::CrossProgram => bench_data.cross_program = value,
                BenchCategory::Redirect => bench_data.redirect = value,
            }
        }

        serde_json::to_string_pretty(&bench_data)
            .context("Failed to serialize `BenchData` to JSON string")
    }
}

impl IntoIterator for BenchDataOuter {
    type Item = (BenchCategory, u64);
    type IntoIter = BTreeMapIntoIter<BenchCategory, u64>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BenchCategory {
    Compute,
    Alloc(usize),
    CounterSync,
    CounterAsync,
    CrossProgram,
    Redirect,
}

impl Display for BenchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchCategory::Compute => write!(f, "compute"),
            BenchCategory::Alloc(size) => write!(f, "alloc-{size}"),
            BenchCategory::CounterSync => write!(f, "counter_sync"),
            BenchCategory::CounterAsync => write!(f, "counter_async"),
            BenchCategory::CrossProgram => write!(f, "cross_program"),
            BenchCategory::Redirect => write!(f, "redirect"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchData {
    pub compute: u64,
    pub alloc: BTreeMap<usize, u64>,
    pub counter: CounterBenchData,
    pub cross_program: u64,
    pub redirect: u64,
}

pub struct BenchDataFile(File);

impl BenchDataFile {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .context("Failed to open or create bench data file")?;

        Ok(Self(file))
    }

    pub fn lock_exclusive(&mut self) -> Result<()> {
        self.0
            .lock_exclusive()
            .context("Failed to lock bench data file for writing")
    }

    pub fn unlock(&self) -> Result<()> {
        <File as FileExt>::unlock(&self.0).context("Failed to unlock bench data file")
    }

    pub fn read_bench_data(&mut self) -> Result<BenchDataOuter> {
        let mut content = String::new();
        self.0
            .read_to_string(&mut content)
            .context("Failed reading bench data bytes to string")?;
        let bench_data =
            BenchDataOuter::from_json_str(&content).context("Failed to deserialize bench data")?;

        Ok(bench_data)
    }

    pub fn update_bench_data(&mut self, updated: BenchDataOuter) -> Result<()> {
        // Serialize back
        let bench_data_string = updated
            .into_json_string()
            .context("Failed to serialize updated bench data")?;

        // Write updated bench data
        self.0.set_len(0).context("Failed to erase file content")?;
        self.0
            .seek(SeekFrom::Start(0))
            .context("Failed to seek to the start of the file")?;
        self.0
            .write_all(bench_data_string.as_bytes())
            .context("Failed to write serialized bench data to file")?;
        self.0
            .flush()
            .context("Failed to flush bench data to file")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterBenchData {
    pub async_call: u64,
    pub sync_call: u64,
}

pub fn store_bench_data(f: impl FnOnce(&mut BenchDataOuter)) -> Result<()> {
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bench_data.json");

    store_bench_data_to_file(path, f)
}

fn store_bench_data_to_file(
    path: impl AsRef<Path>,
    f: impl FnOnce(&mut BenchDataOuter),
) -> Result<()> {
    let mut file = BenchDataFile::open(path).context("Failed to create `BenchDataFile`")?;

    file.lock_exclusive().unwrap_or_else(|e| {
        panic!("Failed to lock bench data file for writing: {e}");
    });

    let mut bench_data = file
        .read_bench_data()
        .context("Failed to read existing bench data")?;

    // Handle bench data
    f(&mut bench_data);

    file.update_bench_data(bench_data)
        .context("Failed to update bench data")?;

    // Unlock the file
    file.unlock()
        .context("Failed to unlock bench data file after writing")
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
