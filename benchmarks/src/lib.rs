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
    AllocBenchDataSerde, BenchCategory, BenchCategoryComparison, BenchCategoryComparisonReport,
    BenchData, BenchDataSerde, ComputeBenchDataSerde, CounterBenchDataSerde,
    CrossProgramBenchDataSerde, ExampleBenchDataSerde, RedirectBenchDataSerde,
    StorageMillionDataSerde, StorageStressDataSerde,
};
pub use file::BenchDataFile;
#[cfg(feature = "gas-profile")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "gas-profile")]
use std::fs;
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

#[cfg(feature = "gas-profile")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GasProfileBucketSerde {
    pub category: String,
    pub label: String,
    pub amount: u64,
}

#[cfg(feature = "gas-profile")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GasProfileArtifactSerde {
    pub benchmark: String,
    pub total_gas: u64,
    pub buckets: Vec<GasProfileBucketSerde>,
}

#[cfg(feature = "gas-profile")]
fn gas_profile_root() -> Option<PathBuf> {
    env::var_os("SAILS_GAS_PROFILE_DIR").map(PathBuf::from)
}

#[cfg(feature = "gas-profile")]
pub fn write_gas_profile_artifact(
    benchmark: &str,
    total_gas: u64,
    buckets: Vec<(String, String, u64)>,
) -> Result<()> {
    let Some(root) = gas_profile_root() else {
        return Ok(());
    };

    fs::create_dir_all(root.join("profiles"))
        .context("Failed to create gas profile artifact directory")?;

    let artifact = GasProfileArtifactSerde {
        benchmark: benchmark.to_owned(),
        total_gas,
        buckets: buckets
            .iter()
            .map(|(category, label, amount)| GasProfileBucketSerde {
                category: category.clone(),
                label: label.clone(),
                amount: *amount,
            })
            .collect(),
    };

    let json_path = root.join("profiles").join(format!("{benchmark}.json"));
    let folded_path = root.join("profiles").join(format!("{benchmark}.folded"));

    fs::write(
        &json_path,
        serde_json::to_vec_pretty(&artifact).context("Failed to encode gas profile artifact")?,
    )
    .with_context(|| {
        format!(
            "Failed to write gas profile artifact to {}",
            json_path.display()
        )
    })?;

    let folded = buckets
        .into_iter()
        .map(|(category, label, amount)| format!("{benchmark};{category};{label} {amount}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&folded_path, format!("{folded}\n")).with_context(|| {
        format!(
            "Failed to write folded gas profile artifact to {}",
            folded_path.display()
        )
    })?;

    Ok(())
}

#[cfg(feature = "gas-profile")]
pub fn read_gas_profile_artifacts() -> Result<Vec<GasProfileArtifactSerde>> {
    let Some(root) = gas_profile_root() else {
        return Ok(Vec::new());
    };

    let profiles = root.join("profiles");
    if !profiles.exists() {
        return Ok(Vec::new());
    }

    let mut artifacts = Vec::new();
    for entry in fs::read_dir(&profiles)
        .with_context(|| format!("Failed to read gas profiles from {}", profiles.display()))?
    {
        let entry = entry.context("Failed to read gas profile directory entry")?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }

        let bytes = fs::read(&path)
            .with_context(|| format!("Failed to read gas profile {}", path.display()))?;
        let artifact: GasProfileArtifactSerde = serde_json::from_slice(&bytes)
            .with_context(|| format!("Failed to decode gas profile {}", path.display()))?;
        artifacts.push(artifact);
    }

    artifacts.sort_by(|left, right| left.benchmark.cmp(&right.benchmark));
    Ok(artifacts)
}

#[cfg(feature = "gas-profile")]
pub fn write_gas_profile_summary(
    summary: &std::collections::BTreeMap<String, u64>,
    comparison_markdown: &str,
) -> Result<()> {
    let Some(root) = gas_profile_root() else {
        return Ok(());
    };

    fs::create_dir_all(&root).context("Failed to create gas profile output directory")?;

    let summary_path = root.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(summary).context("Failed to encode gas profile summary")?,
    )
    .with_context(|| format!("Failed to write summary to {}", summary_path.display()))?;

    let comparison_path = root.join("comparison.md");
    fs::write(&comparison_path, comparison_markdown).with_context(|| {
        format!(
            "Failed to write gas profile comparison markdown to {}",
            comparison_path.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        BenchDataSerde, ComputeBenchDataSerde, CounterBenchDataSerde, CrossProgramBenchDataSerde,
        ExampleBenchDataSerde, RedirectBenchDataSerde, StorageStressDataSerde,
    };
    use std::{
        collections::BTreeMap,
        io::{Read, Seek, SeekFrom, Write},
        thread,
    };
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_not_overwritten() {
        // Create initial bench data.
        let initial_bench_data = BenchDataSerde {
            compute: ComputeBenchDataSerde { median: 123 },
            alloc: Default::default(),
            counter: CounterBenchDataSerde {
                async_call: 53,
                sync_call: 35,
            },
            cross_program: CrossProgramBenchDataSerde { median: 42 },
            redirect: RedirectBenchDataSerde { median: 4242 },
            message_stack: Default::default(),
            storage: StorageStressDataSerde(BTreeMap::from([(
                "sails_static_balance_prepare_1024".to_owned(),
                777,
            )])),
            storage_million: StorageMillionDataSerde(BTreeMap::from([(
                "static_balance_prepare_1000000".to_owned(),
                999,
            )])),
            examples: ExampleBenchDataSerde(BTreeMap::from([(
                "aggregator_btree_prepare_1024".to_owned(),
                888,
            )])),
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
                compute: ComputeBenchDataSerde { median: 42 },
                alloc: Default::default(),
                counter: CounterBenchDataSerde {
                    async_call: 84,
                    sync_call: 126,
                },
                cross_program: CrossProgramBenchDataSerde { median: 0 },
                redirect: RedirectBenchDataSerde { median: 4343 },
                message_stack: Default::default(),
                storage: StorageStressDataSerde(BTreeMap::from([(
                    "sails_static_balance_prepare_1024".to_owned(),
                    777,
                )])),
                storage_million: StorageMillionDataSerde(BTreeMap::from([(
                    "static_balance_prepare_1000000".to_owned(),
                    999,
                )])),
                examples: ExampleBenchDataSerde(BTreeMap::from([(
                    "aggregator_btree_prepare_1024".to_owned(),
                    888,
                )])),
            },
        )
    }
}
