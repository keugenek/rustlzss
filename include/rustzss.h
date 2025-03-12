#ifndef RUSTZSS_H
#define RUSTZSS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

/**
 * Opaque struct representing an LZSS context
 */
typedef struct LzssContext LzssContext;

/**
 * Create a new LZSS context with specified parameters
 *
 * @param window_size Size of the sliding window (up to 65535)
 * @param min_match_length Minimum match length for encoding
 * @return Pointer to the LZSS context or NULL on failure
 */
LzssContext* lzss_create(int window_size, int min_match_length);

/**
 * Free resources used by the LZSS context
 *
 * @param context LZSS context created with lzss_create
 */
void lzss_destroy(LzssContext* context);

/**
 * Compress data using LZSS algorithm
 *
 * @param context LZSS context created with lzss_create
 * @param input Pointer to input data buffer
 * @param input_size Size of the input data in bytes
 * @param output Pointer to output buffer (must be pre-allocated)
 * @param output_size Size of the output buffer in bytes
 * @param compressed_size Pointer to where the actual compressed size will be stored
 * @return 0 on success, negative error code on failure
 */
int lzss_compress(
    const LzssContext* context,
    const unsigned char* input,
    unsigned long input_size,
    unsigned char* output,
    unsigned long output_size,
    unsigned long* compressed_size
);

/**
 * Decompress data using LZSS algorithm
 *
 * @param context LZSS context created with lzss_create
 * @param input Pointer to compressed data buffer
 * @param input_size Size of the compressed data in bytes
 * @param output Pointer to output buffer (must be pre-allocated)
 * @param output_size Size of the output buffer in bytes
 * @param decompressed_size Pointer to where the actual decompressed size will be stored
 * @return 0 on success, negative error code on failure
 */
int lzss_decompress(
    const LzssContext* context,
    const unsigned char* input,
    unsigned long input_size,
    unsigned char* output,
    unsigned long output_size,
    unsigned long* decompressed_size
);

/**
 * Estimate the maximum compressed size for a given input size
 *
 * This is useful for pre-allocating output buffers.
 * In the worst case, LZSS compression can result in slight expansion,
 * especially for incompressible data (e.g., already compressed or random data).
 *
 * @param input_size Size of the input data in bytes
 * @return Estimated maximum compressed size in bytes
 */
unsigned long lzss_max_compressed_size(unsigned long input_size);

/**
 * Get the original size of compressed data without decompressing it
 *
 * This function extracts the original size from the header of the compressed data
 *
 * @param compressed_data Pointer to compressed data buffer
 * @param compressed_size Size of the compressed data in bytes
 * @return Original uncompressed size in bytes, or 0 if invalid
 */
unsigned long lzss_get_original_size(
    const unsigned char* compressed_data,
    unsigned long compressed_size
);

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // RUSTZSS_H