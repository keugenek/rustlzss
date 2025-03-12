use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rustzss::LZSS;

fn generate_random_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.gen::<u8>());
    }
    data
}

fn generate_repeated_data(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let pattern = b"This is a test pattern with some repeated content. ";
    
    while data.len() < size {
        data.extend_from_slice(pattern);
    }
    
    data.truncate(size);
    data
}

fn generate_text_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(size);
    
    // Common English letter frequencies
    let alphabet = b"etaoinshrdlucmfwypvbgkjqxz ";
    let weights = [
        12.0, 9.0, 8.0, 7.5, 7.0, 6.5, 6.0, 5.5, 5.0, 4.5, 4.0, 3.5, 3.0, 
        2.5, 2.0, 1.8, 1.5, 1.3, 1.0, 0.8, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 15.0
    ];
    
    // Generate weighted random text
    let total_weight: f32 = weights.iter().sum();
    while data.len() < size {
        let r: f32 = rng.gen::<f32>() * total_weight;
        let mut cumulative = 0.0;
        for (i, &w) in weights.iter().enumerate() {
            cumulative += w;
            if r <= cumulative {
                data.push(alphabet[i]);
                break;
            }
        }
    }
    
    data
}

fn basic_benchmark(c: &mut Criterion) {
    let lzss = LZSS::new(4096, 3);
    
    // Random data - 10KB
    let random_data = generate_random_data(1000_000);
    let compressed_random = lzss.compress(&random_data);
    
    // Repeated data - 10KB  
    let repeated_data = generate_repeated_data(1000_000);
    let compressed_repeated = lzss.compress(&repeated_data);
    
    // Text data - 10KB
    let text_data = generate_text_data(1000_000);
    let compressed_text = lzss.compress(&text_data);
    
    // Compression benchmarks
    c.bench_function("compress_random", |b| {
        b.iter(|| lzss.compress(black_box(&random_data)))
    });
    
    c.bench_function("compress_repeated", |b| {
        b.iter(|| lzss.compress(black_box(&repeated_data)))
    });
    
    c.bench_function("compress_text", |b| {
        b.iter(|| lzss.compress(black_box(&text_data)))
    });
    
    // Decompression benchmarks
    c.bench_function("decompress_random", |b| {
        b.iter(|| lzss.decompress(black_box(&compressed_random)))
    });
    
    c.bench_function("decompress_repeated", |b| {
        b.iter(|| lzss.decompress(black_box(&compressed_repeated)))
    });
    
    c.bench_function("decompress_text", |b| {
        b.iter(|| lzss.decompress(black_box(&compressed_text)))
    });
    
    // Print compression ratios
    println!("\nCompression ratios:");
    println!("  Random data: {:.2}%", 
             (compressed_random.len() as f64) / (random_data.len() as f64) * 100.0);
    println!("  Repeated data: {:.2}%", 
             (compressed_repeated.len() as f64) / (repeated_data.len() as f64) * 100.0);
    println!("  Text data: {:.2}%", 
             (compressed_text.len() as f64) / (text_data.len() as f64) * 100.0);
}

fn window_size_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Window size comparison");
    
    // Generate a Wikipedia-like text (500KB)
    let mut data = Vec::new();
    let pattern = b"War and Peace (Russian: \xD0\x92\xD0\xBE\xD0\xB9\xD0\xBD\xD0\xB0 \xD0\xB8 \xD0\xBC\xD0\xB8\xD1\x80, romanized: Voyna i mir) is a literary work by Leo Tolstoy that chronicles the French invasion of Russia and the impact of the Napoleonic era on Tsarist society through the stories of five Russian aristocratic families. ";
    
    while data.len() < 500_000 {
        data.extend_from_slice(pattern);
        // Add some random variation to make distant matches more realistic
        if data.len() % 5000 < 100 {
            data.extend_from_slice(b"Pierre Bezukhov is the central character and often a voice for Tolstoy's own beliefs or struggles. ");
        } 
        else if data.len() % 8000 < 100 {
            data.extend_from_slice(b"Natasha Rostova is a central character, introduced as not pretty but full of life, romantic and impulsive. ");
        }
    }
    data.truncate(500_000);
    
    // Test different window sizes
    let window_sizes = [1024, 4096, 8192, 16384, 32768];
    
    for &window_size in &window_sizes {
        let lzss = LZSS::new(window_size, 3);
        
        // Measure compression
        group.bench_function(format!("compress_window_{}", window_size), |b| {
            b.iter(|| lzss.compress(black_box(&data)))
        });
        
        // Calculate and print compression ratio
        let compressed = lzss.compress(&data);
        let ratio = (compressed.len() as f64) / (data.len() as f64) * 100.0;
        println!("Window size {}: {:.2}% of original", window_size, ratio);
        
        // Measure decompression
        group.bench_function(format!("decompress_window_{}", window_size), |b| {
            b.iter(|| lzss.decompress(black_box(&compressed)))
        });
    }
    
    group.finish();
}

criterion_group!(benches, basic_benchmark, window_size_benchmark);
criterion_main!(benches);