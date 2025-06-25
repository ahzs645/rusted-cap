// Enhanced main.js with error handling for Electron transcription app
const { app, BrowserWindow, ipcMain } = require('electron');
const { createCaptureSession, init, AudioFormat } = require('cap-electron-capture');

class TranscriptionManager {
  constructor() {
    this.session = null;
    this.isActive = false;
    this.audioBuffer = [];
  }

  async start(config = {}) {
    if (this.isActive) {
      throw new Error('Already recording');
    }

    console.log('Starting transcription with config:', config);

    // Check permissions first
    const permissions = await this.checkPermissions();
    if (!permissions.microphone.granted && config.audio?.microphone) {
      throw new Error('Microphone permission required');
    }

    this.session = createCaptureSession({
      audio: {
        enabled: true,
        systemAudio: true,
        microphone: true,
        segmentDurationMs: config.segmentDuration || 1500, // 1.5s for responsiveness
        format: AudioFormat.WAV, // Use WAV for reliability
        sampleRate: 48000,
        channels: 2,
        ...config.audio
      },
      screen: { 
        enabled: false // Focus on audio for transcription
      }
    });

    try {
      const audioStream = await this.session.start();
      this.isActive = true;

      // Real-time audio processing
      audioStream.on('data', (segment) => {
        this.handleAudioSegment(segment);
      });

      audioStream.on('error', (error) => {
        console.error('Audio stream error:', error);
        this.handleError(error);
      });

      console.log('Transcription started successfully');
      return audioStream;
    } catch (error) {
      this.session = null;
      throw error;
    }
  }

  handleAudioSegment(segment) {
    // Convert for IPC transmission
    const segmentData = {
      data: Array.from(segment.data), // Convert for IPC
      sampleRate: segment.sampleRate,
      timestamp: segment.timestamp,
      source: segment.source,
      duration: segment.duration_ms
    };

    // Send to renderer for processing/transcription
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('audio-segment', segmentData);
    }

    // Optional: Buffer audio for batch processing
    this.audioBuffer.push(segmentData);
    
    // Keep buffer size manageable (last 30 seconds)
    const maxBufferTime = 30000; // 30 seconds in ms
    const cutoffTime = Date.now() - maxBufferTime;
    this.audioBuffer = this.audioBuffer.filter(segment => segment.timestamp > cutoffTime);
  }

  handleError(error) {
    console.error('TranscriptionManager error:', error);
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('transcription-error', {
        message: error.message,
        timestamp: Date.now()
      });
    }
  }

  async stop() {
    if (this.session) {
      console.log('Stopping transcription...');
      await this.session.stop();
      this.session = null;
      this.isActive = false;
      this.audioBuffer = [];
      console.log('Transcription stopped');
    }
  }

  async checkPermissions() {
    try {
      const { checkPermissions } = require('cap-electron-capture');
      const permissionsStr = await checkPermissions();
      return JSON.parse(permissionsStr);
    } catch (error) {
      console.error('Failed to check permissions:', error);
      return {
        microphone: { granted: false },
        systemAudio: { granted: false },
        screenRecording: { granted: false }
      };
    }
  }

  async requestPermissions() {
    try {
      const { requestPermissions } = require('cap-electron-capture');
      const permissionsStr = await requestPermissions();
      return JSON.parse(permissionsStr);
    } catch (error) {
      console.error('Failed to request permissions:', error);
      throw error;
    }
  }

  getSystemAudioInstructions() {
    try {
      const { getSystemAudioSetupInstructions } = require('cap-electron-capture');
      return getSystemAudioSetupInstructions();
    } catch (error) {
      console.error('Failed to get system audio instructions:', error);
      return 'System audio setup instructions not available';
    }
  }

  getBufferedAudio() {
    return [...this.audioBuffer]; // Return copy
  }

  clearBuffer() {
    this.audioBuffer = [];
  }
}

let mainWindow;
const transcriptionManager = new TranscriptionManager();

// App lifecycle
app.whenReady().then(async () => {
  // Initialize the capture library
  try {
    const { init } = require('cap-electron-capture');
    const capabilities = await init();
    console.log('Capture library initialized:', JSON.parse(capabilities));
  } catch (error) {
    console.error('Failed to initialize capture library:', error);
  }

  createWindow();
});

app.on('window-all-closed', async () => {
  // Clean shutdown
  await transcriptionManager.stop();
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  });

  mainWindow.loadFile('index.html');

  // Open DevTools in development
  if (process.env.NODE_ENV === 'development') {
    mainWindow.webContents.openDevTools();
  }
}

// IPC handlers
ipcMain.handle('start-transcription', async (event, config) => {
  try {
    await transcriptionManager.start(config);
    return { success: true };
  } catch (error) {
    console.error('Failed to start transcription:', error);
    return { 
      success: false, 
      error: error.message,
      instructions: error.message.includes('system audio') ? 
        transcriptionManager.getSystemAudioInstructions() : null
    };
  }
});

ipcMain.handle('stop-transcription', async () => {
  try {
    await transcriptionManager.stop();
    return { success: true };
  } catch (error) {
    console.error('Failed to stop transcription:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('check-permissions', async () => {
  try {
    const permissions = await transcriptionManager.checkPermissions();
    return { success: true, permissions };
  } catch (error) {
    console.error('Failed to check permissions:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('request-permissions', async () => {
  try {
    const permissions = await transcriptionManager.requestPermissions();
    return { success: true, permissions };
  } catch (error) {
    console.error('Failed to request permissions:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('get-audio-devices', async () => {
  try {
    const { getAudioDevices } = require('cap-electron-capture');
    const devicesStr = await getAudioDevices();
    const devices = JSON.parse(devicesStr);
    return { success: true, devices };
  } catch (error) {
    console.error('Failed to get audio devices:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('get-system-audio-instructions', () => {
  try {
    const instructions = transcriptionManager.getSystemAudioInstructions();
    return { success: true, instructions };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

ipcMain.handle('get-transcription-status', () => {
  return {
    isActive: transcriptionManager.isActive,
    bufferSize: transcriptionManager.audioBuffer.length
  };
});

ipcMain.handle('get-buffered-audio', () => {
  try {
    const bufferedAudio = transcriptionManager.getBufferedAudio();
    return { success: true, audio: bufferedAudio };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

ipcMain.handle('clear-audio-buffer', () => {
  try {
    transcriptionManager.clearBuffer();
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Handle app shutdown gracefully
process.on('SIGINT', async () => {
  console.log('Received SIGINT, shutting down gracefully...');
  await transcriptionManager.stop();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('Received SIGTERM, shutting down gracefully...');
  await transcriptionManager.stop();
  process.exit(0);
});

module.exports = { TranscriptionManager };
