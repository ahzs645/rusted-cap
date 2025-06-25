use serde::{Deserialize, Serialize};
use napi_derive::napi;

/// Main configuration for the capture session
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Audio capture configuration
    pub audio: AudioCaptureConfig,
    /// Screen capture configuration 
    pub screen: ScreenCaptureConfig,
    /// Output format settings
    pub output: OutputFormat,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            audio: AudioCaptureConfig::default(),
            screen: ScreenCaptureConfig::default(),
            output: OutputFormat::default(),
        }
    }
}

/// Audio capture configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCaptureConfig {
    /// Enable audio capture
    pub enabled: bool,
    /// Capture system audio (computer output)
    pub system_audio: bool,
    /// Capture microphone input
    pub microphone: bool,
    /// Audio sample rate (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Audio channels (1 for mono, 2 for stereo)
    pub channels: u16,
    /// Segment duration in milliseconds for real-time processing
    pub segment_duration_ms: u32,
    /// Audio device ID for microphone (None for default)
    pub microphone_device_id: Option<String>,
    /// Audio format for output
    pub format: AudioFormat,
}

impl Default for AudioCaptureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            system_audio: true,
            microphone: true,
            sample_rate: 44100,
            channels: 2,
            segment_duration_ms: 2000, // 2 seconds for transcription
            microphone_device_id: None,
            format: AudioFormat::Aac,
        }
    }
}

/// Screen capture configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCaptureConfig {
    /// Enable screen capture
    pub enabled: bool,
    /// Display ID to capture (None for primary display)
    pub display_id: Option<u32>,
    /// Capture frame rate
    pub fps: u32,
    /// Capture quality (0-100)
    pub quality: u8,
    /// Include cursor in capture
    pub include_cursor: bool,
    /// Capture specific window ID (None for full screen)
    pub window_id: Option<i64>,
}

impl Default for ScreenCaptureConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Default to audio-only for transcription
            display_id: None,
            fps: 30,
            quality: 80,
            include_cursor: true,
            window_id: None,
        }
    }
}

/// Output format configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    /// Audio output format
    pub audio: AudioFormat,
    /// Video output format (if screen capture is enabled)
    pub video: VideoFormat,
    /// Output directory for segments
    pub output_dir: Option<String>,
    /// Enable real-time streaming
    pub real_time: bool,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self {
            audio: AudioFormat::Aac,
            video: VideoFormat::Mp4,
            output_dir: None,
            real_time: true,
        }
    }
}

/// Supported audio formats
#[napi]
#[derive(Debug, Serialize, Deserialize)]
pub enum AudioFormat {
    /// AAC audio format (recommended for transcription)
    Aac,
    /// MP3 audio format
    Mp3,
    /// WAV audio format
    Wav,
    /// Raw PCM data
    Raw,
}

/// Supported video formats
#[napi]
#[derive(Debug, Serialize, Deserialize)]
pub enum VideoFormat {
    /// MP4 container with H.264
    Mp4,
    /// WebM container
    WebM,
    /// Raw frames
    Raw,
}
