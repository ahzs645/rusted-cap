const { contextBridge, ipcRenderer } = require('electron');

// Expose safe APIs to the renderer process
contextBridge.exposeInMainWorld('electronAPI', {
  // Platform and device information
  getPlatformInfo: () => ipcRenderer.invoke('get-platform-info'),
  
  // Transcription controls
  startTranscription: (config) => ipcRenderer.invoke('start-transcription', config),
  stopTranscription: () => ipcRenderer.invoke('stop-transcription'),
  getTranscriptionStatus: () => ipcRenderer.invoke('get-transcription-status'),
  
  // Event listeners for transcription results (if implemented)
  onTranscriptionResult: (callback) => {
    ipcRenderer.on('transcription-result', (event, data) => callback(data));
  },
  
  onTranscriptionError: (callback) => {
    ipcRenderer.on('transcription-error', (event, error) => callback(error));
  }
});
