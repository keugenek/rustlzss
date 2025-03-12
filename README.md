# RustLZSS

A fast Rust implementation of the LZSS (Lempel-Ziv-Storer-Szymanski) compression algorithm.

## Features

- Fast compression and decompression of byte arrays
- Optimized hash-based matching for better performance
- Configurable window size (up to 65535 bytes) and minimum match length
- Simple API for easy integration
- Robust handling of large files with reliable decompression
- **C/C++ FFI support** for seamless integration with existing C++ codebases

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustzss = "0.1.0"
```

### Basic Example

```rust
use rustzss::LZSS;

fn main() {
    // Create compressor with window size of 16384 bytes and minimum match length of 3
    let lzss = LZSS::new(16384, 3);
    
    // Original data
    let data = b"This is a test string with repetitive content. This is a test string with repetitive content.";
    
    // Compress
    let compressed = lzss.compress(data);
    println!("Original size: {}, Compressed size: {}", data.len(), compressed.len());
    
    // Decompress
    let decompressed = lzss.decompress(&compressed);
    
    // Verify
    assert_eq!(data.to_vec(), decompressed);
}
```

### Command-line Example

The library includes a simple command-line tool for compressing and decompressing files:

```
cargo run --example simple compress input.txt compressed.bin
cargo run --example simple decompress compressed.bin output.txt
```

You can also specify a custom window size (default is 4096):

```
cargo run --example simple compress input.txt compressed.bin 16384
cargo run --example simple decompress compressed.bin output.txt 16384
```

## Algorithm

LZSS compresses data by replacing repeated occurrences of data with references to a single copy of that data existing earlier in the uncompressed data stream. A match is encoded as a pair of numbers (distance, length), where distance indicates how far back the match starts and length indicates the match length.

This implementation uses:

1. Hash-based matching for faster lookups
2. Control bytes to efficiently encode whether the subsequent data is a literal or a match
3. Original data size storage for reliable decompression
4. 2-byte offset encoding supporting larger window sizes (up to 65535 bytes)
5. Optimal handling of matches to maximize compression ratio

## Performance

Benchmarks were run on different types of data with a 4096-byte window and minimum match length of 3:

### Compression Performance
- Random data (10KB): ~1.19ms
- Repeated pattern data (10KB): ~21.3μs (about 56x faster than random data)
- Text-like data (10KB): ~993μs (about 1.2x faster than random data)

### Decompression Performance
- Random data: ~22.4μs
- Repeated pattern data: ~11.6μs (about 2x faster than random data)
- Text-like data: ~24.5μs

### Compression Ratios
- Random data: 112.53% (slightly larger than original)
- Repeated pattern data: 1.61% (excellent compression)
- Text-like data: 106.22% (slightly larger than original)

These results demonstrate that LZSS is particularly effective for compressing data with high repetition, while it may slightly increase the size of random data. The algorithm provides a good balance between compression ratio and performance.

Performance varies with the type of data being compressed:

- For highly repetitive data, compression ratios can be as low as 1-2% of original size
- Random data typically experiences minimal compression (sometimes even slight expansion)
- Text and code files typically compress to about 40-60% of original size, depending on content

### Window Size Impact

The window size has a significant impact on compression performance and ratio, as demonstrated by our benchmarks:

| Window Size | Compression Ratio | Compression Speed |
|-------------|------------------|-------------------|
| 1024        | 4.35%            | 1.30ms            |
| 4096        | 3.60%            | 1.02ms            |
| 8192        | 3.46%            | 0.93ms            |
| 16384       | 2.47%            | 0.71ms            |
| 32768       | 1.62%            | 0.48ms            |

*Benchmark on 500KB of text data with repetitive patterns

Key observations:
- **Larger window sizes** (8192-65535 bytes): Dramatically better compression ratios (up to 62.8% smaller output than with 1KB window) for files with repetitive content spread far apart
- **Medium window sizes** (4096-8192 bytes): Good balance between compression ratio and performance for most general-purpose use cases
- **Smaller window sizes** (256-1024 bytes): Usually suitable for real-time compression of small data chunks

Our implementation's 2-byte offset encoding enables these larger window sizes beyond the original 1-byte limit, greatly improving compression for text documents, code, and other data with patterns that repeat at a distance.

## Tests

The library includes comprehensive tests, including validation with large random buffers:

```
cargo test               # Run basic tests
cargo test -- --ignored  # Include larger test cases (1MB and 10MB)
```

## Benchmarks

Run benchmarks to measure performance:

```
cargo bench
```

## Implementation Details

The implementation focuses on:

1. **Efficiency**: Compression and decompression are optimized for speed
2. **Reliability**: The algorithm handles edge cases properly, including overlapping references and self-referential patterns
3. **Configurability**: Window size (up to 65535 bytes) and minimum match length can be adjusted for different use cases
4. **Safety**: Careful bounds checking prevents out-of-range memory access

## C/C++ Integration

RustLZSS provides seamless FFI (Foreign Function Interface) integration for C and C++ projects, making it easy to include this optimized Rust implementation in existing C++ game engines or applications.

### C API

The C API provides low-level access to the LZSS functionality:

```c
// Include the header
#include "rustzss.h"

