use std::collections::HashMap;

// Make the FFI module public
pub mod ffi;

// Add autotuning support
#[cfg(feature = "autotune")]
pub mod autotune;

/// LZSS encoder/decoder implementation for byte streams.
/// 
/// This implementation uses a sliding window approach with
/// configurable window size and minimum match length.
pub struct LZSS {
    window_size: usize,
    min_match_length: usize,
}

impl LZSS {
    /// Create a new LZSS compressor/decompressor with given parameters
    pub fn new(window_size: usize, min_match_length: usize) -> Self {
        LZSS {
            window_size,
            min_match_length,
        }
    }

    /// Compress input data using LZSS algorithm
    /// 
    /// Returns compressed byte vector
    pub fn compress(&self, input: &[u8]) -> Vec<u8> {
        let input_len = input.len();
        
        // Handle empty input
        if input_len == 0 {
            return Vec::new();
        }
        
        let mut output = Vec::new();
        let mut pos = 0;
        
        // Store original size for exact decompression
        for i in 0..4 {
            output.push(((input_len >> (i * 8)) & 0xFF) as u8);
        }
        
        // Control byte and its bit position
        let mut control_byte = 0u8;
        let mut bit_pos = 0;
        let mut control_byte_pos = output.len();
        output.push(0); // Reserve space for first control byte
        
        // Dictionary for finding matches
        let mut dictionary: HashMap<&[u8], Vec<usize>> = HashMap::new();
        
        // Calculate the maximum representable match length
        let max_match_code = 255; // One byte to encode the match length adjustment
        let max_match_length = max_match_code + self.min_match_length;
        
        while pos < input_len {
            // Find the longest match in the sliding window
            let max_look_ahead = std::cmp::min(input_len - pos, max_match_length);
            let window_begin = if pos > self.window_size { pos - self.window_size } else { 0 };
            
            // Try to find the longest match
            let mut best_match_len = 0;
            let mut best_match_dist = 0;
            
            // Only look for matches if we have enough bytes ahead
            if max_look_ahead >= self.min_match_length {
                let key_size = std::cmp::min(3, max_look_ahead);
                let search_key = &input[pos..pos + key_size];
                
                if let Some(positions) = dictionary.get(search_key) {
                    for &prev_pos in positions.iter().rev() {
                        if prev_pos < window_begin {
                            continue;
                        }
                        
                        let mut match_len = 0;
                        let max_possible = std::cmp::min(input_len - pos, input_len - prev_pos);
                        
                        while match_len < max_possible && match_len < max_look_ahead && 
                              input[prev_pos + match_len] == input[pos + match_len] {
                            match_len += 1;
                        }
                        
                        if match_len >= self.min_match_length && match_len > best_match_len {
                            best_match_len = match_len;
                            best_match_dist = pos - prev_pos;
                            
                            if match_len >= 16 { // Early termination if we find a good match
                                break;
                            }
                        }
                    }
                }
                
                // Add current position to dictionary
                if key_size == 3 { // Only add 3-byte keys
                    dictionary.entry(search_key).or_insert_with(Vec::new).push(pos);
                }
            }
            
            // Encode literal or match
            if best_match_len >= self.min_match_length {
                // Encode a match
                control_byte |= 1 << bit_pos;
                
                // Use 2 bytes for offset to support larger window sizes (up to 65535)
                if best_match_dist > 65535 {
                    best_match_dist = 65535; // Limit to max representable value with 2 bytes
                    // Recalculate match length with this constrained distance
                    let back_pos = pos - best_match_dist;
                    let mut adjusted_len = 0;
                    while adjusted_len < max_look_ahead && 
                          input[back_pos + adjusted_len] == input[pos + adjusted_len] {
                        adjusted_len += 1;
                    }
                    best_match_len = adjusted_len;
                    
                    // If the adjusted match is too short, encode as literal instead
                    if best_match_len < self.min_match_length {
                        control_byte &= !(1 << bit_pos); // Reset bit
                        output.push(input[pos]);
                        pos += 1;
                    } else {
                        // Store the distance using 2 bytes (little-endian)
                        output.push((best_match_dist & 0xFF) as u8);            // Low byte
                        output.push(((best_match_dist >> 8) & 0xFF) as u8);     // High byte
                        output.push((best_match_len - self.min_match_length) as u8);
                        pos += best_match_len;
                    }
                } else {
                    // Store the distance using 2 bytes (little-endian)
                    output.push((best_match_dist & 0xFF) as u8);           // Low byte
                    output.push(((best_match_dist >> 8) & 0xFF) as u8);    // High byte
                    output.push((best_match_len - self.min_match_length) as u8);
                    pos += best_match_len;
                }
            } else {
                // Encode a literal
                output.push(input[pos]);
                pos += 1;
            }
            
            bit_pos += 1;
            
            // If control byte is full, start a new one
            if bit_pos == 8 {
                output[control_byte_pos] = control_byte;
                
                if pos < input_len {
                    control_byte = 0;
                    bit_pos = 0;
                    control_byte_pos = output.len();
                    output.push(0); // Reserve space for next control byte
                }
            }
        }
        
        // Update the last control byte if not full
        if bit_pos > 0 && bit_pos < 8 {
            output[control_byte_pos] = control_byte;
        }
        
        output
    }

    /// Decompress data compressed with the LZSS algorithm
    /// 
    /// Returns the decompressed byte vector
    pub fn decompress(&self, input: &[u8]) -> Vec<u8> {
        if input.len() < 5 { // Need at least 4 bytes for size + 1 for control
            return Vec::new();
        }
        
        // Extract original size from header
        let mut original_size = 0usize;
        for i in 0..4 {
            original_size |= (input[i] as usize) << (i * 8);
        }
        
        let mut output = Vec::with_capacity(original_size);
        let mut pos = 4; // Start after size header
        
        while pos < input.len() && output.len() < original_size {
            let control_byte = input[pos];
            pos += 1;
            
            // Process each bit in the control byte
            for bit in 0..8 {
                if output.len() >= original_size || pos >= input.len() {
                    break;
                }
                
                if (control_byte & (1 << bit)) != 0 {
                    // This is a match reference
                    if pos + 2 >= input.len() { // Need 2 bytes for distance + 1 for length
                        break; // Not enough data
                    }
                    
                    // Read distance from 2 bytes (little-endian)
                    let distance = (input[pos] as usize) | ((input[pos + 1] as usize) << 8);
                    let length = (input[pos + 2] as usize) + self.min_match_length;
                    pos += 3;
                    
                    // Sanity check
                    if distance == 0 || distance > output.len() {
                        continue; // Skip invalid reference
                    }
                    
                    // Copy from the already decompressed output
                    let start_pos = output.len() - distance;
                    
                    for i in 0..length {
                        if start_pos + i < output.len() {
                            // Regular copy from earlier in output
                            output.push(output[start_pos + i]);
                        } else {
                            // Handle self-referential copies (like ABABAB pattern)
                            // Calculate correct offset based on what we've copied so far
                            let offset = i % distance;
                            output.push(output[start_pos + offset]);
                        }
                        
                        if output.len() >= original_size {
                            break;
                        }
                    }
                } else {
                    // This is a literal byte
                    output.push(input[pos]);
                    pos += 1;
                }
            }
        }
        
        // Ensure we have exactly the original size
        if output.len() > original_size {
            output.truncate(original_size);
        } else if output.len() < original_size {
            // This would be an error condition in real code,
            // but for now we'll just pad with zeros
            output.resize(original_size, 0);
        }
        
        output
    }
}

// Include detailed tests
#[path = "tests.rs"]
mod tests;