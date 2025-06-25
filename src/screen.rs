use crate::{config::ScreenCaptureConfig, error::{CaptureError, CaptureResult, ScreenError}};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    /// Display ID
    pub id: u32,
    /// Display name
    pub name: String,
    /// Display resolution
    pub resolution: (u32, u32),
    /// Display width (for easier access)
    pub width: u32,
    /// Display height (for easier access)
    pub height: u32,
    /// Display position
    pub position: (i32, i32),
    /// Is primary display
    pub is_primary: bool,
    /// Display scale factor
    pub scale_factor: f64,
}

/// Window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    /// Window ID
    pub id: i64,
    /// Window title
    pub title: String,
    /// Application name
    pub app_name: String,
    /// Window bounds
    pub bounds: (i32, i32, u32, u32), // x, y, width, height
    /// Is window minimized
    pub is_minimized: bool,
    /// Is window visible
    pub is_visible: bool,
}

/// Screen frame data
#[derive(Debug, Clone)]
pub struct ScreenFrame {
    /// Raw frame data (RGBA)
    pub data: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Timestamp in milliseconds
    pub timestamp: u64,
    /// Frame number
    pub frame_number: u64,
}

/// Screen capture implementation
pub struct ScreenCapture {
    config: ScreenCaptureConfig,
    is_running: Arc<Mutex<bool>>,
    frame_sender: Option<mpsc::UnboundedSender<ScreenFrame>>,
    frame_counter: Arc<Mutex<u64>>,
}

impl ScreenCapture {
    /// Create a new screen capture instance
    pub fn new(config: ScreenCaptureConfig) -> CaptureResult<Self> {
        log::info!("Initializing screen capture with config: {:?}", config);
        
        // Validate configuration
        if config.fps == 0 {
            return Err(CaptureError::Screen(ScreenError::FormatError(
                "FPS must be greater than 0".to_string()
            )));
        }

        if config.quality > 100 {
            return Err(CaptureError::Screen(ScreenError::FormatError(
                "Quality must be between 0 and 100".to_string()
            )));
        }

        Ok(Self {
            config,
            is_running: Arc::new(Mutex::new(false)),
            frame_sender: None,
            frame_counter: Arc::new(Mutex::new(0)),
        })
    }

    /// Start screen capture
    pub async fn start(&mut self) -> CaptureResult<mpsc::UnboundedReceiver<ScreenFrame>> {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return Err(CaptureError::Screen(ScreenError::InitializationFailed(
                "Screen capture is already running".to_string()
            )));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        self.frame_sender = Some(tx.clone());

        // Start platform-specific capture
        self.start_platform_capture(tx).await?;

        *is_running = true;
        log::info!("Screen capture started successfully");

        Ok(rx)
    }

    /// Stop screen capture
    pub async fn stop(&mut self) -> CaptureResult<()> {
        let mut is_running = self.is_running.lock().unwrap();
        if !*is_running {
            return Ok(());
        }

        // Stop capture
        self.stop_platform_capture().await?;
        
        self.frame_sender = None;
        *is_running = false;

        log::info!("Screen capture stopped successfully");
        Ok(())
    }

    /// Start platform-specific capture implementation
    async fn start_platform_capture(&self, tx: mpsc::UnboundedSender<ScreenFrame>) -> CaptureResult<()> {
        #[cfg(target_os = "macos")]
        {
            self.start_macos_capture(tx).await
        }

        #[cfg(target_os = "windows")]
        {
            self.start_windows_capture(tx).await
        }

        #[cfg(target_os = "linux")]
        {
            self.start_linux_capture(tx).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(CaptureError::Platform(
                "Screen capture not supported on this platform".to_string()
            ))
        }
    }

