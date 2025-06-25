use crate::error::{CaptureResult};
use cpal::traits::{HostTrait, DeviceTrait};
use serde::{Deserialize, Serialize};

/// Permission status for different capture types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    /// Microphone permission status
    pub microphone: PermissionState,
    /// Screen recording permission status
    pub screen_recording: PermissionState,
    /// System audio permission status (varies by platform)
    pub system_audio: PermissionState,
}

/// Individual permission state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionState {
    /// Permission granted
    Granted,
    /// Permission denied
    Denied,
    /// Permission not yet requested
    NotRequested,
    /// Permission request in progress
    Requesting,
    /// Permission not applicable on this platform
    NotApplicable,
}

/// Request all necessary permissions for audio and screen capture
pub async fn request_all_permissions() -> CaptureResult<PermissionStatus> {
    let microphone = request_microphone_permission().await?;
    let screen_recording = request_screen_recording_permission().await?;
    let system_audio = request_system_audio_permission().await?;

    Ok(PermissionStatus {
        microphone,
        screen_recording,
        system_audio,
    })
}

/// Check current permission status without requesting
pub async fn check_permissions() -> CaptureResult<PermissionStatus> {
    let microphone = check_microphone_permission().await?;
    let screen_recording = check_screen_recording_permission().await?;
    let system_audio = check_system_audio_permission().await?;

    Ok(PermissionStatus {
        microphone,
        screen_recording,
        system_audio,
    })
}

/// Platform-specific microphone permission handling
async fn request_microphone_permission() -> CaptureResult<PermissionState> {
    #[cfg(target_os = "macos")]
    {
        macos_request_microphone_permission().await
    }

    #[cfg(target_os = "windows")]
    {
        windows_request_microphone_permission().await
    }

    #[cfg(target_os = "linux")]
    {
        linux_request_microphone_permission().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(PermissionState::NotApplicable)
    }
}

async fn check_microphone_permission() -> CaptureResult<PermissionState> {
    // Try to enumerate audio devices as a basic permission check
    match cpal::default_host().default_input_device() {
        Some(_) => Ok(PermissionState::Granted),
        None => Ok(PermissionState::Denied),
    }
}

/// Platform-specific screen recording permission handling
async fn request_screen_recording_permission() -> CaptureResult<PermissionState> {
    #[cfg(target_os = "macos")]
    {
        macos_request_screen_recording_permission().await
    }

    #[cfg(target_os = "windows")]
    {
        // Windows generally doesn't require explicit permission for screen capture
        Ok(PermissionState::Granted)
    }

    #[cfg(target_os = "linux")]
    {
        // Linux permission depends on the capture method (X11, Wayland, etc.)
        Ok(PermissionState::Granted)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(PermissionState::NotApplicable)
    }
}

async fn check_screen_recording_permission() -> CaptureResult<PermissionState> {
    // Basic check - this would need more sophisticated platform-specific checks
    Ok(PermissionState::NotRequested)
}

/// Platform-specific system audio permission handling
async fn request_system_audio_permission() -> CaptureResult<PermissionState> {
    #[cfg(target_os = "macos")]
    {
        macos_request_system_audio_permission().await
    }

    #[cfg(target_os = "windows")]
    {
        windows_request_system_audio_permission().await
    }

    #[cfg(target_os = "linux")]
    {
        linux_request_system_audio_permission().await
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(PermissionState::NotApplicable)
    }
}

async fn check_system_audio_permission() -> CaptureResult<PermissionState> {
    // Check if system audio devices are available
    let host = cpal::default_host();
    
    #[cfg(target_os = "macos")]
    {
        // Look for virtual audio devices or ScreenCaptureKit availability
        if let Ok(mut devices) = host.output_devices() {
            let has_virtual_device = devices.any(|device| {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    name_lower.contains("blackhole") || 
                    name_lower.contains("soundflower") ||
                    name_lower.contains("virtual")
                } else {
                    false
                }
            });
            
            if has_virtual_device {
                Ok(PermissionState::Granted)
            } else {
                Ok(PermissionState::Denied)
            }
        } else {
            Ok(PermissionState::Denied)
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Look for stereo mix or loopback devices
        if let Ok(mut devices) = host.input_devices() {
            let has_loopback = devices.any(|device| {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    name_lower.contains("stereo mix") || 
                    name_lower.contains("what u hear") ||
                    name_lower.contains("loopback")
                } else {
                    false
                }
            });
            
            if has_loopback {
                Ok(PermissionState::Granted)
            } else {
                Ok(PermissionState::Denied)
            }
        } else {
            Ok(PermissionState::Denied)
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Look for monitor devices
        if let Ok(mut devices) = host.input_devices() {
            let has_monitor = devices.any(|device| {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    name_lower.contains("monitor") || 
                    name_lower.contains("output") ||
                    name_lower.contains("sink")
                } else {
                    false
                }
            });
            
            if has_monitor {
                Ok(PermissionState::Granted)
            } else {
                Ok(PermissionState::Denied)
            }
        } else {
            Ok(PermissionState::Denied)
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(PermissionState::NotApplicable)
    }
}

