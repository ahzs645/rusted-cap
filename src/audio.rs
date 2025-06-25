use crate::{config::AudioCaptureConfig, error::{AudioError, CaptureError, CaptureResult}};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream, StreamConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::collections::VecDeque;

pub use crate::config::AudioFormat;

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Device ID
    pub id: String,
    /// Device name
    pub name: String,
    /// Device type (input/output)
    pub device_type: AudioDeviceType,
    /// Default device
    pub is_default: bool,
    /// Supported sample rates
    pub sample_rates: Vec<u32>,
    /// Supported channel counts
    pub channels: Vec<u16>,
}

/// Audio device type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioDeviceType {
    Input,
    Output,
}

/// Audio segment data
#[derive(Debug, Clone)]
pub struct AudioSegment {
    /// Raw audio data
    pub data: Vec<f32>,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Timestamp in milliseconds
    pub timestamp: u64,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Audio source type
    pub source: AudioSource,
}

/// Audio source type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSource {
    Microphone,
    SystemAudio,
    Mixed,
}

/// Audio processor that handles capture and processing
pub struct AudioProcessor {
    config: AudioCaptureConfig,
    microphone_stream: Option<Stream>,
    system_audio_stream: Option<Stream>,
    segment_sender: Option<mpsc::UnboundedSender<AudioSegment>>,
    is_running: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new(config: AudioCaptureConfig) -> CaptureResult<Self> {
        log::info!("Initializing audio processor with config: {:?}", config);
        
        Ok(Self {
            config,
            microphone_stream: None,
            system_audio_stream: None,
            segment_sender: None,
            is_running: Arc::new(Mutex::new(false)),
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// Start audio capture
    pub async fn start(&mut self) -> CaptureResult<mpsc::UnboundedReceiver<AudioSegment>> {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return Err(CaptureError::Audio(AudioError::InitializationFailed(
                "Audio processor is already running".to_string()
            )));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        self.segment_sender = Some(tx.clone());

        // Initialize microphone capture if enabled
        if self.config.microphone {
            self.microphone_stream = Some(self.create_microphone_stream(tx.clone())?);
        }

        // Initialize system audio capture if enabled
        if self.config.system_audio {
            self.system_audio_stream = Some(self.create_system_audio_stream(tx.clone())?);
        }

        // Start streams
        if let Some(ref stream) = self.microphone_stream {
            stream.play().map_err(CaptureError::from)?;
        }

        if let Some(ref stream) = self.system_audio_stream {
            stream.play().map_err(CaptureError::from)?;
        }

        *is_running = true;
        log::info!("Audio processor started successfully");

        Ok(rx)
    }

    /// Stop audio capture
    pub async fn stop(&mut self) -> CaptureResult<()> {
        let mut is_running = self.is_running.lock().unwrap();
        if !*is_running {
            return Ok(());
        }

        // Stop and drop streams
        if let Some(stream) = self.microphone_stream.take() {
            stream.pause().map_err(CaptureError::from)?;
        }

        if let Some(stream) = self.system_audio_stream.take() {
            stream.pause().map_err(CaptureError::from)?;
        }

        self.segment_sender = None;
        *is_running = false;

        log::info!("Audio processor stopped successfully");
        Ok(())
    }

    /// Create microphone input stream
    fn create_microphone_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        let host = cpal::default_host();
        let device = if let Some(ref device_id) = self.config.microphone_device_id {
            // Find specific device by ID
            host.input_devices()?
                .find(|d| d.name().unwrap_or_default() == *device_id)
                .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound(device_id.clone())))?
        } else {
            // Use default input device
            host.default_input_device()
                .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound("default input".to_string())))?
        };

        let config = StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let sample_rate = self.config.sample_rate;
        let channels = self.config.channels;
        let segment_duration_samples = (sample_rate * self.config.segment_duration_ms / 1000) as usize;
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer.lock().unwrap();
                buffer.extend_from_slice(data);

                // Check if we have enough samples for a segment
                if buffer.len() >= segment_duration_samples * channels as usize {
                    let segment_data: Vec<f32> = buffer.drain(..segment_duration_samples * channels as usize).collect();
                    
                    let segment = AudioSegment {
                        data: segment_data,
                        sample_rate,
                        channels,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        duration_ms: (segment_duration_samples as f32 / sample_rate as f32 * 1000.0) as u32,
                        source: AudioSource::Microphone,
                    };

                    if let Err(e) = tx.send(segment) {
                        log::error!("Failed to send microphone audio segment: {}", e);
                    }
                }
            },
            move |err| {
                log::error!("Microphone stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    /// Create system audio capture stream
    fn create_system_audio_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        #[cfg(target_os = "macos")]
        {
            self.create_macos_system_audio_stream(tx)
        }

        #[cfg(target_os = "windows")]
        {
            self.create_windows_system_audio_stream(tx)
        }

        #[cfg(target_os = "linux")]
        {
            self.create_linux_system_audio_stream(tx)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(CaptureError::Platform(
                "System audio capture not supported on this platform".to_string()
            ))
        }
    }

    #[cfg(target_os = "macos")]
    fn create_macos_system_audio_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        // On macOS, we would use ScreenCaptureKit or BlackHole for system audio
        // For now, we'll create a placeholder that would need actual implementation
        log::warn!("macOS system audio capture requires ScreenCaptureKit implementation");
        
        // Placeholder implementation - would need actual ScreenCaptureKit integration
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound("default output".to_string())))?;

        let config = StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // This is a placeholder - real implementation would capture system audio
        let stream = device.build_input_stream(
            &config,
            move |_data: &[f32], _: &cpal::InputCallbackInfo| {
                // Real implementation would capture system audio here
            },
            move |err| {
                log::error!("System audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    #[cfg(target_os = "windows")]
    fn create_windows_system_audio_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        // On Windows, we would use WASAPI loopback mode
        log::warn!("Windows system audio capture requires WASAPI loopback implementation");
        
        // Placeholder - would need actual WASAPI implementation
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound("default output".to_string())))?;

        let config = StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_input_stream(
            &config,
            move |_data: &[f32], _: &cpal::InputCallbackInfo| {
                // Real implementation would capture system audio here
            },
            move |err| {
                log::error!("System audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    #[cfg(target_os = "linux")]
    fn create_linux_system_audio_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        // On Linux, we would use PipeWire or PulseAudio
        log::warn!("Linux system audio capture requires PipeWire/PulseAudio implementation");
        
        // Placeholder - would need actual PipeWire/PulseAudio implementation
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound("default output".to_string())))?;

        let config = StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_input_stream(
            &config,
            move |_data: &[f32], _: &cpal::InputCallbackInfo| {
                // Real implementation would capture system audio here
            },
            move |err| {
                log::error!("System audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }
}

/// Get available audio devices
pub fn get_available_devices() -> CaptureResult<Vec<AudioDevice>> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    // Get input devices
    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                let is_default = host.default_input_device()
                    .and_then(|d| d.name().ok())
                    .map(|default_name| default_name == name)
                    .unwrap_or(false);
                
                let device_info = AudioDevice {
                    id: name.clone(),
                    name,
                    device_type: AudioDeviceType::Input,
                    is_default,
                    sample_rates: vec![44100, 48000], // Could query actual supported rates
                    channels: vec![1, 2], // Could query actual supported channels
                };
                devices.push(device_info);
            }
        }
    }

    // Get output devices  
    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                let is_default = host.default_output_device()
                    .and_then(|d| d.name().ok())
                    .map(|default_name| default_name == name)
                    .unwrap_or(false);
                
                let device_info = AudioDevice {
                    id: name.clone(),
                    name,
                    device_type: AudioDeviceType::Output,
                    is_default,
                    sample_rates: vec![44100, 48000],
                    channels: vec![1, 2],
                };
                devices.push(device_info);
            }
        }
    }

    Ok(devices)
}
