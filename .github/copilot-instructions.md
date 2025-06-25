<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Cap Electron Capture Library - Copilot Instructions

This is a Rust library designed for cross-platform screen capture and audio processing in Electron applications. The library extracts core functionality from Cap's screen recording pipeline and provides Node.js bindings for use in transcription and real-time audio processing applications.

## Project Structure

- **src/lib.rs**: Main library entry point with Node.js FFI bindings
- **src/audio.rs**: Audio capture and processing implementation using cpal
- **src/screen.rs**: Screen capture implementation with platform-specific code
- **src/config.rs**: Configuration structures and defaults
- **src/error.rs**: Error types and handling
- **src/platform.rs**: Platform detection and capabilities
- **index.js**: JavaScript interface for Node.js integration
- **index.d.ts**: TypeScript definitions
- **test.js**: Example usage and testing

## Key Features

1. **Cross-platform Audio Capture**: System audio + microphone using cpal
2. **Screen Capture**: Platform-specific implementations (ScreenCaptureKit/Windows Capture API/PipeWire)
3. **Real-time Processing**: Segmented audio output for transcription services
4. **Node.js Integration**: NAPI bindings for Electron applications
5. **TypeScript Support**: Full type definitions included

## Platform Support

- **macOS**: ScreenCaptureKit for screen/audio capture
- **Windows**: Windows Capture API + WASAPI loopback
- **Linux**: PipeWire/PulseAudio integration

## Usage Context

This library is designed for Electron applications that need:
- Real-time audio transcription
- Screen recording with audio
- System audio capture alongside microphone input
- Cross-platform compatibility

## Development Guidelines

1. **Error Handling**: Use the CaptureError enum for consistent error types
2. **Async Operations**: All capture operations are async using tokio
3. **Memory Safety**: Use Arc<Mutex<>> for shared state between threads
4. **Platform Abstraction**: Implement platform-specific code in separate modules
5. **Node.js Compatibility**: All public APIs should be NAPI-compatible

## Build Process

- Uses napi-rs for Node.js bindings
- Requires platform-specific native dependencies
- Builds as cdylib for Node.js addon usage

## Testing

Run `node test.js` to test basic functionality including:
- Platform capability detection
- Audio device enumeration
- Display detection
- Capture session lifecycle
