cmake_minimum_required(VERSION 3.10)
project(RustLZSSExample CXX)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Path to the Rust library
set(RUSTZSS_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/..")

# Add include directories
include_directories("${RUSTZSS_LIB_DIR}/include")

# Create the C++ example executable
add_executable(cpp_example cpp_example.cpp)

# Find the Rust library
if(WIN32)
    set(RUSTZSS_LIB "${RUSTZSS_LIB_DIR}/target/release/rustzss.dll.lib")
elseif(APPLE)
    set(RUSTZSS_LIB "${RUSTZSS_LIB_DIR}/target/release/librustzss.dylib")
else()
    set(RUSTZSS_LIB "${RUSTZSS_LIB_DIR}/target/release/librustzss.so")
endif()

# Custom command to build the Rust library
add_custom_command(
    OUTPUT ${RUSTZSS_LIB}
    COMMAND cargo build --release
    WORKING_DIRECTORY ${RUSTZSS_LIB_DIR}
    COMMENT "Building Rust LZSS library"
    VERBATIM
)

# Create a custom target for the Rust library
add_custom_target(
    rustzss_lib
    DEPENDS ${RUSTZSS_LIB}
)

# Make cpp_example depend on the Rust library
add_dependencies(cpp_example rustzss_lib)

# Link against the Rust library
target_link_libraries(cpp_example ${RUSTZSS_LIB})