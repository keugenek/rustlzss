#ifndef RUSTZSS_HPP
#define RUSTZSS_HPP

#include "rustzss.h"
#include <vector>
#include <memory>
#include <stdexcept>
#include <string>

namespace rustzss {

/**
 * C++ wrapper for the LZSS compression algorithm
 */
class LZSS {
public:
    /**
     * Constructor
     * 
     * @param windowSize Size of the sliding window (up to 65535)
     * @param minMatchLength Minimum match length for encoding (default: 3)
     * @throws std::runtime_error if context creation fails
     */
    LZSS(int windowSize, int minMatchLength = 3) {
        context_ = lzss_create(windowSize, minMatchLength);
        if (!context_) {
            throw std::runtime_error("Failed to create LZSS context");
        }
    }

    /**
     * Destructor - automatically frees the context
     */
    ~LZSS() {
        if (context_) {
            lzss_destroy(context_);
        }
    }

    // Delete copy constructor and assignment operator to prevent accidental copies
    LZSS(const LZSS&) = delete;
    LZSS& operator=(const LZSS&) = delete;

    // Allow move constructor and assignment
    LZSS(LZSS&& other) noexcept : context_(other.context_) {
        other.context_ = nullptr;
    }

    LZSS& operator=(LZSS&& other) noexcept {
        if (this != &other) {
            if (context_) {
                lzss_destroy(context_);
            }
            context_ = other.context_;
            other.context_ = nullptr;
        }
        return *this;
    }

    /**
     * Compress data
     * 
     * @param data Data to compress
     * @return Compressed data
     * @throws std::runtime_error on compression error
     */
    std::vector<unsigned char> compress(const std::vector<unsigned char>& data) const {
        if (data.empty()) {
            return {};
        }

        // Allocate output buffer with worst-case size
        unsigned long maxSize = lzss_max_compressed_size(data.size());
        std::vector<unsigned char> result(maxSize);
        
        unsigned long compressedSize = 0;
        int status = lzss_compress(
            context_,
            data.data(),
            data.size(),
            result.data(),
            result.size(),
            &compressedSize
        );

        if (status != 0) {
            throw std::runtime_error("Compression failed with error code: " + std::to_string(status));
        }

        // Resize to actual compressed size
        result.resize(compressedSize);
        return result;
    }

    /**
     * Compress data
     * 
     * @param data Pointer to data to compress
     * @param size Size of data in bytes
     * @return Compressed data
     * @throws std::runtime_error on compression error
     */
    std::vector<unsigned char> compress(const unsigned char* data, size_t size) const {
        if (!data || size == 0) {
            return {};
        }

        // Allocate output buffer with worst-case size
        unsigned long maxSize = lzss_max_compressed_size(size);
        std::vector<unsigned char> result(maxSize);
        
        unsigned long compressedSize = 0;
        int status = lzss_compress(
            context_,
            data,
            size,
            result.data(),
            result.size(),
            &compressedSize
        );

        if (status != 0) {
            throw std::runtime_error("Compression failed with error code: " + std::to_string(status));
        }

        // Resize to actual compressed size
        result.resize(compressedSize);
        return result;
    }

    /**
     * Decompress data
     * 
     * @param compressedData Compressed data to decompress
     * @return Decompressed data
     * @throws std::runtime_error on decompression error
     */
    std::vector<unsigned char> decompress(const std::vector<unsigned char>& compressedData) const {
        if (compressedData.empty()) {
            return {};
        }

        // Get original size from header
        unsigned long originalSize = lzss_get_original_size(
            compressedData.data(),
            compressedData.size()
        );

        if (originalSize == 0) {
            throw std::runtime_error("Invalid compressed data or unable to determine original size");
        }

        // Allocate output buffer with original size
        std::vector<unsigned char> result(originalSize);
        
        unsigned long decompressedSize = 0;
        int status = lzss_decompress(
            context_,
            compressedData.data(),
            compressedData.size(),
            result.data(),
            result.size(),
            &decompressedSize
        );

        if (status != 0) {
            throw std::runtime_error("Decompression failed with error code: " + std::to_string(status));
        }

        // Verify size
        if (decompressedSize != originalSize) {
            throw std::runtime_error("Decompressed size does not match expected size");
        }

        return result;
    }

    /**
     * Decompress data
     * 
     * @param compressedData Pointer to compressed data
     * @param compressedSize Size of compressed data in bytes
     * @return Decompressed data
     * @throws std::runtime_error on decompression error
     */
    std::vector<unsigned char> decompress(const unsigned char* compressedData, size_t compressedSize) const {
        if (!compressedData || compressedSize == 0) {
            return {};
        }

        // Get original size from header
        unsigned long originalSize = lzss_get_original_size(
            compressedData,
            compressedSize
        );

        if (originalSize == 0) {
            throw std::runtime_error("Invalid compressed data or unable to determine original size");
        }

        // Allocate output buffer with original size
        std::vector<unsigned char> result(originalSize);
        
        unsigned long decompressedSize = 0;
        int status = lzss_decompress(
            context_,
            compressedData,
            compressedSize,
            result.data(),
            result.size(),
            &decompressedSize
        );

        if (status != 0) {
            throw std::runtime_error("Decompression failed with error code: " + std::to_string(status));
        }

        // Verify size
        if (decompressedSize != originalSize) {
            throw std::runtime_error("Decompressed size does not match expected size");
        }

        return result;
    }

private:
    LzssContext* context_;
};

/**
 * Helper function to get the maximum compressed size for a given input size
 */
inline size_t maxCompressedSize(size_t inputSize) {
    return lzss_max_compressed_size(inputSize);
}

} // namespace rustzss

#endif // RUSTZSS_HPP