    /// Stop platform-specific capture
    async fn stop_platform_capture(&self) -> CaptureResult<()> {
        // Platform-specific cleanup would go here
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn start_macos_capture(&self, tx: mpsc::UnboundedSender<ScreenFrame>) -> CaptureResult<()> {
        // macOS implementation using ScreenCaptureKit
        log::info!("Starting macOS screen capture");
        
        // This would use the screencapturekit crate
        // For now, we'll simulate frame capture
        let fps = self.config.fps;
        let frame_counter = self.frame_counter.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let frame_interval = std::time::Duration::from_millis(1000 / fps as u64);
            let mut interval = tokio::time::interval(frame_interval);

            while *is_running.lock().unwrap() {
                interval.tick().await;

                // Simulate frame capture - real implementation would use ScreenCaptureKit
                let mut counter = frame_counter.lock().unwrap();
                *counter += 1;

                let frame = ScreenFrame {
                    data: vec![0; 1920 * 1080 * 4], // Placeholder RGBA data
                    width: 1920,
                    height: 1080,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    frame_number: *counter,
                };

                if tx.send(frame).is_err() {
                    log::error!("Failed to send screen frame - receiver dropped");
                    break;
                }
            }
        });

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn start_windows_capture(&self, tx: mpsc::UnboundedSender<ScreenFrame>) -> CaptureResult<()> {
        // Windows implementation using Windows Capture API
        log::info!("Starting Windows screen capture");
        
        // This would use the windows crate and Windows.Graphics.Capture API
        // For now, we'll simulate frame capture
        let fps = self.config.fps;
        let frame_counter = self.frame_counter.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let frame_interval = std::time::Duration::from_millis(1000 / fps as u64);
            let mut interval = tokio::time::interval(frame_interval);

            while *is_running.lock().unwrap() {
                interval.tick().await;

                // Simulate frame capture - real implementation would use Windows Capture API
                let mut counter = frame_counter.lock().unwrap();
                *counter += 1;

                let frame = ScreenFrame {
                    data: vec![0; 1920 * 1080 * 4], // Placeholder RGBA data
                    width: 1920,
                    height: 1080,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    frame_number: *counter,
                };

                if tx.send(frame).is_err() {
                    log::error!("Failed to send screen frame - receiver dropped");
                    break;
                }
            }
        });

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn start_linux_capture(&self, tx: mpsc::UnboundedSender<ScreenFrame>) -> CaptureResult<()> {
        // Linux implementation using PipeWire or X11
        log::info!("Starting Linux screen capture");
        
        // This would use PipeWire or X11/Wayland capture
        // For now, we'll simulate frame capture
        let fps = self.config.fps;
        let frame_counter = self.frame_counter.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let frame_interval = std::time::Duration::from_millis(1000 / fps as u64);
            let mut interval = tokio::time::interval(frame_interval);

            while *is_running.lock().unwrap() {
                interval.tick().await;

                // Simulate frame capture - real implementation would use PipeWire/X11
                let mut counter = frame_counter.lock().unwrap();
                *counter += 1;

                let frame = ScreenFrame {
                    data: vec![0; 1920 * 1080 * 4], // Placeholder RGBA data
                    width: 1920,
                    height: 1080,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    frame_number: *counter,
                };

                if tx.send(frame).is_err() {
                    log::error!("Failed to send screen frame - receiver dropped");
                    break;
                }
            }
        });

        Ok(())
    }
}

/// Get available displays for screen capture
pub fn get_available_displays() -> CaptureResult<Vec<Display>> {
    #[cfg(target_os = "macos")]
    {
        get_macos_displays()
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_displays()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_displays()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(vec![]) // Return empty list for unsupported platforms
    }
}

/// Get available windows for window capture
pub fn get_available_windows() -> CaptureResult<Vec<Window>> {
    #[cfg(target_os = "macos")]
    {
        get_macos_windows()
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_windows()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(vec![]) // Return empty list for unsupported platforms
    }
}

#[cfg(target_os = "macos")]
fn get_macos_displays() -> CaptureResult<Vec<Display>> {
    // 1. Check permissions first (Cap's approach)
    if !check_screen_recording_permission() {
        log::warn!("Screen recording permission may not be granted");
    }
    
    // 2. Try ScreenCaptureKit first (modern approach - requires macOS 12.3+)
    match get_displays_screencapturekit() {
        Ok(displays) => {
            log::info!("✅ Successfully enumerated {} displays via ScreenCaptureKit", displays.len());
            Ok(displays)
        },
        Err(e) => {
            log::warn!("❌ ScreenCaptureKit failed: {}, falling back to CoreGraphics", e);
            get_displays_coregraphics()
        }
    }
}

