const fs = require('fs');

// Helper function to clean up test directories with Windows compatibility
async function cleanupTestDir(testDir) {
    // Windows sometimes holds file locks briefly after operations
    // Just retry a few times with delays
    let retries = 5;
    while (retries > 0) {
        try {
            if (fs.existsSync(testDir)) {
                fs.rmSync(testDir, { recursive: true, force: true, maxRetries: 3 });
            }
            return; // Success
        } catch (err) {
            retries--;
            if (retries === 0) {
                // Final attempt failed, just warn and continue
                console.warn(`Warning: Could not clean up ${testDir}: ${err.message}`);
                return;
            }
            // Wait a bit for Windows to release file handles
            await new Promise(resolve => setTimeout(resolve, 200));
        }
    }
}

module.exports = { cleanupTestDir };