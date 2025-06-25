//! HLS Segmentation and Playlist Generation
//! 
//! Implements Cap's HLS streaming approach with real-time segment management

use crate::error::{CaptureError, CaptureResult};
use super::{HLSConfig, EncodedAudioSegment, EncodedVideoSegment};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// HLS segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HLSSegment {
    /// Segment sequence number
    pub sequence_number: u32,
    /// Segment duration in seconds
    pub duration: f64,
    /// Video segment URL/path
    pub video_url: String,
    /// Audio segment URL/path
    pub audio_url: String,
    /// Combined segment URL/path (optional)
    pub combined_url: Option<String>,
    /// Timestamp when segment was created
    pub timestamp: u64,
    /// Byte size of video data
    pub video_size: usize,
    /// Byte size of audio data
    pub audio_size: usize,
}

/// HLS playlist following Cap's structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HLSPlaylist {
    /// Playlist version
    pub version: u32,
    /// Target duration in seconds
    pub target_duration: u32,
    /// Media sequence number
    pub media_sequence: u32,
    /// List of segments
    pub segments: Vec<HLSSegment>,
    /// Whether the playlist is complete
    pub end_list: bool,
}

/// HLS segmenter following Cap's approach
pub struct HLSSegmenter {
    config: HLSConfig,
    segments: VecDeque<HLSSegment>,
    sequence_counter: u32,
    user_id: String,
    video_id: String,
}

impl HLSSegmenter {
    /// Create new HLS segmenter
    pub fn new(config: HLSConfig, user_id: String, video_id: String) -> Self {
        Self {
            config,
            segments: VecDeque::new(),
            sequence_counter: 0,
            user_id,
            video_id,
        }
    }

    /// Create HLS segment from encoded audio and video data
    pub fn create_hls_segment(
        &mut self,
        audio_segment: EncodedAudioSegment,
        video_segment: Option<EncodedVideoSegment>,
    ) -> CaptureResult<HLSSegment> {
        let duration = audio_segment.duration.max(
            video_segment.as_ref().map(|v| v.duration).unwrap_or(0.0)
        );

        let segment = HLSSegment {
            sequence_number: self.sequence_counter,
            duration,
            video_url: if video_segment.is_some() {
                format!("video/video_recording_{}.ts", self.sequence_counter)
            } else {
                String::new()
            },
            audio_url: format!("audio/audio_recording_{}.aac", self.sequence_counter),
            combined_url: if video_segment.is_some() {
                Some(format!("combined-source/segment_{}.ts", self.sequence_counter))
            } else {
                None
            },
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            video_size: video_segment.as_ref().map(|v| v.data.len()).unwrap_or(0),
            audio_size: audio_segment.data.len(),
        };

        // Add to segments queue
        self.segments.push_back(segment.clone());

        // Maintain playlist size limit
        while self.segments.len() > self.config.playlist_size {
            self.segments.pop_front();
        }

        self.sequence_counter += 1;

        log::debug!("Created HLS segment {} (duration: {:.2}s, video: {} bytes, audio: {} bytes)",
                   segment.sequence_number, segment.duration, segment.video_size, segment.audio_size);

        Ok(segment)
    }

    /// Generate HLS playlist in M3U8 format following Cap's structure
    pub fn generate_m3u8_playlist(&self, playlist_type: PlaylistType) -> String {
        let mut playlist = String::new();

        // M3U8 header
        playlist.push_str("#EXTM3U\n");
        playlist.push_str("#EXT-X-VERSION:3\n");
        playlist.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", self.config.target_duration));

        // Media sequence (oldest segment sequence)
        if let Some(first_segment) = self.segments.front() {
            playlist.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", first_segment.sequence_number));
        }

        // Add segments
        for segment in &self.segments {
            playlist.push_str(&format!("#EXTINF:{:.3},\n", segment.duration));
            
            match playlist_type {
                PlaylistType::Video => {
                    if !segment.video_url.is_empty() {
                        playlist.push_str(&format!("{}\n", segment.video_url));
                    }
                },
                PlaylistType::Audio => {
                    playlist.push_str(&format!("{}\n", segment.audio_url));
                },
                PlaylistType::Combined => {
                    if let Some(combined_url) = &segment.combined_url {
                        playlist.push_str(&format!("{}\n", combined_url));
                    }
                },
            }
        }

        playlist
    }

