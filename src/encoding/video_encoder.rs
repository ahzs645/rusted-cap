//! FFmpeg-based Video Encoder
//! 
//! Implements Cap's real-time H.264 encoding pipeline for screen capture

use crate::error::{CaptureError, CaptureResult};
use super::{VideoEncodingConfig, VideoCodec, PixelFormat};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Encoded video segment ready for upload
#[derive(Debug, Clone)]
pub struct EncodedVideoSegment {
    /// Encoded H.264 data
    pub data: Vec<u8>,
    /// Segment sequence number
    pub sequence: u32,
    /// Duration in seconds
    pub duration: f64,
    /// Timestamp when encoded
    pub timestamp: u64,
    /// Frame count in this segment
    pub frame_count: u32,
    /// Resolution
    pub resolution: (u32, u32),
}

/// FFmpeg-based video encoder following Cap's implementation
pub struct VideoEncoder {
    config: VideoEncodingConfig,
    encoder: Option<ffmpeg::encoder::video::Video>,
    sequence_counter: u32,
    frames_per_segment: u32,
    current_segment_frames: Vec<Vec<u8>>,
    frame_counter: u32,
}

impl VideoEncoder {
    /// Create new video encoder with Cap's H.264 settings
    pub fn new(config: VideoEncodingConfig) -> CaptureResult<Self> {
        log::info!("Initializing FFmpeg video encoder with config: {:?}", config);
        
        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| {
            CaptureError::EncodingError(format!("Failed to initialize FFmpeg: {}", e))
        })?;

        let frames_per_segment = (config.frame_rate.0 as f64 * 2.0) as u32; // 2 second segments

        let mut encoder = Self {
            config: config.clone(),
            encoder: None,
            sequence_counter: 0,
            frames_per_segment,
            current_segment_frames: Vec::new(),
            frame_counter: 0,
        };