/// Request screen recording permission (Cap's approach)
#[cfg(target_os = "macos")]
pub async fn request_screen_recording_permission() -> CaptureResult<bool> {
    use screencapturekit::shareable_content::SCShareableContent;
    
    log::info!("🔐 Requesting screen recording permission...");
    
    // Try to access ScreenCaptureKit - this will trigger permission prompt if needed
    match SCShareableContent::get() {
        Ok(_content) => {
            log::info!("✅ Screen recording permission granted");
            Ok(true)
        },
        Err(error) => {
            log::error!("❌ Screen recording permission request failed: {}", error);
            log::info!("💡 To grant permission:");
            log::info!("   1. Open System Preferences > Privacy & Security");
            log::info!("   2. Go to Screen Recording");
            log::info!("   3. Add this application");
            log::info!("   4. Restart the application");
            Ok(false)
        }
    }
}

#[cfg(target_os = "macos")]
fn get_displays_screencapturekit() -> CaptureResult<Vec<Display>> {
    use screencapturekit::shareable_content::SCShareableContent;
    
    log::debug!("Attempting to use ScreenCaptureKit for display enumeration");
    
    // 1. Get shareable content (this includes all displays)
    let content = match SCShareableContent::get() {
        Ok(content) => content,
        Err(e) => {
            return Err(CaptureError::Screen(ScreenError::InitializationFailed(
                format!("Failed to get shareable content: {}. Grant Screen Recording permission in System Preferences > Privacy & Security > Screen Recording.", e)
            )));
        }
    };
    
    // 2. Get all displays from shareable content
    let sc_displays = content.displays();
    
    if sc_displays.is_empty() {
        return Err(CaptureError::Screen(ScreenError::DisplayNotFound(
            "No displays found via ScreenCaptureKit".to_string()
        )));
    }
    
    let mut displays = Vec::new();
    
    // 3. Convert SCDisplay objects to our Display struct using real API calls
    for (index, _sc_display) in sc_displays.iter().enumerate() {
        let display = Display {
            id: get_display_id_from_index(index)?,
            name: get_display_name_from_index(index)?,
            width: get_display_width_from_index(index)?,
            height: get_display_height_from_index(index)?,
            resolution: get_display_resolution_from_index(index)?,
            position: get_display_position_from_index(index)?,
            is_primary: is_primary_display_from_index(index)?,
            scale_factor: get_scale_factor_from_index(index)?,
        };
        displays.push(display);
    }
    
    log::info!("Successfully enumerated {} displays via ScreenCaptureKit", displays.len());
    Ok(displays)
}

// Real ScreenCaptureKit API implementations using CoreGraphics integration (Cap's approach)
#[cfg(target_os = "macos")]
fn get_display_id_from_index(index: usize) -> CaptureResult<u32> {
    use core_graphics::display::CGDisplay;
    
    // Get real display IDs from CoreGraphics (which ScreenCaptureKit uses internally)
    match CGDisplay::active_displays() {
        Ok(display_ids) => {
            if let Some(&display_id) = display_ids.get(index) {
                Ok(display_id)
            } else {
                Err(CaptureError::Screen(ScreenError::DisplayNotFound(
                    format!("Display {} not found", index)
                )))
            }
        },
        Err(_) => Err(CaptureError::Screen(ScreenError::InitializationFailed(
            "Failed to get display IDs".to_string()
        ))),
    }
}

#[cfg(target_os = "macos")]
fn get_display_name_from_index(index: usize) -> CaptureResult<String> {
    use core_graphics::display::CGDisplay;
    
    // Get real display names using the same logic as CoreGraphics fallback
    if let Ok(display_ids) = CGDisplay::active_displays() {
        if let Some(&display_id) = display_ids.get(index) {
            let main_display = CGDisplay::main();
            if display_id == main_display.id {
                return Ok("Built-in Display (ScreenCaptureKit)".to_string());
            } else {
                return Ok(format!("External Display {} (ScreenCaptureKit)", display_id));
            }
        }
    }
    
    // Final fallback
    Ok(format!("ScreenCaptureKit Display {}", index))
}

