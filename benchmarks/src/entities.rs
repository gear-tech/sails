use anyhow::{Context, Result};
use itertools::Either;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, btree_map::IntoIter as BTreeMapIntoIter},
    fmt::Display,
};

/// A collection holding benchmark data categorized by [`BenchCategory`].
pub struct BenchData(BTreeMap<BenchCategory, u64>);

impl BenchData {
    /// Creates a new `BenchData` instance from a JSON string.
    pub fn from_json_str(str: &str) -> Result<Self> {
        let data: BenchDataSerde = serde_json::from_str(str)
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

    /// Update compute benchmark category value.
    pub fn update_compute_bench(&mut self, value: u64) {
        self.0.insert(BenchCategory::Compute, value);
    }

    /// Update allocation benchmark category value.
    pub fn update_alloc_bench(&mut self, size: usize, value: u64) {
        self.0.insert(BenchCategory::Alloc(size), value);
    }

    /// Update counter benchmark category value.
    pub fn update_counter_bench(&mut self, is_async: bool, value: u64) {
        if is_async {
            self.0.insert(BenchCategory::CounterAsync, value);
        } else {
            self.0.insert(BenchCategory::CounterSync, value);
        }
    }

    /// Update cross-program benchmark category value.
    pub fn update_cross_program_bench(&mut self, value: u64) {
        self.0.insert(BenchCategory::CrossProgram, value);
    }

    /// Update redirect benchmark category value.
    pub fn update_redirect_bench(&mut self, value: u64) {
        self.0.insert(BenchCategory::Redirect, value);
    }

    /// Convert the benchmark data into a JSON string.
    pub fn into_json_string(self) -> Result<String> {
        let mut bench_data = BenchDataSerde::default();
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

impl IntoIterator for BenchData {
    type Item = (BenchCategory, u64);
    type IntoIter = BTreeMapIntoIter<BenchCategory, u64>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Benchmark data stored in the benchmarks file.
///
/// This struct is used to serialize and deserialize benchmark data
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchDataSerde {
    pub compute: u64,
    pub alloc: BTreeMap<usize, u64>,
    pub counter: CounterBenchDataSerde,
    pub cross_program: u64,
    pub redirect: u64,
}

/// Counter test benchmark data stored in the benchmarks file.
///
/// This struct is used to serialize and deserialize benchmark data
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterBenchDataSerde {
    pub async_call: u64,
    pub sync_call: u64,
}

/// Benchmark category that can be read (written) from (to) the benchmarks file.
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

/// Comparison entity for benchmark categories.
#[derive(Debug)]
pub struct BenchCategoryComparison {
    category: BenchCategory,
    current: u64,
    other: u64,
    diff: i64,
    diff_percent: f64,
    status: Either<ThresholdPassStatus, PerformanceStatus>,
}

impl BenchCategoryComparison {
    pub fn new(
        category: BenchCategory,
        current: u64,
        other: u64,
        maybe_threshold: Option<f64>,
    ) -> Self {
        let diff = current as i64 - other as i64;
        let diff_percent = (diff as f64 / other as f64) * 100.0;
        let status = match maybe_threshold {
            Some(threshold) => {
                let exceeds = diff_percent.abs() > threshold;
                if exceeds {
                    Either::Left(ThresholdPassStatus::Fail)
                } else {
                    Either::Left(ThresholdPassStatus::Pass)
                }
            }
            None => {
                if diff_percent.abs() < 1.0 {
                    // [0,..1.0)
                    Either::Right(PerformanceStatus::NoChange)
                } else if diff_percent < -5.0 {
                    // [-inf, -5.0)
                    Either::Right(PerformanceStatus::SignificantImprovement)
                } else if diff_percent < 0.0 {
                    // [-5.0, 0.0)
                    Either::Right(PerformanceStatus::MinorImprovement)
                } else if diff_percent < 5.0 {
                    // [0.0, 5.0)
                    Either::Right(PerformanceStatus::MinorRegression)
                } else {
                    // [5.0, inf)
                    Either::Right(PerformanceStatus::SignificantRegression)
                }
            }
        };

        Self {
            category,
            current,
            other,
            diff,
            diff_percent,
            status,
        }
    }

    pub fn has_failed_threshold(&self) -> bool {
        self.status
            .as_ref()
            .left()
            .map(|status| matches!(status, ThresholdPassStatus::Fail))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy)]
enum ThresholdPassStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy)]
enum PerformanceStatus {
    SignificantImprovement,
    MinorImprovement,
    NoChange,
    SignificantRegression,
    MinorRegression,
}

/// Report structure for benchmark category comparison.
///
/// This struct is a placeholder to formatted benchmark comparison data.
/// The formatted data is later decided on a client side how to be displayed.
pub struct BenchCategoryComparisonReport {
    pub category: String,
    pub current: String,
    pub other: String,
    pub diff_sign: &'static str,
    pub diff: String,
    pub diff_percent_sign: &'static str,
    pub diff_percent: f64,
    pub status: &'static str,
}

impl From<BenchCategoryComparison> for BenchCategoryComparisonReport {
    fn from(comparison: BenchCategoryComparison) -> Self {
        let category = comparison.category.to_string();
        let current = Self::format_number(comparison.current);
        let other = Self::format_number(comparison.other);
        let diff_sign = if comparison.diff >= 0 { "+" } else { "-" };
        let diff = Self::format_number(comparison.diff.unsigned_abs());
        let diff_percent_sign = if comparison.diff_percent >= 0.0 {
            "+"
        } else {
            ""
        };
        let diff_percent = comparison.diff_percent;
        let status = Self::status_to_str(&comparison.status);

        BenchCategoryComparisonReport {
            category,
            current,
            other,
            diff_sign,
            diff,
            diff_percent_sign,
            diff_percent,
            status,
        }
    }
}

impl BenchCategoryComparisonReport {
    fn format_number(num: u64) -> String {
        let num_str = num.to_string();
        let mut result = String::new();

        for (i, ch) in num_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push('_');
            }
            result.push(ch);
        }

        result.chars().rev().collect()
    }

    fn status_to_str(status: &Either<ThresholdPassStatus, PerformanceStatus>) -> &'static str {
        match status {
            Either::Left(ThresholdPassStatus::Pass) => "✅ PASS",
            Either::Left(ThresholdPassStatus::Fail) => "❌ FAIL",
            Either::Right(PerformanceStatus::SignificantImprovement) => "🚀",
            Either::Right(PerformanceStatus::MinorImprovement) => "👍",
            Either::Right(PerformanceStatus::NoChange) => "✅",
            Either::Right(PerformanceStatus::SignificantRegression) => "❌",
            Either::Right(PerformanceStatus::MinorRegression) => "⚠️",
        }
    }
}
