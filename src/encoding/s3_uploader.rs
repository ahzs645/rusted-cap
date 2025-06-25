//! S3 Uploader for Real-time Streaming
//! 
//! Implements Cap's S3 upload strategy for real-time HLS streaming

use crate::error::{CaptureError, CaptureResult};
use super::{EncodedAudioSegment, EncodedVideoSegment, HLSSegment, S3ContentType};
use aws_sdk_s3::{Client, config::Region};
use aws_config::load_from_env;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

/// S3 upload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// S3 bucket name
    pub bucket: String,
    /// AWS region
    pub region: String,
    /// Upload timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to use accelerated transfer
    pub accelerated_transfer: bool,
    /// Custom endpoint (for S3-compatible services)
    pub endpoint: Option<String>,
}

/// S3 uploader for real-time streaming
pub struct S3Uploader {
    client: Client,
    config: UploadConfig,
    user_id: String,
    video_id: String,
}

impl S3Uploader {
    /// Create new S3 uploader
    pub async fn new(config: UploadConfig, user_id: String, video_id: String) -> CaptureResult<Self> {
        log::info!("Initializing S3 uploader for user {} video {}", user_id, video_id);

        // Load AWS configuration
        let aws_config = load_from_env().await;
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);

        // Set region
        s3_config_builder = s3_config_builder.region(Region::new(config.region.clone()));

        // Set custom endpoint if provided
        if let Some(endpoint) = &config.endpoint {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
        }