    /// Generate master playlist for multi-stream playback
    pub fn generate_master_playlist(&self) -> String {
        let mut playlist = String::new();

        playlist.push_str("#EXTM3U\n");
        playlist.push_str("#EXT-X-VERSION:3\n");

        // Video stream
        playlist.push_str("#EXT-X-STREAM-INF:BANDWIDTH=2000000,RESOLUTION=1920x1080\n");
        playlist.push_str("video/stream.m3u8\n");

        // Audio stream
        playlist.push_str("#EXT-X-STREAM-INF:BANDWIDTH=128000\n");
        playlist.push_str("audio/stream.m3u8\n");

        playlist
    }

    /// Get current segments
    pub fn get_segments(&self) -> Vec<HLSSegment> {
        self.segments.iter().cloned().collect()
    }

    /// Get the latest segment
    pub fn get_latest_segment(&self) -> Option<&HLSSegment> {
        self.segments.back()
    }

    /// Generate S3 key for segment
    pub fn generate_s3_key(&self, segment: &HLSSegment, content_type: S3ContentType) -> String {
        match content_type {
            S3ContentType::VideoSegment => {
                format!("{}/{}/video/video_recording_{}.ts", 
                       self.user_id, self.video_id, segment.sequence_number)
            },
            S3ContentType::AudioSegment => {
                format!("{}/{}/audio/audio_recording_{}.aac", 
                       self.user_id, self.video_id, segment.sequence_number)
            },
            S3ContentType::CombinedSegment => {
                format!("{}/{}/combined-source/segment_{}.ts", 
                       self.user_id, self.video_id, segment.sequence_number)
            },
            S3ContentType::VideoPlaylist => {
                format!("{}/{}/video/stream.m3u8", self.user_id, self.video_id)
            },
            S3ContentType::AudioPlaylist => {
                format!("{}/{}/audio/stream.m3u8", self.user_id, self.video_id)
            },
            S3ContentType::CombinedPlaylist => {
                format!("{}/{}/combined-source/stream.m3u8", self.user_id, self.video_id)
            },
            S3ContentType::MasterPlaylist => {
                format!("{}/{}/stream.m3u8", self.user_id, self.video_id)
            },
        }
    }

    /// Clear all segments (for cleanup)
    pub fn clear_segments(&mut self) {
        self.segments.clear();
        self.sequence_counter = 0;
    }
}

/// Type of HLS playlist to generate
#[derive(Debug, Clone)]
pub enum PlaylistType {
    Video,
    Audio,
    Combined,
}

/// S3 content type for different segment types
#[derive(Debug, Clone)]
pub enum S3ContentType {
    VideoSegment,
    AudioSegment,
    CombinedSegment,
    VideoPlaylist,
    AudioPlaylist,
    CombinedPlaylist,
    MasterPlaylist,
}

impl S3ContentType {
    /// Get MIME type for S3 upload
    pub fn mime_type(&self) -> &'static str {
        match self {
            S3ContentType::VideoSegment | S3ContentType::CombinedSegment => "video/mp2t",
            S3ContentType::AudioSegment => "audio/aac",
            S3ContentType::VideoPlaylist | 
            S3ContentType::AudioPlaylist | 
            S3ContentType::CombinedPlaylist | 
            S3ContentType::MasterPlaylist => "application/vnd.apple.mpegurl",
        }
    }
}

/// Create HLS segmenter with Cap's default settings
pub fn create_cap_hls_segmenter(user_id: String, video_id: String) -> HLSSegmenter {
    let config = HLSConfig {
        segment_duration: 2.0, // 2-second segments like Cap
        target_duration: 2,
        playlist_size: 5, // Keep last 5 segments
    };

    HLSSegmenter::new(config, user_id, video_id)
}
