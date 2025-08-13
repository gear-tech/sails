use anyhow::{Context, Result, anyhow};
use benchmarks::{BenchCategory, BenchDataFile, BenchDataOuter};
use clap::Parser;
use itertools::Either;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "bench-analyzer")]
#[command(about = "A tool for analyzing benchmark data differences and comparisons")]
struct Cli {
    /// Current benchmark data file
    #[arg(long)]
    current: PathBuf,

    /// Other benchmark data file
    #[arg(long)]
    other: PathBuf,

    /// Threshold percentage for failure
    #[arg(long)]
    threshold: Option<f64>,

    /// Report markdown file
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    compare_bench(cli.current, cli.other, cli.output, cli.threshold)
}

fn compare_bench(
    current: PathBuf,
    other: PathBuf,
    report_output: Option<PathBuf>,
    threshold: Option<f64>,
) -> Result<()> {
    let (current_data, other_data) = get_bench_data(current, other)?;

    let mut report = String::new();
    report.push_str("## 🔬 Benchmark Comparison\n\n");
    report.push_str("| Benchmark | Current | Baseline | Change | Change % | Status |\n");
    report.push_str("|-----------|---------|----------|---------|----------|--------|\n");

    let mut threshold_failed = threshold.map(|_| false);
    let mut report = current_data
        .into_iter()
        .zip(other_data)
        .map(
            |((current_category, current_value), (other_category, other_value))| {
                assert_eq!(current_category, other_category, "Categories do not match");

                let comparison = BenchCategoryComparison::new(
                    current_category,
                    current_value,
                    other_value,
                    threshold,
                );

                if matches!(threshold_failed, Some(false)) && comparison.has_failed_threshold() {
                    let _ = threshold_failed.insert(true);
                }

                comparison
            },
        )
        .fold(report, |mut report, comparison| {
            add_comparison_to_report(&mut report, comparison);
            report
        });

    add_report_conclusion(&mut report, threshold, threshold_failed);

    // Printing the final report
    println!("{report}");

    if matches!(threshold_failed, Some(true)) {
        return Err(anyhow!("Benchmark contains tests failing the threshold."));
    }

    if let Some(report_output) = report_output {
        fs::write(&report_output, &report).context("Failed to write report output")?;

        println!(
            "\nComparison table written to '{}'",
            report_output.display()
        );
    }

    Ok(())
}

fn get_bench_data(current: PathBuf, previous: PathBuf) -> Result<(BenchDataOuter, BenchDataOuter)> {
    let mut current_file =
        BenchDataFile::open(current).context("Failed to open current benchmark data file")?;
    let mut previous_file =
        BenchDataFile::open(previous).context("Failed to open previous benchmark data file")?;

    let current_data = current_file.read_bench_data()?;
    let previous_data = previous_file.read_bench_data()?;

    Ok((current_data, previous_data))
}

#[derive(Debug)]
struct BenchCategoryComparison {
    category: BenchCategory,
    current: u64,
    other: u64,
    diff: i64,
    diff_percent: f64,
    status: Either<ThresholdPassStatus, PerformanceStatus>,
}

impl BenchCategoryComparison {
    fn new(
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

    fn has_failed_threshold(&self) -> bool {
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

fn add_comparison_to_report(report: &mut String, comparison: BenchCategoryComparison) {
    let BenchCategoryComparison {
        category,
        current,
        other,
        diff,
        diff_percent,
        status,
    } = comparison;

    let current = format_number(current);
    let other = format_number(other);
    let diff_sign = if diff >= 0 { "+" } else { "-" };
    let diff_percent_sign = if diff_percent >= 0.0 { "+" } else { "" };
    let diff = format_number(diff.unsigned_abs());
    let status_str = status_to_str(&status);

    report.push_str(&format!(
        "| {category} | {current} | {other} | {diff_sign}{diff} | {diff_percent_sign}{diff_percent:.2}% | {status_str} |\n",
    ));
}

fn add_report_conclusion(
    report: &mut String,
    threshold: Option<f64>,
    threshold_failed: Option<bool>,
) {
    match threshold_failed {
        Some(true) => {
            let threshold = threshold.expect("threshold is required when threshold_failed is true");
            let err_str = format!("\n❌ Benchmark threshold {threshold:.1}% check failed!\n");
            report.push_str(&err_str);
        }
        Some(false) => {
            report.push_str("\n✅ All benchmark differences are within acceptable thresholds.");
        }
        None => {
            report.push_str("\n### Legend\n- 🚀 Significant improvement (>5% reduction)\n- ✅ No significant change or minor improvement\n- ⚠️ Minor regression (<5% increase)\n- ❌ Significant regression (>5% increase)\n");
        }
    }
}
