/**
 * Cap Electron Capture - Encoding and Streaming Example
 * 
 * This example demonstrates Cap's real-time audio/video encoding pipeline
 * with FFmpeg, HLS segmentation, and S3 streaming capabilities.
 */

const cap = require('.');

async function testCapEncodingPipeline() {
    console.log('🎬 Cap Electron Capture - Encoding Pipeline Test');
    console.log('================================================');
    
    try {
        // 1. Initialize and check capabilities
        console.log('\n1. Initializing Cap library...');
        const initResult = cap.init();
        console.log('Init result:', JSON.parse(initResult));
        
        // 2. Check encoding capabilities
        console.log('\n2. Checking encoding capabilities...');
        const encodingCaps = cap.getEncodingCapabilities();
        const capabilities = JSON.parse(encodingCaps);
        console.log('Encoding capabilities:', JSON.stringify(capabilities, null, 2));
        
        // 3. Request permissions for screen and audio capture
        console.log('\n3. Requesting permissions...');
        const permissionsResult = await cap.requestPermissions();
        const permissions = JSON.parse(permissionsResult);
        console.log('Permissions:', permissions);
        
        // 4. Create recording configuration with Cap's settings
        console.log('\n4. Creating recording configuration...');
        const recordingConfig = {
            // Audio settings for transcription
            audio: {
                enabled: true,
                system_audio: true,
                microphone: true,
                sample_rate: 48000,
                channels: 2,
                segment_duration_ms: 2000, // 2 second segments like Cap
                format: "Aac"
            },
            
            // Screen capture settings
            screen: {
                enabled: true,
                fps: 30,
                quality: 80,
                include_cursor: true
            },
            
            // Encoding settings (Cap's pipeline)
            encoding: {
                // Audio encoding for transcription
                audio: {
                    codec: "AAC",
                    bitrate: 128000, // 128kbps for transcription
                    sample_rate: 48000,
                    channels: 2,
                    channel_layout: "Stereo"
                },
                
                // Video encoding for screen recording
                video: {
                    codec: "H264",
                    bitrate: 2000000, // 2Mbps
                    frame_rate: [30, 1], // 30fps
                    resolution: [1920, 1080],
                    pixel_format: "YUV420P",
                    hardware_acceleration: true
                },
                
                // HLS segmentation (Cap's approach)
                hls: {
                    segment_duration: 2.0, // 2-second segments
                    target_duration: 2,
                    playlist_size: 5
                }
            },
            
            // User and upload settings
            user_id: "test_user_123",
            s3_bucket: "cap-recordings", // Optional for local testing
            enable_transcription: true,
            enable_streaming: false // Set to true for S3 streaming
        };
        
        console.log('Recording config:', JSON.stringify(recordingConfig, null, 2));
        
        // 5. Create recording pipeline
        console.log('\n5. Creating recording pipeline...');
        const pipelineResult = await cap.createRecordingPipeline(JSON.stringify(recordingConfig));
        const pipelineInfo = JSON.parse(pipelineResult);
        console.log('Pipeline created:', pipelineInfo);
        
        const sessionId = pipelineInfo.session_id;
        
        // 6. Start recording
        console.log('\n6. Starting recording...');
        const recordingResult = await cap.startRecording(sessionId);
        const session = JSON.parse(recordingResult);
        console.log('Recording started:', session);
        
        // Show stream URLs if available
        if (session.stream_urls) {
            console.log('\n📺 Stream URLs:');
            console.log('Master playlist:', session.stream_urls.master);
            console.log('Video stream:', session.stream_urls.video);
            console.log('Audio stream:', session.stream_urls.audio);
            console.log('Combined stream:', session.stream_urls.combined);
        }
        
        // 7. Record for a few seconds
        console.log('\n7. Recording for 10 seconds...');
        console.log('🔴 RECORDING IN PROGRESS');
        console.log('Audio encoding: AAC 128kbps');
        console.log('Video encoding: H.264 2Mbps');
        console.log('HLS segments: 2 second duration');
        
        // Simulate real-time processing
        for (let i = 0; i < 10; i++) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            console.log(`  📊 ${i + 1}/10 seconds - Processing...`);
        }
        
        // 8. Stop recording
        console.log('\n8. Stopping recording...');
        const stopResult = await cap.stopRecording(sessionId);
        const finalSession = JSON.parse(stopResult);
        console.log('Recording stopped:', finalSession);
        
        // 9. Show final results
        console.log('\n✅ Recording Complete!');
        console.log('==================');
        console.log('Session ID:', finalSession.id);
        console.log('Status:', finalSession.status);
        
        if (finalSession.final_stats) {
            console.log('Final Statistics:');
            console.log('  Duration:', finalSession.final_stats.total_duration, 'seconds');
            console.log('  Segments:', finalSession.final_stats.total_segments);
            console.log('  Total bytes:', finalSession.final_stats.total_bytes);
            console.log('  Average FPS:', finalSession.final_stats.avg_fps);
        }
        
        if (finalSession.files) {
            console.log('Generated Files:');
            console.log('  Master playlist:', finalSession.files.master_playlist);
            console.log('  Final video:', finalSession.files.final_video);
        }
        
        console.log('\n🎉 Cap encoding pipeline test completed successfully!');
        
    } catch (error) {
        console.error('❌ Error during encoding pipeline test:', error);
        process.exit(1);
    }
}

// Additional utility functions to demonstrate specific features
async function testEncodingCapabilities() {
    console.log('\n🔧 Testing Encoding Capabilities');
    console.log('================================');
    
    try {
        const caps = JSON.parse(cap.getEncodingCapabilities());
        
        console.log('Supported Audio Codecs:', caps.audio_codecs);
        console.log('Supported Video Codecs:', caps.video_codecs);
        console.log('Container Formats:', caps.container_formats);
        
        console.log('\nStreaming Capabilities:');
        console.log('  HLS Support:', caps.streaming.hls);
        console.log('  Segment Duration:', caps.streaming.segment_duration, 'seconds');
        console.log('  Max Bitrate:', caps.streaming.max_bitrate, 'bps');
        
        console.log('\nHardware Acceleration:');
        console.log('  Available:', caps.hardware_acceleration.available);
        console.log('  Platforms:', caps.hardware_acceleration.platforms);
        
        console.log('\nDefault Settings:');
        console.log('  Audio:', caps.default_settings.audio);
        console.log('  Video:', caps.default_settings.video);
        
    } catch (error) {
        console.error('Error testing encoding capabilities:', error);
    }
}

async function main() {
    console.log('Cap Electron Capture - Encoding & Streaming Test Suite');
    console.log('======================================================');
    
    // Test encoding capabilities
    await testEncodingCapabilities();
    
    // Test full pipeline
    await testCapEncodingPipeline();
}

// Run the tests
if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    testCapEncodingPipeline,
    testEncodingCapabilities
};
