#[cfg(feature = "autotune")]
use rustzss::autotune::{
    AssetInfo, AssetType, Tuner, TunerConfig, quick_benchmark, scan_directory
};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Duration;

#[cfg(feature = "autotune")]
fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "benchmark" => {
            if args.len() < 3 {
                eprintln!("Error: Missing directory path for benchmark.");
                print_usage();
                process::exit(1);
            }
            benchmark_directory(&args[2], args.get(3).map(|s| s.parse().unwrap_or(10)))?;
        }
        "tune" => {
            if args.len() < 3 {
                eprintln!("Error: Missing directory path for tuning.");
                print_usage();
                process::exit(1);
            }
            
            // Parse ratio priority
            let ratio_priority = args.get(3)
                .map(|s| s.parse().unwrap_or(0.5))
                .unwrap_or(0.5);
                
            tune_directory(&args[2], ratio_priority)?;
        }
        "profile" => {
            if args.len() < 3 {
                eprintln!("Error: Missing directory path for profiling.");
                print_usage();
                process::exit(1);
            }
            
            // Generate profile
            profile_asset_types(&args[2])?;
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage();
            process::exit(1);
        }
    }

    Ok(())
}

#[cfg(not(feature = "autotune"))]
fn main() {
    eprintln!("The autotune feature is not enabled. Please rebuild with --features=autotune");
    process::exit(1);
}

fn print_usage() {
    println!("RustLZSS Autotuner - Optimize LZSS compression for game assets");
    println!("\nUsage:");
    println!("  autotune benchmark <directory> [max_files]");
    println!("    - Run benchmark on assets in the directory with default parameters");
    println!("  autotune tune <directory> [ratio_priority]");
    println!("    - Tune parameters for assets in the directory");
    println!("    - ratio_priority: A value between 0.0 (prioritize speed) and 1.0 (prioritize compression ratio)");
    println!("  autotune profile <directory>");
    println!("    - Generate optimal parameter profiles for different asset types");
    println!("  autotune help");
    println!("    - Display this help message");
}

#[cfg(feature = "autotune")]
fn benchmark_directory(dir_path: &str, max_files: Option<usize>) -> io::Result<()> {
    println!("Scanning directory {} for assets...", dir_path);
    let mut assets = scan_directory(dir_path, max_files)?;
    
    println!("Found {} assets", assets.len());
    if assets.is_empty() {
        println!("No assets found to benchmark");
        return Ok(());
    }
    
    // Group assets by type
    let mut asset_groups: HashMap<AssetType, Vec<AssetInfo>> = HashMap::new();
    for asset in assets.drain(..) {
        asset_groups.entry(asset.asset_type).or_default().push(asset);
    }
    
    // Benchmark each type
    for (asset_type, mut group) in asset_groups {
        println!("\nBenchmarking {:?} assets ({} files)", asset_type, group.len());
        
        let mut total_size = 0;
        let mut total_compressed_size = 0;
        let mut count = 0;
        
        for asset in group.iter_mut().take(5) { // Limit to 5 per group for brevity
            println!("\n{} ({:?}, {} bytes):", asset.filename(), asset.asset_type, asset.size);
            
            // Run quick benchmark
            if let Some(result) = quick_benchmark(asset) {
                println!("  Compression ratio: {:.2}%", result.compression_ratio_percent());
                println!("  Compression throughput: {:.2} MB/s", result.compression_throughput());
                println!("  Decompression throughput: {:.2} MB/s", result.decompression_throughput());
                
                total_size += result.original_size;
                total_compressed_size += result.compressed_size;
                count += 1;
            }
        }
        
        // Print aggregate stats
        if count > 0 {
            let avg_ratio = (total_compressed_size as f64) / (total_size as f64) * 100.0;
            println!("\nSummary for {:?} assets:", asset_type);
            println!("  Average compression ratio: {:.2}%", avg_ratio);
            println!("  Total original size: {} bytes", total_size);
            println!("  Total compressed size: {} bytes", total_compressed_size);
        }
    }
    
    Ok(())
}

