const { getDisplays, getAudioDevices, checkPermissions } = require('./index');

console.log("🍎 Testing Cap-Style macOS Display Enumeration\n");

async function testDisplayEnumeration() {
    try {
        // Test permissions first
        console.log("� Checking Permissions:");
        try {
            const permissions = checkPermissions();
            console.log(JSON.stringify(permissions, null, 2));
        } catch (error) {
            console.log(`⚠️  Permission check failed: ${error.message}`);
        }
        console.log();

        // Test display enumeration
        console.log("🖥️  Enumerating Displays:");
        const displaysJson = getDisplays();
        const displays = JSON.parse(displaysJson);
        
        if (!Array.isArray(displays)) {
            console.log("❌ Parsed displays is not an array");
            console.log("Parsed value:", displays);
            return;
        }
        
        if (displays.length === 0) {
            console.log("❌ No displays found");
            return;
        }

        console.log(`✅ Found ${displays.length} display(s):`);
        
        displays.forEach((display, index) => {
            console.log(`\n📺 Display ${index + 1}:`);
            console.log(`  ID: ${display.id}`);
            console.log(`  Name: "${display.name}"`);
            console.log(`  Resolution: ${display.width} x ${display.height} (${display.resolution[0]} x ${display.resolution[1]})`);
            console.log(`  Position: (${display.position[0]}, ${display.position[1]})`);
            console.log(`  Primary: ${display.is_primary ? "Yes" : "No"}`);
            console.log(`  Scale Factor: ${display.scale_factor}x`);
            
            // Cap-style analysis
            const isRetina = display.scale_factor >= 2.0;
            const isLargeDisplay = display.width >= 2560;
            const aspectRatio = (display.width / display.height).toFixed(2);
            
            console.log(`  🔍 Analysis:`);
            console.log(`    - Display Type: ${isRetina ? "Retina" : "Standard"} (${isLargeDisplay ? "Large" : "Normal"} size)`);
            console.log(`    - Aspect Ratio: ${aspectRatio}:1`);
            console.log(`    - Total Pixels: ${(display.width * display.height / 1000000).toFixed(1)}M`);
        });

        // Test audio devices for comparison
        console.log("\n🎤 Audio Devices (for reference):");
        try {
            const audioDevicesJson = getAudioDevices();
            const audioDevices = JSON.parse(audioDevicesJson);
            console.log(`✅ Found ${audioDevices.length} audio device(s)`);
            audioDevices.forEach((device, index) => {
                console.log(`  ${index + 1}. ${device.name} (${device.device_type})`);
            });
        } catch (error) {
            console.log(`❌ Audio enumeration failed: ${error.message}`);
        }

        // Summary in Cap style
        console.log("\n📊 Cap-Style Summary:");
        const primaryDisplay = displays.find(d => d.is_primary);
        if (primaryDisplay) {
            console.log(`Primary: ${primaryDisplay.name} @ ${primaryDisplay.width}x${primaryDisplay.height} (${primaryDisplay.scale_factor}x)`);
        }
        
        const totalPixels = displays.reduce((sum, d) => sum + (d.width * d.height), 0);
        console.log(`Total workspace: ${(totalPixels / 1000000).toFixed(1)}M pixels across ${displays.length} display(s)`);
        
        const hasRetina = displays.some(d => d.scale_factor >= 2.0);
        console.log(`Retina detected: ${hasRetina ? "Yes" : "No"}`);

    } catch (error) {
        console.error("❌ Test failed:", error);
        
        // Check if it's a permission issue
        if (error.message.includes("permission") || error.message.includes("Permission")) {
            console.log("\n🔧 Permission Fix:");
            console.log("1. Open System Preferences > Privacy & Security");
            console.log("2. Go to Screen Recording");
            console.log("3. Add Node.js or your terminal app");
            console.log("4. Restart this test");
        }
    }
}

console.log("🚀 Starting display enumeration test...\n");
testDisplayEnumeration().then(() => {
    console.log("\n✨ Test completed!");
}).catch(error => {
    console.error("💥 Unexpected error:", error);
});
