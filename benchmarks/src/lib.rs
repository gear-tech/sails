//! TODO [sab]

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

#[cfg(test)]
mod benchmarks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchData {
    pub compute: u64,
    pub alloc: BTreeMap<u32, u64>,
    pub counter: CounterBenchData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterBenchData {
    pub async_call: u64,
    pub sync_call: u64,
}

pub fn store_bench_data(f: impl FnOnce(&mut BenchData)) -> Result<()> {
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bench_data.json");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .context("Failed to open or create bench data file")?;

    let _lock_res = file.lock_exclusive().unwrap_or_else(|e| {
        panic!("Failed to lock bench data file for writing: {e}");
    });

    let mut bench_data = read_bench_data(&mut file).expect("Failed to read bench data file");
    f(&mut bench_data);

    erase_file_content(&mut file)?;

    serde_json::to_writer_pretty(&mut file, &bench_data)
        .context("Failed to serialize bench data to JSON")?;

    <File as FileExt>::unlock(&file)
        .context("Failed to unlock bench data file")
        .map_err(Into::into)
}

fn read_bench_data(file: &mut File) -> Result<BenchData> {
    let mut content = String::new();
    file.read_to_string(&mut content)
        .context("Failed reading bench data bytes to string")?;

    serde_json::from_str(&content)
        .context("Failed to deserialize bench data")
        .map_err(Into::into)
}

fn erase_file_content(file: &mut File) -> Result<()> {
    file.set_len(0).context("Failed to erase file content")?;

    file.seek(SeekFrom::Start(0))
        .context("Failed to seek to the start of the file")?;

    Ok(())
}

// todo [sab] add test