#[cfg(target_os = "macos")]
fn get_display_width_from_index(index: usize) -> CaptureResult<u32> {
    use core_graphics::display::CGDisplay;
    
    // Get real display dimensions from CoreGraphics
    if let Ok(display_ids) = CGDisplay::active_displays() {
        if let Some(&display_id) = display_ids.get(index) {
            let display = CGDisplay::new(display_id);
            let bounds = display.bounds();
            return Ok(bounds.size.width as u32);
        }
    }
    
    Err(CaptureError::Screen(ScreenError::DisplayNotFound(
        format!("Could not get width for display {}", index)
    )))
}

#[cfg(target_os = "macos")]
fn get_display_height_from_index(index: usize) -> CaptureResult<u32> {
    use core_graphics::display::CGDisplay;
    
    // Get real display dimensions from CoreGraphics
    if let Ok(display_ids) = CGDisplay::active_displays() {
        if let Some(&display_id) = display_ids.get(index) {
            let display = CGDisplay::new(display_id);
            let bounds = display.bounds();
            return Ok(bounds.size.height as u32);
        }
    }
    
    Err(CaptureError::Screen(ScreenError::DisplayNotFound(
        format!("Could not get height for display {}", index)
    )))
}

#[cfg(target_os = "macos")]
fn get_display_resolution_from_index(index: usize) -> CaptureResult<(u32, u32)> {
    let width = get_display_width_from_index(index)?;
    let height = get_display_height_from_index(index)?;
    Ok((width, height))
}

#[cfg(target_os = "macos")]
fn get_display_position_from_index(index: usize) -> CaptureResult<(i32, i32)> {
    use core_graphics::display::CGDisplay;
    
    // Get real display position from CoreGraphics
    if let Ok(display_ids) = CGDisplay::active_displays() {
        if let Some(&display_id) = display_ids.get(index) {
            let display = CGDisplay::new(display_id);
            let bounds = display.bounds();
            return Ok((bounds.origin.x as i32, bounds.origin.y as i32));
        }
    }
    
    Err(CaptureError::Screen(ScreenError::DisplayNotFound(
        format!("Could not get position for display {}", index)
    )))
}

#[cfg(target_os = "macos")]
fn is_primary_display_from_index(index: usize) -> CaptureResult<bool> {
    use core_graphics::display::CGDisplay;
    
    // Check if this is the main display using CoreGraphics
    if let Ok(display_ids) = CGDisplay::active_displays() {
        if let Some(&display_id) = display_ids.get(index) {
            let main_display = CGDisplay::main();
            return Ok(display_id == main_display.id);
        }
    }
    
    // Fallback: First display from ScreenCaptureKit is typically primary
    Ok(index == 0)
}

#[cfg(target_os = "macos")]
fn get_scale_factor_from_index(index: usize) -> CaptureResult<f64> {
    use core_graphics::display::CGDisplay;
    
    // Get real scale factor using CoreGraphics
    if let Ok(display_ids) = CGDisplay::active_displays() {
        if let Some(&display_id) = display_ids.get(index) {
            let display = CGDisplay::new(display_id);
            return Ok(get_display_scale_factor_coregraphics(&display));
        }
    }
    
    // Default for modern Mac displays
    Ok(2.0)
}

