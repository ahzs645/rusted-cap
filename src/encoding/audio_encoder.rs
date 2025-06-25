//! FFmpeg-based Audio Encoder
//! 
//! Implements Cap's real-time AAC encoding pipeline for audio transcription

use crate::error::{CaptureError, CaptureResult};
use super::{AudioEncodingConfig, AudioCodec, AudioChannelLayout};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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

/// FFmpeg-based audio encoder following Cap's implementation
pub struct AudioEncoder {
    config: AudioEncodingConfig,
    encoder: Option<ffmpeg::encoder::audio::Audio>,
    sequence_counter: u32,
    samples_per_segment: usize,
    current_segment_samples: Vec<f32>,
}

impl AudioEncoder {
    /// Create new audio encoder with Cap's AAC settings
    pub fn new(config: AudioEncodingConfig) -> CaptureResult<Self> {
        log::info!("Initializing FFmpeg audio encoder with config: {:?}", config);
        
        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| {
            CaptureError::EncodingError(format!("Failed to initialize FFmpeg: {}", e))
        })?;

        let mut encoder = Self {
            config: config.clone(),
            encoder: None,
            sequence_counter: 0,
            samples_per_segment: (config.sample_rate as f64 * 2.0) as usize, // 2 second segments
            current_segment_samples: Vec::new(),
        };

        encoder.initialize_encoder()?;
        Ok(encoder)
    }

    /// Initialize the AAC encoder with Cap's settings
    fn initialize_encoder(&mut self) -> CaptureResult<()> {
        let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::AAC)
            .ok_or_else(|| CaptureError::EncodingError("AAC codec not found".to_string()))?;

        let context = ffmpeg::codec::context::Context::new();
        let mut encoder = context.encoder().audio().map_err(|e| {
            CaptureError::EncodingError(format!("Failed to create audio encoder context: {}", e))
        })?;

        // Configure encoder with Cap's settings
        encoder.set_bit_rate(self.config.bitrate as usize);
        encoder.set_sample_rate(self.config.sample_rate as i32);
        encoder.set_channels(self.config.channels as i32);
        
        // Set channel layout
        let channel_layout = match self.config.channel_layout {
            AudioChannelLayout::Mono => ffmpeg::channel_layout::MONO,
            AudioChannelLayout::Stereo => ffmpeg::channel_layout::STEREO,
            AudioChannelLayout::Surround51 => ffmpeg::channel_layout::_5POINT1,
        };
        encoder.set_channel_layout(channel_layout);
        
        // Set sample format (floating point)
        encoder.set_format(ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar));

        let encoder = encoder.open_as(codec).map_err(|e| {
            CaptureError::EncodingError(format!("Failed to open AAC encoder: {}", e))
        })?;

        self.encoder = Some(encoder);
        log::info!("AAC encoder initialized successfully");
        Ok(())
    }

    /// Process audio samples and encode to AAC segments
    pub fn process_audio(&mut self, pcm_data: &[f32]) -> CaptureResult<Vec<EncodedAudioSegment>> {
        let mut segments = Vec::new();
        
        // Add samples to current segment buffer
        self.current_segment_samples.extend_from_slice(pcm_data);

        // Check if we have enough samples for a complete segment
        while self.current_segment_samples.len() >= self.samples_per_segment {
            let segment_data: Vec<f32> = self.current_segment_samples
                .drain(..self.samples_per_segment)
                .collect();

            let encoded_segment = self.encode_audio_segment(&segment_data)?;
            segments.push(encoded_segment);
        }

        Ok(segments)
    }

    /// Encode a single audio segment to AAC
    fn encode_audio_segment(&mut self, pcm_data: &[f32]) -> CaptureResult<EncodedAudioSegment> {
        let encoder = self.encoder.as_mut()
            .ok_or_else(|| CaptureError::EncodingError("Encoder not initialized".to_string()))?;

        // Create FFmpeg audio frame
        let mut frame = ffmpeg::frame::Audio::new(
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
            pcm_data.len() / self.config.channels as usize,
            ffmpeg::channel_layout::STEREO
        );

        frame.set_rate(self.config.sample_rate as i32);

        // Copy PCM data to frame
        // For stereo, we need to split interleaved data into planar format
        if self.config.channels == 2 {
            let samples_per_channel = pcm_data.len() / 2;
            let mut left_channel = Vec::with_capacity(samples_per_channel);
            let mut right_channel = Vec::with_capacity(samples_per_channel);

            for i in 0..samples_per_channel {
                left_channel.push(pcm_data[i * 2]);
                right_channel.push(pcm_data[i * 2 + 1]);
            }

            // Copy to frame planes
            frame.plane_mut::<f32>(0)[..samples_per_channel].copy_from_slice(&left_channel);
            frame.plane_mut::<f32>(1)[..samples_per_channel].copy_from_slice(&right_channel);
        } else {
            // Mono
            frame.plane_mut::<f32>(0)[..pcm_data.len()].copy_from_slice(pcm_data);
        }

        // Encode frame to AAC packet
        let mut packet = ffmpeg::packet::Packet::empty();
        encoder.encode(&frame, &mut packet).map_err(|e| {
            CaptureError::EncodingError(format!("Failed to encode audio frame: {}", e))
        })?;

        let encoded_data = if let Some(data) = packet.data() {
            data.to_vec()
        } else {
            Vec::new()
        };

        let segment = EncodedAudioSegment {
            data: encoded_data,
            sequence: self.sequence_counter,
            duration: 2.0, // Cap uses 2-second segments
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
        };

        self.sequence_counter += 1;
        
        log::debug!("Encoded audio segment {} ({} bytes)", 
                   segment.sequence, segment.data.len());

        Ok(segment)
    }

    /// Flush any remaining audio data
    pub fn flush(&mut self) -> CaptureResult<Vec<EncodedAudioSegment>> {
        let mut segments = Vec::new();

        // Encode any remaining samples
        if !self.current_segment_samples.is_empty() {
            // Pad with silence if needed
            let remaining_samples = self.samples_per_segment - self.current_segment_samples.len();
            self.current_segment_samples.resize(self.samples_per_segment, 0.0);

            let segment_data = self.current_segment_samples.clone();
            let encoded_segment = self.encode_audio_segment(&segment_data)?;
            segments.push(encoded_segment);

            self.current_segment_samples.clear();
        }

        // Flush encoder
        if let Some(encoder) = &mut self.encoder {
            let mut packet = ffmpeg::packet::Packet::empty();
            while encoder.flush(&mut packet).is_ok() {
                if let Some(data) = packet.data() {
                    let segment = EncodedAudioSegment {
                        data: data.to_vec(),
                        sequence: self.sequence_counter,
                        duration: 2.0,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        sample_rate: self.config.sample_rate,
                        channels: self.config.channels,
                    };

                    self.sequence_counter += 1;
                    segments.push(segment);
                }
            }
        }

        Ok(segments)
    }
}

impl Drop for AudioEncoder {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            log::error!("Error flushing audio encoder: {}", e);
        }
    }
}

/// Create an audio encoder with Cap's default settings for transcription
pub fn create_transcription_encoder() -> CaptureResult<AudioEncoder> {
    let config = super::AudioEncodingConfig {
        codec: AudioCodec::AAC,
        bitrate: 128000, // 128kbps - optimal for transcription
        sample_rate: 48000,
        channels: 2,
        channel_layout: AudioChannelLayout::Stereo,
    };

    AudioEncoder::new(config)
}
