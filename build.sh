#!/bin/bash

# Build script for Cap Electron Capture Library

echo "🔧 Building Cap Electron Capture Library..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
fi

echo "📦 Installing Node.js dependencies..."
npm install

echo "🦀 Building Rust native module..."
cargo build --release

echo "🔗 Building Node.js bindings..."
npm run build

echo "✅ Build completed successfully!"
echo ""
echo "🧪 Run tests with: npm test"
echo "📚 See README.md for usage examples"
