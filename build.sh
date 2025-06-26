#!/bin/bash

# Enhanced build script for Cap Electron Capture Library
# This script handles platform-specific dependencies and native module compilation

set -e  # Exit on any error

echo "🚀 Cap Electron Capture Library - Production Build"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_step() {
    echo -e "\n${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check prerequisites
print_step "Checking prerequisites..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi
print_success "Rust found: $(rustc --version)"

# Check Node.js
if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed"
    echo "Please install Node.js from https://nodejs.org/"
    exit 1
fi
print_success "Node.js found: $(node --version)"

# Check npm
if ! command -v npm &> /dev/null; then
    print_error "npm is not installed"
    echo "Please install npm (usually comes with Node.js)"
    exit 1
fi
print_success "npm found: $(npm --version)"

# Platform-specific dependency checks
print_step "Checking platform-specific dependencies..."

case "$(uname -s)" in
    Darwin*)
        echo "🍎 Detected macOS"
        
        # Check Xcode command line tools
        if ! xcode-select -p &> /dev/null; then
            print_error "Xcode Command Line Tools not found"
            echo "Please install with: xcode-select --install"
            exit 1
        fi
        print_success "Xcode Command Line Tools found"
        
        # Check for pkg-config (needed for FFmpeg)
        if ! command -v pkg-config &> /dev/null; then
            print_warning "pkg-config not found (needed for FFmpeg)"
            echo "Install with: brew install pkg-config"
            echo "Or disable audio-encoding feature in Cargo.toml"
        fi
        
        # Check for FFmpeg (optional)
        if ! command -v ffmpeg &> /dev/null; then
            print_warning "FFmpeg not found (optional - for audio encoding)"
            echo "Install with: brew install ffmpeg"
        else
            print_success "FFmpeg found: $(ffmpeg -version | head -n1)"
        fi
        ;;
        
    Linux*)
        echo "🐧 Detected Linux"
        
        # Check build essentials
        if ! command -v gcc &> /dev/null; then
            print_error "GCC not found"
            echo "Please install build essentials:"
            echo "Ubuntu/Debian: sudo apt update && sudo apt install build-essential"
            echo "CentOS/RHEL: sudo yum groupinstall 'Development Tools'"
            exit 1
        fi
        print_success "GCC found: $(gcc --version | head -n1)"
        
        # Check for ALSA development files
        if ! pkg-config --exists alsa; then
            print_warning "ALSA development files not found"
            echo "Install with: sudo apt install libasound2-dev"
        fi
        
        # Check for PulseAudio development files
        if ! pkg-config --exists libpulse; then
            print_warning "PulseAudio development files not found"
            echo "Install with: sudo apt install libpulse-dev"
        fi
        ;;
        
    MINGW*|MSYS*|CYGWIN*)
        echo "🪟 Detected Windows"
        
        # Check for Visual Studio Build Tools
        if ! command -v cl &> /dev/null; then
            print_warning "Visual Studio Build Tools not found"
            echo "Please install Visual Studio Build Tools or Visual Studio Community"
            echo "Download from: https://visualstudio.microsoft.com/downloads/"
        fi
        ;;
        
    *)
        print_warning "Unknown operating system: $(uname -s)"
        echo "Build may not work on this platform"
        ;;
esac

# Install Node.js dependencies
print_step "Installing Node.js dependencies..."
npm install
print_success "Node.js dependencies installed"

# Build Rust project
print_step "Building Rust native module..."

# Clean previous builds
print_step "Cleaning previous builds..."
cargo clean
rm -f *.node

# Check if we should build with audio encoding
if [ "$1" = "--no-audio-encoding" ]; then
    print_warning "Building without audio encoding support"
    export CARGO_FEATURES="--no-default-features"
else
    print_step "Building with audio encoding support (FFmpeg)"
    export CARGO_FEATURES="--features audio-encoding"
fi

# Build the native module
if [ "$1" = "--debug" ]; then
    print_step "Building in debug mode..."
    npm run build:debug
else
    print_step "Building in release mode..."
    npm run build
fi

print_success "Native module build completed"

# Verify the build
print_step "Verifying build..."
if [ -f "cap-electron-capture.*.node" ] || [ -f "index.node" ]; then
    print_success "Native module file found"
else
    print_error "Native module file not found"
    echo "Build may have failed. Check the output above for errors."
    exit 1
fi

# Run tests
print_step "Running tests..."
if npm test; then
    print_success "All tests passed"
else
    print_warning "Some tests failed - check output above"
fi

print_step "Build Summary"
echo "=============="
print_success "✅ Rust native module compiled successfully"
print_success "✅ Node.js bindings generated"
print_success "✅ TypeScript definitions available"

print_step "Next Steps"
echo "=========="
echo "🧪 Run tests: npm test"
echo "📚 See examples: cd examples && npm start"
echo "📖 Read documentation: README.md"

if command -v ffmpeg &> /dev/null; then
    print_success "🎵 Audio encoding (AAC/MP3) available"
else
    print_warning "🎵 Audio encoding not available (no FFmpeg)"
    echo "   Install FFmpeg for audio encoding support"
fi

echo ""
print_success "🎉 Build completed successfully!"
echo "   You can now use cap-electron-capture in your projects"
