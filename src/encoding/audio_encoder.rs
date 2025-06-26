//! Real FFmpeg-based Audio Encoder Implementation
//! 
//! Following Cap's architecture and patterns

use crate::error::{CaptureError, CaptureResult};
use super::{AudioEncodingConfig, AudioCodec, AudioChannelLayout};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ffmpeg::{
    codec::{context, encoder},
    format::{sample::Type, Sample},
    threading::Config,
    ChannelLayout,
};

/// Encoded audio segment ready for upload
#[derive(Debug, Clone)]
pub struct EncodedAudioSegment {
    /// Encoded AAC data
    pub data: Vec<u8>,
    /// Segment sequence number
    pub sequence: u32,
    /// Duration in seconds
    pub duration: f64,
    /// Timestamp when encoded
    pub timestamp: u64,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
}

/// Error types following Cap's pattern
#[derive(Debug)]
pub enum AudioEncodingError {
    FFmpeg(ffmpeg::Error),
    TaskLaunch(String),
    Other(String),
}

impl From<ffmpeg::Error> for AudioEncodingError {
    fn from(error: ffmpeg::Error) -> Self {
        AudioEncodingError::FFmpeg(error)
    }
}

impl From<AudioEncodingError> for CaptureError {
    fn from(error: AudioEncodingError) -> Self {
        match error {
            AudioEncodingError::FFmpeg(e) => CaptureError::EncodingError(format!("FFmpeg error: {}", e)),
            AudioEncodingError::TaskLaunch(e) => CaptureError::EncodingError(format!("Task launch error: {}", e)),
            AudioEncodingError::Other(e) => CaptureError::EncodingError(e),
        }
    }
}

/// Audio encoder trait following Cap's pattern
pub trait AudioEncoderTrait {
    fn queue_frame(&mut self, frame: ffmpeg::frame::Audio) -> Result<Vec<u8>, AudioEncodingError>;
    fn finish(&mut self) -> Result<Vec<u8>, AudioEncodingError>;
}

/// AAC encoder following Cap's implementation pattern
pub struct AACEncoder {
    encoder: encoder::Audio,
    packet: ffmpeg::Packet,
    resampler: Option<ffmpeg::software::resampling::Context>,
    resampled_frame: ffmpeg::frame::Audio,
    buffer: Vec<VecDeque<u8>>,
    config: AudioEncodingConfig,
    sequence_counter: u32,
    pts: i64,
    samples_per_segment: usize,
    current_segment_samples: Vec<f32>,
}

impl AACEncoder {
    // ✅ Use Planar consistently (from cap-media/src/encoders/aac.rs)
    const OUTPUT_SAMPLE_FORMAT: Sample = Sample::F32(Type::Planar);
    
    /// Create new AAC encoder following Cap's factory pattern
    pub fn new(config: AudioEncodingConfig) -> Result<Self, AudioEncodingError> {
        // Initialize FFmpeg following Cap's pattern
        ffmpeg::init().map_err(|e| AudioEncodingError::Other(format!("FFmpeg init: {}", e)))?;
        
        let codec = encoder::find_by_name("aac")
            .ok_or_else(|| AudioEncodingError::TaskLaunch("Could not find AAC codec".into()))?;
            
        let mut encoder_ctx = context::Context::new_with_codec(codec);
        encoder_ctx.set_threading(Config::count(4));
        let mut encoder = encoder_ctx.encoder().audio()?;

        // Configure encoder following Cap's pattern
        let rate = {
            let mut rates = codec
                .audio()
                .unwrap()
                .rates()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            rates.sort();

            let Some(&rate) = rates
                .iter()
                .find(|r| **r >= config.sample_rate as i32)
                .or(rates.first())
            else {
                return Err(AudioEncodingError::TaskLaunch(format!(
                    "AAC Codec does not support sample rate {}",
                    config.sample_rate
                )));
            };
            rate
        };

        let channel_layout = ChannelLayout::default(config.channels as i32);
        
        // ✅ Cap's resampler logic - check if input differs from output
        // Input is typically Packed (interleaved) from external sources
        let input_format = Sample::F32(Type::Packed); // What we actually receive
        let output_format = Self::OUTPUT_SAMPLE_FORMAT; // Planar for AAC
        let output_rate = rate as u32;
        
        let resampler = if (input_format, channel_layout, config.sample_rate) != 
                          (output_format, channel_layout, output_rate) {
            Some(ffmpeg::software::resampler(
                (input_format, channel_layout, config.sample_rate),
                (output_format, channel_layout, output_rate),
            )?)
        } else {
            None
        };

        encoder.set_bit_rate(config.bitrate as usize);
        encoder.set_rate(rate);
        encoder.set_format(Self::OUTPUT_SAMPLE_FORMAT);
        encoder.set_channel_layout(channel_layout);
        encoder.set_time_base(ffmpeg::Rational(1, rate));

        let encoder = encoder.open()?;

        Ok(Self {
            encoder,
            packet: ffmpeg::Packet::empty(),
            resampler,
            resampled_frame: ffmpeg::frame::Audio::empty(),
            buffer: vec![VecDeque::new(); config.channels as usize],
            config: config.clone(),
            sequence_counter: 0,
            pts: 0,
            samples_per_segment: (config.sample_rate as f64 * 2.0) as usize, // Cap's 2-second segments
            current_segment_samples: Vec::new(),
        })
    }

