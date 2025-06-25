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
        log::info!("Initializing macOS system audio capture via ScreenCaptureKit");
        
        // For now, we'll use a hybrid approach:
        // 1. Try to create a loopback device capture (requires virtual audio setup)
        // 2. Fall back to a placeholder that guides users to set up system audio
        
        let host = cpal::default_host();
        
        // Look for BlackHole or similar virtual audio device
        let system_device = if let Ok(mut devices) = host.output_devices() {
            devices.find(|device| {
                if let Ok(name) = device.name() {
                    name.to_lowercase().contains("blackhole") || 
                    name.to_lowercase().contains("soundflower") ||
                    name.to_lowercase().contains("virtual")
                } else {
                    false
                }
            })
        } else {
            None
        };

        let device = if let Some(virtual_device) = system_device {
            log::info!("Found virtual audio device for system audio capture");
            virtual_device
        } else {
            log::warn!("No virtual audio device found. System audio capture requires BlackHole or similar virtual audio driver. Using default output device as placeholder.");
            host.default_output_device()
                .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound("default output".to_string())))?
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

        // Note: This is a simplified implementation. Full ScreenCaptureKit integration would require:
        // 1. Requesting screen recording permission
        // 2. Creating SCStream with audio capture enabled
        // 3. Implementing SCStreamDelegate for audio callbacks
        // 4. Converting CMSampleBuffer to our audio format
        
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer.lock().unwrap();
                buffer.extend_from_slice(data);

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
                        source: AudioSource::SystemAudio,
                    };

                    if let Err(e) = tx.send(segment) {
                        log::error!("Failed to send system audio segment: {}", e);
                    }
                }
            },
            move |err| {
                log::error!("System audio stream error: {}", err);
            },
            None,
        ).map_err(|e| CaptureError::Audio(AudioError::StreamError(
            format!("Failed to create macOS system audio stream. Consider installing BlackHole for proper system audio capture: {}", e)
        )))?;

        Ok(stream)
    }

    #[cfg(target_os = "windows")]
    fn create_windows_system_audio_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;
        use windows::core::*;
        
        log::info!("Initializing Windows system audio capture via WASAPI loopback");

        // For a complete WASAPI loopback implementation, we would need:
        // 1. CoInitialize COM
        // 2. Create IMMDeviceEnumerator
        // 3. Get default render device
        // 4. Activate IAudioClient in loopback mode
        // 5. Start capture loop
        
        // Since this is complex and requires unsafe code, we'll provide a hybrid approach:
        // Use cpal with a loopback device if available, otherwise provide guidance
        
        let host = cpal::default_host();
        
        // Try to find a stereo mix or similar loopback device
        let loopback_device = if let Ok(devices) = host.input_devices() {
            devices.find(|device| {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    name_lower.contains("stereo mix") || 
                    name_lower.contains("what u hear") ||
                    name_lower.contains("loopback") ||
                    name_lower.contains("wave out mix")
                } else {
                    false
                }
            })
        } else {
            None
        };

        let device = if let Some(loopback) = loopback_device {
            log::info!("Found Windows loopback device for system audio capture");
            loopback
        } else {
            log::warn!("No loopback device found. Enable 'Stereo Mix' in Windows sound settings for system audio capture");
            // Fall back to default output device for now
            host.default_output_device()
                .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound(
                    "No system audio device found. Please enable 'Stereo Mix' in Windows Sound settings".to_string()
                )))?
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
                        source: AudioSource::SystemAudio,
                    };

                    if let Err(e) = tx.send(segment) {
                        log::error!("Failed to send system audio segment: {}", e);
                    }
                }
            },
            move |err| {
                log::error!("System audio stream error: {}", err);
            },
            None,
        ).map_err(|e| CaptureError::Audio(AudioError::StreamError(
            format!("Failed to create Windows system audio stream. Enable 'Stereo Mix' in sound settings: {}", e)
        )))?;

        Ok(stream)
    }

    #[cfg(target_os = "linux")]
    fn create_linux_system_audio_stream(&self, tx: mpsc::UnboundedSender<AudioSegment>) -> CaptureResult<Stream> {
        log::info!("Initializing Linux system audio capture via PipeWire/PulseAudio");
        
        let host = cpal::default_host();
        
        // On Linux, look for monitor devices (PulseAudio) or similar system audio sources
        let monitor_device = if let Ok(devices) = host.input_devices() {
            devices.find(|device| {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    name_lower.contains("monitor") || 
                    name_lower.contains("output") ||
                    name_lower.contains("sink") ||
                    name_lower.contains("loopback")
                } else {
                    false
                }
            })
        } else {
            None
        };

        let device = if let Some(monitor) = monitor_device {
            log::info!("Found Linux monitor device for system audio capture");
            monitor
        } else {
            log::warn!("No monitor device found. Configure PulseAudio/PipeWire for system audio capture");
            host.default_input_device()
                .ok_or_else(|| CaptureError::Audio(AudioError::DeviceNotFound(
                    "No system audio device found. Configure PulseAudio monitor or PipeWire loopback".to_string()
                )))?
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
                        source: AudioSource::SystemAudio,
                    };

                    if let Err(e) = tx.send(segment) {
                        log::error!("Failed to send system audio segment: {}", e);
                    }
                }
            },
            move |err| {
                log::error!("System audio stream error: {}", err);
            },
            None,
        ).map_err(|e| CaptureError::Audio(AudioError::StreamError(
            format!("Failed to create Linux system audio stream. Configure PipeWire/PulseAudio: {}", e)
        )))?;

        Ok(stream)
    }

    /// Mix audio from multiple sources
    pub fn mix_audio_sources(&self, mic_data: &[f32], system_data: &[f32]) -> Vec<f32> {
        let max_len = mic_data.len().max(system_data.len());
        let mut mixed = Vec::with_capacity(max_len);
        
        for i in 0..max_len {
            let mic_sample = mic_data.get(i).copied().unwrap_or(0.0);
            let system_sample = system_data.get(i).copied().unwrap_or(0.0);
            
            // Simple mixing with gain adjustment to prevent clipping
            let mixed_sample = (mic_sample * 0.6 + system_sample * 0.6).clamp(-1.0, 1.0);
            mixed.push(mixed_sample);
        }
        
        mixed
    }

    /// Convert PCM audio to different formats
    pub fn convert_to_format(&self, pcm_data: &[f32], format: AudioFormat) -> CaptureResult<Vec<u8>> {
        match format {
            AudioFormat::Raw => Ok(pcm_data.iter().flat_map(|&f| f.to_le_bytes()).collect()),
            AudioFormat::Wav => self.encode_wav(pcm_data),
            AudioFormat::Mp3 => self.encode_mp3(pcm_data),
            AudioFormat::Aac => self.encode_aac(pcm_data),
        }
    }

    /// Encode to WAV format
    fn encode_wav(&self, pcm_data: &[f32]) -> CaptureResult<Vec<u8>> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        
        let spec = hound::WavSpec {
            channels: self.config.channels,
            sample_rate: self.config.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec)
                .map_err(|e| CaptureError::Audio(AudioError::EncodingError(e.to_string())))?;
            
            for &sample in pcm_data {
                writer.write_sample(sample)
                    .map_err(|e| CaptureError::Audio(AudioError::EncodingError(e.to_string())))?;
            }
            
            writer.finalize()
                .map_err(|e| CaptureError::Audio(AudioError::EncodingError(e.to_string())))?;
        }

        Ok(cursor.into_inner())
    }

    /// Encode to MP3 format (placeholder - would need actual MP3 encoder)
    fn encode_mp3(&self, _pcm_data: &[f32]) -> CaptureResult<Vec<u8>> {
        // For now, return error suggesting WAV format
        Err(CaptureError::Audio(AudioError::EncodingError(
            "MP3 encoding not yet implemented. Use WAV or Raw format.".to_string()
        )))
    }

    /// Encode to AAC format (placeholder - would need actual AAC encoder)
    fn encode_aac(&self, _pcm_data: &[f32]) -> CaptureResult<Vec<u8>> {
        // For now, return error suggesting WAV format
        Err(CaptureError::Audio(AudioError::EncodingError(
            "AAC encoding not yet implemented. Use WAV or Raw format.".to_string()
        )))
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
