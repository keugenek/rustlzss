use rustzss::LZSS;
use std::io::{self, Read, Write};
use std::fs::File;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 4 {
        eprintln!("Usage: {} <compress|decompress> <input_file> <output_file> [window_size]", args[0]);
        eprintln!("\nWindow size is optional (default: 4096). Recommended values: 4096, 8192, 16384, 32768");
        std::process::exit(1);
    }
    
    let mode = &args[1];
    let input_filename = &args[2];
    let output_filename = &args[3];
    
    // Parse window size if provided
    let window_size = if args.len() > 4 {
        match args[4].parse::<usize>() {
            Ok(size) => size,
            Err(_) => {
                eprintln!("Invalid window size: {}. Using default (4096)", args[4]);
                4096
            }
        }
    } else {
        4096
    };
    
    let min_match_length = 3;
    
    // Read input file
    let mut input_file = File::open(input_filename)?;
    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;
    
    // Create LZSS instance with configured window size
    let lzss = LZSS::new(window_size, min_match_length);
    
    // Process data
    let output_data = match mode.as_str() {
        "compress" => {
            println!("Compressing {} to {} with window size {}", input_filename, output_filename, window_size);
            let start = std::time::Instant::now();
            let compressed = lzss.compress(&input_data);
            let duration = start.elapsed();
            let ratio = (compressed.len() as f64) / (input_data.len() as f64) * 100.0;
            println!("Compressed {} bytes to {} bytes in {:.2?} ({:.2}% of original size)",
                input_data.len(), compressed.len(), duration, ratio);
            compressed
        },
        "decompress" => {
            println!("Decompressing {} to {} with window size {}", input_filename, output_filename, window_size);
            let start = std::time::Instant::now();
            let decompressed = lzss.decompress(&input_data);
            let duration = start.elapsed();
            println!("Decompressed {} bytes to {} bytes in {:.2?}",
                input_data.len(), decompressed.len(), duration);
            decompressed
        },
        _ => {
            eprintln!("Invalid mode: {}. Use 'compress' or 'decompress'", mode);
            std::process::exit(1);
        }
    };
    
    // Write output file
    let mut output_file = File::create(output_filename)?;
    output_file.write_all(&output_data)?;
    
    Ok(())
}