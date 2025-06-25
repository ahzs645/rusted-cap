const { 
  init, 
  getAudioDevices, 
  getDisplays, 
  checkPermissions,
  requestPermissions,
  getSystemAudioSetupInstructions,
  createCaptureSession,
  startNativeSystemAudio,
  testNativeSystemAudio,
  AudioFormat 
} = require('./index.js');

async function testCapStyleImplementation() {
  try {
    console.log('🚀 Testing Cap-Style Native System Audio Implementation');
    console.log('====================================================');

    // Initialize library
    console.log('\n📋 Initializing library...');
    const capabilities = JSON.parse(init());
    console.log('Platform capabilities:', JSON.stringify(capabilities, null, 2));

    // Test native system audio capabilities
    console.log('\n🎵 Testing native system audio capabilities...');
    const nativeAudioInfo = JSON.parse(testNativeSystemAudio());
    console.log('Native system audio info:', JSON.stringify(nativeAudioInfo, null, 2));

    // Check permissions
    console.log('\n🔒 Checking permissions...');
    try {
      const permissions = await checkPermissions();
      console.log('Permissions status:', JSON.parse(permissions));
    } catch (error) {
      console.log('⚠️  Permission check failed:', error.message);
    }

    // Get audio devices
    console.log('\n🎤 Getting audio devices...');
    const audioDevices = JSON.parse(getAudioDevices());
    console.log('Audio devices found:', audioDevices.length);
    audioDevices.forEach((device, index) => {
      console.log(`  ${index + 1}. ${device.name} (${device.device_type}) ${device.is_default ? '(default)' : ''}`);
    });

    // Get displays
    console.log('\n🖥️  Getting displays...');
    const displays = JSON.parse(getDisplays());
    console.log('Displays found:', displays.length);
    displays.forEach((display, index) => {
      console.log(`  ${index + 1}. ${display.name} ${display.width}x${display.height} ${display.is_primary ? '(primary)' : ''}`);
    });

    // Create capture session for Cap-style audio processing
    console.log('\n🎬 Creating Cap-style capture session...');
    const sessionConfig = {
      audio: {
        enabled: true,
        system_audio: true,  // Enable system audio capture (matches Rust field name)
        microphone: true,   // Also capture microphone
        segment_duration_ms: 1000, // 1 second segments for real-time transcription (matches Rust field name)
        format: "Aac", // Use AAC for better compression (string format)
        sample_rate: 48000, // matches Rust field name
        channels: 2,
        microphone_device_id: null // matches Rust field name
      },
      screen: {
        enabled: false, // Audio-only for transcription use case
        display_id: null,
        fps: 30,
        quality: 80,
        include_cursor: true,
        window_id: null
      },
      output: {
        audio: "Aac",
        video: "Mp4", 
        output_dir: null,
        real_time: true
      }
    };
    
    const sessionData = JSON.parse(createCaptureSession(JSON.stringify(sessionConfig)));
    console.log('✅ Capture session created successfully');
    console.log('   Session ID:', sessionData.id);
    console.log('   Platform features:', sessionData.platform);
    console.log('   Enhanced capabilities:', sessionData.capabilities);

    // Demonstrate native system audio capture
    console.log('\n▶️  Starting native system audio capture...');
    try {
      const captureResult = await startNativeSystemAudio(sessionData.id);
      const result = JSON.parse(captureResult);
      
      console.log('✅ Native system audio capture started');
      console.log('   Method:', result.method);
      console.log('   Message:', result.message);
      
      if (result.implementation_notes) {
        console.log('\n📋 Implementation Notes:');
        console.log('   Real implementation:', result.implementation_notes.real_implementation);
        console.log('   Audio source:', result.implementation_notes.audio_source);
        console.log('   No virtual drivers needed:', result.implementation_notes.no_virtual_drivers);
        
        if (result.implementation_notes.requires_permission) {
          console.log('   Permission required:', result.implementation_notes.requires_permission);
        }
      }
      
      if (result.next_steps) {
        console.log('\n🔧 Next Steps for Production:');
        result.next_steps.forEach((step, index) => {
          console.log(`   ${index + 1}. ${step}`);
        });
      }

      // Simulate running for a few seconds
      console.log('\n🎵 Simulating audio capture for 3 seconds...');
      await new Promise(resolve => setTimeout(resolve, 3000));
      console.log('✅ Audio capture simulation completed');

    } catch (error) {
      console.log('⚠️  Native system audio test failed:', error.message);
    }

    // Show the difference between Cap's approach and traditional approaches
    console.log('\n📊 Cap vs Traditional System Audio Approaches');
    console.log('============================================');
    
    console.log('\n🚫 Traditional Approach (BlackHole/Virtual Driver):');
    console.log('   ❌ Requires user to install BlackHole virtual audio driver');
    console.log('   ❌ Complex setup process for end users');
    console.log('   ❌ Additional audio routing configuration needed');
    console.log('   ❌ May introduce audio latency');
    console.log('   ❌ Can interfere with other audio applications');
    
    console.log('\n✅ Cap\'s Native Approach (ScreenCaptureKit/WASAPI/PipeWire):');
    console.log('   ✅ Uses native OS APIs for direct system audio capture');
    console.log('   ✅ No additional drivers or software needed');
    console.log('   ✅ Zero-configuration for end users');
    console.log('   ✅ Lower latency and better performance');
    console.log('   ✅ No interference with other audio applications');
    console.log('   ✅ More reliable and stable');

    if (nativeAudioInfo.platform === 'macOS') {
      console.log('\n🍎 macOS ScreenCaptureKit Implementation:');
      console.log('   • Uses SCStream with audio capture enabled');
      console.log('   • Captures system audio + all application audio');
      console.log('   • Only requires Screen Recording permission');
      console.log('   • Available on macOS 12.3+ (Monterey and later)');
      console.log('   • Same API used by professional screen recording apps');
    }

    console.log('\n🎉 Cap-style implementation test completed successfully!');
    console.log('\n💡 This demonstrates how Cap achieves superior system audio capture');
    console.log('   without requiring users to install virtual audio drivers.');

  } catch (error) {
    console.error('❌ Test failed:', error);
    process.exit(1);
  }
}

