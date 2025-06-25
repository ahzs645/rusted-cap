//! Cap Electron Capture Library
//! 
//! A cross-platform screen capture and audio processing library designed for
//! integration with Electron applications. This library extracts the core
//! functionality from Cap's screen recording pipeline for use in transcription
//! and real-time audio processing applications.

use napi_derive::napi;

pub mod audio;
pub mod screen;
pub mod config;
pub mod error;
pub mod platform;

// Re-export main types
pub use audio::{AudioProcessor, AudioSegment};
pub use screen::{ScreenCapture};
pub use config::{CaptureConfig, OutputFormat, AudioCaptureConfig, ScreenCaptureConfig};
pub use error::{CaptureError, CaptureResult};

/// Initialize the library and check platform capabilities
#[napi]
pub fn init() -> napi::Result<String> {
    env_logger::init();
    log::info!("Cap Electron Capture library initialized");
    
    let capabilities = platform::get_platform_capabilities();
    Ok(serde_json::to_string(&capabilities)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize capabilities: {}", e)))?)
}

/// Get information about available audio devices
#[napi]
pub fn get_audio_devices() -> napi::Result<String> {
    let devices = audio::get_available_devices()
        .map_err(|e| napi::Error::from_reason(format!("Failed to get audio devices: {}", e)))?;
    
    Ok(serde_json::to_string(&devices)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize devices: {}", e)))?)
}

/// Get information about available displays for screen capture
#[napi]
pub fn get_displays() -> napi::Result<String> {
    let displays = screen::get_available_displays()
        .map_err(|e| napi::Error::from_reason(format!("Failed to get displays: {}", e)))?;
    
    Ok(serde_json::to_string(&displays)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize displays: {}", e)))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_init() {
        let result = init();
        assert!(result.is_ok());
    }
}
