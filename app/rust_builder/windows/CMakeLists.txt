# Builds app/rust with Cargokit and bundles the resulting shared library.
cmake_minimum_required(VERSION 3.14)

set(PROJECT_NAME "jixel_native")
project(${PROJECT_NAME} LANGUAGES CXX)

include("../cargokit/cmake/cargokit.cmake")
# Flutter reaches this folder through a symlink (a junction on Windows) in
# windows/flutter/ephemeral/.plugin_symlinks, so "../../rust" would resolve
# from there. Point at the crate from the app's own CMake root instead.
file(RELATIVE_PATH jixel_rust_dir "${CMAKE_CURRENT_SOURCE_DIR}" "${CMAKE_SOURCE_DIR}/../rust")
apply_cargokit(${PROJECT_NAME} ${jixel_rust_dir} jixel_bridge "")

set(
  jixel_native_bundled_libraries
  "${${PROJECT_NAME}_cargokit_lib}"
  PARENT_SCOPE
)
