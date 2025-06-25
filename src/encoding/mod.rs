//! Cap Electron Capture - Encoding Module
//! 
//! This module implements Cap's FFmpeg-based encoding pipeline for both
//! audio and video processing, following their real-time HLS streaming approach.

pub mod audio_encoder;
pub mod video_encoder;
pub mod hls;
pub mod s3_uploader;

pub use audio_encoder::{AudioEncoder, EncodedAudioSegment};
pub use video_encoder::{VideoEncoder, EncodedVideoSegment};
pub use hls::{HLSSegmenter, HLSSegment, HLSPlaylist};
pub use s3_uploader::{S3Uploader, UploadConfig};

use crate::error::CaptureResult;
use serde::{Deserialize, Serialize};

/// Encoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    /// Audio encoding settings
    pub audio: AudioEncodingConfig,
    /// Video encoding settings
    pub video: VideoEncodingConfig,
    /// HLS segmentation settings
    pub hls: HLSConfig,
    /// Upload settings
    pub upload: Option<UploadConfig>,
}

/// Audio encoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEncodingConfig {
    /// Audio codec (AAC)
    pub codec: AudioCodec,
    /// Bitrate in bits per second
    pub bitrate: u32,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Channel layout
    pub channel_layout: AudioChannelLayout,
}

/// Video encoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEncodingConfig {
    /// Video codec (H.264)
    pub codec: VideoCodec,
    /// Bitrate in bits per second
    pub bitrate: u32,
    /// Frame rate (fps)
    pub frame_rate: (u32, u32),
    /// Video resolution
    pub resolution: (u32, u32),
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Hardware acceleration
    pub hardware_acceleration: bool,
}

/// HLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HLSConfig {
    /// Segment duration in seconds
    pub segment_duration: f64,
    /// Target duration for playlist
    pub target_duration: u32,
    /// Number of segments to keep in playlist
    pub playlist_size: usize,
}

/// Audio codec options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCodec {
    AAC,
}

/// Video codec options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
}

/// Audio channel layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioChannelLayout {
    Mono,
    Stereo,
    Surround51,
}

/// Pixel format for video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PixelFormat {
    YUV420P,
    RGBA,
    BGRA,
}

impl Default for EncodingConfig {
    fn default() -> Self {
        Self {
            audio: AudioEncodingConfig::default(),
            video: VideoEncodingConfig::default(),
            hls: HLSConfig::default(),
            upload: None,
        }
    }
}

impl Default for AudioEncodingConfig {
    fn default() -> Self {
        Self {
            codec: AudioCodec::AAC,
            bitrate: 128000, // 128kbps for transcription
            sample_rate: 48000,
            channels: 2,
            channel_layout: AudioChannelLayout::Stereo,
        }
    }
}

impl Default for VideoEncodingConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            bitrate: 2000000, // 2Mbps
            frame_rate: (30, 1), // 30fps
            resolution: (1920, 1080),
            pixel_format: PixelFormat::YUV420P,
            hardware_acceleration: true,
        }
    }
}

impl Default for HLSConfig {
    fn default() -> Self {
        Self {
            segment_duration: 2.0, // 2-second segments like Cap
            target_duration: 2,
            playlist_size: 5,
        }
    }
}
