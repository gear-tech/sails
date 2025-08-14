use anyhow::{Context, Result, anyhow};
use benchmarks::{
    BenchCategoryComparison, BenchCategoryComparisonReport, BenchData, BenchDataFile,
};
use clap::Parser;
use std::{fs, path::PathBuf};

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

    // Create unfinished report.
    let mut report = current_data
        .into_iter()
        .zip(other_data)
        .map(
            // Create a comparison entity for each benchmark category.
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
        .fold(initialize_report(), |mut report, comparison| {
            // Add each comparison to the report.
            add_comparison_to_report(&mut report, comparison);
            report
        });

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
