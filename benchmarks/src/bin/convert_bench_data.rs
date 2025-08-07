use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct BenchData {
    compute: u64,
    alloc: HashMap<String, u64>,
    counter: HashMap<String, u64>,
    cross_program: u64,
    redirect: u64,
}

#[derive(Serialize)]
struct BenchmarkEntry {
    name: String,
    unit: String,
    value: u64,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Benchmark Data Converter");
        println!();
        println!("USAGE:");
        println!("    {} [INPUT_FILE] [OUTPUT_FILE]", args[0]);
        println!();
        println!("ARGUMENTS:");
        println!("    INPUT_FILE    Input JSON file (default: bench_data.json)");
        println!("    OUTPUT_FILE   Output JSON file (default: bench_data_converted.json)");
        println!();
        println!("DESCRIPTION:");
        println!("    Converts benchmark data from the original nested JSON format");
        println!("    to the GitHub Actions benchmark format with name, unit, and value fields.");
        return Ok(());
    }
    
    let input_file = args.get(1).unwrap_or(&"bench_data.json".to_string()).clone();
    let output_file = args.get(2).unwrap_or(&"bench_data_converted.json".to_string()).clone();
    
    // Read the input file
    let input_path = Path::new(&input_file);
    if !input_path.exists() {
        eprintln!("Input file '{}' does not exist", input_file);
        std::process::exit(1);
    }
    
    let content = fs::read_to_string(input_path)?;
    let bench_data: BenchData = serde_json::from_str(&content)?;
    
    let mut entries = Vec::new();
    
    // Add compute benchmark
    entries.push(BenchmarkEntry {
        name: "Compute".to_string(),
        unit: "gas".to_string(),
        value: bench_data.compute,
    });
    
    // Add alloc benchmarks (sorted by key for consistent ordering)
    let mut alloc_keys: Vec<_> = bench_data.alloc.keys().collect();
    alloc_keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));
    
    for key in alloc_keys {
        let value = bench_data.alloc[key];
        entries.push(BenchmarkEntry {
            name: format!("alloc - {}", key),
            unit: "gas".to_string(),
            value,
        });
    }
    
    // Add counter benchmarks
    for (key, value) in &bench_data.counter {
        entries.push(BenchmarkEntry {
            name: format!("counter - {}", key),
            unit: "gas".to_string(),
            value: *value,
        });
    }
    
    // Add cross_program benchmark
    entries.push(BenchmarkEntry {
        name: "cross_program".to_string(),
        unit: "gas".to_string(),
        value: bench_data.cross_program,
    });
    
    // Add redirect benchmark
    entries.push(BenchmarkEntry {
        name: "redirect".to_string(),
        unit: "gas".to_string(),
        value: bench_data.redirect,
    });
    
    // Write the output file
    let output_json = serde_json::to_string_pretty(&entries)?;
    fs::write(&output_file, output_json)?;
    
    println!("Successfully converted '{}' to '{}'", input_file, output_file);
    println!("Generated {} benchmark entries", entries.len());
    
    Ok(())
}
