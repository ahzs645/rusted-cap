# Cap Electron Capture Library

A high-performance, cross-platform screen capture and audio processing library designed for Electron applications. Built with Rust for maximum performance and reliability, with seamless Node.js integration for real-time audio transcription and screen recording applications.

## ✨ Features

### 🎵 Advanced Audio Capture
- **System Audio + Microphone**: Capture both system audio and microphone simultaneously
- **Real-time Segmentation**: Audio segments optimized for transcription services (1-2 second chunks)
- **Multiple Formats**: Support for WAV, Raw PCM, MP3, and AAC encoding
- **Cross-platform**: Native implementations for macOS, Windows, and Linux
- **Low Latency**: Optimized for real-time processing and transcription

### 🖥️ Screen Capture
- **Multi-display Support**: Capture from multiple monitors
- **Window-specific Capture**: Target individual applications
- **High Performance**: Hardware-accelerated capture when available
- **Flexible Output**: Multiple video formats and quality settings

### 🔒 Permission Management
- **Automatic Permission Requests**: Streamlined permission flow for all capture types
- **Status Checking**: Real-time permission status monitoring
- **Platform-specific Guidance**: Detailed setup instructions for each OS

### 🚀 Electron Integration
- **TypeScript Support**: Complete type definitions included
- **Event-driven Architecture**: Real-time audio/video streaming via events
- **Error Handling**: Comprehensive error types with contextual messages
- **Background Processing**: Non-blocking capture with async/await support

## 🏗️ Architecture

### Platform-Specific Audio Capture

#### macOS
- **ScreenCaptureKit**: Native system audio capture
- **Virtual Audio Drivers**: BlackHole/SoundFlower support
- **Core Audio**: High-quality microphone input

#### Windows
- **WASAPI Loopback**: System audio capture
- **Stereo Mix**: Legacy audio mixing support
- **Windows Capture API**: Modern capture methods

#### Linux
- **PipeWire**: Modern audio subsystem support
- **PulseAudio**: Monitor device capture
- **ALSA**: Low-level audio interface

## 🚀 Quick Start

### Installation

```bash
npm install cap-electron-capture
```

### Basic Usage

```javascript
const { init, createCaptureSession, AudioFormat } = require('cap-electron-capture');

// Initialize the library
const capabilities = JSON.parse(init());
console.log('Platform capabilities:', capabilities);

// Create a capture session for transcription
const session = createCaptureSession({
  audio: {
    enabled: true,
    systemAudio: true,
    microphone: true,
    segmentDurationMs: 1500, // 1.5 second segments
    format: AudioFormat.WAV,
    sampleRate: 48000,
    channels: 2
  },
  screen: {
    enabled: false // Audio-only for transcription
  }
});

// Start capturing
const audioStream = await session.start();

// Handle real-time audio segments
audioStream.on('data', (segment) => {
  console.log('Received audio segment:', {
    timestamp: segment.timestamp,
    duration: segment.duration_ms,
    source: segment.source, // 'Microphone', 'SystemAudio', or 'Mixed'
    sampleRate: segment.sampleRate,
    dataLength: segment.data.length
  });
  
  // Send to transcription service
  sendToTranscriptionService(segment);
});

// Stop when done
await session.stop();
```

## 🔧 Electron Integration

### Complete Transcription App Example

```javascript
// main.js
const { app, BrowserWindow, ipcMain } = require('electron');
const { createCaptureSession, checkPermissions, AudioFormat } = require('cap-electron-capture');

class TranscriptionManager {
  constructor() {
    this.session = null;
    this.isActive = false;
  }

  async start(config = {}) {
    if (this.isActive) throw new Error('Already recording');

    // Check permissions first
    const permissions = JSON.parse(await checkPermissions());
    if (permissions.microphone !== 'Granted') {
      throw new Error('Microphone permission required');
    }

    this.session = createCaptureSession({
      audio: {
        enabled: true,
        systemAudio: true,
        microphone: true,
        segmentDurationMs: 1500,
        format: AudioFormat.WAV,
        ...config.audio
      }
    });

    const audioStream = await this.session.start();
    this.isActive = true;

    // Real-time audio processing
    audioStream.on('data', (segment) => {
      mainWindow.webContents.send('audio-segment', {
        data: Array.from(segment.data),
        sampleRate: segment.sampleRate,
        timestamp: segment.timestamp,
        source: segment.source
      });
    });

    return audioStream;
  }

  async stop() {
    if (this.session) {
      await this.session.stop();
      this.session = null;
      this.isActive = false;
    }
  }
}

const transcriptionManager = new TranscriptionManager();

// IPC handlers
ipcMain.handle('start-transcription', async (event, config) => {
  try {
    await transcriptionManager.start(config);
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

ipcMain.handle('stop-transcription', async () => {
  await transcriptionManager.stop();
  return { success: true };
});
```

### Renderer Process