    /// Process audio samples following Cap's real-time pattern
    pub fn process_audio(&mut self, pcm_data: &[f32]) -> Result<Vec<EncodedAudioSegment>, AudioEncodingError> {
        let mut segments = Vec::new();
        
        // Add samples to current segment buffer
        self.current_segment_samples.extend_from_slice(pcm_data);

        // Process complete segments (Cap's 2-second segments)
        while self.current_segment_samples.len() >= self.samples_per_segment {
            let segment_data: Vec<f32> = self.current_segment_samples
                .drain(..self.samples_per_segment)
                .collect();

            let encoded_segment = self.encode_audio_segment(&segment_data)?;
            segments.push(encoded_segment);
        }

        Ok(segments)
    }

    /// Encode a single audio segment by breaking it into proper frame sizes
    fn encode_audio_segment(&mut self, pcm_data: &[f32]) -> Result<EncodedAudioSegment, AudioEncodingError> {
        let frame_size = 1024; // AAC frame size in samples per channel
        let samples_per_frame = frame_size * self.config.channels as usize;
        let mut encoded_data = Vec::new();
        
        // Process the PCM data in chunks that fit the encoder's frame size
        for chunk_start in (0..pcm_data.len()).step_by(samples_per_frame) {
            let chunk_end = (chunk_start + samples_per_frame).min(pcm_data.len());
            let chunk = &pcm_data[chunk_start..chunk_end];
            
            // Skip incomplete frames at the end (will be handled in next segment or flush)
            if chunk.len() < samples_per_frame {
                break;
            }
            
            let frame = self.create_audio_frame(chunk)?;
            let raw_frame_data = self.queue_frame(frame)?;
            
            // ✅ Add ADTS header to make it playable
            if !raw_frame_data.is_empty() {
                let adts_frame = self.add_adts_header(&raw_frame_data);
                encoded_data.extend(adts_frame);
            }
        }
        
        let segment = EncodedAudioSegment {
            data: encoded_data,
            sequence: self.sequence_counter,
            duration: 2.0, // Cap's 2-second segments
            // ✅ Use PTS-based timestamp calculation (Cap's approach)
            timestamp: (self.pts as f64 / self.config.sample_rate as f64 * 1000.0) as u64,
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
        };
        
        self.sequence_counter += 1;
        
        Ok(segment)
    }

    /// Create audio frame from PCM data with proper format consistency
    fn create_audio_frame(&mut self, pcm_data: &[f32]) -> Result<ffmpeg::frame::Audio, AudioEncodingError> {
        let samples_per_channel = pcm_data.len() / self.config.channels as usize;
        
        // AAC encoder frame size is typically 1024 samples per channel
        let frame_size = 1024;
        let actual_samples = samples_per_channel.min(frame_size);
        
        // ✅ Create frames in INPUT format (Packed) - resampler will convert to output format
        let input_format = Sample::F32(Type::Packed);
        let mut frame = ffmpeg::frame::Audio::new(
            input_format, // Use input format, not output format
            actual_samples,
            ChannelLayout::default(self.config.channels as i32),
        );
        
        frame.set_rate(self.config.sample_rate);
        frame.set_pts(Some(self.pts));
        
        // ✅ For Packed format, copy directly (interleaved data)
        let frame_data = frame.data_mut(0);
        let samples_to_copy = actual_samples * self.config.channels as usize;
        let byte_count = samples_to_copy * std::mem::size_of::<f32>();
        
        unsafe {
            let src_ptr = pcm_data.as_ptr() as *const u8;
            let dst_ptr = frame_data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, byte_count);
        }
        