// macOS-specific permission implementations
#[cfg(target_os = "macos")]
async fn macos_request_microphone_permission() -> CaptureResult<PermissionState> {
    // On macOS, microphone permission is handled by the system when accessing the device
    // We can test access by trying to create an input stream
    
    use cpal::traits::*;
    
    let host = cpal::default_host();
    match host.default_input_device() {
        Some(device) => {
            // Try to get device name and supported configs to test access
            match device.name() {
                Ok(_) => {
                    log::info!("Microphone access appears to be available");
                    Ok(PermissionState::Granted)
                },
                Err(_) => {
                    log::warn!("Microphone access may be restricted");
                    Ok(PermissionState::Denied)
                }
            }
        },
        None => {
            log::warn!("No default input device found");
            Ok(PermissionState::Denied)
        }
    }
}

#[cfg(target_os = "macos")]
async fn macos_request_screen_recording_permission() -> CaptureResult<PermissionState> {
    // On macOS 10.15+, screen recording requires explicit permission
    // The system will prompt the user when screen capture is first attempted
    
    log::info!("Screen recording permission check on macOS");
    log::info!("If screen capture fails, grant permission in:");
    log::info!("System Preferences > Security & Privacy > Privacy > Screen Recording");
    
    // For now, assume permission is needed and will be requested by the system
    Ok(PermissionState::NotRequested)
}

#[cfg(target_os = "macos")]
async fn macos_request_system_audio_permission() -> CaptureResult<PermissionState> {
    // System audio on macOS requires either:
    // 1. Virtual audio driver (BlackHole)
    // 2. ScreenCaptureKit with audio enabled
    
    log::info!("System audio capture requires virtual audio device (BlackHole) or ScreenCaptureKit permission");
    Ok(PermissionState::NotRequested)
}

// Windows-specific permission implementations
#[cfg(target_os = "windows")]
async fn windows_request_microphone_permission() -> CaptureResult<PermissionState> {
    // Windows 10+ has microphone privacy settings
    log::info!("Microphone permission managed by Windows privacy settings");
    Ok(PermissionState::NotRequested)
}

#[cfg(target_os = "windows")]
async fn windows_request_system_audio_permission() -> CaptureResult<PermissionState> {
    // System audio requires enabling Stereo Mix or using WASAPI loopback
    log::info!("System audio requires 'Stereo Mix' to be enabled in sound settings");
    Ok(PermissionState::NotRequested)
}

// Linux-specific permission implementations
#[cfg(target_os = "linux")]
async fn linux_request_microphone_permission() -> CaptureResult<PermissionState> {
    // Linux permissions depend on user groups and PulseAudio/PipeWire configuration
    log::info!("Microphone access depends on user permissions and audio system configuration");
    Ok(PermissionState::NotRequested)
}

#[cfg(target_os = "linux")]
async fn linux_request_system_audio_permission() -> CaptureResult<PermissionState> {
    // System audio requires monitor devices or loopback configuration
    log::info!("System audio requires PulseAudio monitor devices or PipeWire loopback configuration");
    Ok(PermissionState::NotRequested)
}

/// Get platform-specific guidance for enabling system audio capture
pub fn get_system_audio_setup_instructions() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macOS System Audio Setup:\n\
        1. Install BlackHole virtual audio driver: https://existential.audio/blackhole/\n\
        2. Or enable ScreenCaptureKit permissions in System Preferences > Security & Privacy > Screen Recording\n\
        3. Restart your application after setup"
    }

    #[cfg(target_os = "windows")]
    {
        "Windows System Audio Setup:\n\
        1. Right-click speaker icon in system tray\n\
        2. Select 'Open Sound settings'\n\
        3. Click 'Sound Control Panel'\n\
        4. Go to Recording tab\n\
        5. Right-click empty space, select 'Show Disabled Devices'\n\
        6. Enable 'Stereo Mix' if available\n\
        7. Set as default recording device"
    }

    #[cfg(target_os = "linux")]
    {
        "Linux System Audio Setup:\n\
        1. For PulseAudio: Enable monitor devices\n\
           pactl load-module module-loopback\n\
        2. For PipeWire: Configure virtual devices\n\
           pw-loopback\n\
        3. Check available devices with: pactl list sources"
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "System audio capture not supported on this platform"
    }
}
