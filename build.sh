#!/bin/bash

# set -e

# Pull the latest changes and update submodules
git pull
git submodule update --init --recursive

# Detect the operating system
os=$(uname)

# Define the directory path
dir="./bin/$os/"

# Create the directory if it does not exist
mkdir -p "$dir"

cd cpp

case "$os" in
  "Linux")
    echo "Running on Linux"
    cpu_num=$(nproc)
    make config=release clean
    make config=release -j"$cpu_num"
    ;;
  "Darwin")
    echo "Running on macOS"
    cpu_num=$(sysctl -n hw.ncpu)
    if ! command -v premake5 &> /dev/null; then
      brew install premake
    fi
    premake5 gmake --cc=clang
    make config=release -j"$cpu_num"
    ;;
  *)
    echo "Unknown operating system"
    exit 1
    ;;
esac

cd ..

# Copy the built shared objects to the target directory
cp -f ./cpp/build/bin/Release/*.so "$dir"
cp -f ./cpp/build/bin/Release/*.dylib "$dir"

# Check if cargo command exists
if command -v cargo &> /dev/null; then
  cd rust
  cargo build --release
  cd ..

  for file in ./rust/target/release/lib*.so; do
    cp -f "$file" "$dir$(basename "$file" | sed 's/^lib//')"
  done

  for file in ./rust/target/release/lib*.dylib; do
    cp -f "$file" "$dir$(basename "$file" | sed 's/^lib//')"
  done
else
  echo "Rust not installed, skipping Rust build."
fi

read -p "Press [Enter] key to continue..."