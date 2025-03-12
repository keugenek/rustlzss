use crate::LZSS;
use std::time::{Duration, Instant};
use std::fmt;

use super::asset_loader::AssetInfo;

/// Results from a compression benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression time
    pub compression_time: Duration,
    /// Decompression time
    pub decompression_time: Duration,
    /// The compression parameters used
    pub parameters: CompressionParameters,
    /// Asset information
    pub asset_info: Option<String>,
}

impl BenchmarkResult {
    /// Calculate the compression ratio (compressed / original)
    pub fn compression_ratio(&self) -> f64 {
        self.compressed_size as f64 / self.original_size as f64
    }
    
    /// Calculate the compression ratio as a percentage
    pub fn compression_ratio_percent(&self) -> f64 {
        self.compression_ratio() * 100.0
    }
    
    /// Calculate the compression throughput (MB/s)
    pub fn compression_throughput(&self) -> f64 {
        let seconds = self.compression_time.as_secs_f64();
        if seconds > 0.0 {
            (self.original_size as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }
    
    /// Calculate the decompression throughput (MB/s)
    pub fn decompression_throughput(&self) -> f64 {
        let seconds = self.decompression_time.as_secs_f64();
        if seconds > 0.0 {
            (self.original_size as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }
    
    /// Combined score that balances compression ratio and speed
    /// Higher is better
    pub fn score(&self) -> f64 {
        // Weight factors (can be adjusted)
        let ratio_weight = 0.4;
        let speed_weight = 0.6;
        
        // Compression ratio score (lower is better, so we invert it)
        let ratio_score = 1.0 / self.compression_ratio();
        
        // Speed score (higher is better)
        let compression_speed = self.compression_throughput();
        let decompression_speed = self.decompression_throughput();
        let speed_score = (compression_speed + decompression_speed) / 2.0;
        
        // Normalize speed score to be in a similar range to ratio_score
        let normalized_speed_score = speed_score / 100.0;
        
        // Combined score
        (ratio_score * ratio_weight) + (normalized_speed_score * speed_weight)
    }
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Benchmark Results:")?;
        if let Some(asset) = &self.asset_info {
            writeln!(f, "Asset: {}", asset)?;
        }
        writeln!(f, "Parameters: {}", self.parameters)?;
        writeln!(f, "Original size: {} bytes", self.original_size)?;
        writeln!(f, "Compressed size: {} bytes", self.compressed_size)?;
        writeln!(f, "Compression ratio: {:.2}%", self.compression_ratio_percent())?;
        writeln!(f, "Compression time: {:?}", self.compression_time)?;
        writeln!(f, "Decompression time: {:?}", self.decompression_time)?;
        writeln!(f, "Compression throughput: {:.2} MB/s", self.compression_throughput())?;
        writeln!(f, "Decompression throughput: {:.2} MB/s", self.decompression_throughput())?;
        write!(f, "Score: {:.2}", self.score())
    }
}

/// Parameters for configuring the LZSS compression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompressionParameters {
    /// Window size in bytes
    pub window_size: usize,
    /// Minimum match length
    pub min_match_length: usize,
}

impl CompressionParameters {
    /// Create a new set of compression parameters
    pub fn new(window_size: usize, min_match_length: usize) -> Self {
        CompressionParameters {
            window_size,
            min_match_length,
        }
    }
    
    /// Create an LZSS instance with these parameters
    pub fn create_lzss(&self) -> LZSS {
        LZSS::new(self.window_size, self.min_match_length)
    }
}

impl fmt::Display for CompressionParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "win_size={}, min_match={}",
            self.window_size, self.min_match_length
        )
    }
}

/// Runs a benchmark with the given data and compression parameters
pub fn run_benchmark(
    data: &[u8], 
    parameters: CompressionParameters,
    asset_info: Option<&AssetInfo>,
    runs: usize,
) -> BenchmarkResult {
    let lzss = parameters.create_lzss();
    
    // Run multiple times for more reliable results
    let mut total_compression_time = Duration::new(0, 0);
    let mut total_decompression_time = Duration::new(0, 0);
    let mut compressed = Vec::new();
    
    for i in 0..runs {
        // Measure compression time
        let start = Instant::now();
        compressed = lzss.compress(data);
        let end = Instant::now();
        total_compression_time += end.duration_since(start);
        
        // Measure decompression time (skip first run for warming up)
        if i > 0 {
            let start = Instant::now();
            let decompressed = lzss.decompress(&compressed);
            let end = Instant::now();
            total_decompression_time += end.duration_since(start);
            
            // Verify correctness
            assert_eq!(decompressed.len(), data.len(), "Decompressed size mismatch");
            assert_eq!(decompressed, data, "Decompressed data mismatch");
        }
    }
    
    // Calculate average times (divide by runs count, but skip first decompression run)
    let avg_compression_time = total_compression_time / runs as u32;
    let avg_decompression_time = total_decompression_time / (runs - 1) as u32;
    
    BenchmarkResult {
        original_size: data.len(),
        compressed_size: compressed.len(),
        compression_time: avg_compression_time,
        decompression_time: avg_decompression_time,
        parameters,
        asset_info: asset_info.map(|info| format!("{} ({})", info.filename(), info.asset_type)),
    }
}