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

criterion_group!(benches, basic_benchmark);
criterion_main!(benches);