#[cfg(feature = "autotune")]
fn tune_directory(dir_path: &str, ratio_priority: f64) -> io::Result<()> {
    println!("Scanning directory {} for assets...", dir_path);
    let mut assets = scan_directory(dir_path, Some(50))?; // Limit to 50 files for reasonable tuning time
    
    println!("Found {} assets for tuning", assets.len());
    if assets.is_empty() {
        println!("No assets found to tune");
        return Ok(());
    }
    
    // Configure tuner
    let config = TunerConfig {
        benchmark_runs: 2,
        max_tuning_time: Some(Duration::from_secs(300)), // 5 minute limit
        max_iterations: 20,
        ratio_priority,
        random_seed: None,
        parallel: true,
    };
    
    println!("Starting parameter tuning with ratio_priority = {:.2}", ratio_priority);
    println!("This may take a few minutes...");
    
    // Create tuner and run tuning
    let mut tuner = Tuner::new(config);
    let result = tuner.tune_for_assets(&mut assets);
    
    // Print results
    println!("\nTuning Results:");
    println!("Time taken: {:?}", result.tuning_time);
    println!("Parameters tested: {}", result.iterations);
    
    println!("\nBest Overall Parameters:");
    println!("Window Size: {}, Min Match Length: {}", 
             result.best_parameters.window_size, 
             result.best_parameters.min_match_length);
    println!("Compression Ratio: {:.2}%", result.best_result.compression_ratio_percent());
    println!("Compression Throughput: {:.2} MB/s", result.best_result.compression_throughput());
    println!("Decompression Throughput: {:.2} MB/s", result.best_result.decompression_throughput());
    
    println!("\nBest Ratio Parameters (regardless of speed):");
    println!("Window Size: {}, Min Match Length: {}", 
             result.best_ratio_parameters.window_size, 
             result.best_ratio_parameters.min_match_length);
    
    println!("\nBest Speed Parameters (regardless of ratio):");
    println!("Window Size: {}, Min Match Length: {}", 
             result.best_speed_parameters.window_size, 
             result.best_speed_parameters.min_match_length);
    
    // Save tuning results to a file
    let output_path = Path::new(dir_path).join("rustzss_tuning_results.txt");
    let mut file = File::create(&output_path)?;
    
    writeln!(file, "RustLZSS Tuning Results")?;
    writeln!(file, "======================")?;
    writeln!(file, "Assets directory: {}", dir_path)?;
    writeln!(file, "Number of assets tested: {}", assets.len())?;
    writeln!(file, "Ratio priority: {:.2}", ratio_priority)?;
    writeln!(file, "Time taken: {:?}", result.tuning_time)?;
    writeln!(file, "Parameters tested: {}", result.iterations)?;
    
    writeln!(file, "\nRecommended Parameters:")?;
    writeln!(file, "Window Size: {}", result.best_parameters.window_size)?;
    writeln!(file, "Min Match Length: {}", result.best_parameters.min_match_length)?;
    writeln!(file, "Compression Ratio: {:.2}%", result.best_result.compression_ratio_percent())?;
    
    println!("\nResults saved to {}", output_path.display());
    
    Ok(())
}

#[cfg(feature = "autotune")]
fn profile_asset_types(dir_path: &str) -> io::Result<()> {
    println!("Scanning directory {} for assets...", dir_path);
    let mut assets = scan_directory(dir_path, Some(100))?; // Limit to 100 files
    
    println!("Found {} assets for profiling", assets.len());
    if assets.is_empty() {
        println!("No assets found to profile");
        return Ok(());
    }
    
    // Configure tuner
    let config = TunerConfig {
        benchmark_runs: 2,
        max_tuning_time: Some(Duration::from_secs(600)), // 10 minute limit
        max_iterations: 15,  // Fewer iterations per asset type
        ratio_priority: 0.5, // Balanced approach
        random_seed: None,
        parallel: true,
    };
    
    println!("Generating asset type profiles...");
    println!("This may take several minutes...");
    
    // Create tuner and generate profiles
    let mut tuner = Tuner::new(config);
    let profiles = tuner.generate_asset_profiles(&mut assets);
    
    // Print and save results
    println!("\nAsset Type Profiles:");
    
    let output_path = Path::new(dir_path).join("rustzss_asset_profiles.txt");
    let mut file = File::create(&output_path)?;
    
    writeln!(file, "RustLZSS Asset Type Profiles")?;
    writeln!(file, "============================")?;
    writeln!(file, "Assets directory: {}", dir_path)?;
    writeln!(file, "Number of assets analyzed: {}", assets.len())?;
    
    writeln!(file, "\nOptimal parameters for each asset type:")?;
    
    for (asset_type, params) in &profiles {
        println!("{:?}: Window Size = {}, Min Match Length = {}", 
                 asset_type, params.window_size, params.min_match_length);
        
        writeln!(file, "{:?}:", asset_type)?;
        writeln!(file, "  Window Size: {}", params.window_size)?;
        writeln!(file, "  Min Match Length: {}", params.min_match_length)?;
    }
    
    // Generate code snippet
    writeln!(file, "\n// Code snippet for easy integration:")?;
    writeln!(file, "fn get_optimal_parameters(asset_type: AssetType) -> (usize, usize) {{")?;
    writeln!(file, "    match asset_type {{")?;
    
    for (asset_type, params) in &profiles {
        writeln!(file, "        AssetType::{:?} => ({}, {}),", 
                 asset_type, params.window_size, params.min_match_length)?;
    }
    
    writeln!(file, "        _ => (4096, 3), // Default parameters")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    
    println!("\nProfiles saved to {}", output_path.display());
    
    Ok(())
}