const fs = require('fs');

// Helper function to clean up test directories with Windows compatibility
async function cleanupTestDir(testDir, db) {
    // Close database if provided
    if (db && db.close) {
        try {
            await db.close();
        } catch (err) {
            console.warn(`Warning: Could not close database: ${err.message}`);
        }
    }
    
    // Add retry logic for Windows
    let retries = 3;
    while (retries > 0) {
        try {
            if (fs.existsSync(testDir)) {
                fs.rmSync(testDir, { recursive: true, force: true, maxRetries: 3 });
            }
            break;
        } catch (err) {
            if (retries === 1 || (err.code !== 'ENOTEMPTY' && err.code !== 'EBUSY')) {
                console.warn(`Warning: Could not clean up test directory: ${err.message}`);
                break;
            }
            retries--;
            await new Promise(resolve => setTimeout(resolve, 100));
        }
    }
}

module.exports = { cleanupTestDir };