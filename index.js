/**
 * Cap Electron Capture Library
 * 
 * A cross-platform screen capture and audio processing library designed for
 * integration with Electron applications.
 */

// For now, we'll provide a mock implementation until the native module is built
let native;

try {
  // Try to load the native binding
  native = require('./cap-electron-capture.node');
} catch (error) {
  console.warn('Native module not found, using mock implementation for testing');
  
  // Mock implementation for development/testing
  native = {
    init: () => JSON.stringify({
      platform: 'MacOS',
      audio: {
        systemAudio: true,
        microphone: true,
        sampleRates: [44100, 48000],
        formats: ['AAC', 'MP3', 'WAV'],
        inputDevices: 2,
        outputDevices: 1
      },
      screen: {
        supported: true,
        displayCount: 1,
        windowCapture: true,
        maxResolution: [3840, 2160],
        frameRates: [15, 30, 60]
      },
      systemVersion: '14.0',
      permissions: {
        microphone: 'NotRequested',
        screenRecording: 'NotRequested',
        systemAudio: 'NotRequested'
      }
    }),
    
    getAudioDevices: () => JSON.stringify([
      {
        id: 'default-input',
        name: 'Built-in Microphone',
        device_type: 'Input',
        is_default: true,
        sample_rates: [44100, 48000],
        channels: [1, 2]
      },
      {
        id: 'default-output',
        name: 'Built-in Speakers',
        device_type: 'Output',
        is_default: true,
        sample_rates: [44100, 48000],
        channels: [2]
      }
    ]),
    
    getDisplays: () => JSON.stringify([
      {
        id: 0,
        name: 'Built-in Display',
        width: 1920,
        height: 1080,
        x: 0,
        y: 0,
        is_primary: true,
        scale_factor: 2.0,
        refresh_rate: 60
      }
    ]),

    checkPermissions: () => Promise.resolve(JSON.stringify({
      microphone: 'NotRequested',
      screenRecording: 'NotRequested',
      systemAudio: 'NotRequested'
    })),

    requestPermissions: () => Promise.resolve(JSON.stringify({
      microphone: 'Granted',
      screenRecording: 'Granted', 
      systemAudio: 'Denied'
    })),

    getSystemAudioSetupInstructions: () => 
      "macOS System Audio Setup:\n" +
      "1. Install BlackHole virtual audio driver: https://existential.audio/blackhole/\n" +
      "2. Or enable ScreenCaptureKit permissions in System Preferences > Security & Privacy > Screen Recording\n" +
      "3. Restart your application after setup"
  };
}

/**
 * Initialize the library and check platform capabilities
 * @returns {Object} Platform capabilities information
 */
function init() {
  const capabilities = native.init();
  return JSON.parse(capabilities);
}

/**
 * Get available audio devices
 * @returns {Array} List of available audio devices
 */
function getAudioDevices() {
  const devices = native.getAudioDevices();
  return JSON.parse(devices);
}

/**
 * Get available displays for screen capture
 * @returns {Array} List of available displays
 */
function getDisplays() {
  const displays = native.getDisplays();
  return JSON.parse(displays);
}

/**
 * Check current permission status without requesting
 * @returns {Promise<Object>} Permission status for all capture types
 */
async function checkPermissions() {
  const permissions = await native.checkPermissions();
  return JSON.parse(permissions);
}

/**
 * Request all necessary permissions for audio and screen capture
 * @returns {Promise<Object>} Permission status after requesting
 */
async function requestPermissions() {
  const permissions = await native.requestPermissions();
  return JSON.parse(permissions);
}

/**
 * Get platform-specific instructions for enabling system audio capture
 * @returns {string} Setup instructions
 */
function getSystemAudioSetupInstructions() {
  return native.getSystemAudioSetupInstructions();
}

/**
 * Create a new capture session
 * @param {Object} config - Capture configuration
 * @returns {Object} Mock capture session for testing
 */
function createCaptureSession(config = {}) {
  // Provide default configuration
  const defaultConfig = {
    audio: {
      enabled: true,
      systemAudio: true,
      microphone: true,
      sampleRate: 44100,
      channels: 2,
      segmentDurationMs: 2000,
      microphoneDeviceId: null,
      format: 'Aac'
    },
    screen: {
      enabled: false,
      displayId: null,
      fps: 30,
      quality: 80,
      includeCursor: true,
      windowId: null
    },
    output: {
      audio: 'Aac',
      video: 'Mp4',
      outputDir: null,
      realTime: true
    }
  };

  // Merge user config with defaults
  const mergedConfig = {
    ...defaultConfig,
    ...config,
    audio: { ...defaultConfig.audio, ...config.audio },
    screen: { ...defaultConfig.screen, ...config.screen },
    output: { ...defaultConfig.output, ...config.output }
  };

  // Return a mock session for testing
  return {
    config: mergedConfig,
    _isActive: false,
    
    async start() {
      if (this._isActive) {
        throw new Error('Session is already active');
      }
      console.log('Mock: Starting capture session with config:', this.config);
      this._isActive = true;
      return Promise.resolve();
    },
    
    async stop() {
      if (!this._isActive) {
        throw new Error('Session is not active');
      }
      console.log('Mock: Stopping capture session');
      this._isActive = false;
      return Promise.resolve();
    },
    
    async isActive() {
      return Promise.resolve(this._isActive);
    }
  };
}

/**
 * Audio formats enum
 */
const AudioFormat = {
  AAC: 'Aac',
  MP3: 'Mp3',
  WAV: 'Wav',
  RAW: 'Raw'
};

/**
 * Video formats enum
 */
const VideoFormat = {
  MP4: 'Mp4',
  WEBM: 'WebM',
  RAW: 'Raw'
};

/**
 * Audio source types enum
 */
const AudioSource = {
  MICROPHONE: 'Microphone',
  SYSTEM_AUDIO: 'SystemAudio',
  MIXED: 'Mixed'
};

/**
 * Permission states enum
 */
const PermissionState = {
  GRANTED: 'Granted',
  DENIED: 'Denied',
  NOT_REQUESTED: 'NotRequested',
  REQUESTING: 'Requesting',
  NOT_APPLICABLE: 'NotApplicable'
};

module.exports = {
  // Main functions
  init,
  getAudioDevices,
  getDisplays,
  createCaptureSession,
  
  // Permission functions
  checkPermissions,
  requestPermissions,
  getSystemAudioSetupInstructions,
  
  // Enums and constants
  AudioFormat,
  VideoFormat,
  AudioSource,
  PermissionState
};
