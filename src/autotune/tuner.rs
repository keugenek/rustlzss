use std::collections::HashMap;
use std::time::Duration;
use rand::{seq::SliceRandom, Rng};
use rayon::prelude::*;

use super::asset_loader::AssetInfo;
use super::benchmark::{BenchmarkResult, CompressionParameters, run_benchmark};

/// Configuration for parameter tuning
#[derive(Debug, Clone)]
pub struct TunerConfig {
    /// Number of benchmark runs for each parameter set
    pub benchmark_runs: usize,
    /// Maximum time to spend tuning (if specified)
    pub max_tuning_time: Option<Duration>,
    /// Maximum number of parameter sets to try
    pub max_iterations: usize,
    /// Weighted priority for compression ratio vs speed (1.0 = only ratio, 0.0 = only speed)
    pub ratio_priority: f64,
    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
    /// Whether to enable parallel tuning
    pub parallel: bool,
}

impl Default for TunerConfig {
    fn default() -> Self {
        TunerConfig {
            benchmark_runs: 3,
            max_tuning_time: None,
            max_iterations: 30,
            ratio_priority: 0.5,
            random_seed: None,
            parallel: true,
        }
    }
}

/// Optimal parameters found by the tuner
#[derive(Debug, Clone)]
pub struct TuningResult {
    /// The best parameters found
    pub best_parameters: CompressionParameters,
    /// The benchmark result for the best parameters
    pub best_result: BenchmarkResult,
    /// All benchmark results evaluated during tuning
    pub all_results: Vec<BenchmarkResult>,
    /// Parameters optimized for best ratio (regardless of speed)
    pub best_ratio_parameters: CompressionParameters,
    /// Parameters optimized for best speed (regardless of ratio)
    pub best_speed_parameters: CompressionParameters,
    /// How long the tuning process took
    pub tuning_time: Duration,
    /// How many iterations were performed
    pub iterations: usize,
}

/// Parameter tuner for finding optimal LZSS parameters
pub struct Tuner {
    config: TunerConfig,
    results: Vec<BenchmarkResult>,
    best_score: f64,
    best_parameters: Option<CompressionParameters>,
    best_ratio: f64,
    best_ratio_parameters: Option<CompressionParameters>,
    best_speed: f64,
    best_speed_parameters: Option<CompressionParameters>,
    parameter_space: Vec<CompressionParameters>,
    tested_parameters: HashMap<CompressionParameters, BenchmarkResult>,
}

impl Tuner {
    /// Create a new parameter tuner with the given configuration
    pub fn new(config: TunerConfig) -> Self {
        let mut parameter_space = Vec::new();
        
        // Generate parameter space (window sizes and min match lengths)
        let window_sizes = [
            256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65535
        ];
        
        let min_match_lengths = [2, 3, 4, 5, 6, 8];
        
        for &window_size in &window_sizes {
            for &min_match in &min_match_lengths {
                parameter_space.push(CompressionParameters::new(window_size, min_match));
            }
        }
        
        // Shuffle the parameter space for better exploration
        let mut rng = match config.random_seed {
            Some(seed) => rand::rngs::StdRng::seed_from_u64(seed),
            None => rand::thread_rng(),
        };
        
        parameter_space.shuffle(&mut rng);
        
        Tuner {
            config,
            results: Vec::new(),
            best_score: 0.0,
            best_parameters: None,
            best_ratio: f64::MAX,
            best_ratio_parameters: None,
            best_speed: 0.0,
            best_speed_parameters: None,
            parameter_space,
            tested_parameters: HashMap::new(),
        }
    }
    
