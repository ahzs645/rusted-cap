#!/usr/bin/env node

/**
 * Cap Electron Capture - Audio File Output Test
 * 
 * Tests the real FFmpeg audio encoder by generating PCM audio data,
 * encoding it to AAC, and writing the output to local files for verification.
 */

const cap = require('./index.js');
const fs = require('fs');
const path = require('path');

console.log('🎵 Cap Audio Encoder - File Output Test');
console.log('=====================================\n');

async function generateTestAudio(duration = 5.0, sampleRate = 48000, channels = 2) {
    console.log(`📊 Generating ${duration}s of test audio (${sampleRate}Hz, ${channels} channels)...`);
    
    const totalSamples = Math.floor(duration * sampleRate * channels);
    const pcmData = new Float32Array(totalSamples);
    
    // Generate a test tone (440Hz sine wave + 880Hz harmonic)
    for (let i = 0; i < totalSamples; i += channels) {
        const time = (i / channels) / sampleRate;
        const sample = Math.sin(2 * Math.PI * 440 * time) * 0.3 + 
                      Math.sin(2 * Math.PI * 880 * time) * 0.1;
        
        // Write to both channels (stereo)
        pcmData[i] = sample;     // Left channel
        if (channels > 1) {
            pcmData[i + 1] = sample * 0.8; // Right channel (slightly quieter)
        }
    }
    
    console.log(`✅ Generated ${totalSamples} PCM samples (${pcmData.length * 4} bytes)`);
    return Array.from(pcmData);
}