```javascript
// renderer.js
const { ipcRenderer } = require('electron');

class TranscriptionUI {
  constructor() {
    this.isRecording = false;
    this.audioSegments = [];
  }

  async startTranscription() {
    const result = await ipcRenderer.invoke('start-transcription', {
      audio: {
        segmentDurationMs: 1500,
        sampleRate: 48000
      }
    });

    if (result.success) {
      this.isRecording = true;
      this.updateUI();
    } else {
      console.error('Failed to start transcription:', result.error);
    }
  }

  async stopTranscription() {
    const result = await ipcRenderer.invoke('stop-transcription');
    if (result.success) {
      this.isRecording = false;
      this.updateUI();
    }
  }

  handleAudioSegment(segment) {
    // Process audio segment for transcription
    this.audioSegments.push(segment);
    
    // Send to transcription service (OpenAI Whisper, Google Speech-to-Text, etc.)
    this.transcribeSegment(segment);
  }

  async transcribeSegment(segment) {
    // Example: Send to transcription service
    // const transcript = await this.sendToWhisper(segment);
    // this.displayTranscript(transcript);
  }
}

// Listen for audio segments
ipcRenderer.on('audio-segment', (event, segment) => {
  transcriptionUI.handleAudioSegment(segment);
});
```

## 🔒 Permission Management

### Checking Permissions

```javascript
const { checkPermissions, requestPermissions, getSystemAudioSetupInstructions } = require('cap-electron-capture');

// Check current permission status
const permissions = JSON.parse(await checkPermissions());
console.log('Permissions:', permissions);
// {
//   microphone: 'Granted',
//   screenRecording: 'NotRequested',
//   systemAudio: 'Denied'
// }

// Request permissions if needed
if (permissions.microphone !== 'Granted') {
  await requestPermissions();
}

// Get setup instructions for system audio
if (permissions.systemAudio !== 'Granted') {
  console.log(getSystemAudioSetupInstructions());
}
```

### Platform-Specific Setup

#### macOS
```bash
# Install BlackHole for system audio capture
brew install blackhole-2ch
```

#### Windows
1. Enable "Stereo Mix" in Windows Sound settings
2. Or use VB-Audio Virtual Cable

#### Linux
```bash
# For PulseAudio
pactl load-module module-loopback

# For PipeWire
pw-loopback
```

## 📊 Audio Processing

### Audio Mixing

```javascript
// The library automatically handles mixing system audio and microphone
const session = createCaptureSession({
  audio: {
    systemAudio: true,
    microphone: true,
    // Audio will be automatically mixed
  }
});
```

### Format Conversion

```javascript
// Capture in different formats
const session = createCaptureSession({
  audio: {
    format: AudioFormat.WAV, // or Raw, MP3, AAC
    sampleRate: 48000,
    channels: 2
  }
});
```

## 🛠️ API Reference

### Core Functions

#### `init(): string`
Initialize the library and get platform capabilities.

#### `checkPermissions(): Promise<string>`
Check current permission status for all capture types.

#### `requestPermissions(): Promise<string>`
Request necessary permissions from the user.

#### `getAudioDevices(): string`
Get list of available audio input/output devices.

#### `getDisplays(): string`
Get list of available displays for screen capture.

#### `getSystemAudioSetupInstructions(): string`
Get platform-specific instructions for enabling system audio.

### CaptureSession

#### `createCaptureSession(config): CaptureSession`
Create a new capture session with the specified configuration.

#### `session.start(): Promise<AudioStream>`
Start the capture session and return an audio stream.

#### `session.stop(): Promise<void>`
Stop the capture session.

#### `session.isActive(): Promise<boolean>`
Check if the session is currently active.

### Configuration Types

```typescript
interface CaptureConfig {
  audio: AudioCaptureConfig;
  screen?: ScreenCaptureConfig;
}

interface AudioCaptureConfig {
  enabled: boolean;
  systemAudio: boolean;
  microphone: boolean;
  segmentDurationMs: number;
  format: AudioFormat;
  sampleRate: number;
  channels: number;
  microphoneDeviceId?: string;
}

enum AudioFormat {
  Raw = 'Raw',
  Wav = 'Wav', 
  Mp3 = 'Mp3',
  Aac = 'Aac'
}
```

## 🏗️ Building from Source

### Prerequisites

- Rust 1.70+
- Node.js 16+
- Platform-specific dependencies:
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools
  - Linux: build-essential, libasound2-dev

### Build Steps

```bash
# Clone the repository
git clone https://github.com/cap-so/cap-electron-capture
cd cap-electron-capture

# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test
```

### Development Build

```bash
# Build in debug mode (faster compilation)
npm run build:debug

# Watch for changes
npm run dev
```

## 🐛 Troubleshooting

### Common Issues

#### System Audio Not Working

**macOS**: Install BlackHole virtual audio driver
```bash
brew install blackhole-2ch
```

**Windows**: Enable "Stereo Mix" in Sound settings
1. Right-click speaker icon → Open Sound settings
2. Sound Control Panel → Recording tab
3. Right-click → Show Disabled Devices
4. Enable "Stereo Mix"

**Linux**: Configure PulseAudio/PipeWire loopback
```bash
pactl load-module module-loopback
```

#### Permission Errors

Use the built-in permission management:
```javascript
const instructions = getSystemAudioSetupInstructions();
console.log(instructions);
```

#### Build Errors

Ensure all native dependencies are installed:
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential libasound2-dev

# Windows - Install Visual Studio Build Tools
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built on top of the excellent [cpal](https://github.com/RustAudio/cpal) audio library
- Inspired by the original [Cap](https://github.com/cap-so/cap) screen recording app
- Uses [napi-rs](https://github.com/napi-rs/napi-rs) for seamless Node.js integration

---

**Perfect for building:**
- 🎙️ Real-time transcription applications
- 📹 Screen recording with audio
- 🎵 Audio analysis tools
- 📱 Voice-controlled applications
- 🤖 AI-powered audio processing
