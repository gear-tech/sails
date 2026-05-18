use anyhow::{Context, Result, anyhow};
use benchmarks::{
    BenchCategory, BenchCategoryComparison, BenchCategoryComparisonReport, BenchData, BenchDataFile,
};
use clap::Parser;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "bench-analyzer")]
#[command(
    about = "A tool for analyzing benchmark data by comparing current and previous benchmark results."
)]
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

    analyze_benches(cli.current, cli.other, cli.output, cli.threshold)
}

fn analyze_benches(
    current: PathBuf,
    other: PathBuf,
    report_output: Option<PathBuf>,
    threshold: Option<f64>,
) -> Result<()> {
    // Get benches data from the provided files.
    let (current_data, other_data) = get_bench_data(current, other)?;

    // Flag to track if any benchmarks fail the threshold check.
    let mut threshold_failed = threshold.map(|_| false);

    let current_data: BTreeMap<_, _> = current_data.into_iter().collect();
    let other_data: BTreeMap<_, _> = other_data.into_iter().collect();
    let categories = current_data
        .keys()
        .chain(other_data.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    // Create unfinished report.
    let mut report = initialize_report();
    for category in categories {
        match (current_data.get(&category), other_data.get(&category)) {
            (Some(current_value), Some(other_value)) => {
                let comparison =
                    BenchCategoryComparison::new(category, *current_value, *other_value, threshold);

                if matches!(threshold_failed, Some(false)) && comparison.has_failed_threshold() {
                    let _ = threshold_failed.insert(true);
                }

                add_comparison_to_report(&mut report, comparison);
            }
            (Some(current_value), None) => {
                mark_threshold_failed_on_shape_change(&mut threshold_failed);
                add_added_benchmark_to_report(&mut report, category, *current_value);
            }
            (None, Some(other_value)) => {
                mark_threshold_failed_on_shape_change(&mut threshold_failed);
                add_removed_benchmark_to_report(&mut report, category, *other_value);
            }
            (None, None) => unreachable!("category is sourced from one of the maps"),
        }
    }

    // Finish the report.
    add_report_conclusion(&mut report, threshold, threshold_failed);

    // Printing the finalized report.
    println!("{report}");

    // If any benchmarks failed the threshold check, return an error.
    if matches!(threshold_failed, Some(true)) {
        return Err(anyhow!("Benchmark contains tests failing the threshold."));
    }

    // If an output path is provided, write the report to that file.
    if let Some(report_output) = report_output {
        fs::write(&report_output, &report).context("Failed to write report output")?;

        println!(
            "\nComparison table written to '{}'",
            report_output.display()
        );
    }

    Ok(())
}

fn get_bench_data(current: PathBuf, previous: PathBuf) -> Result<(BenchData, BenchData)> {
    let mut current_file =
        BenchDataFile::open(current).context("Failed to open current benchmark data file")?;
    let mut previous_file =
        BenchDataFile::open(previous).context("Failed to open previous benchmark data file")?;

    let current_data = current_file.read_bench_data()?;
    let previous_data = previous_file.read_bench_data()?;

    Ok((current_data, previous_data))
}

fn initialize_report() -> String {
    let mut report = String::new();
    report.push_str("## 🔬 Benchmark Comparison\n\n");
    report.push_str("| Benchmark | Current | Baseline | Change | Change % | Status |\n");
    report.push_str("|-----------|---------|----------|---------|----------|--------|\n");

    report
}

fn add_comparison_to_report(report: &mut String, comparison: BenchCategoryComparison) {
    let BenchCategoryComparisonReport {
        category,
        current,
        other,
        diff_sign,
        diff,
        diff_percent_sign,
        diff_percent,
        status,
    } = comparison.into();
    report.push_str(&format!(
        "| {category} | {current} | {other} | {diff_sign}{diff} | {diff_percent_sign}{diff_percent:.2}% | {status} |\n",
    ));
}

fn add_added_benchmark_to_report(report: &mut String, category: BenchCategory, current: u64) {
    report.push_str(&format!(
        "| {category} | {current} | - | - | - | 🆕 New benchmark |\n",
    ));
}

fn add_removed_benchmark_to_report(report: &mut String, category: BenchCategory, other: u64) {
    report.push_str(&format!(
        "| {category} | - | {other} | - | - | ⚠️ Missing from current |\n",
    ));
}

fn mark_threshold_failed_on_shape_change(threshold_failed: &mut Option<bool>) {
    if matches!(threshold_failed, Some(false)) {
        let _ = threshold_failed.insert(true);
    }
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
            report.push_str("\n### Legend\n- 🚀 Significant improvement (>5% reduction)\n- 👍 Minor improvement (<5% reduction)\n- ✅ No significant change\n- ⚠️ Minor regression (<5% increase)\n- ❌ Significant regression (>5% increase)\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benchmarks::{BenchDataSerde, ComputeBenchDataSerde, StorageStressDataSerde};
    use std::collections::BTreeMap;

    #[test]
    fn report_includes_benchmarks_missing_from_baseline() {
        let dir = tempfile::tempdir().unwrap();
        let current_path = dir.path().join("current.json");
        let baseline_path = dir.path().join("baseline.json");
        let output_path = dir.path().join("comparison.md");
        let storage_key = "sails_static_balance_read_existing_1024".to_owned();

        let current = BenchDataSerde {
            compute: ComputeBenchDataSerde { median: 1 },
            storage: StorageStressDataSerde(BTreeMap::from([(storage_key.clone(), 810_912_211)])),
            ..Default::default()
        };
        let baseline = BenchDataSerde {
            compute: ComputeBenchDataSerde { median: 1 },
            ..Default::default()
        };

        fs::write(
            &current_path,
            serde_json::to_string_pretty(&current).unwrap(),
        )
        .unwrap();
        fs::write(
            &baseline_path,
            serde_json::to_string_pretty(&baseline).unwrap(),
        )
        .unwrap();

        analyze_benches(current_path, baseline_path, Some(output_path.clone()), None).unwrap();

        let report = fs::read_to_string(output_path).unwrap();
        assert!(report.contains(&format!("storage_{storage_key}")));
        assert!(report.contains("New benchmark"));
    }

    #[test]
    fn threshold_mode_fails_when_categories_are_missing_from_baseline() {
        let dir = tempfile::tempdir().unwrap();
        let current_path = dir.path().join("current.json");
        let baseline_path = dir.path().join("baseline.json");
        let storage_key = "sails_static_balance_read_existing_1024".to_owned();

        let current = BenchDataSerde {
            compute: ComputeBenchDataSerde { median: 1 },
            storage: StorageStressDataSerde(BTreeMap::from([(storage_key, 810_912_211)])),
            ..Default::default()
        };
        let baseline = BenchDataSerde {
            compute: ComputeBenchDataSerde { median: 1 },
            ..Default::default()
        };

        fs::write(
            &current_path,
            serde_json::to_string_pretty(&current).unwrap(),
        )
        .unwrap();
        fs::write(
            &baseline_path,
            serde_json::to_string_pretty(&baseline).unwrap(),
        )
        .unwrap();

        let error = analyze_benches(current_path, baseline_path, None, Some(1.0)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Benchmark contains tests failing the threshold."
        );
    }
}