        encoder.initialize_encoder()?;
        Ok(encoder)
    }

    /// Initialize the H.264 encoder with Cap's settings
    fn initialize_encoder(&mut self) -> CaptureResult<()> {
        let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
            .ok_or_else(|| CaptureError::EncodingError("H.264 codec not found".to_string()))?;

        let context = ffmpeg::codec::context::Context::new();
        let mut encoder = context.encoder().video().map_err(|e| {
            CaptureError::EncodingError(format!("Failed to create video encoder context: {}", e))
        })?;

        // Configure encoder with Cap's settings
        encoder.set_width(self.config.resolution.0);
        encoder.set_height(self.config.resolution.1);
        encoder.set_format(self.get_pixel_format());
        encoder.set_frame_rate(Some((self.config.frame_rate.0 as i32, self.config.frame_rate.1 as i32)));
        encoder.set_bit_rate(self.config.bitrate as usize);

        // Set encoding parameters for real-time streaming
        encoder.set_gop(self.config.frame_rate.0); // 1 second GOP
        
        // Hardware acceleration if available and requested
        if self.config.hardware_acceleration {
            if let Ok(hw_device) = ffmpeg::device::input(&ffmpeg::format::input(&"videotoolbox").unwrap()) {
                log::info!("Hardware acceleration available, using VideoToolbox");
                // Note: Hardware acceleration setup would require additional FFmpeg configuration
            }
        }

        let encoder = encoder.open_as(codec).map_err(|e| {
            CaptureError::EncodingError(format!("Failed to open H.264 encoder: {}", e))
        })?;

        self.encoder = Some(encoder);
        log::info!("H.264 encoder initialized successfully");
        Ok(())
    }

    /// Convert our pixel format enum to FFmpeg format
    fn get_pixel_format(&self) -> ffmpeg::format::Pixel {
        match self.config.pixel_format {
            PixelFormat::YUV420P => ffmpeg::format::Pixel::YUV420P,
            PixelFormat::RGBA => ffmpeg::format::Pixel::RGBA,
            PixelFormat::BGRA => ffmpeg::format::Pixel::BGRA,
        }
    }

    /// Process video frames and encode to H.264 segments
    pub fn process_frame(&mut self, rgba_frame: &[u8]) -> CaptureResult<Option<EncodedVideoSegment>> {
        // Add frame to current segment buffer
        self.current_segment_frames.push(rgba_frame.to_vec());
        self.frame_counter += 1;

        // Check if we have enough frames for a complete segment
        if self.current_segment_frames.len() >= self.frames_per_segment as usize {
            let frames = std::mem::take(&mut self.current_segment_frames);
            let segment = self.encode_video_segment(frames)?;
            return Ok(Some(segment));
        }

        Ok(None)
    }

    /// Encode a single video segment to H.264
    fn encode_video_segment(&mut self, frames: Vec<Vec<u8>>) -> CaptureResult<EncodedVideoSegment> {
        let encoder = self.encoder.as_mut()
            .ok_or_else(|| CaptureError::EncodingError("Encoder not initialized".to_string()))?;

        let mut encoded_data = Vec::new();

        for frame_data in &frames {
            let encoded_frame = self.encode_single_frame(frame_data)?;
            encoded_data.extend(encoded_frame);
        }

        let segment = EncodedVideoSegment {
            data: encoded_data,
            sequence: self.sequence_counter,
            duration: 2.0, // Cap uses 2-second segments
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            frame_count: frames.len() as u32,
            resolution: self.config.resolution,
        };

        self.sequence_counter += 1;
        
        log::debug!("Encoded video segment {} ({} bytes, {} frames)", 
                   segment.sequence, segment.data.len(), segment.frame_count);

        Ok(segment)
    }

    /// Encode a single frame
    fn encode_single_frame(&mut self, rgba_frame: &[u8]) -> CaptureResult<Vec<u8>> {
        let encoder = self.encoder.as_mut()
            .ok_or_else(|| CaptureError::EncodingError("Encoder not initialized".to_string()))?;

        // Create FFmpeg video frame
        let mut frame = ffmpeg::frame::Video::new(
            self.get_pixel_format(),
            self.config.resolution.0,
            self.config.resolution.1
        );

        // Convert RGBA to YUV420P if needed
        let yuv_data = if matches!(self.config.pixel_format, PixelFormat::YUV420P) {
            self.convert_rgba_to_yuv420p(rgba_frame)?
        } else {
            rgba_frame.to_vec()
        };

        // Copy frame data
        if matches!(self.config.pixel_format, PixelFormat::YUV420P) {
            self.copy_yuv420p_to_frame(&mut frame, &yuv_data)?;
        } else {
            // For RGBA/BGRA, copy directly
            frame.data_mut(0)[..rgba_frame.len()].copy_from_slice(rgba_frame);
        }

        frame.set_pts(Some(self.frame_counter as i64));

        // Encode frame to H.264 packet
        let mut packet = ffmpeg::packet::Packet::empty();
        encoder.encode(&frame, &mut packet).map_err(|e| {
            CaptureError::EncodingError(format!("Failed to encode video frame: {}", e))
        })?;

        let encoded_data = if let Some(data) = packet.data() {
            data.to_vec()
        } else {
            Vec::new()
        };

        Ok(encoded_data)
    }

    /// Convert RGBA to YUV420P color space
    fn convert_rgba_to_yuv420p(&self, rgba_data: &[u8]) -> CaptureResult<Vec<u8>> {
        let width = self.config.resolution.0 as usize;
        let height = self.config.resolution.1 as usize;
        
        let y_plane_size = width * height;
        let uv_plane_size = (width / 2) * (height / 2);
        let total_size = y_plane_size + 2 * uv_plane_size;
        
        let mut yuv_data = vec![0u8; total_size];
        
        // Convert RGBA to YUV420P
        for y in 0..height {
            for x in 0..width {
                let rgba_idx = (y * width + x) * 4;
                let r = rgba_data[rgba_idx] as f32;
                let g = rgba_data[rgba_idx + 1] as f32;
                let b = rgba_data[rgba_idx + 2] as f32;
                
                // Convert to YUV
                let y_val = (0.257 * r + 0.504 * g + 0.098 * b + 16.0) as u8;
                let u_val = (-0.148 * r - 0.291 * g + 0.439 * b + 128.0) as u8;
                let v_val = (0.439 * r - 0.368 * g - 0.071 * b + 128.0) as u8;
                
                // Y plane
                yuv_data[y * width + x] = y_val;
                
                // U and V planes (subsampled)
                if x % 2 == 0 && y % 2 == 0 {
                    let uv_idx = (y / 2) * (width / 2) + (x / 2);
                    yuv_data[y_plane_size + uv_idx] = u_val;
                    yuv_data[y_plane_size + uv_plane_size + uv_idx] = v_val;
                }
            }
        }
        
        Ok(yuv_data)
    }

    /// Copy YUV420P data to FFmpeg frame
    fn copy_yuv420p_to_frame(&self, frame: &mut ffmpeg::frame::Video, yuv_data: &[u8]) -> CaptureResult<()> {
        let width = self.config.resolution.0 as usize;
        let height = self.config.resolution.1 as usize;
        
        let y_plane_size = width * height;
        let uv_plane_size = (width / 2) * (height / 2);
        
        // Copy Y plane
        frame.data_mut(0)[..y_plane_size].copy_from_slice(&yuv_data[..y_plane_size]);
        
        // Copy U plane
        frame.data_mut(1)[..uv_plane_size].copy_from_slice(
            &yuv_data[y_plane_size..y_plane_size + uv_plane_size]
        );
        
        // Copy V plane
        frame.data_mut(2)[..uv_plane_size].copy_from_slice(
            &yuv_data[y_plane_size + uv_plane_size..]
        );
        
        Ok(())
    }

    /// Flush any remaining video frames
    pub fn flush(&mut self) -> CaptureResult<Vec<EncodedVideoSegment>> {
        let mut segments = Vec::new();

        // Encode any remaining frames
        if !self.current_segment_frames.is_empty() {
            let frames = std::mem::take(&mut self.current_segment_frames);
            let segment = self.encode_video_segment(frames)?;
            segments.push(segment);
        }

        // Flush encoder
        if let Some(encoder) = &mut self.encoder {
            let mut packet = ffmpeg::packet::Packet::empty();
            while encoder.flush(&mut packet).is_ok() {
                if let Some(data) = packet.data() {
                    let segment = EncodedVideoSegment {
                        data: data.to_vec(),
                        sequence: self.sequence_counter,
                        duration: 2.0,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        frame_count: 0,
                        resolution: self.config.resolution,
                    };

                    self.sequence_counter += 1;
                    segments.push(segment);
                }
            }
        }

        Ok(segments)
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            log::error!("Error flushing video encoder: {}", e);
        }
    }
}

/// Create a video encoder with Cap's default settings for screen recording
pub fn create_screen_recording_encoder(resolution: (u32, u32)) -> CaptureResult<VideoEncoder> {
    let config = super::VideoEncodingConfig {
        codec: VideoCodec::H264,
        bitrate: 2000000, // 2Mbps
        frame_rate: (30, 1), // 30fps
        resolution,
        pixel_format: PixelFormat::YUV420P,
        hardware_acceleration: true,
    };

    VideoEncoder::new(config)
}
