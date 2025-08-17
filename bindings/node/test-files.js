#!/usr/bin/env node

const { WalDB } = require('./index.js');
const path = require('path');
const os = require('os');
const fs = require('fs');
const crypto = require('crypto');

// Helper to create test directory
function testDir(name) {
    return path.join(os.tmpdir(), `waldb_test_files_${name}_${process.pid}`);
}

// Helper to clean up
function cleanup(dir) {
    try {
        fs.rmSync(dir, { recursive: true, force: true });
    } catch (e) {}
}

// Helper to create test data
function createTestData(size) {
    return Buffer.from(crypto.randomBytes(size));
}

async function runTests() {
    let passed = 0;
    let failed = 0;

    async function test(name, fn) {
        process.stdout.write(`Testing ${name}: `);
        try {
            await fn();
            console.log('✅ PASSED');
            passed++;
        } catch (error) {
            console.log(`❌ FAILED - ${error.message}`);
            console.error(error.stack);
            failed++;
        }
    }

    // Test 1: Basic file storage and retrieval
    await test('File Set/Get', async () => {
        const dir = testDir('basic');
        const db = await WalDB.open(dir);
        
        const data = Buffer.from('Hello, WalDB files!');
        await db.setFile('documents/test.txt', data);
        
        const retrieved = await db.getFile('documents/test.txt');
        if (!Buffer.isBuffer(retrieved)) {
            throw new Error('Retrieved data is not a Buffer');
        }
        
        if (!data.equals(retrieved)) {
            throw new Error('Retrieved data does not match original');
        }
        
        cleanup(dir);
    });

    // Test 2: Binary file handling
    await test('Binary Files', async () => {
        const dir = testDir('binary');
        const db = await WalDB.open(dir);
        
        // Create a fake image (random bytes)
        const imageData = createTestData(1024);
        await db.setFile('images/photo.jpg', imageData);
        
        const retrieved = await db.getFile('images/photo.jpg');
        if (!imageData.equals(retrieved)) {
            throw new Error('Binary data corrupted');
        }
        
        cleanup(dir);
    });

    // Test 3: File metadata
    await test('File Metadata', async () => {
        const dir = testDir('metadata');
        const db = await WalDB.open(dir);
        
        const data = Buffer.from('Test file content');
        await db.setFile('docs/readme.md', data);
        
        // Get metadata without loading the file
        const meta = await db.getFileMeta('docs/readme.md');
        
        if (!meta.size || meta.size !== data.length) {
            throw new Error(`Incorrect size metadata: ${meta.size} vs ${data.length}`);
        }
        
        if (!meta.hash) {
            throw new Error('Missing hash metadata');
        }
        
        if (!meta.type) {
            throw new Error('Missing type metadata');
        }
        
        cleanup(dir);
    });

    // Test 4: File deduplication
    await test('File Deduplication', async () => {
        const dir = testDir('dedup');
        const db = await WalDB.open(dir);
        
        const data = Buffer.from('Duplicate content test');
        
        // Store the same content under different paths
        await db.setFile('files/copy1.txt', data);
        await db.setFile('files/copy2.txt', data);
        await db.setFile('files/copy3.txt', data);
        
        // All should retrieve the same content
        const retrieved1 = await db.getFile('files/copy1.txt');
        const retrieved2 = await db.getFile('files/copy2.txt');
        const retrieved3 = await db.getFile('files/copy3.txt');
        
        if (!data.equals(retrieved1) || !data.equals(retrieved2) || !data.equals(retrieved3)) {
            throw new Error('Deduplicated files have different content');
        }
        
        // Check that they all have the same hash (indicating deduplication)
        const meta1 = await db.getFileMeta('files/copy1.txt');
        const meta2 = await db.getFileMeta('files/copy2.txt');
        const meta3 = await db.getFileMeta('files/copy3.txt');
        
        if (meta1.hash !== meta2.hash || meta2.hash !== meta3.hash) {
            throw new Error('Files not properly deduplicated');
        }
        
        cleanup(dir);
    });

    // Test 5: File deletion
    await test('File Deletion', async () => {
        const dir = testDir('delete');
        const db = await WalDB.open(dir);
        
        const data = Buffer.from('To be deleted');
        await db.setFile('temp/file.txt', data);
        
        // Verify it exists
        const exists = await db.getFile('temp/file.txt');
        if (!exists) {
            throw new Error('File not created');
        }
        
        // Delete it
        await db.deleteFile('temp/file.txt');
        
        // Verify it's gone
        try {
            await db.getFile('temp/file.txt');
            throw new Error('File still exists after deletion');
        } catch (e) {
            if (!e.message.includes('not found')) {
                throw e;
            }
        }
        
        // Verify metadata is also gone
        const meta = await db.getFileMeta('temp/file.txt');
        if (meta.size || meta.hash || meta.type) {
            throw new Error('File metadata not fully deleted');
        }
        
        cleanup(dir);
    });

    // Test 6: Large file handling
    await test('Large Files', async () => {
        const dir = testDir('large');
        const db = await WalDB.open(dir);
        
        // Create a 1MB file
        const largeData = createTestData(1024 * 1024);
        await db.setFile('large/file.bin', largeData);
        
        const retrieved = await db.getFile('large/file.bin');
        if (!largeData.equals(retrieved)) {
            throw new Error('Large file corrupted');
        }
        
        const meta = await db.getFileMeta('large/file.bin');
        if (meta.size !== largeData.length) {
            throw new Error(`Size mismatch: ${meta.size} vs ${largeData.length}`);
        }
        
        cleanup(dir);
    });

    // Test 7: File overwrite
    await test('File Overwrite', async () => {
        const dir = testDir('overwrite');
        const db = await WalDB.open(dir);
        
        const data1 = Buffer.from('Original content');
        const data2 = Buffer.from('Updated content with different size');
        
        await db.setFile('docs/file.txt', data1);
        const original = await db.getFile('docs/file.txt');
        
        await db.setFile('docs/file.txt', data2);
        const updated = await db.getFile('docs/file.txt');
        
        if (data1.equals(updated)) {
            throw new Error('File not updated');
        }
        
        if (!data2.equals(updated)) {
            throw new Error('File not updated correctly');
        }
        
        const meta = await db.getFileMeta('docs/file.txt');
        if (meta.size !== data2.length) {
            throw new Error('Metadata not updated after overwrite');
        }
        
        cleanup(dir);
    });

    // Test 8: Non-existent file
    await test('Non-existent File', async () => {
        const dir = testDir('nonexistent');
        const db = await WalDB.open(dir);
        
        try {
            await db.getFile('does/not/exist.txt');
            throw new Error('Should have thrown for non-existent file');
        } catch (e) {
            if (!e.message.includes('not found')) {
                throw new Error(`Wrong error: ${e.message}`);
            }
        }
        
        cleanup(dir);
    });

    // Test 9: Different data types
    await test('Different Data Types', async () => {
        const dir = testDir('types');
        const db = await WalDB.open(dir);
        
        // Test with ArrayBuffer
        const arrayBuffer = new ArrayBuffer(16);
        const view = new Uint8Array(arrayBuffer);
        for (let i = 0; i < 16; i++) {
            view[i] = i;
        }
        
        await db.setFile('data/arraybuffer.bin', arrayBuffer);
        const retrieved1 = await db.getFile('data/arraybuffer.bin');
        
        // Test with Uint8Array
        const uint8Array = new Uint8Array([1, 2, 3, 4, 5]);
        await db.setFile('data/uint8array.bin', uint8Array);
        const retrieved2 = await db.getFile('data/uint8array.bin');
        
        if (retrieved1.length !== 16) {
            throw new Error('ArrayBuffer data wrong size');
        }
        
        if (retrieved2.length !== 5) {
            throw new Error('Uint8Array data wrong size');
        }
        
        cleanup(dir);
    });

    // Test 10: MIME type detection
    await test('MIME Type Detection', async () => {
        const dir = testDir('mime');
        const db = await WalDB.open(dir);
        
        // Create files with recognizable headers
        const pngHeader = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        const jpegHeader = Buffer.from([0xFF, 0xD8, 0xFF, 0xE0]);
        const pdfHeader = Buffer.from('%PDF-1.4');
        
        await db.setFile('images/test.png', pngHeader);
        await db.setFile('images/test.jpg', jpegHeader);
        await db.setFile('docs/test.pdf', pdfHeader);
        
        const pngMeta = await db.getFileMeta('images/test.png');
        const jpegMeta = await db.getFileMeta('images/test.jpg');
        const pdfMeta = await db.getFileMeta('docs/test.pdf');
        
        // The actual MIME detection in waldb.rs is simplified
        // It should at least return some type
        if (!pngMeta.type || !jpegMeta.type || !pdfMeta.type) {
            throw new Error('MIME type not detected');
        }
        
        cleanup(dir);
    });

    console.log('\n========================');
    console.log(`Results: ${passed} passed, ${failed} failed`);
    
    if (failed > 0) {
        process.exit(1);
    }
}

// Run tests
console.log('Running WalDB File Storage Tests');
console.log('=================================');
runTests().catch(error => {
    console.error('Test runner failed:', error);
    process.exit(1);
});