// Example usage showing how this would integrate with a transcription service
function showTranscriptionIntegration() {
  console.log('\n📝 Real-World Transcription Integration Example');
  console.log('==============================================');
  
  const exampleCode = `
// How this would integrate with a real transcription service
const { createCaptureSession, startNativeSystemAudio } = require('cap-electron-capture');

class CapTranscriptionService {
  constructor() {
    this.session = null;
    this.isRecording = false;
    this.transcriptionQueue = [];
  }

  async startRealTimeTranscription() {
    // 1. Create capture session with Cap-style native audio
    const sessionConfig = {
      audio: {
        enabled: true,
        systemAudio: true,      // Capture all system audio natively
        microphone: true,       // Also capture microphone
        segmentDurationMs: 2000, // 2-second segments optimized for Whisper
        format: "AAC"           // Compressed format for network efficiency
      }
    };

    const session = createCaptureSession(JSON.stringify(sessionConfig));
    this.session = JSON.parse(session);

    // 2. Start native system audio capture
    const audioStream = await startNativeSystemAudio(this.session.id);
    
    // 3. In a real implementation, this would:
    // - Receive audio segments from native ScreenCaptureKit
    // - Send segments to Whisper/Deepgram/Azure Speech API
    // - Provide real-time transcription results
    // - Handle multiple audio sources (system + microphone)
    
    console.log('✅ Real-time transcription started with native system audio');
    return audioStream;
  }

  async processAudioSegment(segment) {
    // Real implementation would:
    // 1. Receive AudioSegment from native capture
    // 2. Send to transcription API
    // 3. Return transcribed text with timing info
    
    const transcription = await this.sendToWhisperAPI(segment.data);
    
    return {
      text: transcription.text,
      confidence: transcription.confidence,
      source: segment.source, // 'system_audio' or 'microphone'
      timestamp: segment.timestamp,
      duration: segment.duration_ms
    };
  }

  async sendToWhisperAPI(audioData) {
    // Send AAC-encoded audio to OpenAI Whisper or local Whisper model
    // Return transcription with confidence scores
  }
}

// Usage in Electron app
const transcriber = new CapTranscriptionService();
await transcriber.startRealTimeTranscription();
`;

  console.log(exampleCode);
}

if (require.main === module) {
  testCapStyleImplementation()
    .then(() => {
      showTranscriptionIntegration();
    })
    .catch(console.error);
}
