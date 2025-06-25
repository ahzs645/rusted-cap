// Renderer process script for the Electron transcription demo

let isRecording = false;
let platformInfo = null;

// DOM elements
const startBtn = document.getElementById('start-btn');
const stopBtn = document.getElementById('stop-btn');
const refreshBtn = document.getElementById('refresh-btn');
const statusArea = document.getElementById('status-area');
const transcriptionOutput = document.getElementById('transcription-output');
const systemAudioCheckbox = document.getElementById('system-audio');
const microphoneCheckbox = document.getElementById('microphone');
const segmentDurationInput = document.getElementById('segment-duration');
const platformInfoContainer = document.getElementById('platform-info');

// Initialize the app
async function initialize() {
    await loadPlatformInfo();
    setupEventListeners();
    updateUI();
}

// Load platform and device information
async function loadPlatformInfo() {
    try {
        showStatus('Loading platform information...', 'info');
        const result = await window.electronAPI.getPlatformInfo();
        
        if (result.success) {
            platformInfo = result.data;
            displayPlatformInfo();
            showStatus('Platform information loaded successfully', 'success');
        } else {
            throw new Error(result.error);
        }
    } catch (error) {
        console.error('Failed to load platform info:', error);
        showStatus(`Failed to load platform info: ${error.message}`, 'error');
    }
}

// Display platform information in the UI
function displayPlatformInfo() {
    if (!platformInfo) return;
    
    const { capabilities, audioDevices } = platformInfo;
    
    platformInfoContainer.innerHTML = `
        <div class="info-card">
            <h3>🖥️ Platform</h3>
            <p><strong>OS:</strong> ${capabilities.platform}</p>
            <p><strong>Version:</strong> ${capabilities.systemVersion}</p>
        </div>
        
        <div class="info-card">
            <h3>🎵 Audio Capabilities</h3>
            <p><strong>System Audio:</strong> ${capabilities.audio.systemAudio ? '✅' : '❌'}</p>
            <p><strong>Microphone:</strong> ${capabilities.audio.microphone ? '✅' : '❌'}</p>
            <p><strong>Input Devices:</strong> ${capabilities.audio.inputDevices}</p>
            <p><strong>Output Devices:</strong> ${capabilities.audio.outputDevices}</p>
        </div>
        
        <div class="info-card">
            <h3>🖥️ Screen Capabilities</h3>
            <p><strong>Supported:</strong> ${capabilities.screen.supported ? '✅' : '❌'}</p>
            <p><strong>Displays:</strong> ${capabilities.screen.displayCount}</p>
            <p><strong>Window Capture:</strong> ${capabilities.screen.windowCapture ? '✅' : '❌'}</p>
        </div>
        
        <div class="info-card">
            <h3>🎤 Audio Devices</h3>
            <ul class="device-list">
                ${audioDevices.map(device => 
                    `<li>${device.name} (${device.deviceType}) ${device.isDefault ? '(default)' : ''}</li>`
                ).join('')}
            </ul>
        </div>
        
        <div class="info-card">
            <h3>🔐 Permissions</h3>
            <p><strong>Microphone:</strong> ${capabilities.permissions.microphone}</p>
            <p><strong>Screen Recording:</strong> ${capabilities.permissions.screenRecording}</p>
            <p><strong>System Audio:</strong> ${capabilities.permissions.systemAudio}</p>
        </div>
    `;
}

// Setup event listeners
function setupEventListeners() {
    startBtn.addEventListener('click', startTranscription);
    stopBtn.addEventListener('click', stopTranscription);
    refreshBtn.addEventListener('click', () => {
        loadPlatformInfo();
    });

    // Listen for transcription results (if implemented)
    window.electronAPI.onTranscriptionResult((data) => {
        appendTranscriptionResult(data);
    });

    window.electronAPI.onTranscriptionError((error) => {
        showStatus(`Transcription error: ${error.message}`, 'error');
    });
}

// Start transcription
async function startTranscription() {
    try {
        const config = {
            systemAudio: systemAudioCheckbox.checked,
            microphone: microphoneCheckbox.checked,
            segmentDuration: parseInt(segmentDurationInput.value)
        };

        if (!config.systemAudio && !config.microphone) {
            showStatus('Please select at least one audio source (system audio or microphone)', 'error');
            return;
        }

        showStatus('Starting transcription...', 'info');
        const result = await window.electronAPI.startTranscription(config);

        if (result.success) {
            isRecording = true;
            updateUI();
            showStatus(
                `<div class="recording-indicator">
                    <span class="recording-dot"></span>
                    Recording in progress
                </div>`,
                'success'
            );
            appendTranscriptionResult({
                text: `🎤 Transcription started with config: ${JSON.stringify(config, null, 2)}`,
                timestamp: new Date().toISOString(),
                type: 'system'
            });
        } else {
            throw new Error(result.error);
        }
    } catch (error) {
        console.error('Failed to start transcription:', error);
        showStatus(`Failed to start transcription: ${error.message}`, 'error');
    }
}

// Stop transcription
async function stopTranscription() {
    try {
        showStatus('Stopping transcription...', 'info');
        const result = await window.electronAPI.stopTranscription();

        if (result.success) {
            isRecording = false;
            updateUI();
            showStatus('Transcription stopped successfully', 'success');
            appendTranscriptionResult({
                text: '🛑 Transcription stopped',
                timestamp: new Date().toISOString(),
                type: 'system'
            });
        } else {
            throw new Error(result.error);
        }
    } catch (error) {
        console.error('Failed to stop transcription:', error);
        showStatus(`Failed to stop transcription: ${error.message}`, 'error');
    }
}

// Update UI based on recording state
function updateUI() {
    startBtn.disabled = isRecording;
    stopBtn.disabled = !isRecording;
    
    // Disable config controls while recording
    systemAudioCheckbox.disabled = isRecording;
    microphoneCheckbox.disabled = isRecording;
    segmentDurationInput.disabled = isRecording;
}

// Show status message
function showStatus(message, type = 'info') {
    statusArea.innerHTML = `<div class="status ${type}">${message}</div>`;
    
    // Auto-clear non-error status after 5 seconds
    if (type !== 'error') {
        setTimeout(() => {
            if (statusArea.innerHTML.includes(message)) {
                statusArea.innerHTML = '';
            }
        }, 5000);
    }
}

// Append transcription result to output
function appendTranscriptionResult(data) {
    const timestamp = new Date(data.timestamp).toLocaleTimeString();
    const prefix = data.type === 'system' ? '[SYSTEM]' : `[${data.source || 'AUDIO'}]`;
    
    const resultLine = `${timestamp} ${prefix} ${data.text}\n`;
    transcriptionOutput.textContent += resultLine;
    
    // Auto-scroll to bottom
    transcriptionOutput.scrollTop = transcriptionOutput.scrollHeight;
}

// Check transcription status periodically
async function checkStatus() {
    try {
        const result = await window.electronAPI.getTranscriptionStatus();
        if (result.success && result.isActive !== isRecording) {
            isRecording = result.isActive;
            updateUI();
        }
    } catch (error) {
        console.error('Failed to check status:', error);
    }
}

// Initialize the app when DOM is loaded
document.addEventListener('DOMContentLoaded', initialize);

// Check status every 2 seconds
setInterval(checkStatus, 2000);
