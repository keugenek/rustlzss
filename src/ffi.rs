use crate::LZSS;
use std::os::raw::{c_char, c_int, c_uchar, c_ulong};
use std::slice;

/// Opaque struct to hold the LZSS compressor instance
pub struct LzssContext {
    lzss: LZSS,
}

/// Create a new LZSS context with specified parameters
///
/// # Parameters
/// * `window_size` - Size of the sliding window (up to 65535)
/// * `min_match_length` - Minimum match length for encoding
///
/// # Returns
/// Pointer to the LZSS context or null on failure
#[no_mangle]
pub extern "C" fn lzss_create(window_size: c_int, min_match_length: c_int) -> *mut LzssContext {
    if window_size <= 0 || min_match_length <= 0 || window_size > 65535 {
        return std::ptr::null_mut();
    }

    let lzss = LZSS::new(window_size as usize, min_match_length as usize);
    let context = Box::new(LzssContext { lzss });
    Box::into_raw(context)
}

/// Free resources used by the LZSS context
///
/// # Parameters
/// * `context` - LZSS context created with lzss_create
#[no_mangle]
pub extern "C" fn lzss_destroy(context: *mut LzssContext) {
    if !context.is_null() {
        unsafe {
            drop(Box::from_raw(context));
        }
    }
}

/// Compress data using LZSS algorithm
///
/// # Parameters
/// * `context` - LZSS context created with lzss_create
/// * `input` - Pointer to input data buffer
/// * `input_size` - Size of the input data in bytes
/// * `output` - Pointer to output buffer (must be pre-allocated)
/// * `output_size` - Size of the output buffer in bytes
/// * `compressed_size` - Pointer to where the actual compressed size will be stored
///
/// # Returns
/// 0 on success, negative error code on failure
#[no_mangle]
pub extern "C" fn lzss_compress(
    context: *const LzssContext,
    input: *const c_uchar,
    input_size: c_ulong,
    output: *mut c_uchar,
    output_size: c_ulong,
    compressed_size: *mut c_ulong,
) -> c_int {
    if context.is_null() || input.is_null() || output.is_null() || compressed_size.is_null() {
        return -1; // Invalid parameters
    }

    unsafe {
        let lzss = &(*context).lzss;
        let input_slice = slice::from_raw_parts(input, input_size as usize);
        
        // Compress the data
        let compressed_data = lzss.compress(input_slice);
        
        // Ensure output buffer is large enough
        if compressed_data.len() > output_size as usize {
            return -2; // Output buffer too small
        }
        
        // Copy compressed data to output buffer
        let output_slice = slice::from_raw_parts_mut(output, output_size as usize);
        output_slice[..compressed_data.len()].copy_from_slice(&compressed_data);
        
        // Store the actual compressed size
        *compressed_size = compressed_data.len() as c_ulong;
        
        0 // Success
    }
}

/// Decompress data using LZSS algorithm
///
/// # Parameters
/// * `context` - LZSS context created with lzss_create
/// * `input` - Pointer to compressed data buffer
/// * `input_size` - Size of the compressed data in bytes
/// * `output` - Pointer to output buffer (must be pre-allocated)
/// * `output_size` - Size of the output buffer in bytes
/// * `decompressed_size` - Pointer to where the actual decompressed size will be stored
///
/// # Returns
/// 0 on success, negative error code on failure
#[no_mangle]
pub extern "C" fn lzss_decompress(
    context: *const LzssContext,
    input: *const c_uchar,
    input_size: c_ulong,
    output: *mut c_uchar,
    output_size: c_ulong,
    decompressed_size: *mut c_ulong,
) -> c_int {
    if context.is_null() || input.is_null() || output.is_null() || decompressed_size.is_null() {
        return -1; // Invalid parameters
    }

    unsafe {
        let lzss = &(*context).lzss;
        let input_slice = slice::from_raw_parts(input, input_size as usize);
        
        // Decompress the data
        let decompressed_data = lzss.decompress(input_slice);
        
        // Ensure output buffer is large enough
        if decompressed_data.len() > output_size as usize {
            return -2; // Output buffer too small
        }
        
        // Copy decompressed data to output buffer
        let output_slice = slice::from_raw_parts_mut(output, output_size as usize);
        output_slice[..decompressed_data.len()].copy_from_slice(&decompressed_data);
        
        // Store the actual decompressed size
        *decompressed_size = decompressed_data.len() as c_ulong;
        
        0 // Success
    }
}

/// Estimate the maximum compressed size for a given input size
///
/// This is useful for pre-allocating output buffers.
/// In the worst case, LZSS compression can result in slight expansion,
/// especially for incompressible data (e.g., already compressed or random data).
///
/// # Parameters
/// * `input_size` - Size of the input data in bytes
///
/// # Returns
/// Estimated maximum compressed size in bytes
#[no_mangle]
pub extern "C" fn lzss_max_compressed_size(input_size: c_ulong) -> c_ulong {
    // 4 bytes for original size + control bytes (1 per 8 bytes worst case) + worst case of all literals
    let control_bytes = (input_size + 7) / 8;
    (4 + control_bytes + input_size) as c_ulong
}

/// Get the original size of compressed data without decompressing it
/// 
/// This function extracts the original size from the header of the compressed data
///
/// # Parameters
/// * `compressed_data` - Pointer to compressed data buffer
/// * `compressed_size` - Size of the compressed data in bytes
///
/// # Returns
/// Original uncompressed size in bytes, or 0 if invalid
#[no_mangle]
pub extern "C" fn lzss_get_original_size(
    compressed_data: *const c_uchar,
    compressed_size: c_ulong,
) -> c_ulong {
    if compressed_data.is_null() || compressed_size < 4 {
        return 0; // Invalid parameters
    }

    unsafe {
        let bytes = slice::from_raw_parts(compressed_data, 4);
        let mut original_size = 0usize;
        
        for i in 0..4 {
            original_size |= (bytes[i] as usize) << (i * 8);
        }
        
        original_size as c_ulong
    }
}