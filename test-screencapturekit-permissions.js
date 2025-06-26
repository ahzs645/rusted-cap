#!/usr/bin/env node

/**
 * Test ScreenCaptureKit Permissions and System Audio Functionality
 * 
 * This test validates that:
 * 1. ScreenCaptureKit is available on the system
 * 2. Screen recording permissions are properly checked
 * 3. System audio capture can be initialized
 */

const { 
    init,
    createCaptureSession, 
    startNativeSystemAudio,
    checkPermissions,
    requestPermissions,
    getSystemAudioSetupInstructions
} = require('./index.js');

async function testScreenCaptureKitPermissions() {
    console.log('🔐 Testing ScreenCaptureKit Permissions');
    console.log('=====================================');

    try {
        // 1. Initialize and check platform capabilities
        console.log('\n1. Checking Platform Capabilities...');
        const capabilities = JSON.parse(await init());
        console.log('   Platform:', capabilities.platform || 'Unknown');
        console.log('   ScreenCaptureKit Available:', capabilities.screencapturekit || false);
        
        // 2. Check current permissions
        console.log('\n2. Checking Current Permissions...');
        const permissions = JSON.parse(await checkPermissions());
        console.log('   Screen Recording:', permissions.screenRecording || false);
        console.log('   Microphone:', permissions.microphone || false);
        console.log('   Camera:', permissions.camera || false);
        
        // 3. Get system audio setup instructions
        console.log('\n3. System Audio Setup Instructions:');
        const instructions = getSystemAudioSetupInstructions();
        console.log('  ', instructions);
        
        // 4. Test system audio session creation
        console.log('\n4. Testing System Audio Session...');
        const config = JSON.stringify({
            audio: { 
                enabled: true, 
                systemAudio: true,
                microphone: false,
                sampleRate: 48000,
                channels: 2,
                segmentDurationMs: 1000,
                format: 'Aac'
            },
            screen: { 
                enabled: false,
                displayId: null,
                fps: 30,
                quality: 80,
                includeCursor: true,
                windowId: null
            },
            output: {
                audio: 'Aac',
                video: 'Mp4',
                realTime: true,
                outputDir: null
            }
        });
        
        const sessionData = await createCaptureSession(config);
        const session = JSON.parse(sessionData);
        console.log('   ✅ Session created:', session.id);
        
        // 5. Test starting ScreenCaptureKit system audio
        console.log('\n5. Testing ScreenCaptureKit System Audio Start...');
        const result = await startNativeSystemAudio(session.id);
        const audioResult = JSON.parse(result);
        
        console.log('   Status:', audioResult.status);
        console.log('   Method:', audioResult.method);
        console.log('   Implementation:', audioResult.implementation);
        
        if (audioResult.status === 'started') {
            console.log('   ✅ System audio ready!');
            console.log('   Audio Config:', audioResult.audio_config);
            console.log('   Permission:', audioResult.requires_permission);
        } else if (audioResult.status === 'error') {
            console.log('   ❌ Error:', audioResult.error);
            console.log('   Message:', audioResult.message);
            
            // If permission error, try to request permissions
            if (audioResult.error.includes('permission') || audioResult.error.includes('Permission')) {
                console.log('\n6. Requesting Permissions...');
                try {
                    const permissionResult = JSON.parse(await requestPermissions());
                    console.log('   Permission request result:', permissionResult);
                } catch (e) {
                    console.log('   Permission request failed:', e.message);
                }
            }
        }
        
        console.log('\n🎉 ScreenCaptureKit Permission Test Complete!');
        console.log('=====================================');
        
        // Summary
        console.log('\n📋 Summary:');
        console.log('   • ScreenCaptureKit Available:', capabilities.screencapturekit || false);
        console.log('   • Screen Recording Permission:', permissions.screenRecording || false);
        console.log('   • System Audio Status:', audioResult.status);
        
        if (audioResult.status === 'error') {
            console.log('\n💡 Next Steps:');
            console.log('   1. Enable Screen Recording permission in System Preferences');
            console.log('   2. Go to: System Preferences > Privacy & Security > Screen Recording');
            console.log('   3. Add and enable your application (Terminal, VS Code, etc.)');
            console.log('   4. Restart the application and run this test again');
        }
        
    } catch (error) {
        console.error('❌ Test failed:', error);
        process.exit(1);
    }
}

// Run the test
testScreenCaptureKitPermissions().catch(console.error);
