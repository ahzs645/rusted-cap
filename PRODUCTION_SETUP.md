# Production Setup Guide - Cap Electron Capture

This guide will help you get the Cap Electron Capture library working in a production environment with native system audio capture.

## Quick Start

### 1. Basic Installation

```bash
# Clone and build
git clone https://github.com/cap-so/cap-electron-capture
cd cap-electron-capture

# Run enhanced build script
./build-enhanced.sh

# Or manual build
npm install && npm run build
```

### 2. Test Installation

```bash
npm test
```

## Platform-Specific Setup for System Audio

### macOS - Native System Audio

**Option 1: BlackHole Virtual Driver (Recommended)**
```bash
# Install BlackHole
brew install blackhole-2ch

# Or download from: https://existential.audio/blackhole/
```

**Option 2: ScreenCaptureKit (Future)**
- Requires macOS 12.3+
- Native integration coming in future updates
- No additional setup needed

**Configuration:**
1. Open Audio MIDI Setup (Applications > Utilities)
2. Create Multi-Output Device
3. Add BlackHole and your speakers
4. Set as system output
5. Configure app to capture from BlackHole

### Windows - WASAPI Loopback

**Enable Stereo Mix:**
1. Right-click sound icon in system tray
2. Open "Sound settings" → "Sound Control Panel"
3. Recording tab → Right-click → "Show Disabled Devices"
4. Enable "Stereo Mix" or "What U Hear"

**For Modern Windows (Windows 10/11):**
- The library will automatically use WASAPI loopback
- No additional configuration needed

### Linux - PipeWire/PulseAudio

**PulseAudio Setup:**
```bash
# Load monitor modules
pactl load-module module-loopback source=<sink>.monitor

# Or enable in /etc/pulse/default.pa
echo "load-module module-loopback source=auto_null.monitor" >> ~/.config/pulse/default.pa
```

**PipeWire Setup:**
```bash
# PipeWire automatically provides monitor sources
# Check available sources:
pw-cat --record --list-targets
```

## Advanced Configuration

### FFmpeg Audio Encoding

**Install FFmpeg:**

macOS:
```bash
brew install ffmpeg pkg-config
```

Ubuntu/Debian:
```bash
sudo apt update
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev pkg-config
```

Windows:
```bash
# Using vcpkg
vcpkg install ffmpeg
```

**Build with FFmpeg:**
```bash
# Enable audio encoding feature
npm run build

# Or build without FFmpeg
./build-enhanced.sh --no-audio-encoding
```

### Electron Integration

**Main Process (main.js):**
```javascript
const { app, BrowserWindow, ipcMain } = require('electron');
const capture = require('cap-electron-capture');

ipcMain.handle('init-capture', async () => {
  return capture.init();
});

ipcMain.handle('start-capture', async (event, config) => {
  const session = capture.createCaptureSession(config);
  return session.start();
});
```

**Preload Script (preload.js):**
```javascript
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('capture', {
  init: () => ipcRenderer.invoke('init-capture'),
  startCapture: (config) => ipcRenderer.invoke('start-capture', config),
});
```

**Renderer Process:**
```javascript
// Initialize
const capabilities = await window.capture.init();

// Start capture session
const session = await window.capture.startCapture({
  audio: {
    systemAudio: true,
    microphone: true,
    format: 'AAC'
  }
});
```

## Troubleshooting

### Common Issues

**1. "Native module not found"**
```bash
# Rebuild native module
npm run clean && npm run build
```

**2. "Permission denied" (macOS)**
```bash
# Grant microphone permission
# System Preferences > Security & Privacy > Microphone

# Grant screen recording permission  
# System Preferences > Security & Privacy > Screen Recording
```

**3. "No system audio captured"**

macOS:
- Install BlackHole virtual audio driver
- Configure Multi-Output Device in Audio MIDI Setup

Windows:
- Enable "Stereo Mix" in recording devices
- Update audio drivers

Linux:
- Check PulseAudio/PipeWire monitor sources
- Load loopback modules

**4. "FFmpeg encoding failed"**
```bash
# Check FFmpeg installation
ffmpeg -version

# Install missing dependencies
brew install ffmpeg pkg-config  # macOS
sudo apt install libavcodec-dev  # Ubuntu
```

### Debug Mode

```bash
# Build in debug mode
npm run build:debug

# Run with debug logging
NODE_ENV=development npm test
```

### Performance Optimization

**For Real-time Transcription:**
```javascript
const config = {
  audio: {
    segmentDuration: 1000,  // 1 second segments
    sampleRate: 16000,      // Lower sample rate for speech
    format: 'AAC',          // Compressed format
    channels: 1             // Mono for speech
  }
};
```

**For High-Quality Recording:**
```javascript
const config = {
  audio: {
    segmentDuration: 5000,  // 5 second segments
    sampleRate: 48000,      // Higher sample rate
    format: 'WAV',          // Uncompressed
    channels: 2             // Stereo
  }
};
```

## Production Deployment

### Building for Distribution

```bash
# Build for all platforms
npm run build

# Create platform-specific builds
npm run build -- --target x86_64-apple-darwin
npm run build -- --target x86_64-pc-windows-msvc
npm run build -- --target x86_64-unknown-linux-gnu
```

### Electron App Distribution

1. Include native module in app bundle
2. Handle permission requests gracefully
3. Provide fallback for unsupported platforms
4. Test on target platforms

### Security Considerations

- Request permissions explicitly
- Handle permission denials gracefully
- Validate audio data before processing
- Use secure audio processing pipelines

## API Reference

See `index.d.ts` for complete TypeScript definitions.

**Key Methods:**
- `init()` - Initialize library and check capabilities
- `getAudioDevices()` - List available audio devices
- `checkPermissions()` - Check current permission status
- `requestPermissions()` - Request necessary permissions
- `createCaptureSession(config)` - Create new capture session

**Events:**
- `audio` - New audio segment available
- `error` - Capture error occurred
- `permission-denied` - Permission request denied

## Support

For issues and questions:
1. Check this setup guide
2. Review the troubleshooting section
3. Check existing GitHub issues
4. Create a new issue with system details

## Contributing

See CONTRIBUTING.md for development setup and guidelines.
