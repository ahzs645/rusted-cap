# Cap Electron Capture - Examples

This directory contains example implementations showing how to integrate the Cap Electron Capture library with Electron applications.

## 📁 Files Overview

- **`electron-main.js`** - Main Electron process with IPC handlers for capture functionality
- **`preload.js`** - Preload script that exposes safe APIs to the renderer
- **`index.html`** - UI for the transcription demo application
- **`renderer.js`** - Renderer process script handling the user interface
- **`package.json`** - Electron app dependencies and scripts

## 🚀 Quick Start

1. **Navigate to the examples directory:**
   ```bash
   cd examples
   ```

2. **Install Electron:**
   ```bash
   npm install
   ```

3. **Run the demo:**
   ```bash
   npm start
   ```

## 🎯 Demo Features

The example application demonstrates:

- ✅ **Platform Detection** - Shows system capabilities and audio devices
- ✅ **Audio Configuration** - Toggle system audio and microphone capture
- ✅ **Real-time Controls** - Start/stop transcription with live status
- ✅ **Mock Transcription** - Simulates transcription workflow
- ✅ **Modern UI** - Beautiful, responsive interface

## 🔧 Integration Guide

### 1. Main Process Setup

```javascript
const { createCaptureSession, init, AudioFormat } = require('cap-electron-capture');

// Initialize library
const capabilities = init();

// Create capture session for audio-only transcription
const session = createCaptureSession({
  audio: {
    enabled: true,
    systemAudio: true,      // Capture system output (Zoom, YouTube, etc.)
    microphone: true,       // Capture microphone input
    segmentDurationMs: 2000, // 2-second segments for real-time processing
    format: AudioFormat.AAC
  },
  screen: {
    enabled: false // Audio-only for transcription
  }
});

await session.start();
```

### 2. IPC Communication

```javascript
// Main process
ipcMain.handle('start-transcription', async (event, config) => {
  const session = createCaptureSession(config);
  await session.start();
  return { success: true };
});

// Renderer process (via preload)
const result = await window.electronAPI.startTranscription(config);
```

### 3. Real Transcription Integration

To add actual transcription (not included in this demo), you would:

```javascript
// In the main process, after starting capture
const audioStream = await session.start();

// Process audio segments with your transcription service
audioStream.on('data', async (audioSegment) => {
  try {
    // Example with OpenAI Whisper
    const transcription = await whisper.transcribe(audioSegment.data);
    
    // Send result to renderer
    mainWindow.webContents.send('transcription-result', {
      text: transcription.text,
      confidence: transcription.confidence,
      source: audioSegment.source, // 'Microphone' or 'SystemAudio'
      timestamp: audioSegment.timestamp
    });
  } catch (error) {
    mainWindow.webContents.send('transcription-error', error);
  }
});
```

## 🔌 Transcription Services Integration

The library works with any transcription service:

### OpenAI Whisper
```javascript
const transcription = await openai.audio.transcriptions.create({
  file: audioSegment.data,
  model: 'whisper-1',
  language: 'en'
});
```

### Google Speech-to-Text
```javascript
const [response] = await speechClient.recognize({
  audio: { content: audioSegment.data.toString('base64') },
  config: {
    encoding: 'WEBM_OPUS',
    sampleRateHertz: audioSegment.sampleRate,
    languageCode: 'en-US'
  }
});
```

### AWS Transcribe Streaming
```javascript
const transcribeStream = await transcribeService.startStreamTranscription({
  LanguageCode: 'en-US',
  MediaEncoding: 'pcm',
  MediaSampleRateHertz: audioSegment.sampleRate
});
```

## 📱 UI Components

The demo includes several reusable UI components:

### Status Display
```javascript
function showStatus(message, type = 'info') {
  // Shows success, error, or info messages
}
```

### Platform Information
```javascript
function displayPlatformInfo() {
  // Shows system capabilities and available devices
}
```

### Recording Controls
```javascript
function updateUI() {
  // Updates button states based on recording status
}
```

## 🛡️ Security Considerations

The example follows Electron security best practices:

- ✅ **Context Isolation** enabled
- ✅ **Node Integration** disabled in renderer
- ✅ **Preload script** for safe API exposure
- ✅ **IPC validation** for all communications

## 🎛️ Configuration Options

You can customize the capture behavior:

```javascript
const config = {
  audio: {
    enabled: true,
    systemAudio: true,           // Capture computer output
    microphone: true,            // Capture microphone
    sampleRate: 44100,           // Audio quality
    channels: 2,                 // Stereo/mono
    segmentDurationMs: 2000,     // Real-time processing interval
    microphoneDeviceId: 'device-id', // Specific microphone
    format: 'Aac'                // Audio format
  },
  screen: {
    enabled: false,              // Enable for screen + audio recording
    fps: 30,                     // Frame rate
    quality: 80,                 // Video quality (0-100)
    includeCursor: true,         // Show cursor
    displayId: 0                 // Specific display
  },
  output: {
    realTime: true,              // Real-time processing
    outputDir: './recordings'    // Save location
  }
};
```

## 📊 Performance Tips

- **Segment Duration**: Use 1-3 seconds for real-time transcription
- **Audio Format**: AAC provides good compression for streaming
- **Sample Rate**: 44.1kHz is sufficient for speech recognition
- **Memory**: Stop sessions when not needed to free resources

## 🐛 Troubleshooting

### Common Issues

1. **"Native module not found"**
   - Build the native module: `npm run build`
   - Check platform compatibility

2. **"Permission denied"**
   - Grant microphone permissions in system settings
   - Grant screen recording permissions (macOS)

3. **"Audio device not found"**
   - Check available devices with `getAudioDevices()`
   - Verify device IDs match

### Debug Mode

Run with debug output:
```bash
NODE_ENV=development npm start
```

## 📚 Next Steps

1. **Add Real Transcription** - Integrate with Whisper, Google, or AWS
2. **Speaker Diarization** - Separate different speakers
3. **Live Correction** - Allow real-time text editing
4. **Export Options** - Save transcriptions in various formats
5. **Cloud Storage** - Sync transcriptions across devices

## 🤝 Contributing

Have ideas for better examples? Please contribute:

1. Fork the repository
2. Create your feature branch
3. Add your example
4. Submit a pull request

## 📄 License

This example code is provided under the same MIT license as the main library.
