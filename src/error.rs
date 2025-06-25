use thiserror::Error;

/// Result type for capture operations
pub type CaptureResult<T> = Result<T, CaptureError>;

/// Error types for the capture library
#[derive(Error, Debug)]
pub enum CaptureError {
    /// Audio-related errors
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),
    
    /// Screen capture errors
    #[error("Screen capture error: {0}")]
    Screen(#[from] ScreenError),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Platform-specific errors
    #[error("Platform error: {0}")]
    Platform(String),
    
    /// Permission errors (microphone, screen recording, etc.)
    #[error("Permission denied: {0}")]
    Permission(String),
    
    /// Device not found or unavailable
    #[error("Device error: {0}")]
    Device(String),
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// General errors
    #[error("General error: {0}")]
    General(#[from] anyhow::Error),
}

/// Audio-specific errors
#[derive(Error, Debug)]
pub enum AudioError {
    /// Failed to initialize audio system
    #[error("Failed to initialize audio system: {0}")]
    InitializationFailed(String),
    
    /// Audio device not found
    #[error("Audio device not found: {0}")]
    DeviceNotFound(String),
    
    /// Audio format not supported
    #[error("Audio format not supported: {0}")]
    UnsupportedFormat(String),
    
    /// Audio stream error
    #[error("Audio stream error: {0}")]
    StreamError(String),
    
    /// Audio encoding error
    #[error("Audio encoding error: {0}")]
    EncodingError(String),
    
    /// Audio buffer overflow/underflow
    #[error("Audio buffer error: {0}")]
    BufferError(String),
}

/// Screen capture specific errors
#[derive(Error, Debug)]
pub enum ScreenError {
    /// Failed to initialize screen capture
    #[error("Failed to initialize screen capture: {0}")]
    InitializationFailed(String),
    
    /// Display not found
    #[error("Display not found: {0}")]
    DisplayNotFound(String),
    
    /// Window not found
    #[error("Window not found: {0}")]
    WindowNotFound(String),
    
    /// Screen capture permission denied
    #[error("Screen capture permission denied")]
    PermissionDenied,
    
    /// Screen capture format error
    #[error("Screen capture format error: {0}")]
    FormatError(String),
    
    /// Frame capture failed
    #[error("Frame capture failed: {0}")]
    CaptureError(String),
}

impl From<cpal::DevicesError> for CaptureError {
    fn from(err: cpal::DevicesError) -> Self {
        CaptureError::Audio(AudioError::DeviceNotFound(err.to_string()))
    }
}

impl From<cpal::DeviceNameError> for CaptureError {
    fn from(err: cpal::DeviceNameError) -> Self {
        CaptureError::Audio(AudioError::DeviceNotFound(err.to_string()))
    }
}

impl From<cpal::BuildStreamError> for CaptureError {
    fn from(err: cpal::BuildStreamError) -> Self {
        let context = match &err {
            cpal::BuildStreamError::DeviceNotAvailable => "Audio device not available - check if another app is using it",
            cpal::BuildStreamError::InvalidArgument => "Invalid audio configuration - check sample rate and channel count",
            cpal::BuildStreamError::BackendSpecific { err } => {
                &format!("Audio backend error: {}", err.description)
            },
            _ => "Audio stream creation failed"
        };
        CaptureError::Audio(AudioError::StreamError(format!("{}: {}", context, err)))
    }
}

impl From<cpal::PlayStreamError> for CaptureError {
    fn from(err: cpal::PlayStreamError) -> Self {
        CaptureError::Audio(AudioError::StreamError(err.to_string()))
    }
}

impl From<cpal::PauseStreamError> for CaptureError {
    fn from(err: cpal::PauseStreamError) -> Self {
        CaptureError::Audio(AudioError::StreamError(err.to_string()))
    }
}