#[cfg(target_os = "macos")]
fn check_screen_recording_permission() -> bool {
    // This is a simplified permission check
    // Real implementation would use CGPreflightScreenCaptureAccess() or similar
    
    use core_graphics::display::CGDisplay;
    
    // Try to get display information - if this fails, we likely don't have permission
    match CGDisplay::active_displays() {
        Ok(displays) => !displays.is_empty(),
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
fn get_displays_coregraphics() -> CaptureResult<Vec<Display>> {
    use core_graphics::display::*;
    
    // Get all active displays
    let display_ids = CGDisplay::active_displays()
        .map_err(|_| CaptureError::Screen(ScreenError::InitializationFailed(
            "Failed to enumerate displays via CoreGraphics".to_string()
        )))?;
    
    if display_ids.is_empty() {
        return Err(CaptureError::Screen(ScreenError::DisplayNotFound(
            "No active displays found".to_string()
        )));
    }
    
    // Get main display for primary detection
    let main_display = CGDisplay::main();
    
    // Convert CGDisplay objects to our Display struct
    let displays: Vec<Display> = display_ids
        .into_iter()
        .map(|display_id| {
            let display = CGDisplay::new(display_id);
            let bounds = display.bounds();
            
            // Try to get a more descriptive name
            let name = get_display_name_coregraphics(display_id, &main_display)
                .unwrap_or_else(|| format!("Display {}", display_id));
            
            Display {
                id: display_id,
                name,
                width: bounds.size.width as u32,
                height: bounds.size.height as u32,
                resolution: (bounds.size.width as u32, bounds.size.height as u32),
                position: (bounds.origin.x as i32, bounds.origin.y as i32),
                is_primary: display_id == main_display.id,
                scale_factor: get_display_scale_factor_coregraphics(&display),
            }
        })
        .collect();
    
    log::info!("Enumerated {} displays via CoreGraphics", displays.len());
    Ok(displays)
}

#[cfg(target_os = "macos")]
fn get_display_name_coregraphics(display_id: u32, main_display: &core_graphics::display::CGDisplay) -> Option<String> {
    // For now, return a simple name based on whether it's the main display
    // This could be enhanced with IOKit calls for actual display names
    if display_id == main_display.id {
        Some("Built-in Display".to_string())
    } else {
        Some(format!("External Display {}", display_id))
    }
}

#[cfg(target_os = "macos")]
fn get_display_scale_factor_coregraphics(display: &core_graphics::display::CGDisplay) -> f64 {
    // Get the display mode to calculate scale factor
    let bounds = display.bounds();
    
    if let Some(mode) = display.display_mode() {
        let pixel_width = mode.pixel_width() as f64;
        let point_width = bounds.size.width;
        
        if point_width > 0.0 {
            (pixel_width / point_width).max(1.0)
        } else {
            1.0
        }
    } else {
        // Default to 2.0 for Retina displays, 1.0 for others
        // This is a heuristic based on common resolutions
        if bounds.size.width >= 2560.0 { 2.0 } else { 1.0 }
    }
}

#[cfg(target_os = "windows")]
fn get_windows_displays() -> CaptureResult<Vec<Display>> {
    // Would use Win32 API to enumerate displays
    Ok(vec![
        Display {
            id: 0,
            name: "Primary Display".to_string(),
            resolution: (1920, 1080),
            width: 1920,
            height: 1080,
            position: (0, 0),
            is_primary: true,
            scale_factor: 1.0,
        }
    ])
}

#[cfg(target_os = "linux")]
fn get_linux_displays() -> CaptureResult<Vec<Display>> {
    // Would use X11 or Wayland to enumerate displays
    Ok(vec![
        Display {
            id: 0,
            name: "Primary Display".to_string(),
            resolution: (1920, 1080),
            width: 1920,
            height: 1080,
            position: (0, 0),
            is_primary: true,
            scale_factor: 1.0,
        }
    ])
}

#[cfg(target_os = "macos")]
fn get_macos_windows() -> CaptureResult<Vec<Window>> {
    // Would use CoreGraphics to enumerate windows
    Ok(vec![])
}

#[cfg(target_os = "windows")]
fn get_windows_windows() -> CaptureResult<Vec<Window>> {
    // Would use Win32 API to enumerate windows
    Ok(vec![])
}

#[cfg(target_os = "linux")]
fn get_linux_windows() -> CaptureResult<Vec<Window>> {
    // Would use X11 or Wayland to enumerate windows
    Ok(vec![])
}
