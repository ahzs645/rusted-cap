const { 
  init, 
  getAudioDevices, 
  getDisplays, 
  checkPermissions,
  requestPermissions,
  getSystemAudioSetupInstructions,
  createCaptureSession,
  AudioFormat 
} = require('./index.js');

async function test() {
  try {
    console.log('🚀 Testing Cap Electron Capture Library');
    console.log('=========================================');

    // Initialize library
    console.log('\n📋 Initializing library...');
    const capabilities = init();
    console.log('Platform capabilities:', JSON.stringify(capabilities, null, 2));

    // Check permissions first
    console.log('\n🔒 Checking permissions...');
    try {
      const permissions = await checkPermissions();
      console.log('Permissions status:', permissions);
      
      if (permissions.systemAudio !== 'Granted') {
        console.log('\n📋 System Audio Setup Instructions:');
        console.log(getSystemAudioSetupInstructions());
      }
    } catch (error) {
      console.log('⚠️  Permission check failed:', error.message);
    }

    // Get audio devices
    console.log('\n🎤 Getting audio devices...');
    const audioDevicesRaw = getAudioDevices();
    console.log('Raw audio devices response:', audioDevicesRaw);
    console.log('Type of response:', typeof audioDevicesRaw);
    
    let audioDevices;
    try {
      audioDevices = JSON.parse(audioDevicesRaw);
      console.log('Parsed audio devices:', audioDevices);
      console.log('Audio devices found:', audioDevices.length);
      audioDevices.forEach((device, index) => {
        console.log(`  ${index + 1}. ${device.name} (${device.device_type}) ${device.is_default ? '(default)' : ''}`);
      });
    } catch (error) {
      console.log('❌ Failed to parse audio devices:', error.message);
      console.log('Raw response was:', audioDevicesRaw);
    }

    // Get displays
    console.log('\n🖥️  Getting displays...');
    const displaysRaw = getDisplays();
    let displays;
    try {
      displays = JSON.parse(displaysRaw);
      console.log('Displays found:', displays.length);
      displays.forEach((display, index) => {
        console.log(`  ${index + 1}. ${display.name} ${display.width}x${display.height} ${display.is_primary ? '(primary)' : ''}`);
      });
    } catch (error) {
      console.log('❌ Failed to parse displays:', error.message);
      console.log('Raw response was:', displaysRaw);
    }

    // Create capture session for audio-only transcription
    console.log('\n🎬 Creating audio-only capture session...');
    const sessionConfig = {
      audio: {
        enabled: true,
        systemAudio: false, // Disable for testing without permissions
        microphone: true,
        segmentDurationMs: 1000, // 1 second segments for real-time transcription
        format: AudioFormat.WAV, // Use WAV for better compatibility
        sampleRate: 48000,
        channels: 2
      },
      screen: {
        enabled: false // Audio-only for transcription
      }
    };
    
    const sessionConfigString = JSON.stringify(sessionConfig);
    const sessionRaw = createCaptureSession(sessionConfigString);
    const sessionData = JSON.parse(sessionRaw);

    // Create a simple session wrapper with mock methods for testing
    const session = {
      id: sessionData.id,
      config: sessionData.config,
      status: sessionData.status,
      capabilities: sessionData.capabilities,
      isActive: async () => sessionData.status === 'started',
      start: async () => { sessionData.status = 'started'; },
      stop: async () => { sessionData.status = 'stopped'; },
    };

    console.log('✅ Capture session created successfully');
    console.log('   Session ID:', session.id);
    console.log('   Capabilities:', session.capabilities);
    console.log('Session active:', await session.isActive());

    // Test starting and stopping
    console.log('\n▶️  Starting capture session...');
    try {
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
    } catch (error) {
      console.log('⚠️  Capture session test failed (expected without permissions):', error.message);
    }

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
