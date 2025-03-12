use rustzss::LZSS;
use std::time::Instant;
use rand::prelude::*;

fn main() {
    println!("LZSS Test Suite");
    println!("===============\n");
    
    // Test with a small repeating pattern
    let small_pattern = "ABABCBABABCBABABCBABABCBABABCBABABCBABABCBABABCBABABC".repeat(20).into_bytes();
    test_compression_decompression("Small pattern", &small_pattern);
    
    // Test with a small random pattern
    let random_data = generate_random_data(1000);
    test_compression_decompression("Small random", &random_data);
    
    // Test with medium data
    let medium_pattern = "Hello, this is a test of LZSS compression algorithm.".repeat(2000).into_bytes();
    test_compression_decompression("Medium pattern", &medium_pattern);
    
    // Test with large data (1MB)
    println!("Generating 1MB test data...");
    let large_pattern = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".repeat(30000).into_bytes();
    test_compression_decompression("Large pattern (1MB)", &large_pattern);
    
    // Test with 10MB data if user wants to run it
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--huge" {
        println!("Generating 10MB test data...");
        let huge_pattern = generate_random_data(10_000_000);
        test_compression_decompression("Huge pattern (10MB)", &huge_pattern);
    }
    
    println!("All tests completed!");
}

fn generate_random_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.gen::<u8>());
    }
    data
}

fn test_compression_decompression(test_name: &str, data: &[u8]) {
    println!("Running test: {}", test_name);
    println!("Input size: {} bytes", data.len());
    
    // Create compressor
    let lzss = LZSS::new(4096, 3);
    
    // Compress
    let start = Instant::now();
    let compressed = lzss.compress(data);
    let compress_time = start.elapsed();
    
    // Calculate compression ratio
    let ratio = (compressed.len() as f64) / (data.len() as f64) * 100.0;
    println!("Compressed: {} bytes, Ratio: {:.2}%", compressed.len(), ratio);
    println!("Compression time: {:?}", compress_time);
    
    // Decompress
    let start = Instant::now();
    let decompressed = lzss.decompress(&compressed);
    let decompress_time = start.elapsed();
    println!("Decompression time: {:?}", decompress_time);
    
    // Verify
    if data.len() != decompressed.len() {
        println!("FAILED: Size mismatch! Original: {}, Decompressed: {}", 
                data.len(), decompressed.len());
    } else if data != decompressed.as_slice() {
        // Find first mismatch for debugging
        for i in 0..data.len() {
            if data[i] != decompressed[i] {
                println!("FAILED: Content mismatch at position {}! Original: {}, Decompressed: {}", 
                        i, data[i], decompressed[i]);
                break;
            }
        }
    } else {
        println!("PASSED: Original and decompressed data match perfectly");
    }
    println!();
}