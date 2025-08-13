use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

// todo [sab] check diff on negative values

#[derive(Deserialize, Clone)]
struct BenchData {
    compute: u64,
    alloc: HashMap<String, u64>,
    counter: HashMap<String, u64>,
    cross_program: u64,
    redirect: u64,
}

#[derive(Serialize)]
struct DiffResult {
    benchmark: String,
    current: u64,
    previous: u64,
    diff_percent: f64,
    exceeds_threshold: bool,
}

fn calculate_diff_percent(current: u64, previous: u64) -> f64 {
    if previous == 0 {
        if current == 0 {
            0.0
        } else {
            100.0 // Consider any non-zero value as 100% increase from zero
        }
    } else {
        ((current as f64 - previous as f64) / previous as f64) * 100.0
    }
}

fn check_benchmark_value(name: String, current: u64, previous: u64, threshold: f64) -> DiffResult {
    let diff_percent = calculate_diff_percent(current, previous);
    let exceeds_threshold = diff_percent.abs() > threshold;
    
    DiffResult {
        benchmark: name,
        current,
        previous,
        diff_percent,
        exceeds_threshold,
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Benchmark Difference Checker");
        println!();
        println!("USAGE:");
        println!("    {} [CURRENT_FILE] [PREVIOUS_FILE] [THRESHOLD]", args[0]);
        println!();
        println!("ARGUMENTS:");
        println!("    CURRENT_FILE   Current benchmark data (default: bench_data.json)");
        println!("    PREVIOUS_FILE  Previous benchmark data (default: bench_data_previous.json)");
        println!("    THRESHOLD      Threshold percentage for failure (default: 1.0)");
        println!();
        println!("DESCRIPTION:");
        println!("    Compares current vs previous benchmark data and fails if any benchmark");
        println!("    differs by more than the threshold percentage. Exit code 0 = pass, 1 = fail.");
        return Ok(());
    }
    
    let current_file = args.get(1).unwrap_or(&"bench_data.json".to_string()).clone();
    let previous_file = args.get(2).unwrap_or(&"bench_data_previous.json".to_string()).clone();
    let threshold: f64 = args.get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    
    // Check if files exist
    if !Path::new(&current_file).exists() {
        eprintln!("❌ Current file '{}' does not exist", current_file);
        process::exit(1);
    }
    
    if !Path::new(&previous_file).exists() {
        eprintln!("⚠️  Previous file '{}' does not exist - treating as first run", previous_file);
        println!("✅ No previous benchmarks to compare against. Skipping diff check.");
        process::exit(0);
    }
    
    // Read the files
    let current_content = fs::read_to_string(&current_file)?;
    let previous_content = fs::read_to_string(&previous_file)?;
    
    let current_data: BenchData = serde_json::from_str(&current_content)?;
    let previous_data: BenchData = serde_json::from_str(&previous_content)?;
    
    let mut results = Vec::new();
    let mut has_failures = false;
    
    // Check compute
    let diff = check_benchmark_value(
        "compute".to_string(),
        current_data.compute,
        previous_data.compute,
        threshold,
    );
    if diff.exceeds_threshold {
        has_failures = true;
    }
    results.push(diff);
    
    // Check alloc benchmarks
    let mut alloc_keys: std::collections::HashSet<String> = current_data.alloc.keys().cloned().collect();
    alloc_keys.extend(previous_data.alloc.keys().cloned());
    let mut alloc_keys: Vec<_> = alloc_keys.into_iter().collect();
    alloc_keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));
    
    for key in alloc_keys {
        let current_val = current_data.alloc.get(&key).unwrap_or(&0);
        let previous_val = previous_data.alloc.get(&key).unwrap_or(&0);
        let diff = check_benchmark_value(
            format!("alloc-{}", key),
            *current_val,
            *previous_val,
            threshold,
        );
        if diff.exceeds_threshold {
            has_failures = true;
        }
        results.push(diff);
    }
    
    // Check counter benchmarks
    let mut counter_keys: std::collections::HashSet<String> = current_data.counter.keys().cloned().collect();
    counter_keys.extend(previous_data.counter.keys().cloned());
    let counter_keys: Vec<_> = counter_keys.into_iter().collect();
    
    for key in counter_keys {
        let current_val = current_data.counter.get(&key).unwrap_or(&0);
        let previous_val = previous_data.counter.get(&key).unwrap_or(&0);
        let diff = check_benchmark_value(
            format!("counter-{}", key),
            *current_val,
            *previous_val,
            threshold,
        );
        if diff.exceeds_threshold {
            has_failures = true;
        }
        results.push(diff);
    }
    
    // Check cross_program
    let diff = check_benchmark_value(
        "cross_program".to_string(),
        current_data.cross_program,
        previous_data.cross_program,
        threshold,
    );
    if diff.exceeds_threshold {
        has_failures = true;
    }
    results.push(diff);
    
    // Check redirect
    let diff = check_benchmark_value(
        "redirect".to_string(),
        current_data.redirect,
        previous_data.redirect,
        threshold,
    );
    if diff.exceeds_threshold {
        has_failures = true;
    }
    results.push(diff);
    
    // Print results
    println!("🔍 Benchmark Difference Analysis (threshold: {:.1}%)", threshold);
    println!("═══════════════════════════════════════════════════════════");
    
    let mut passed = 0;
    let mut failed = 0;
    
    for result in &results {
        let status = if result.exceeds_threshold {
            failed += 1;
            "❌ FAIL"
        } else {
            passed += 1;
            "✅ PASS"
        };
        
        let sign = if result.diff_percent >= 0.0 { "+" } else { "" };
        println!(
            "{} | {:20} | {:>15} → {:>15} | {}{:>6.2}%",
            status,
            result.benchmark,
            format_number(result.previous),
            format_number(result.current),
            sign,
            result.diff_percent
        );
    }
    
    println!("═══════════════════════════════════════════════════════════");
    println!("📊 Summary: {} passed, {} failed", passed, failed);
    
    if has_failures {
        println!();
        println!("❌ BENCHMARK DIFF CHECK FAILED!");
        println!("   Some benchmarks differ by more than {:.1}% from the previous run.", threshold);
        println!("   This indicates significant performance changes that need investigation.");
        process::exit(1);
    } else {
        println!();
        println!("✅ All benchmark differences are within acceptable threshold ({:.1}%)", threshold);
        process::exit(0);
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
