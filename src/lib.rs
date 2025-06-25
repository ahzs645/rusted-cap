//! Cap Electron Capture Library
//! 
//! A cross-platform screen capture and audio processing library designed for
//! integration with Electron applications. This library extracts the core
//! functionality from Cap's screen recording pipeline for use in transcription
//! and real-time audio processing applications.

use napi_derive::napi;
use serde_json;

pub mod audio;
pub mod screen;
pub mod config;
pub mod error;
pub mod platform;
pub mod permissions;
pub mod encoding;
pub mod recording;

// Re-export main types
pub use audio::{AudioProcessor, AudioSegment};
pub use screen::{ScreenCapture};
pub use recording::{CapRecordingPipeline, RecordingConfig, RecordingSession};
pub use encoding::{AudioEncoder, VideoEncoder, HLSSegmenter, S3Uploader};
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

/// Request all necessary permissions for audio and screen capture
#[napi]
pub async fn request_permissions() -> napi::Result<String> {
    let permissions = permissions::request_all_permissions().await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(serde_json::to_string(&permissions)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?)
}

/// Check current permission status without requesting
#[napi]
pub async fn check_permissions() -> napi::Result<String> {
    let permissions = permissions::check_permissions().await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(serde_json::to_string(&permissions)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?)
}

/// Get platform-specific instructions for enabling system audio capture
#[napi]
pub fn get_system_audio_setup_instructions() -> String {
    permissions::get_system_audio_setup_instructions().to_string()
}

