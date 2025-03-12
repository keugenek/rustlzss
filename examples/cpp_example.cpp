#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <fstream>
#include <cstring>

// Include the C++ wrapper
#include "../include/rustzss.hpp"

// Helper functions
std::vector<unsigned char> readFile(const std::string& filename) {
    std::ifstream file(filename, std::ios::binary);
    if (!file) {
        throw std::runtime_error("Failed to open file: " + filename);
    }
    
    file.seekg(0, std::ios::end);
    size_t size = file.tellg();
    file.seekg(0, std::ios::beg);
    
    std::vector<unsigned char> buffer(size);
    file.read(reinterpret_cast<char*>(buffer.data()), size);
    
    return buffer;
}

void writeFile(const std::string& filename, const std::vector<unsigned char>& data) {
    std::ofstream file(filename, std::ios::binary);
    if (!file) {
        throw std::runtime_error("Failed to open file for writing: " + filename);
    }
    
    file.write(reinterpret_cast<const char*>(data.data()), data.size());
}

void compressFile(const std::string& inputFile, const std::string& outputFile, int windowSize) {
    try {
        // Create LZSS compressor
        rustzss::LZSS lzss(windowSize);
        
        // Read input file
        std::cout << "Reading input file: " << inputFile << std::endl;
        auto data = readFile(inputFile);
        std::cout << "Input size: " << data.size() << " bytes" << std::endl;
        
        // Compress data
        std::cout << "Compressing..." << std::endl;
        auto start = std::chrono::high_resolution_clock::now();
        auto compressed = lzss.compress(data);
        auto end = std::chrono::high_resolution_clock::now();
        
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();
        double ratio = static_cast<double>(compressed.size()) / data.size() * 100.0;
        
        std::cout << "Compressed size: " << compressed.size() << " bytes" << std::endl;
        std::cout << "Compression ratio: " << ratio << "%" << std::endl;
        std::cout << "Compression time: " << duration << " ms" << std::endl;
        
        // Write compressed data to output file
        std::cout << "Writing compressed data to: " << outputFile << std::endl;
        writeFile(outputFile, compressed);
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
}

void decompressFile(const std::string& inputFile, const std::string& outputFile, int windowSize) {
    try {
        // Create LZSS decompressor
        rustzss::LZSS lzss(windowSize);
        
        // Read compressed file
        std::cout << "Reading compressed file: " << inputFile << std::endl;
        auto compressed = readFile(inputFile);
        std::cout << "Compressed size: " << compressed.size() << " bytes" << std::endl;
        
        // Get original size
        unsigned long originalSize = lzss_get_original_size(compressed.data(), compressed.size());
        std::cout << "Original size: " << originalSize << " bytes" << std::endl;
        
        // Decompress data
        std::cout << "Decompressing..." << std::endl;
        auto start = std::chrono::high_resolution_clock::now();
        auto decompressed = lzss.decompress(compressed);
        auto end = std::chrono::high_resolution_clock::now();
        
        auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();
        
        std::cout << "Decompressed size: " << decompressed.size() << " bytes" << std::endl;
        std::cout << "Decompression time: " << duration << " ms" << std::endl;
        
        // Write decompressed data to output file
        std::cout << "Writing decompressed data to: " << outputFile << std::endl;
        writeFile(outputFile, decompressed);
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
}

// Main function - simple command line interface
int main(int argc, char* argv[]) {
    std::cout << "RustLZSS C++ Example" << std::endl;
    std::cout << "====================" << std::endl << std::endl;
    
    if (argc < 4) {
        std::cout << "Usage: " << std::endl;
        std::cout << "  " << argv[0] << " compress <input_file> <output_file> [window_size]" << std::endl;
        std::cout << "  " << argv[0] << " decompress <input_file> <output_file> [window_size]" << std::endl;
        std::cout << std::endl;
        std::cout << "Default window_size is 4096 bytes" << std::endl;
        return 1;
    }
    
    std::string command = argv[1];
    std::string inputFile = argv[2];
    std::string outputFile = argv[3];
    int windowSize = 4096; // Default window size
    
    if (argc >= 5) {
        windowSize = std::atoi(argv[4]);
        if (windowSize <= 0 || windowSize > 65535) {
            std::cerr << "Window size must be between 1 and 65535" << std::endl;
            return 1;
        }
    }
    
    try {
        if (command == "compress") {
            compressFile(inputFile, outputFile, windowSize);
        } else if (command == "decompress") {
            decompressFile(inputFile, outputFile, windowSize);
        } else {
            std::cerr << "Unknown command: " << command << std::endl;
            return 1;
        }
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
    
    std::cout << "Operation completed successfully!" << std::endl;
    return 0;
}