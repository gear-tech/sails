use crate::BenchData;
use anyhow::{Context, Result};
use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

/// A file that holds benchmark data.
pub struct BenchDataFile(File);

impl BenchDataFile {
    /// Opens a benchmark data file.
    /// 
    /// If the file does not exist, the function fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .context("Failed to open or create bench data file")?;

        Ok(Self(file))
    }

    /// Locks the file for exclusive access.
    pub fn lock_exclusive(&mut self) -> Result<()> {
        self.0
            .lock_exclusive()
            .context("Failed to lock bench data file for writing")
    }

    /// Unlocks the file after exclusive access.
    pub fn unlock(&self) -> Result<()> {
        <File as FileExt>::unlock(&self.0).context("Failed to unlock bench data file")
    }

    /// Reads the benchmark data from the file.
    pub fn read_bench_data(&mut self) -> Result<BenchData> {
        let mut content = String::new();
        self.0
            .read_to_string(&mut content)
            .context("Failed reading bench data bytes to string")?;
        let bench_data =
            BenchData::from_json_str(&content).context("Failed to deserialize bench data")?;

        Ok(bench_data)
    }

    /// Converts the benchmark data into a JSON string and writes it to the file.
    pub fn write_bench_data(&mut self, data: BenchData) -> Result<()> {
        // Serialize back
        let bench_data_string = data
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