// Create an LZSS context
LzssContext* context = lzss_create(16384, 3);

// Compress data
unsigned char* input = /* your data */;
unsigned long input_size = /* input size */;
unsigned long max_output_size = lzss_max_compressed_size(input_size);
unsigned char* output = malloc(max_output_size);
unsigned long compressed_size = 0;

int result = lzss_compress(context, input, input_size, output, max_output_size, &compressed_size);
if (result == 0) {
    // Compression successful, compressed_size contains the actual size
}

// Decompress data
unsigned long original_size = lzss_get_original_size(output, compressed_size);
unsigned char* decompressed = malloc(original_size);
unsigned long decompressed_size = 0;

result = lzss_decompress(context, output, compressed_size, decompressed, original_size, &decompressed_size);
if (result == 0) {
    // Decompression successful
}

// Clean up
lzss_destroy(context);
free(output);
free(decompressed);
```

### C++ API

A more convenient C++ wrapper is also provided:

```cpp
// Include the C++ header
#include "rustzss.hpp"

// Create an LZSS compressor
rustzss::LZSS lzss(16384, 3);

// Compress data using std::vector
std::vector<unsigned char> input = /* your data */;
std::vector<unsigned char> compressed = lzss.compress(input);

// Or compress from raw pointers
unsigned char* raw_input = /* pointer to data */;
size_t input_size = /* size of data */;
std::vector<unsigned char> compressed = lzss.compress(raw_input, input_size);

// Decompress data
std::vector<unsigned char> decompressed = lzss.decompress(compressed);

// Error handling is done via C++ exceptions
try {
    auto compressed = lzss.compress(input);
    auto decompressed = lzss.decompress(compressed);
} catch (const std::exception& e) {
    std::cerr << "Error: " << e.what() << std::endl;
}
```

### CMake Integration

You can integrate the Rust library into your C++ project using CMake:

```cmake
# Find the Rust library
set(RUSTZSS_LIB_DIR "/path/to/rustzss")
include_directories("${RUSTZSS_LIB_DIR}/include")

# Find the library based on platform
if(WIN32)
    set(RUSTZSS_LIB "${RUSTZSS_LIB_DIR}/target/release/rustzss.dll.lib")
elseif(APPLE)
    set(RUSTZSS_LIB "${RUSTZSS_LIB_DIR}/target/release/librustzss.dylib")
else()
    set(RUSTZSS_LIB "${RUSTZSS_LIB_DIR}/target/release/librustzss.so")
endif()

# Link against the library
target_link_libraries(your_target ${RUSTZSS_LIB})
```

See the `examples/CMakeLists.txt` file for a complete example of how to build and link against the Rust library from C++.

## License

MIT
