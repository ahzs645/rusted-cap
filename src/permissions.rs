use crate::error::{CaptureResult};
use cpal::traits::{HostTrait};
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
    #[cfg(target_os = "macos")]
    {
        // Cap's approach: Check for ScreenCaptureKit availability instead of virtual drivers
        // ScreenCaptureKit is available on macOS 12.3+ and provides native system audio capture
        use std::process::Command;
        
        // Check if we're on macOS 12.3+ (where ScreenCaptureKit has audio support)
        let os_version_check = Command::new("sw_vers")
            .args(&["-productVersion"])
            .output();
            
        if let Ok(output) = os_version_check {
            let version_str = String::from_utf8_lossy(&output.stdout);
            log::info!("macOS version: {}", version_str.trim());
            
            // For now, assume ScreenCaptureKit is available
            // In a real implementation, we'd check the exact version
            Ok(PermissionState::Denied) // Will become Granted when screen recording permission is given
        } else {
            log::warn!("Could not determine macOS version");
            Ok(PermissionState::Denied)
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Cap's approach: Use WASAPI loopback instead of Stereo Mix
        // WASAPI loopback is available on Windows Vista+ and doesn't require Stereo Mix
        log::info!("Windows WASAPI loopback available for native system audio capture");
        Ok(PermissionState::Granted) // WASAPI loopback doesn't require special permissions
    }

    #[cfg(target_os = "linux")]
    {
        // Cap's approach: Use PipeWire monitor sources natively
        // Check if PipeWire or PulseAudio monitor sources are available
        use std::process::Command;
        
        // Check if PipeWire is available
        let pipewire_check = Command::new("pw-cli")
            .args(&["info"])
            .output();
            
        if pipewire_check.is_ok() {
            log::info!("PipeWire detected - native monitor sources available");
            Ok(PermissionState::Granted)
        } else {
            // Check for PulseAudio
            let pulse_check = Command::new("pactl")
                .args(&["info"])
                .output();
                
            if pulse_check.is_ok() {
                log::info!("PulseAudio detected - monitor sources available");
                Ok(PermissionState::Granted)
            } else {
                log::warn!("Neither PipeWire nor PulseAudio detected");
                Ok(PermissionState::Denied)
            }
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
    // Cap's approach: Use ScreenCaptureKit for native system audio capture
    // This requires Screen Recording permission, not virtual audio drivers
    
    log::info!("System audio capture using ScreenCaptureKit (native approach)");
    log::info!("Requires Screen Recording permission in System Preferences > Security & Privacy > Privacy > Screen Recording");
    
    // The actual permission request would happen when ScreenCaptureKit is initialized
    // For now, indicate that permission setup is needed
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
    // Cap's approach: Use WASAPI loopback mode for native system audio capture
    // No additional setup required - WASAPI loopback is built into Windows Vista+
    
    log::info!("System audio capture using WASAPI loopback (native approach)");
    log::info!("No additional setup required - WASAPI loopback is available by default");
    
    Ok(PermissionState::Granted)
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
    // Cap's approach: Use PipeWire monitor sources or PulseAudio monitors natively
    // These are available by default in modern Linux distributions
    
    log::info!("System audio capture using PipeWire/PulseAudio monitor sources (native approach)");
    log::info!("Monitor sources are available by default in most Linux distributions");
    
    Ok(PermissionState::Granted)
}

/// Get platform-specific guidance for enabling system audio capture
pub fn get_system_audio_setup_instructions() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macOS Native System Audio (Cap's Approach):\n\
        1. Enable Screen Recording permission in System Preferences > Security & Privacy > Privacy > Screen Recording\n\
        2. Add your application to the list of allowed apps\n\
        3. Restart your application after granting permission\n\
        \n\
        ✅ No BlackHole or virtual drivers needed!\n\
        ✅ Uses ScreenCaptureKit for direct system audio capture\n\
        ✅ Available on macOS 12.3+ (Monterey and later)"
    }

    #[cfg(target_os = "windows")]
    {
        "Windows Native System Audio (Cap's Approach):\n\
        1. No setup required! WASAPI loopback is built into Windows\n\
        2. Your application can capture system audio directly\n\
        \n\
        ✅ No Stereo Mix configuration needed!\n\
        ✅ Uses WASAPI loopback for direct system audio capture\n\
        ✅ Available on Windows Vista and later"
    }

    #[cfg(target_os = "linux")]
    {
        "Linux Native System Audio (Cap's Approach):\n\
        1. No setup required for most distributions!\n\
        2. Uses PipeWire or PulseAudio monitor sources\n\
        3. If needed, check available sources: pactl list sources short\n\
        \n\
        ✅ No manual loopback configuration needed!\n\
        ✅ Uses native monitor sources for direct system audio capture\n\
        ✅ Works with PipeWire and PulseAudio"
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "Native system audio capture not supported on this platform"
    }
}
