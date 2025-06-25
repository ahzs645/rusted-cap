# Cap Electron Capture

A cross-platform screen capture and audio processing library designed for integration with Electron applications. This library extracts the core functionality from [Cap's](https://github.com/cap-so/cap) screen recording pipeline for use in transcription and real-time audio processing applications.

## Features

- 🎤 **Real-time Audio Capture**: System audio + microphone input
- 🖥️ **Cross-platform Screen Capture**: macOS, Windows, and Linux support
- ⚡ **Low Latency**: Optimized for real-time processing
- 🔊 **Audio Segmentation**: Configurable segment duration for transcription services
- 📱 **Electron Ready**: Node.js bindings for seamless Electron integration
- 🎯 **TypeScript Support**: Full type definitions included
- 🛡️ **Memory Safe**: Written in Rust with proper error handling

## Platform Support

| Platform | Screen Capture | System Audio | Microphone |
|----------|----------------|--------------|------------|
| macOS    | ✅ ScreenCaptureKit | ✅ | ✅ |
| Windows  | ✅ Windows Capture API | ✅ WASAPI Loopback | ✅ |
| Linux    | ✅ PipeWire/X11 | ✅ PipeWire/PulseAudio | ✅ |

## Installation

```bash
npm install cap-electron-capture
```

Or using yarn:

```bash
yarn add cap-electron-capture
```

## Quick Start

### Audio-Only Transcription (Recommended)

```javascript
const { createCaptureSession, AudioFormat } = require('cap-electron-capture');

async function startTranscription() {
  // Create audio-only capture session
  const session = createCaptureSession({
    audio: {
      enabled: true,
      systemAudio: true,      // Capture computer output
      microphone: true,       // Capture microphone input  
      segmentDurationMs: 2000, // 2-second segments
      format: AudioFormat.AAC
    },
    screen: {
      enabled: false // Audio-only for transcription
    }
  });

  // Start capturing
  await session.start();
  console.log('Audio capture started!');

  // Stop after 10 seconds
  setTimeout(async () => {
    await session.stop();
    console.log('Audio capture stopped!');
  }, 10000);
}

startTranscription().catch(console.error);
```

### Full Screen + Audio Recording

```javascript
const { createCaptureSession } = require('cap-electron-capture');

const session = createCaptureSession({
  audio: {
    enabled: true,
    systemAudio: true,
    microphone: true,
    segmentDurationMs: 1000
  },
  screen: {
    enabled: true,
    fps: 30,
    quality: 80,
    includeCursor: true
  }
});

await session.start();
```

## API Reference

### Functions

#### `init(): PlatformCapabilities`
Initialize the library and get platform capabilities.

```javascript
const capabilities = init();
console.log(capabilities.platform); // 'MacOS', 'Windows', 'Linux'
console.log(capabilities.audio.systemAudio); // true/false
```

#### `getAudioDevices(): AudioDevice[]`
Get available audio input/output devices.

```javascript
const devices = getAudioDevices();
devices.forEach(device => {
  console.log(`${device.name} (${device.deviceType})`);
});
```

#### `getDisplays(): Display[]`
Get available displays for screen capture.

```javascript
const displays = getDisplays();
displays.forEach(display => {
  console.log(`${display.name}: ${display.resolution[0]}x${display.resolution[1]}`);
});
```

#### `createCaptureSession(config?: CaptureConfig): CaptureSession`
Create a new capture session with optional configuration.

### Configuration

```typescript
interface CaptureConfig {
  audio: {
    enabled: boolean;
    systemAudio: boolean;        // Capture system output
    microphone: boolean;         // Capture microphone input
    sampleRate: number;          // 44100, 48000, etc.
    channels: number;            // 1 (mono) or 2 (stereo)
    segmentDurationMs: number;   // Segment length for real-time processing
    microphoneDeviceId?: string; // Specific device ID (optional)
    format: AudioFormat;         // 'Aac', 'Mp3', 'Wav', 'Raw'
  };
  screen: {
    enabled: boolean;
    displayId?: number;          // Specific display (optional)
    fps: number;                 // Frame rate (15, 30, 60)
    quality: number;             // 0-100 quality
    includeCursor: boolean;      // Include cursor in capture
    windowId?: number;           // Specific window (optional)
  };
  output: {
    audio: AudioFormat;
    video: VideoFormat;
    outputDir?: string;          // Output directory (optional)
    realTime: boolean;           // Real-time processing
  };
}
```

### CaptureSession Class

```typescript
class CaptureSession {
  constructor(config: CaptureConfig);
  
  async start(): Promise<void>;
  async stop(): Promise<void>;
  async isActive(): Promise<boolean>;
}
```

## Electron Integration Example

### Main Process (main.js)

```javascript
const { app, BrowserWindow, ipcMain } = require('electron');
const { createCaptureSession, init } = require('cap-electron-capture');

let mainWindow;
let captureSession;

app.whenReady().then(() => {
  // Initialize capture library
  const capabilities = init();
  console.log('Platform capabilities:', capabilities);

  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  });

  // Handle start transcription
  ipcMain.handle('start-transcription', async () => {
    try {
      captureSession = createCaptureSession({
        audio: {
          enabled: true,
          systemAudio: true,
          microphone: true,
          segmentDurationMs: 2000
        },
        screen: { enabled: false }
      });

      await captureSession.start();
      return { success: true };
    } catch (error) {
      return { success: false, error: error.message };
    }
  });

  // Handle stop transcription
  ipcMain.handle('stop-transcription', async () => {
    if (captureSession) {
      await captureSession.stop();
      captureSession = null;
    }
    return { success: true };
  });
});
```

### Preload Script (preload.js)

```javascript
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('captureAPI', {
  startTranscription: () => ipcRenderer.invoke('start-transcription'),
  stopTranscription: () => ipcRenderer.invoke('stop-transcription')
});
```

### Renderer Process (renderer.js)

```javascript
let isRecording = false;

document.getElementById('start-btn').addEventListener('click', async () => {
  if (!isRecording) {
    const result = await window.captureAPI.startTranscription();
    if (result.success) {
      isRecording = true;
      updateUI();
    }
  }
});

document.getElementById('stop-btn').addEventListener('click', async () => {
  if (isRecording) {
    await window.captureAPI.stopTranscription();
    isRecording = false;
    updateUI();
  }
});
```

## Use Cases

### 1. Real-time Transcription Apps
Capture both system audio and microphone for meeting transcription:

```javascript
const session = createCaptureSession({
  audio: {
    enabled: true,
    systemAudio: true,  // Capture Zoom/Teams audio
    microphone: true,   // Capture user voice
    segmentDurationMs: 1500 // Fast transcription
  }
});
```

### 2. Screen Recording with Commentary
Record screen with microphone narration:

```javascript
const session = createCaptureSession({
  audio: {
    enabled: true,
    systemAudio: false, // No system audio
    microphone: true    // Only microphone
  },
  screen: {
    enabled: true,
    fps: 30,
    quality: 90
  }
});
```

### 3. System Audio Analysis
Monitor and analyze system audio output:

```javascript
const session = createCaptureSession({
  audio: {
    enabled: true,
    systemAudio: true,  // Only system audio
    microphone: false,
    segmentDurationMs: 500 // High frequency analysis
  }
});
```

## Error Handling

```javascript
try {
  const session = createCaptureSession(config);
  await session.start();
} catch (error) {
  if (error.message.includes('Permission denied')) {
    console.log('Please grant microphone/screen recording permissions');
  } else if (error.message.includes('Device not found')) {
    console.log('Audio device not available');
  } else {
    console.error('Capture error:', error);
  }
}
```

## Building from Source

### Prerequisites

- Rust 1.70+
- Node.js 16+
- Platform-specific tools:
  - **macOS**: Xcode command line tools
  - **Windows**: Visual Studio Build Tools
  - **Linux**: GCC, ALSA/PulseAudio dev packages

### Build Steps

```bash
# Clone repository
git clone https://github.com/cap-so/cap.git
cd cap/cap-audio

# Install dependencies
npm install

# Build native module
npm run build

# Run tests
npm test
```

## Permissions

### macOS
- **Microphone**: Automatically requested when needed
- **Screen Recording**: Must be granted in System Preferences > Security & Privacy

### Windows
- **Microphone**: Automatically requested when needed
- **Screen Capture**: No special permissions required

### Linux
- **Microphone**: Handled by system audio server
- **Screen Capture**: May require Wayland portal permissions

## Performance Tips

1. **Segment Duration**: Use 1-3 seconds for transcription, longer for storage
2. **Audio Format**: AAC provides best compression for real-time use
3. **Sample Rate**: 44.1kHz is sufficient for speech transcription
4. **Memory Usage**: Stop sessions when not needed to free resources

## Troubleshooting

### Common Issues

**Audio device not found**
```javascript
// List available devices first
const devices = getAudioDevices();
console.log('Available devices:', devices);
```

**Permission denied**
```javascript
// Check platform capabilities
const capabilities = init();
console.log('Permissions:', capabilities.permissions);
```

**High CPU usage**
- Reduce segment frequency (increase `segmentDurationMs`)
- Lower audio sample rate if acceptable
- Disable screen capture if only audio is needed

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

## Related Projects

- [Cap](https://github.com/cap-so/cap) - The main Cap screen recording application
- [scap](https://github.com/gyroflow/scap) - Screen capture library
- [cpal](https://github.com/rustaudio/cpal) - Cross-platform audio I/O library
