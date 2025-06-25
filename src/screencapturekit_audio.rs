use crate::{config::AudioCaptureConfig, error::{CaptureError, CaptureResult}};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Audio segment for real-time transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSegment {
    /// Raw audio data
    pub data: Vec<u8>,
    /// Audio source (microphone, system, etc.)
    pub source: String,
    /// Segment duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp when captured
    pub timestamp: u64,
    /// Audio format (PCM, AAC, etc.)
    pub format: String,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u32,
}

/// Cap-style ScreenCaptureKit integration for macOS system audio
#[cfg(target_os = "macos")]
pub struct ScreenCaptureKitAudio {
    config: AudioCaptureConfig,
    stream: Option<screencapturekit::SCStream>,
    delegate: Option<AudioStreamDelegate>,
}

#[cfg(target_os = "macos")]
impl ScreenCaptureKitAudio {
    pub fn new(config: AudioCaptureConfig) -> CaptureResult<Self> {
        Ok(Self {
            config,
            stream: None,
            delegate: None,
        })
    }

    /// Start ScreenCaptureKit audio capture (Cap's approach)
    pub async fn start_capture(&mut self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<()> {
        use screencapturekit::*;

        // 1. Create SCStreamConfiguration with audio enabled
        let mut config = SCStreamConfiguration::new();
        config.set_captures_audio(true);
        config.set_sample_rate(self.config.sample_rate as i32);
        config.set_channel_count(self.config.channels as i32);
        config.set_excludes_current_process_audio(false);

        // 2. Get available content
        let content = SCShareableContent::current()
            .map_err(|e| CaptureError::Platform(format!("Failed to get shareable content: {}", e)))?;

        // 3. Get primary display
        let displays = content.displays();
        let display = displays.first()
            .ok_or_else(|| CaptureError::Platform("No displays found".to_string()))?;

        // 4. Create content filter for display capture (includes system audio)
        let filter = SCContentFilter::new_with_display_excluding_apps_excepting_windows(
            display,
            &[], // Exclude no apps
            &[]  // Exception windows
        );

        // 5. Create delegate for handling audio callbacks
        let delegate = AudioStreamDelegate::new(tx);
        self.delegate = Some(delegate.clone());

        // 6. Create and start stream
        let stream = SCStream::new(&filter, &config, &delegate)
            .map_err(|e| CaptureError::Platform(format!("Failed to create SCStream: {}", e)))?;

        stream.start_capture()
            .map_err(|e| CaptureError::Platform(format!("Failed to start capture: {}", e)))?;

        self.stream = Some(stream);

        log::info!("ScreenCaptureKit audio capture started");
        Ok(())
    }

    pub async fn stop_capture(&mut self) -> CaptureResult<()> {
        if let Some(stream) = self.stream.take() {
            stream.stop_capture()
                .map_err(|e| CaptureError::Platform(format!("Failed to stop capture: {}", e)))?;
        }
        self.delegate = None;
        log::info!("ScreenCaptureKit audio capture stopped");
        Ok(())
    }
}

/// ScreenCaptureKit stream delegate for audio callbacks
#[cfg(target_os = "macos")]
#[derive(Clone)]
pub struct AudioStreamDelegate {
    audio_sender: mpsc::UnboundedSender<AudioSegment>,
}

#[cfg(target_os = "macos")]
impl AudioStreamDelegate {
    pub fn new(audio_sender: mpsc::UnboundedSender<AudioSegment>) -> Self {
        Self { audio_sender }
    }

    /// Convert CMSampleBuffer to PCM audio data (Cap's approach)
    fn convert_sample_buffer_to_pcm(&self, sample_buffer: &screencapturekit::CMSampleBuffer) -> Vec<u8> {
        // This would contain the actual Core Media sample buffer conversion
        // For now, return mock data
        vec![0u8; 1024] // Mock PCM data
    }
}

#[cfg(target_os = "macos")]
impl screencapturekit::SCStreamDelegate for AudioStreamDelegate {
    fn stream_did_output_sample_buffer(
        &self,
        _stream: &screencapturekit::SCStream,
        sample_buffer: screencapturekit::CMSampleBuffer,
        output_type: screencapturekit::SCStreamOutputType,
    ) {
        // Only process audio samples
        if output_type == screencapturekit::SCStreamOutputType::Audio {
            let audio_data = self.convert_sample_buffer_to_pcm(&sample_buffer);
            
            let segment = AudioSegment {
                data: audio_data,
                source: "system_audio".to_string(),
                duration_ms: 100, // Segment duration
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                format: "PCM".to_string(),
                sample_rate: 48000,
                channels: 2,
            };

            // Send audio segment for processing
            if let Err(_) = self.audio_sender.send(segment) {
                log::warn!("Failed to send audio segment - receiver dropped");
            }
        }
    }
}

/// Simple microphone capture using cpal (for consistency)
pub struct MicrophoneCapture {
    config: AudioCaptureConfig,
}

impl MicrophoneCapture {
    pub fn new(config: AudioCaptureConfig) -> CaptureResult<Self> {
        Ok(Self { config })
    }

    pub async fn start_capture(&mut self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<()> {
        // Simple microphone capture implementation
        // For now, just log that it would start
        log::info!("Microphone capture started (mock implementation)");
        
        // In a real implementation, this would:
        // 1. Use cpal to get default input device
        // 2. Create input stream with callback
        // 3. Process audio data in segments
        // 4. Send AudioSegments through tx
        
        Ok(())
    }

    pub async fn stop_capture(&mut self) -> CaptureResult<()> {
        log::info!("Microphone capture stopped");
        Ok(())
    }
}