        let s3_config = s3_config_builder.build();
        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            config,
            user_id,
            video_id,
        })
    }

    /// Upload audio segment immediately (real-time streaming)
    pub async fn upload_audio_segment_realtime(&self, segment: EncodedAudioSegment) -> CaptureResult<String> {
        let key = format!("{}/{}/audio/audio_recording_{}.aac", 
                         self.user_id, self.video_id, segment.sequence);

        self.upload_data_with_timeout(
            &key,
            segment.data,
            S3ContentType::AudioSegment.mime_type()
        ).await?;

        log::debug!("Uploaded audio segment {} to S3: {}", segment.sequence, key);
        Ok(key)
    }

    /// Upload video segment immediately (real-time streaming)
    pub async fn upload_video_segment_realtime(&self, segment: EncodedVideoSegment) -> CaptureResult<String> {
        let key = format!("{}/{}/video/video_recording_{}.ts", 
                         self.user_id, self.video_id, segment.sequence);

        self.upload_data_with_timeout(
            &key,
            segment.data,
            S3ContentType::VideoSegment.mime_type()
        ).await?;

        log::debug!("Uploaded video segment {} to S3: {}", segment.sequence, key);
        Ok(key)
    }

    /// Upload combined audio+video segment
    pub async fn upload_combined_segment(&self, 
                                       audio_segment: EncodedAudioSegment,
                                       video_segment: EncodedVideoSegment) -> CaptureResult<String> {
        // For combined segments, we need to mux audio and video
        // This is a simplified approach - in production, use FFmpeg muxing
        let combined_data = self.combine_audio_video(audio_segment.data, video_segment.data)?;
        
        let key = format!("{}/{}/combined-source/segment_{}.ts", 
                         self.user_id, self.video_id, video_segment.sequence);

        self.upload_data_with_timeout(
            &key,
            combined_data,
            S3ContentType::CombinedSegment.mime_type()
        ).await?;

        log::debug!("Uploaded combined segment {} to S3: {}", video_segment.sequence, key);
        Ok(key)
    }

    /// Update HLS playlist after new segment
    pub async fn update_playlist(&self, playlist_content: String, content_type: S3ContentType) -> CaptureResult<String> {
        let key = match content_type {
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
            _ => return Err(CaptureError::Upload("Invalid playlist content type".to_string())),
        };

        self.upload_data_with_timeout(
            &key,
            playlist_content.into_bytes(),
            content_type.mime_type()
        ).await?;

        log::debug!("Updated playlist: {}", key);
        Ok(key)
    }

    /// Upload data with timeout (for real-time guarantees)
    async fn upload_data_with_timeout(&self, key: &str, data: Vec<u8>, content_type: &str) -> CaptureResult<()> {
        let upload_future = self.client
            .put_object()
            .bucket(&self.config.bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send();

        // Apply timeout for real-time streaming requirements
        timeout(Duration::from_secs(self.config.timeout_seconds), upload_future)
            .await
            .map_err(|_| CaptureError::Upload(format!("Upload timeout for key: {}", key)))?
            .map_err(|e| CaptureError::Upload(format!("Failed to upload {}: {}", key, e)))?;

        Ok(())
    }

    /// Simple audio+video combination (in production, use proper muxing)
    fn combine_audio_video(&self, audio_data: Vec<u8>, video_data: Vec<u8>) -> CaptureResult<Vec<u8>> {
        // This is a simplified approach for demonstration
        // In production, use FFmpeg to properly mux audio and video streams
        let mut combined = Vec::with_capacity(audio_data.len() + video_data.len());
        combined.extend(video_data); // Video first
        combined.extend(audio_data); // Audio after
        Ok(combined)
    }

    /// Batch upload multiple segments (for efficiency)
    pub async fn batch_upload_segments(&self, 
                                     audio_segments: Vec<EncodedAudioSegment>,
                                     video_segments: Vec<EncodedVideoSegment>) -> CaptureResult<Vec<String>> {
        let mut uploaded_keys = Vec::new();

        // Upload in parallel for better performance
        let upload_futures = audio_segments.into_iter().map(|segment| {
            self.upload_audio_segment_realtime(segment)
        }).chain(video_segments.into_iter().map(|segment| {
            self.upload_video_segment_realtime(segment)
        }));

        let results = futures::future::join_all(upload_futures).await;

        for result in results {
            match result {
                Ok(key) => uploaded_keys.push(key),
                Err(e) => {
                    log::error!("Failed to upload segment: {}", e);
                    return Err(e);
                }
            }
        }

        log::info!("Batch uploaded {} segments", uploaded_keys.len());
        Ok(uploaded_keys)
    }

    /// Generate presigned URL for client-side access
    pub async fn generate_presigned_url(&self, key: &str, expiry_seconds: u64) -> CaptureResult<String> {
        let request = self.client
            .get_object()
            .bucket(&self.config.bucket)
            .key(key);

        let presigned_url = request
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    Duration::from_secs(expiry_seconds)
                ).map_err(|e| CaptureError::Upload(format!("Failed to create presigning config: {}", e)))?
            )
            .await
            .map_err(|e| CaptureError::Upload(format!("Failed to generate presigned URL: {}", e)))?;

        Ok(presigned_url.uri().to_string())
    }

    /// Delete old segments (cleanup)
    pub async fn cleanup_old_segments(&self, segment_sequences: Vec<u32>) -> CaptureResult<()> {
        let delete_futures = segment_sequences.into_iter().map(|seq| {
            let audio_key = format!("{}/{}/audio/audio_recording_{}.aac", 
                                   self.user_id, self.video_id, seq);
            let video_key = format!("{}/{}/video/video_recording_{}.ts", 
                                   self.user_id, self.video_id, seq);
            
            async move {
                // Delete audio segment
                if let Err(e) = self.client.delete_object()
                    .bucket(&self.config.bucket)
                    .key(&audio_key)
                    .send()
                    .await {
                    log::warn!("Failed to delete audio segment {}: {}", audio_key, e);
                }

                // Delete video segment
                if let Err(e) = self.client.delete_object()
                    .bucket(&self.config.bucket)
                    .key(&video_key)
                    .send()
                    .await {
                    log::warn!("Failed to delete video segment {}: {}", video_key, e);
                }
            }
        });

        futures::future::join_all(delete_futures).await;
        log::info!("Cleaned up old segments");
        Ok(())
    }
}

/// Create S3 uploader with Cap's production settings
pub fn create_cap_s3_uploader(bucket: String, user_id: String, video_id: String) -> UploadConfig {
    UploadConfig {
        bucket,
        region: "us-east-1".to_string(), // Cap's primary region
        timeout_seconds: 10, // 10 second timeout for real-time streaming
        accelerated_transfer: true,
        endpoint: None,
    }
}
