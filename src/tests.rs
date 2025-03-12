#[cfg(test)]
mod tests {
    use crate::LZSS;
    use rand::prelude::*;
    use std::time::Instant;

    // Generate random data of specified size
    fn generate_random_data(size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(rng.gen::<u8>());
        }
        data
    }

    // Generate repeating pattern data of specified size
    fn generate_pattern_data(size: usize) -> Vec<u8> {
        let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut data = Vec::with_capacity(size);
        while data.len() < size {
            data.extend_from_slice(pattern);
        }
        data.truncate(size);
        data
    }

    // Test a compression/decompression cycle and return success/failure
    fn test_compression_cycle(lzss: &LZSS, data: &[u8], name: &str, print_results: bool) -> bool {
        if print_results {
            println!("Testing with {}: {} bytes", name, data.len());
        }

        let start = Instant::now();
        let compressed = lzss.compress(data);
        let compress_time = start.elapsed();

        let start = Instant::now();
        let decompressed = lzss.decompress(&compressed);
        let decompress_time = start.elapsed();

        let ratio = (compressed.len() as f64) / (data.len() as f64) * 100.0;
        
        if print_results {
            println!("  Original: {} bytes", data.len());
            println!("  Compressed: {} bytes ({:.2}%)", compressed.len(), ratio);
            println!("  Compression time: {:?}", compress_time);
            println!("  Decompression time: {:?}", decompress_time);
        }

        // Verify the data matches
        let matches = data.len() == decompressed.len() && data == decompressed.as_slice();
        
        if print_results {
            println!("  Result: {}", if matches { "PASSED" } else { "FAILED" });
            
            if !matches {
                if data.len() != decompressed.len() {
                    println!("  Size mismatch: original {} bytes, decompressed {} bytes",
                             data.len(), decompressed.len());
                } else {
                    // Find first mismatch
                    for i in 0..data.len() {
                        if data[i] != decompressed[i] {
                            println!("  First mismatch at position {}: expected {}, got {}",
                                     i, data[i], decompressed[i]);
                            break;
                        }
                    }
                }
            }
        }
        
        matches
    }

    #[test]
    fn test_empty_data() {
        let lzss = LZSS::new(4096, 3);
        let data = Vec::new();
        let compressed = lzss.compress(&data);
        let decompressed = lzss.decompress(&compressed);
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_minimal_data() {
        let lzss = LZSS::new(4096, 3);
        let data = vec![42];  // Just a single byte
        let compressed = lzss.compress(&data);
        let decompressed = lzss.decompress(&compressed);
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_small_pattern() {
        let lzss = LZSS::new(4096, 3);
        let data = b"ABABCBABABCBABABCBABABCBABABCBABABCBABABCBABABCBABABC".to_vec();
        assert!(test_compression_cycle(&lzss, &data, "Small repeating pattern", true));
    }

    #[test]
    fn test_repeated_pattern() {
        let lzss = LZSS::new(4096, 3);
        let mut data = Vec::new();
        for _ in 0..1000 {
            data.extend_from_slice(b"HelloWorld");
        }
        assert!(test_compression_cycle(&lzss, &data, "Medium repeating pattern", true));
    }

    #[test]
    fn test_random_small() {
        let lzss = LZSS::new(4096, 3);
        let data = generate_random_data(1000);
        assert!(test_compression_cycle(&lzss, &data, "Small random data", true));
    }

    #[test]
    fn test_random_medium() {
        let lzss = LZSS::new(4096, 3);
        let data = generate_random_data(100_000);
        assert!(test_compression_cycle(&lzss, &data, "Medium random data", true));
    }

    #[test]
    #[ignore] // This test may take some time
    fn test_random_large() {
        let lzss = LZSS::new(4096, 3);
        let data = generate_random_data(1_000_000); // 1MB
        assert!(test_compression_cycle(&lzss, &data, "Large random data (1MB)", true));
    }

    #[test]
    #[ignore] // This test takes considerable time and memory
    fn test_random_huge() {
        let lzss = LZSS::new(4096, 3);
        let data = generate_random_data(10_000_000); // 10MB
        assert!(test_compression_cycle(&lzss, &data, "Huge random data (10MB)", true));
    }

    #[test]
    fn test_pattern_large() {
        let lzss = LZSS::new(4096, 3);
        let data = generate_pattern_data(1_000_000); // 1MB
        assert!(test_compression_cycle(&lzss, &data, "Large pattern data (1MB)", true));
    }

    #[test]
    #[ignore] // This test takes considerable time and memory
    fn test_pattern_huge() {
        let lzss = LZSS::new(4096, 3);
        let data = generate_pattern_data(10_000_000); // 10MB
        assert!(test_compression_cycle(&lzss, &data, "Huge pattern data (10MB)", true));
    }

    #[test]
    fn test_various_window_sizes() {
        let data = generate_pattern_data(100_000);
        
        println!("\nTesting different window sizes:");
        for &window_size in &[256, 512, 1024, 4096, 8192, 16384] {
            let lzss = LZSS::new(window_size, 3);
            let name = format!("Window size {}", window_size);
            assert!(test_compression_cycle(&lzss, &data, &name, true));
        }
    }

    #[test]
    fn test_various_min_match_lengths() {
        let data = generate_pattern_data(100_000);
        
        println!("\nTesting different minimum match lengths:");
        for &min_match in &[2, 3, 4, 5, 6, 8] {
            let lzss = LZSS::new(4096, min_match);
            let name = format!("Min match length {}", min_match);
            assert!(test_compression_cycle(&lzss, &data, &name, true));
        }
    }
}