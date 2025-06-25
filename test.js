const { init, getAudioDevices, getDisplays, createCaptureSession, AudioFormat } = require('./index.js');

async function test() {
  try {
    console.log('🚀 Testing Cap Electron Capture Library');
    console.log('=========================================');

    // Initialize library
    console.log('\n📋 Initializing library...');
    const capabilities = init();
    console.log('Platform capabilities:', JSON.stringify(capabilities, null, 2));

    // Get audio devices
    console.log('\n🎤 Getting audio devices...');
    const audioDevices = getAudioDevices();
    console.log('Audio devices found:', audioDevices.length);
    audioDevices.forEach((device, index) => {
      console.log(`  ${index + 1}. ${device.name} (${device.deviceType}) ${device.isDefault ? '(default)' : ''}`);
    });

    // Get displays
    console.log('\n🖥️  Getting displays...');
    const displays = getDisplays();
    console.log('Displays found:', displays.length);
    displays.forEach((display, index) => {
      console.log(`  ${index + 1}. ${display.name} ${display.resolution[0]}x${display.resolution[1]} ${display.isPrimary ? '(primary)' : ''}`);
    });

    // Create capture session for audio-only transcription
    console.log('\n🎬 Creating audio-only capture session...');
    const session = createCaptureSession({
      audio: {
        enabled: true,
        systemAudio: true,
        microphone: true,
        segmentDurationMs: 1000, // 1 second segments for real-time transcription
        format: AudioFormat.AAC
      },
      screen: {
        enabled: false // Audio-only for transcription
      }
    });

    console.log('✅ Capture session created successfully');
    console.log('Session active:', await session.isActive());

    // Test starting and stopping
    console.log('\n▶️  Starting capture session...');
    await session.start();
    console.log('✅ Capture session started');
    console.log('Session active:', await session.isActive());

    // Let it run for a short time
    console.log('🎵 Capturing audio for 3 seconds...');
    await new Promise(resolve => setTimeout(resolve, 3000));

    console.log('\n⏹️  Stopping capture session...');
    await session.stop();
    console.log('✅ Capture session stopped');
    console.log('Session active:', await session.isActive());

    console.log('\n🎉 All tests completed successfully!');

  } catch (error) {
    console.error('❌ Test failed:', error);
    process.exit(1);
  }
}

// Example usage for Electron transcription app
function electronTranscriptionExample() {
  console.log('\n📝 Example: Electron Transcription App Usage');
  console.log('===========================================');
  
  const exampleCode = `
// In your Electron main process
const { createCaptureSession, AudioFormat } = require('cap-electron-capture');

class TranscriptionService {
  constructor() {
    this.session = null;
    this.isRecording = false;
  }

  async startTranscription() {
    // Create audio-only capture session
    this.session = createCaptureSession({
      audio: {
        enabled: true,
        systemAudio: true,      // Capture computer audio
        microphone: true,       // Capture microphone
        segmentDurationMs: 2000, // 2-second segments for transcription
        format: AudioFormat.AAC
      },
      screen: {
        enabled: false // Audio-only for transcription
      }
    });

    // Start capturing
    const audioStream = await this.session.start();
    this.isRecording = true;

    // Process audio segments in real-time
    audioStream.on('data', (audioSegment) => {
      // Send to transcription service (Whisper, Deepgram, etc.)
      this.processAudioSegment(audioSegment);
    });

    return audioStream;
  }

  async stopTranscription() {
    if (this.session) {
      await this.session.stop();
      this.isRecording = false;
    }
  }

  processAudioSegment(segment) {
    // Send audio data to transcription API
    console.log(\`Processing \${segment.durationMs}ms of \${segment.source} audio\`);
    
    // Example: Send to Whisper API
    // const transcription = await whisper.transcribe(segment.data);
    // this.sendToRenderer('transcription', { text: transcription, source: segment.source });
  }
}

// Usage
const transcriptionService = new TranscriptionService();
await transcriptionService.startTranscription();
`;

  console.log(exampleCode);
}

if (require.main === module) {
  test()
    .then(() => {
      electronTranscriptionExample();
    })
    .catch(console.error);
}