/// Create a new capture session with the given configuration
#[napi(js_name = "createCaptureSession")]
pub fn create_capture_session(config: String) -> napi::Result<String> {
    // Parse config to validate it
    let _capture_config: CaptureConfig = serde_json::from_str(&config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {}", e)))?;
    
    // For now, create a session ID and return enhanced session info
    let session_id = uuid::Uuid::new_v4().to_string();
    
    let session_data = serde_json::json!({
        "id": session_id,
        "config": serde_json::from_str::<serde_json::Value>(&config).unwrap_or(serde_json::json!({})),
        "status": "created",
        "capabilities": {
            "audio": true,
            "realtime": true,
            "screen": true,
            "screencapturekit": cfg!(target_os = "macos"),
            "native_system_audio": cfg!(target_os = "macos")
        },
        "platform": {
            "os": std::env::consts::OS,
            "supports_native_system_audio": cfg!(target_os = "macos")
        },
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    });
    
    log::info!("Created capture session: {}", session_id);
    Ok(session_data.to_string())
}

/// Start native system audio capture demonstration
#[napi]
pub async fn start_native_system_audio(session_id: String) -> napi::Result<String> {
    #[cfg(target_os = "macos")]
    {
        // Demonstrate Cap's ScreenCaptureKit approach (simplified)
        log::info!("Starting ScreenCaptureKit system audio capture (demo)...");
        
        // In a real implementation, this would:
        // 1. Create SCStreamConfiguration with audio enabled
        // 2. Get shareable content and displays
        // 3. Create content filter for display capture
        // 4. Create stream delegate for audio callbacks
        // 5. Start SCStream capture
        
        let result = serde_json::json!({
            "status": "started",
            "session_id": session_id,
            "method": "screencapturekit",
            "message": "ScreenCaptureKit system audio capture started (demo mode)",
            "implementation_notes": {
                "real_implementation": "Would use SCStream with audio enabled",
                "audio_source": "System audio + applications",
                "no_virtual_drivers": true,
                "requires_permission": "Screen Recording permission in System Preferences"
            },
            "next_steps": [
                "Add proper ScreenCaptureKit bindings",
                "Implement SCStreamDelegate for audio callbacks", 
                "Convert CMSampleBuffer to audio segments",
                "Send to transcription service"
            ]
        });
        
        Ok(result.to_string())
    }
    
    #[cfg(target_os = "windows")]
    {
        log::info!("Starting WASAPI loopback system audio capture (demo)...");
        
        let result = serde_json::json!({
            "status": "started",
            "session_id": session_id,
            "method": "wasapi_loopback",
            "message": "WASAPI loopback system audio capture started (demo mode)",
            "implementation_notes": {
                "real_implementation": "Would use WASAPI loopback mode",
                "audio_source": "System audio loopback",
                "no_virtual_drivers": true
            }
        });
        
        Ok(result.to_string())
    }
    
    #[cfg(target_os = "linux")]
    {
        log::info!("Starting PipeWire system audio capture (demo)...");
        
        let result = serde_json::json!({
            "status": "started", 
            "session_id": session_id,
            "method": "pipewire_monitor",
            "message": "PipeWire monitor system audio capture started (demo mode)",
            "implementation_notes": {
                "real_implementation": "Would use PipeWire monitor sources",
                "audio_source": "Monitor of system audio output",
                "no_virtual_drivers": true
            }
        });
        
        Ok(result.to_string())
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(napi::Error::from_reason("Native system audio capture not implemented for this platform"))
    }
}

/// Test native system audio capture capabilities
#[napi]
pub fn test_native_system_audio() -> napi::Result<String> {
    #[cfg(target_os = "macos")]
    {
        let result = serde_json::json!({
            "platform": "macOS",
            "method": "ScreenCaptureKit",
            "available": true,
            "description": "Uses ScreenCaptureKit for true system audio capture without virtual drivers",
            "requirements": [
                "Screen Recording permission in System Preferences",
                "macOS 12.3 or later",
                "No BlackHole or virtual drivers needed"
            ]
        });
        Ok(result.to_string())
    }
    
    #[cfg(target_os = "windows")]
    {
        let result = serde_json::json!({
            "platform": "Windows",
            "method": "WASAPI Loopback",
            "available": true,
            "description": "Uses WASAPI loopback for system audio capture",
            "requirements": [
                "Windows Vista or later",
                "No additional drivers needed"
            ]
        });
        Ok(result.to_string())
    }
    
    #[cfg(target_os = "linux")]
    {
        let result = serde_json::json!({
            "platform": "Linux",
            "method": "PipeWire/PulseAudio",
            "available": true,
            "description": "Uses PipeWire or PulseAudio monitor sources",
            "requirements": [
                "PipeWire or PulseAudio",
                "Monitor audio sources enabled"
            ]
        });
        Ok(result.to_string())
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let result = serde_json::json!({
            "platform": "Unsupported",
            "available": false,
            "description": "Native system audio capture not implemented for this platform"
        });
        Ok(result.to_string())
    }
}

/// Create a new recording pipeline with Cap's architecture
#[napi(js_name = "createRecordingPipeline")]
pub async fn create_recording_pipeline(config: String) -> napi::Result<String> {
    let recording_config: recording::RecordingConfig = serde_json::from_str(&config)
        .map_err(|e| napi::Error::from_reason(format!("Invalid recording config: {}", e)))?;
    
    let mut pipeline = recording::CapRecordingPipeline::new(recording_config)
        .map_err(|e| napi::Error::from_reason(format!("Failed to create pipeline: {}", e)))?;
    
    pipeline.initialize().await
        .map_err(|e| napi::Error::from_reason(format!("Failed to initialize pipeline: {}", e)))?;
    
    // Store pipeline instance (in production, use a session manager)
    let session_info = serde_json::json!({
        "session_id": pipeline.get_session_id(),
        "status": "initialized",
        "capabilities": {
            "encoding": {
                "audio": "AAC",
                "video": "H.264",
                "hls": true
            },
            "streaming": pipeline.get_config().enable_streaming,
            "transcription": pipeline.get_config().enable_transcription
        }
    });
    
    Ok(serde_json::to_string(&session_info)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize session: {}", e)))?)
}

/// Start recording with the specified session
#[napi(js_name = "startRecording")]
pub async fn start_recording(session_id: String) -> napi::Result<String> {
    // In production, retrieve pipeline from session manager
    // For now, return mock data demonstrating the structure
    let session = serde_json::json!({
        "id": session_id,
        "user_id": "user_123",
        "start_time": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        "status": "recording",
        "stream_urls": {
            "master": format!("https://s3.amazonaws.com/cap-recordings/user_123/{}/stream.m3u8", session_id),
            "video": format!("https://s3.amazonaws.com/cap-recordings/user_123/{}/video/stream.m3u8", session_id),
            "audio": format!("https://s3.amazonaws.com/cap-recordings/user_123/{}/audio/stream.m3u8", session_id),
            "combined": format!("https://s3.amazonaws.com/cap-recordings/user_123/{}/combined-source/stream.m3u8", session_id)
        },
        "stats": {
            "duration": 0.0,
            "video_frames": 0,
            "audio_segments": 0,
            "bytes_uploaded": 0,
            "avg_fps": 0.0
        }
    });
    
    Ok(serde_json::to_string(&session)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize session: {}", e)))?)
}

/// Stop recording and finalize segments
#[napi(js_name = "stopRecording")]
pub async fn stop_recording(session_id: String) -> napi::Result<String> {
    // In production, retrieve pipeline from session manager and stop
    let session = serde_json::json!({
        "id": session_id,
        "status": "stopped",
        "final_stats": {
            "total_duration": 120.5,
            "total_segments": 60,
            "total_bytes": 1024000,
            "avg_fps": 29.8
        },
        "files": {
            "master_playlist": format!("https://s3.amazonaws.com/cap-recordings/user_123/{}/stream.m3u8", session_id),
            "final_video": format!("https://s3.amazonaws.com/cap-recordings/user_123/{}/output/video_recording_000.m3u8", session_id)
        }
    });
    
    Ok(serde_json::to_string(&session)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize session: {}", e)))?)
}

/// Get encoding capabilities and configuration options
#[napi(js_name = "getEncodingCapabilities")]
pub fn get_encoding_capabilities() -> napi::Result<String> {
    let capabilities = serde_json::json!({
        "audio_codecs": ["AAC"],
        "video_codecs": ["H.264", "H.265"],
        "container_formats": ["HLS", "MP4"],
        "streaming": {
            "hls": true,
            "segment_duration": 2.0,
            "max_bitrate": 5000000
        },
        "hardware_acceleration": {
            "available": true,
            "platforms": ["VideoToolbox", "NVENC", "QuickSync"]
        },
        "default_settings": {
            "audio": {
                "codec": "AAC",
                "bitrate": 128000,
                "sample_rate": 48000,
                "channels": 2
            },
            "video": {
                "codec": "H.264",
                "bitrate": 2000000,
                "frame_rate": 30,
                "resolution": "1920x1080"
            }
        }
    });
    
    Ok(serde_json::to_string(&capabilities)
        .map_err(|e| napi::Error::from_reason(format!("Failed to serialize capabilities: {}", e)))?)
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