        self.pts += actual_samples as i64;
        
        Ok(frame)
    }

    /// Flush remaining audio data
    pub fn flush(&mut self) -> Result<Vec<EncodedAudioSegment>, AudioEncodingError> {
        let mut segments = Vec::new();

        // Encode any remaining samples
        if !self.current_segment_samples.is_empty() {
            // Pad with silence if needed
            self.current_segment_samples.resize(self.samples_per_segment, 0.0);

            let segment_data = self.current_segment_samples.clone();
            let encoded_segment = self.encode_audio_segment(&segment_data)?;
            segments.push(encoded_segment);

            self.current_segment_samples.clear();
        }
        
        // Flush resampler if needed
        if let Some(resampler) = &mut self.resampler {
            loop {
                if resampler.delay().is_none() {
                    break;
                }
                
                if resampler.flush(&mut self.resampled_frame).is_err() {
                    break;
                }
                
                if self.resampled_frame.samples() == 0 {
                    break;
                }
                
                // Create a temporary reference to avoid borrowing issues
                let resampled_frame = &self.resampled_frame;
                let encoded_data = {
                    self.encoder.send_frame(resampled_frame)?;
                    
                    let mut data = Vec::new();
                    while self.encoder.receive_packet(&mut self.packet).is_ok() {
                        if let Some(packet_data) = self.packet.data() {
                            data.extend_from_slice(packet_data);
                        }
                    }
                    data
                };
                
                if !encoded_data.is_empty() {
                    let segment = EncodedAudioSegment {
                        data: encoded_data,
                        sequence: self.sequence_counter,
                        duration: 0.0, // Flush data
                        // ✅ Use PTS-based timestamp calculation
                        timestamp: (self.pts as f64 / self.config.sample_rate as f64 * 1000.0) as u64,
                        sample_rate: self.config.sample_rate,
                        channels: self.config.channels,
                    };
                    
                    self.sequence_counter += 1;
                    segments.push(segment);
                }
            }
        }
        
        // Send EOF to encoder
        self.encoder.send_eof()?;
        
        // Get remaining packets
        while self.encoder.receive_packet(&mut self.packet).is_ok() {
            let encoded_data = self.packet.data().unwrap_or(&[]).to_vec();
            if !encoded_data.is_empty() {
                let segment = EncodedAudioSegment {
                    data: encoded_data,
                    sequence: self.sequence_counter,
                    duration: 0.0, // Final flush
                    // ✅ Use PTS-based timestamp calculation
                    timestamp: (self.pts as f64 / self.config.sample_rate as f64 * 1000.0) as u64,
                    sample_rate: self.config.sample_rate,
                    channels: self.config.channels,
                };
                
                self.sequence_counter += 1;
                segments.push(segment);
            }
        }
        
        Ok(segments)
    }

    /// Encode a single frame (Cap's pattern)
    fn encode_frame(&mut self, frame: &ffmpeg::frame::Audio) -> Result<Vec<u8>, AudioEncodingError> {
        self.encoder.send_frame(frame)?;
        
        let mut encoded_data = Vec::new();
        while self.encoder.receive_packet(&mut self.packet).is_ok() {
            if let Some(data) = self.packet.data() {
                encoded_data.extend_from_slice(data);
            }
        }
        
        Ok(encoded_data)
    }

    /// Add ADTS headers to AAC data for proper container format (Cap's approach)
    fn add_adts_header(&self, aac_data: &[u8]) -> Vec<u8> {
        let frame_length = aac_data.len() + 7; // ADTS header is 7 bytes
        let mut adts_header = vec![0u8; 7];
        
        // ADTS fixed header
        adts_header[0] = 0xFF;
        adts_header[1] = 0xF1; // MPEG-4, Layer 0, no CRC
        
        // Profile (2 bits) + Sample rate index (4 bits) + Private bit (1 bit) + Channel config (3 bits)
        let profile = 1u8; // AAC LC
        let sample_rate_index = match self.config.sample_rate {
            96000 => 0u8,
            88200 => 1u8,
            64000 => 2u8,
            48000 => 3u8,
            44100 => 4u8,
            32000 => 5u8,
            24000 => 6u8,
            22050 => 7u8,
            16000 => 8u8,
            12000 => 9u8,
            11025 => 10u8,
            8000 => 11u8,
            _ => 4u8, // Default to 44.1kHz
        };
        
        let channels = self.config.channels as u8;
        adts_header[2] = (profile << 6) | (sample_rate_index << 2) | ((channels >> 2) & 0x1);
        adts_header[3] = ((channels & 0x3) << 6) | (((frame_length >> 11) & 0x3) as u8);
        adts_header[4] = ((frame_length >> 3) & 0xFF) as u8;
        adts_header[5] = (((frame_length & 0x7) << 5) as u8) | 0x1F;
        adts_header[6] = 0xFC;
        
        let mut result = adts_header;
        result.extend_from_slice(aac_data);
        result
    }
}

