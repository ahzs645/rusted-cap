use crate::CaptureResult;
use crate::screencapturekit::is_screencapturekit_available;
use cpal::traits::HostTrait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Platform capabilities information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Operating system
    pub platform: Platform,
    /// Audio capture capabilities
    pub audio: AudioCapabilities,
    /// Screen capture capabilities
    pub screen: ScreenCapabilities,
    /// System version
    pub system_version: String,
    /// Available permissions
    pub permissions: PermissionStatus,
    /// ScreenCaptureKit availability (macOS only)
    pub screencapturekit: bool,
}

/// Platform types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    Unknown,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::MacOS => write!(f, "macOS"),
            Platform::Windows => write!(f, "Windows"),
            Platform::Linux => write!(f, "Linux"),
            Platform::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Audio capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCapabilities {
    /// System audio capture supported
    pub system_audio: bool,
    /// Microphone capture supported
    pub microphone: bool,
    /// Available sample rates
    pub sample_rates: Vec<u32>,
    /// Supported audio formats
    pub formats: Vec<String>,
    /// Number of available input devices
    pub input_devices: usize,
    /// Number of available output devices
    pub output_devices: usize,
}

/// Screen capture capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCapabilities {
    /// Screen capture supported
    pub supported: bool,
    /// Number of available displays
    pub display_count: usize,
    /// Window capture supported
    pub window_capture: bool,
    /// Maximum resolution supported
    pub max_resolution: Option<(u32, u32)>,
    /// Supported frame rates
    pub frame_rates: Vec<u32>,
}

/// Permission status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    /// Microphone permission
    pub microphone: Permission,
    /// Screen recording permission
    pub screen_recording: Permission,
    /// System audio permission (macOS only)
    pub system_audio: Permission,
}

/// Permission state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Granted,
    Denied,
    NotDetermined,
    NotRequired,
}

/// Get platform capabilities
pub fn get_platform_capabilities() -> PlatformCapabilities {
    let platform = detect_platform();
    
    PlatformCapabilities {
        audio: get_audio_capabilities(),
        screen: get_screen_capabilities(),
        system_version: get_system_version(),
        permissions: get_permission_status(),
        screencapturekit: is_screencapturekit_available(),
        platform,
    }
}

/// Detect the current platform
fn detect_platform() -> Platform {
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
    
    #[cfg(target_os = "windows")]
    return Platform::Windows;
    
    #[cfg(target_os = "linux")]
    return Platform::Linux;
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return Platform::Unknown;
}

/// Get audio capabilities for the current platform
fn get_audio_capabilities() -> AudioCapabilities {
    // Try to enumerate audio devices using cpal
    let host = cpal::default_host();
    
    let input_devices = host.input_devices()
        .map(|devices| devices.count())
        .unwrap_or(0);
    
    let output_devices = host.output_devices()
        .map(|devices| devices.count())
        .unwrap_or(0);
    
    AudioCapabilities {
        system_audio: supports_system_audio(),
        microphone: input_devices > 0,
        sample_rates: vec![44100, 48000, 96000],
        formats: vec!["PCM".to_string(), "AAC".to_string(), "MP3".to_string()],
        input_devices,
        output_devices,
    }
}

/// Get screen capture capabilities for the current platform
fn get_screen_capabilities() -> ScreenCapabilities {
    ScreenCapabilities {
        supported: supports_screen_capture(),
        display_count: get_display_count(),
        window_capture: supports_window_capture(),
        max_resolution: get_max_resolution(),
        frame_rates: vec![15, 30, 60],
    }
}

/// Get system version string
fn get_system_version() -> String {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
    
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string() // Could use WinAPI to get detailed version
    }
    
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .map(|line| line.split('=').nth(1).unwrap_or("Unknown").trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "Linux".to_string())
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "Unknown".to_string()
    }
}

/// Get current permission status
fn get_permission_status() -> PermissionStatus {
    PermissionStatus {
        microphone: check_microphone_permission(),
        screen_recording: check_screen_recording_permission(),
        system_audio: check_system_audio_permission(),
    }
}

/// Check if system audio capture is supported
fn supports_system_audio() -> bool {
    #[cfg(target_os = "macos")]
    return true; // ScreenCaptureKit supports system audio
    
    #[cfg(target_os = "windows")]
    return true; // WASAPI supports loopback
    
    #[cfg(target_os = "linux")]
    return true; // PipeWire/PulseAudio support
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return false;
}

/// Check if screen capture is supported
fn supports_screen_capture() -> bool {
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    return true;
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return false;
}

/// Check if window capture is supported
fn supports_window_capture() -> bool {
    supports_screen_capture() // Same platforms generally support both
}

/// Get number of displays
fn get_display_count() -> usize {
    // This would need platform-specific implementation
    // For now, return 1 as a default
    1
}

/// Get maximum resolution
fn get_max_resolution() -> Option<(u32, u32)> {
    // This would need platform-specific implementation
    // For now, return common 4K resolution
    Some((3840, 2160))
}

/// Check microphone permission
fn check_microphone_permission() -> Permission {
    // This would need platform-specific implementation
    Permission::NotDetermined
}

/// Check screen recording permission
fn check_screen_recording_permission() -> Permission {
    // This would need platform-specific implementation
    Permission::NotDetermined
}

/// Check system audio permission (mainly for macOS)
fn check_system_audio_permission() -> Permission {
    #[cfg(target_os = "macos")]
    return Permission::NotDetermined; // Would need CoreAudio/ScreenCaptureKit check
    
    #[cfg(not(target_os = "macos"))]
    return Permission::NotRequired;
}

/// Request microphone permission
pub async fn request_microphone_permission() -> CaptureResult<bool> {
    // Platform-specific permission request implementation
    #[cfg(target_os = "macos")]
    {
        // Would use AVAudioSession or similar
        Ok(true)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true) // Other platforms typically don't require explicit permission requests
    }
}

/// Request screen recording permission
pub async fn request_screen_recording_permission() -> CaptureResult<bool> {
    // Platform-specific permission request implementation
    #[cfg(target_os = "macos")]
    {
        // Would use ScreenCaptureKit permission request
        Ok(true)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true) // Other platforms typically don't require explicit permission requests
    }
}