async function testAudioFileOutput() {
    try {
        // 1. Initialize the library
        console.log('1. Initializing Cap library...');
        const initResult = await cap.init();
        const init = JSON.parse(initResult);
        console.log('   Platform:', init.platform);
        console.log('   Audio support:', init.audio.system_audio ? '✅' : '❌');
        
        // 2. Create audio encoder with transcription settings
        console.log('\n2. Creating audio encoder...');
        const audioConfig = {
            codec: 'AAC',
            bitrate: 128000,
            sample_rate: 48000,
            channels: 2,
            channel_layout: 'Stereo'
        };
        
        console.log('   Config:', JSON.stringify(audioConfig, null, 2));
        
        // 3. Generate test PCM audio data
        console.log('\n3. Generating test audio data...');
        const pcmData = await generateTestAudio(5.0, 48000, 2); // 5 seconds of audio
        console.log(`   Generated: ${pcmData.length} samples (${(pcmData.length * 4 / 1024).toFixed(1)} KB)`);
        
        // 4. Create a test recording configuration
        console.log('\n4. Creating recording configuration for audio-only test...');
        const recordingConfig = {
            audio: {
                enabled: true,
                system_audio: false, // Don't need actual system audio for this test
                microphone: false,   // Don't need microphone for this test
                sample_rate: 48000,
                channels: 2,
                segment_duration_ms: 2000,
                format: 'Aac'
            },
            screen: {
                enabled: false, // Audio-only test
                fps: 30,
                quality: 80,
                include_cursor: true
            },
            encoding: {
                audio: audioConfig,
                video: {
                    codec: 'H264',
                    bitrate: 2000000,
                    frame_rate: [30, 1],
                    resolution: [1920, 1080],
                    pixel_format: 'YUV420P',
                    hardware_acceleration: true
                },
                hls: {
                    segment_duration: 2,
                    target_duration: 2,
                    playlist_size: 5
                }
            },
            user_id: 'audio_test_user',
            s3_bucket: 'test-audio-bucket',
            enable_transcription: true,
            enable_streaming: false
        };
        
        // 5. Create recording pipeline
        console.log('\n5. Creating audio recording pipeline...');
        const pipeline = await cap.createRecordingPipeline(JSON.stringify(recordingConfig));
        const pipelineResult = JSON.parse(pipeline);
        console.log('   Pipeline created:', pipelineResult.session_id);
        
        // 6. Start recording session
        console.log('\n6. Starting recording session...');
        const session = await cap.startRecording(pipelineResult.session_id);
        const sessionResult = JSON.parse(session);
        console.log('   Session started:', sessionResult.status);
        
        // 7. Process audio data in chunks (simulate real-time audio)
        console.log('\n7. Processing audio data in segments...');
        const chunkSize = 48000 * 2 * 2; // 2 seconds worth of samples (48kHz * 2 channels * 2 seconds)
        const segments = [];
        
        for (let offset = 0; offset < pcmData.length; offset += chunkSize) {
            const chunk = pcmData.slice(offset, offset + chunkSize);
            const segmentNumber = Math.floor(offset / chunkSize) + 1;
            const totalSegments = Math.ceil(pcmData.length / chunkSize);
            
            console.log(`   📦 Processing segment ${segmentNumber}/${totalSegments} (${chunk.length} samples)...`);
            
            // Convert Float32Array to Buffer for NAPI
            const chunkBuffer = Buffer.allocUnsafe(chunk.length * 4);
            for (let i = 0; i < chunk.length; i++) {
                chunkBuffer.writeFloatLE(chunk[i], i * 4);
            }
            
            // Process this chunk through the encoder
            try {
                const resultStr = await cap.processAudioChunk(pipelineResult.session_id, Array.from(chunkBuffer));
                const result = JSON.parse(resultStr);
                
                if (result && result.segments) {
                    segments.push(...result.segments);
                    console.log(`      ✅ Encoded ${result.segments.length} AAC segments`);
                    
                    // Log first segment details
                    if (result.segments.length > 0) {
                        const seg = result.segments[0];
                        console.log(`         Segment ${seg.sequence}: ${seg.size_bytes} bytes, ${seg.duration}s`);
                    }
                }
            } catch (err) {
                console.warn(`      ⚠️  Warning processing chunk: ${err.message}`);
            }
        }
        
        // 8. Flush any remaining data
        console.log('\n8. Flushing encoder...');
        try {
            const flushResultStr = await cap.flushEncoder(pipelineResult.session_id);
            const flushResult = JSON.parse(flushResultStr);
            
            if (flushResult && flushResult.segments) {
                segments.push(...flushResult.segments);
                console.log(`   ✅ Flushed ${flushResult.segments.length} additional segments`);
            }
        } catch (err) {
            console.warn(`   ⚠️  Warning flushing encoder: ${err.message}`);
        }
        
        // 9. Stop recording
        console.log('\n9. Stopping recording...');
        const stopResult = await cap.stopRecording(pipelineResult.session_id);
        const stopResultData = JSON.parse(stopResult);
        console.log('   Recording stopped:', stopResultData.status);
        
        // 10. Write encoded segments to files
        console.log('\n10. Writing encoded audio segments to files...');
        const outputDir = path.join(__dirname, 'test-audio-output');
        
        // Create output directory
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
            console.log(`    📁 Created output directory: ${outputDir}`);
        }
        
        // Write individual AAC segments
        let totalBytes = 0;
        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];
            const filename = `segment_${String(i).padStart(3, '0')}.aac`;
            const filepath = path.join(outputDir, filename);
            
            const buffer = Buffer.from(segment.data);
            fs.writeFileSync(filepath, buffer);
            totalBytes += buffer.length;
            
            console.log(`    💾 ${filename}: ${buffer.length} bytes (${segment.duration}s, seq: ${segment.sequence})`);
        }
        
        // Write combined AAC file
        if (segments.length > 0) {
            console.log('\n11. Creating combined AAC file...');
            const combinedFilename = 'test_audio_combined.aac';
            const combinedFilepath = path.join(outputDir, combinedFilename);
            
            const combinedData = Buffer.concat(segments.map(seg => Buffer.from(seg.data)));
            fs.writeFileSync(combinedFilepath, combinedData);
            
            console.log(`    💾 ${combinedFilename}: ${combinedData.length} bytes`);
        }
        
        // Write original PCM data for reference
        console.log('\n12. Writing reference PCM file...');
        const pcmFilename = 'test_audio_reference.pcm';
        const pcmFilepath = path.join(outputDir, pcmFilename);
        
        const pcmBuffer = Buffer.allocUnsafe(pcmData.length * 4);
        for (let i = 0; i < pcmData.length; i++) {
            pcmBuffer.writeFloatLE(pcmData[i], i * 4);
        }
        fs.writeFileSync(pcmFilepath, pcmBuffer);
        console.log(`    💾 ${pcmFilename}: ${pcmBuffer.length} bytes (32-bit float PCM)`);
        
        // 13. Summary
        console.log('\n📊 Test Results Summary');
        console.log('=======================');
        console.log(`Input PCM Data: ${(pcmData.length * 4 / 1024).toFixed(1)} KB (${pcmData.length} samples)`);
        console.log(`Encoded Segments: ${segments.length}`);
        console.log(`Total AAC Data: ${(totalBytes / 1024).toFixed(1)} KB`);
        console.log(`Compression Ratio: ${((pcmData.length * 4) / totalBytes).toFixed(1)}:1`);
        console.log(`Output Directory: ${outputDir}`);
        
        // List all files created
        console.log('\n📁 Files Created:');
        const files = fs.readdirSync(outputDir).sort();
        files.forEach(file => {
            const filepath = path.join(outputDir, file);
            const stats = fs.statSync(filepath);
            console.log(`   ${file}: ${(stats.size / 1024).toFixed(1)} KB`);
        });
        
        console.log('\n✅ Audio file output test completed successfully!');
        console.log('\n🎧 To test the encoded audio:');
        console.log(`   cd ${outputDir}`);
        console.log(`   ffplay test_audio_combined.aac  # Play the AAC file`);
        console.log(`   ffplay -f f32le -ar 48000 -ac 2 test_audio_reference.pcm  # Play original PCM`);
        
        return {
            success: true,
            segments_created: segments.length,
            total_bytes: totalBytes,
            output_directory: outputDir,
            files: files
        };
        
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        console.error('Stack trace:', error.stack);
        return { success: false, error: error.message };
    }
}

// Run the test
if (require.main === module) {
    testAudioFileOutput()
        .then(result => {
            if (result.success) {
                console.log('\n🎉 Test completed successfully!');
                process.exit(0);
            } else {
                console.log('\n💥 Test failed!');
                process.exit(1);
            }
        })
        .catch(error => {
            console.error('Unhandled error:', error);
            process.exit(1);
        });
}

module.exports = { testAudioFileOutput };
