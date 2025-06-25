const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { createCaptureSession, init, getAudioDevices, AudioFormat } = require('../index.js');

let mainWindow;
let captureSession;

// Initialize the library
console.log('🚀 Initializing Cap Electron Capture...');
const capabilities = init();
console.log('Platform capabilities:', capabilities);

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

  // Load the app
  mainWindow.loadFile(path.join(__dirname, 'index.html'));

  // Open DevTools in development
  if (process.env.NODE_ENV === 'development') {
    mainWindow.webContents.openDevTools();
  }
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// IPC Handlers for transcription functionality

ipcMain.handle('get-platform-info', async () => {
  try {
    return {
      success: true,
      data: {
        capabilities: init(),
        audioDevices: getAudioDevices()
      }
    };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

ipcMain.handle('start-transcription', async (event, config) => {
  try {
    if (captureSession) {
      return { success: false, error: 'Transcription is already running' };
    }

    // Create audio-only capture session for transcription
    captureSession = createCaptureSession({
      audio: {
        enabled: true,
        systemAudio: config?.systemAudio ?? true,
        microphone: config?.microphone ?? true,
        segmentDurationMs: config?.segmentDuration ?? 2000,
        format: AudioFormat.AAC
      },
      screen: {
        enabled: false // Audio-only for transcription
      }
    });

    await captureSession.start();
    console.log('✅ Transcription session started');

    // In a real implementation, you would process audio segments here
    // and send transcription results back to the renderer
    
    return { success: true };
  } catch (error) {
    console.error('❌ Failed to start transcription:', error);
    captureSession = null;
    return { success: false, error: error.message };
  }
});

ipcMain.handle('stop-transcription', async () => {
  try {
    if (!captureSession) {
      return { success: false, error: 'No transcription session running' };
    }

    await captureSession.stop();
    captureSession = null;
    console.log('✅ Transcription session stopped');

    return { success: true };
  } catch (error) {
    console.error('❌ Failed to stop transcription:', error);
    return { success: false, error: error.message };
  }
});

ipcMain.handle('get-transcription-status', async () => {
  try {
    const isActive = captureSession ? await captureSession.isActive() : false;
    return { success: true, isActive };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Handle app termination
app.on('before-quit', async () => {
  if (captureSession) {
    try {
      await captureSession.stop();
      console.log('🛑 Stopped transcription session on app quit');
    } catch (error) {
      console.error('Error stopping transcription on quit:', error);
    }
  }
});