    /// Tune parameters for a single asset
    pub fn tune_for_asset(&mut self, asset: &mut AssetInfo) -> TuningResult {
        let data = match asset.data() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error loading asset {}: {}", asset.filename(), e);
                return self.empty_result();
            }
        };
        
        self.tune_for_data(data, Some(asset))
    }
    
    /// Tune parameters for a collection of assets
    pub fn tune_for_assets(&mut self, assets: &mut [AssetInfo]) -> TuningResult {
        let start_time = std::time::Instant::now();
        let mut iterations = 0;
        
        // Try each parameter set on all assets
        while iterations < self.config.max_iterations && !self.parameter_space.is_empty() {
            let params = self.parameter_space.remove(0);
            
            // Skip if we've already tested these parameters
            if self.tested_parameters.contains_key(&params) {
                continue;
            }
            
            // Run benchmarks for each asset with these parameters
            let results: Vec<BenchmarkResult> = if self.config.parallel {
                // Load all asset data first to avoid IO during parallel execution
                let asset_data: Vec<_> = assets
                    .iter_mut()
                    .filter_map(|asset| asset.data().ok().map(|data| (data.to_vec(), asset)))
                    .collect();
                
                asset_data.par_iter()
                    .map(|(data, asset)| {
                        run_benchmark(data, params, Some(*asset), self.config.benchmark_runs)
                    })
                    .collect()
            } else {
                assets.iter_mut()
                    .filter_map(|asset| {
                        asset.data().ok().map(|data| {
                            run_benchmark(data, params, Some(asset), self.config.benchmark_runs)
                        })
                    })
                    .collect()
            };
            
            if results.is_empty() {
                continue;
            }
            
            // Calculate aggregate scores
            let avg_ratio = results.iter().map(|r| r.compression_ratio()).sum::<f64>() / results.len() as f64;
            let avg_speed = results.iter().map(|r| (r.compression_throughput() + r.decompression_throughput()) / 2.0).sum::<f64>() / results.len() as f64;
            
            // Calculate combined score with user-defined priority
            let ratio_score = 1.0 / avg_ratio; // Invert ratio so higher is better
            let speed_score = avg_speed / 100.0; // Normalize to a similar range
            let combined_score = (ratio_score * self.config.ratio_priority) + (speed_score * (1.0 - self.config.ratio_priority));
            
            // Track best parameters
            if combined_score > self.best_score {
                self.best_score = combined_score;
                self.best_parameters = Some(params);
            }
            
            // Track best ratio parameters
            if avg_ratio < self.best_ratio {
                self.best_ratio = avg_ratio;
                self.best_ratio_parameters = Some(params);
            }
            
            // Track best speed parameters
            if avg_speed > self.best_speed {
                self.best_speed = avg_speed;
                self.best_speed_parameters = Some(params);
            }
            
            // Store results for this parameter set
            let aggregate_result = BenchmarkResult {
                original_size: results.iter().map(|r| r.original_size).sum(),
                compressed_size: results.iter().map(|r| r.compressed_size).sum(),
                compression_time: Duration::from_secs_f64(results.iter().map(|r| r.compression_time.as_secs_f64()).sum::<f64>() / results.len() as f64),
                decompression_time: Duration::from_secs_f64(results.iter().map(|r| r.decompression_time.as_secs_f64()).sum::<f64>() / results.len() as f64),
                parameters: params,
                asset_info: Some(format!("Aggregate of {} assets", results.len())),
            };
            
            self.results.push(aggregate_result.clone());
            self.tested_parameters.insert(params, aggregate_result);
            
            iterations += 1;
            
            // Check if we've exceeded our time budget
            if let Some(max_time) = self.config.max_tuning_time {
                if start_time.elapsed() >= max_time {
                    break;
                }
            }
        }
        
        // Create tuning result
        self.create_tuning_result(start_time.elapsed(), iterations)
    }
    
    /// Tune parameters for a single data buffer
    pub fn tune_for_data(&mut self, data: &[u8], asset: Option<&AssetInfo>) -> TuningResult {
        let start_time = std::time::Instant::now();
        let mut iterations = 0;
        
        // Try each parameter set
        let mut i = 0;
        while i < self.parameter_space.len() && iterations < self.config.max_iterations {
            let params = self.parameter_space[i];
            i += 1;
            
            // Skip if we've already tested these parameters
            if self.tested_parameters.contains_key(&params) {
                continue;
            }
            
            // Run benchmark with these parameters
            let result = run_benchmark(data, params, asset, self.config.benchmark_runs);
            
            // Calculate scores
            let ratio = result.compression_ratio();
            let speed = (result.compression_throughput() + result.decompression_throughput()) / 2.0;
            
            // Calculate combined score with user-defined priority
            let ratio_score = 1.0 / ratio; // Invert ratio so higher is better
            let speed_score = speed / 100.0; // Normalize to a similar range
            let combined_score = (ratio_score * self.config.ratio_priority) + (speed_score * (1.0 - self.config.ratio_priority));
            
            // Track best parameters
            if combined_score > self.best_score {
                self.best_score = combined_score;
                self.best_parameters = Some(params);
            }
            
            // Track best ratio parameters
            if ratio < self.best_ratio {
                self.best_ratio = ratio;
                self.best_ratio_parameters = Some(params);
            }
            
            // Track best speed parameters
            if speed > self.best_speed {
                self.best_speed = speed;
                self.best_speed_parameters = Some(params);
            }
            
            self.results.push(result.clone());
            self.tested_parameters.insert(params, result);
            
            iterations += 1;
            
            // Check if we've exceeded our time budget
            if let Some(max_time) = self.config.max_tuning_time {
                if start_time.elapsed() >= max_time {
                    break;
                }
            }
        }
        
        // Create tuning result
        self.create_tuning_result(start_time.elapsed(), iterations)
    }
    
    /// Create a tuning result based on current state
    fn create_tuning_result(&self, duration: Duration, iterations: usize) -> TuningResult {
        if let (Some(best_params), Some(best_ratio_params), Some(best_speed_params)) = 
            (self.best_parameters, self.best_ratio_parameters, self.best_speed_parameters) {
            
            TuningResult {
                best_parameters: best_params,
                best_result: self.tested_parameters[&best_params].clone(),
                all_results: self.results.clone(),
                best_ratio_parameters: best_ratio_params,
                best_speed_parameters: best_speed_params,
                tuning_time: duration,
                iterations,
            }
        } else {
            self.empty_result()
        }
    }
    
    /// Create an empty tuning result for error cases
    fn empty_result(&self) -> TuningResult {
        let default_params = CompressionParameters::new(4096, 3);
        
        TuningResult {
            best_parameters: default_params,
            best_result: BenchmarkResult {
                original_size: 0,
                compressed_size: 0,
                compression_time: Duration::new(0, 0),
                decompression_time: Duration::new(0, 0),
                parameters: default_params,
                asset_info: None,
            },
            all_results: Vec::new(),
            best_ratio_parameters: default_params,
            best_speed_parameters: default_params,
            tuning_time: Duration::new(0, 0),
            iterations: 0,
        }
    }
    
    /// Generate a set of optimal parameters for different asset types
    pub fn generate_asset_profiles(&mut self, assets: &mut [AssetInfo]) -> HashMap<super::asset_loader::AssetType, CompressionParameters> {
        // Group assets by type
        use super::asset_loader::AssetType;
        let mut asset_groups: HashMap<AssetType, Vec<&mut AssetInfo>> = HashMap::new();
        
        for asset in assets {
            asset_groups.entry(asset.asset_type).or_default().push(asset);
        }
        
        // Tune parameters for each asset type
        let mut profiles = HashMap::new();
        
        for (asset_type, group) in asset_groups {
            if asset_type == AssetType::Unknown || group.is_empty() {
                continue;
            }
            
            println!("Tuning for asset type: {:?} ({} assets)", asset_type, group.len());
            
            // Take a sample if the group is large
            let sample: Vec<_> = if group.len() > 5 {
                let mut rng = rand::thread_rng();
                group.choose_multiple(&mut rng, 5).cloned().collect()
            } else {
                group
            };
            
            // Reset tuner state
            self.results.clear();
            self.best_score = 0.0;
            self.best_parameters = None;
            self.best_ratio = f64::MAX;
            self.best_ratio_parameters = None;
            self.best_speed = 0.0;
            self.best_speed_parameters = None;
            self.tested_parameters.clear();
            self.parameter_space = self.parameter_space.clone();
            
            // Tune for this asset type
            let result = self.tune_for_assets(&mut sample.iter_mut().copied().collect::<Vec<_>>());
            profiles.insert(asset_type, result.best_parameters);
            
            println!("  Best parameters: {}", result.best_parameters);
            println!("  Compression ratio: {:.2}%", result.best_result.compression_ratio_percent());
        }
        
        profiles
    }
}

/// Perform a quick benchmark with standard parameters on the given asset
pub fn quick_benchmark(asset: &mut AssetInfo) -> Option<BenchmarkResult> {
    match asset.data() {
        Ok(data) => {
            let params = CompressionParameters::new(4096, 3);
            Some(run_benchmark(data, params, Some(asset), 1))
        },
        Err(e) => {
            eprintln!("Error loading asset {}: {}", asset.filename(), e);
            None
        }
    }
}