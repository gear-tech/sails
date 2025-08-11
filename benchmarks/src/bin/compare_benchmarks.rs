use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
struct BenchData {
    compute: u64,
    alloc: HashMap<String, u64>,
    counter: HashMap<String, u64>,
    cross_program: u64,
    redirect: u64,
}

#[derive(Serialize)]
struct ComparisonResult {
    name: String,
    current: u64,
    baseline: u64,
    change: i64,
    change_percent: f64,
    status: String,
}

fn format_gas(gas: u64) -> String {
    let gas_str = gas.to_string();
    let mut result = String::new();
    
    for (i, ch) in gas_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('_');
        }
        result.push(ch);
    }
    
    result.chars().rev().collect()
}

fn calculate_change_status(change_percent: f64) -> String {
    if change_percent.abs() < 1.0 {
        "✅".to_string() // No significant change
    } else if change_percent < -5.0 {
        "🚀".to_string() // Significant improvement
    } else if change_percent < 0.0 {
        "✅".to_string() // Minor improvement
    } else if change_percent < 5.0 {
        "⚠️".to_string() // Minor regression
    } else {
        "❌".to_string() // Significant regression
    }
}

fn compare_values(name: String, current: u64, baseline: u64) -> ComparisonResult {
    let change = current as i64 - baseline as i64;
    let change_percent = if baseline > 0 {
        (change as f64 / baseline as f64) * 100.0
    } else {
        0.0
    };
    let status = calculate_change_status(change_percent);

    ComparisonResult {
        name,
        current,
        baseline,
        change,
        change_percent,
        status,
    }
}

fn generate_markdown_table(comparisons: &[ComparisonResult]) -> String {
    let mut markdown = String::new();
    
    markdown.push_str("## 🔬 Benchmark Comparison\n\n");
    markdown.push_str("| Benchmark | Current | Baseline | Change | Change % | Status |\n");
    markdown.push_str("|-----------|---------|----------|---------|----------|--------|\n");
    
    for comp in comparisons {
        let change_sign = if comp.change >= 0 { "+" } else { "" };
        markdown.push_str(&format!(
            "| {} | {} | {} | {}{} | {}{:.2}% | {} |\n",
            comp.name,
            format_gas(comp.current),
            format_gas(comp.baseline),
            change_sign,
            format_gas(comp.change.abs() as u64),
            if comp.change_percent >= 0.0 { "+" } else { "" },
            comp.change_percent,
            comp.status
        ));
    }
    
    markdown.push_str("\n### Legend\n");
    markdown.push_str("- 🚀 Significant improvement (>5% reduction)\n");
    markdown.push_str("- ✅ No significant change or minor improvement\n");
    markdown.push_str("- ⚠️ Minor regression (<5% increase)\n");
    markdown.push_str("- ❌ Significant regression (>5% increase)\n");
    
    markdown
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Benchmark Comparison Tool");
        println!();
        println!("USAGE:");
        println!("    {} [CURRENT_FILE] [BASELINE_FILE] [OUTPUT_FILE]", args[0]);
        println!();
        println!("ARGUMENTS:");
        println!("    CURRENT_FILE   Current benchmark data (default: bench_data.json)");
        println!("    BASELINE_FILE  Baseline benchmark data (default: baseline.json)");
        println!("    OUTPUT_FILE    Output markdown file (default: comparison.md)");
        println!();
        println!("DESCRIPTION:");
        println!("    Compares two benchmark JSON files and generates a markdown table");
        println!("    showing the differences with status indicators.");
        return Ok(());
    }
    
    let current_file = args.get(1).unwrap_or(&"bench_data.json".to_string()).clone();
    let baseline_file = args.get(2).unwrap_or(&"baseline.json".to_string()).clone();
    let output_file = args.get(3).unwrap_or(&"comparison.md".to_string()).clone();
    
    // Read the files
    if !Path::new(&current_file).exists() {
        eprintln!("Current file '{}' does not exist", current_file);
        std::process::exit(1);
    }
    
    if !Path::new(&baseline_file).exists() {
        eprintln!("Baseline file '{}' does not exist", baseline_file);
        std::process::exit(1);
    }
    
    let current_content = fs::read_to_string(&current_file)?;
    let baseline_content = fs::read_to_string(&baseline_file)?;
    
    let current_data: BenchData = serde_json::from_str(&current_content)?;
    let baseline_data: BenchData = serde_json::from_str(&baseline_content)?;
    
    let mut comparisons = Vec::new();
    
    // Compare compute
    comparisons.push(compare_values(
        "Compute".to_string(),
        current_data.compute,
        baseline_data.compute,
    ));
    
    // Compare alloc benchmarks (get all keys from both datasets)
    let mut alloc_keys: std::collections::HashSet<String> = current_data.alloc.keys().cloned().collect();
    alloc_keys.extend(baseline_data.alloc.keys().cloned());
    let mut alloc_keys: Vec<_> = alloc_keys.into_iter().collect();
    alloc_keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));
    
    for key in alloc_keys {
        let current_val = current_data.alloc.get(&key).unwrap_or(&0);
        let baseline_val = baseline_data.alloc.get(&key).unwrap_or(&0);
        comparisons.push(compare_values(
            format!("alloc - {}", key),
            *current_val,
            *baseline_val,
        ));
    }
    
    // Compare counter benchmarks
    let mut counter_keys: std::collections::HashSet<String> = current_data.counter.keys().cloned().collect();
    counter_keys.extend(baseline_data.counter.keys().cloned());
    let counter_keys: Vec<_> = counter_keys.into_iter().collect();
    
    for key in counter_keys {
        let current_val = current_data.counter.get(&key).unwrap_or(&0);
        let baseline_val = baseline_data.counter.get(&key).unwrap_or(&0);
        comparisons.push(compare_values(
            format!("counter - {}", key),
            *current_val,
            *baseline_val,
        ));
    }
    
    // Compare cross_program
    comparisons.push(compare_values(
        "cross_program".to_string(),
        current_data.cross_program,
        baseline_data.cross_program,
    ));
    
    // Compare redirect
    comparisons.push(compare_values(
        "redirect".to_string(),
        current_data.redirect,
        baseline_data.redirect,
    ));
    
    // Generate markdown table
    let markdown = generate_markdown_table(&comparisons);
    
    // Write to file
    fs::write(&output_file, &markdown)?;
    
    // Also output to stdout for GitHub Actions
    println!("{}", markdown);
    
    // Summary
    let total_benchmarks = comparisons.len();
    let improvements = comparisons.iter().filter(|c| c.change_percent < -1.0).count();
    let regressions = comparisons.iter().filter(|c| c.change_percent > 1.0).count();
    let no_change = total_benchmarks - improvements - regressions;
    
    println!("📊 **Summary**: {} total benchmarks - {} improvements, {} no significant change, {} regressions",
             total_benchmarks, improvements, no_change, regressions);
    
    println!("\nComparison table written to '{}'", output_file);
    
    Ok(())
}