impl AudioEncoderTrait for AACEncoder {
    fn queue_frame(&mut self, frame: ffmpeg::frame::Audio) -> Result<Vec<u8>, AudioEncodingError> {
        if let Some(resampler) = &mut self.resampler {
            resampler.run(&frame, &mut self.resampled_frame)?;
            // Clone the frame to avoid borrow checker issues
            let resampled_frame = self.resampled_frame.clone();
            self.encode_frame(&resampled_frame)
        } else {
            self.encode_frame(&frame)
        }
    }

    fn finish(&mut self) -> Result<Vec<u8>, AudioEncodingError> {
        let segments = self.flush()?;
        Ok(segments.into_iter().flat_map(|s| s.data).collect())
    }
}

/// Thread-safe wrapper for the audio encoder (Cap's pattern)
#[derive(Clone)]
pub struct AudioEncoder {
    inner: Arc<Mutex<AACEncoder>>,
    config: AudioEncodingConfig,
}

impl AudioEncoder {
    /// Create new audio encoder with real FFmpeg AAC encoding
    pub fn new(config: AudioEncodingConfig) -> CaptureResult<Self> {
        log::info!("Initializing real FFmpeg audio encoder with config: {:?}", config);
        
        let encoder = AACEncoder::new(config.clone())
            .map_err(|e| CaptureError::from(e))?;
        
        log::info!("FFmpeg audio encoder initialized successfully");
        
        Ok(Self {
            inner: Arc::new(Mutex::new(encoder)),
            config,
        })
    }

    /// Process audio samples and encode to AAC segments (Cap's real-time approach)
    pub fn process_audio(&mut self, pcm_data: &[f32]) -> CaptureResult<Vec<EncodedAudioSegment>> {
        let mut inner = self.inner.lock().map_err(|e| {
            CaptureError::EncodingError(format!("Failed to acquire encoder lock: {}", e))
        })?;

        let segments = inner.process_audio(pcm_data)
            .map_err(|e| CaptureError::from(e))?;

        Ok(segments)
    }

    /// Flush any remaining audio data (final cleanup)
    pub fn flush(&mut self) -> CaptureResult<Vec<EncodedAudioSegment>> {
        let mut inner = self.inner.lock().map_err(|e| {
            CaptureError::EncodingError(format!("Failed to acquire encoder lock: {}", e))
        })?;

        let segments = inner.flush()
            .map_err(|e| CaptureError::from(e))?;

        log::debug!("Flushed audio encoder with {} remaining segments", segments.len());
        Ok(segments)
    }
}

impl Drop for AudioEncoder {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            log::error!("Error flushing audio encoder during drop: {}", e);
        }
    }
}

/// Factory function following Cap's pattern
pub fn create_aac_encoder(config: AudioEncodingConfig) -> CaptureResult<AudioEncoder> {
    AudioEncoder::new(config)
}

/// Create transcription-optimized encoder following Cap's defaults
pub fn create_transcription_encoder() -> CaptureResult<AudioEncoder> {
    let config = AudioEncodingConfig {
        codec: AudioCodec::AAC,
        bitrate: 128000,     // 128kbps for transcription
        sample_rate: 48000,  // Cap's standard
        channels: 2,         // Stereo
        channel_layout: AudioChannelLayout::Stereo,
    };
    
    create_aac_encoder(config)
}

/// Simple encoder wrapper for easy usage
pub struct SimpleAudioEncoder {
    inner: AudioEncoder,
}

impl SimpleAudioEncoder {
    pub fn new() -> CaptureResult<Self> {
        Ok(Self {
            inner: create_transcription_encoder()?,
        })
    }

    pub fn encode(&mut self, pcm_data: &[f32]) -> CaptureResult<Vec<EncodedAudioSegment>> {
        self.inner.process_audio(pcm_data)
    }

    pub fn finish(mut self) -> CaptureResult<Vec<EncodedAudioSegment>> {
        self.inner.flush()
    }
}
