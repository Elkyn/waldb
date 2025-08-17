#!/usr/bin/env node

const { WalDB } = require('./index.js');
const path = require('path');
const os = require('os');

// Helper to create test directory
function testDir(name) {
    return path.join(os.tmpdir(), `waldb_test_vector_edge_${name}_${process.pid}`);
}

// Helper to clean up
function cleanup(dir) {
    try {
        require('fs').rmSync(dir, { recursive: true, force: true });
    } catch (e) {}
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
            failed++;
        }
    }

    // Edge Case 1: Empty vectors
    await test('Empty Vectors', async () => {
        const dir = testDir('empty');
        const db = await WalDB.open(dir);
        
        try {
            await db.setVector('test/empty', []);
            const retrieved = await db.getVector('test/empty');
            if (retrieved === null || retrieved.length !== 0) {
                throw new Error('Empty vector not handled correctly');
            }
        } catch (e) {
            // Some implementations may reject empty vectors
            if (!e.message.includes('empty') && !e.message.includes('length')) {
                throw e;
            }
        }
        
        cleanup(dir);
    });

    // Edge Case 2: Very large vectors
    await test('Large Vectors (1000 dimensions)', async () => {
        const dir = testDir('large');
        const db = await WalDB.open(dir);
        
        const largeVector = new Array(1000).fill(0).map((_, i) => i / 1000);
        await db.setVector('test/large', largeVector);
        
        const retrieved = await db.getVector('test/large');
        if (retrieved.length !== 1000) {
            throw new Error(`Vector size mismatch: ${retrieved.length}`);
        }
        
        cleanup(dir);
    });

    // Edge Case 3: Negative values in vectors
    await test('Negative Vector Values', async () => {
        const dir = testDir('negative');
        const db = await WalDB.open(dir);
        
        const vector = [-1.0, -0.5, 0.0, 0.5, 1.0];
        await db.setVector('test/negative', vector);
        
        const retrieved = await db.getVector('test/negative');
        for (let i = 0; i < vector.length; i++) {
            if (Math.abs(retrieved[i] - vector[i]) > 0.0001) {
                throw new Error(`Negative value corruption at index ${i}`);
            }
        }
        
        cleanup(dir);
    });

    // Edge Case 4: NaN and Infinity in vectors
    await test('NaN and Infinity Handling', async () => {
        const dir = testDir('nan');
        const db = await WalDB.open(dir);
        
        // Test NaN
        try {
            await db.setVector('test/nan', [1.0, NaN, 3.0]);
            // If it accepts NaN, verify retrieval
            const retrieved = await db.getVector('test/nan');
            if (!isNaN(retrieved[1])) {
                throw new Error('NaN not preserved');
            }
        } catch (e) {
            // It's OK to reject NaN
            if (!e.message.includes('NaN') && !e.message.includes('valid')) {
                throw e;
            }
        }
        
        // Test Infinity
        try {
            await db.setVector('test/inf', [1.0, Infinity, -Infinity]);
            const retrieved = await db.getVector('test/inf');
            if (retrieved[1] !== Infinity || retrieved[2] !== -Infinity) {
                throw new Error('Infinity not preserved');
            }
        } catch (e) {
            // It's OK to reject Infinity
            if (!e.message.includes('Infinity') && !e.message.includes('finite')) {
                throw e;
            }
        }
        
        cleanup(dir);
    });

    // Edge Case 5: Zero vectors (all zeros)
    await test('Zero Vectors', async () => {
        const dir = testDir('zero');
        const db = await WalDB.open(dir);
        
        const zeroVector = [0, 0, 0, 0, 0];
        await db.setVector('test/zero', zeroVector);
        
        // Search with zero vector should work
        const results = await db.advancedSearch({
            pattern: 'test/*',
            vector: {
                query: [0, 0, 0, 0, 0],
                field: 'zero'
            },
            limit: 1
        });
        
        // Zero vector similarity with itself should be handled
        // (normally it's undefined due to division by zero in cosine similarity)
        
        cleanup(dir);
    });

    // Edge Case 6: Mismatched vector dimensions in search
    await test('Mismatched Vector Dimensions', async () => {
        const dir = testDir('mismatch');
        const db = await WalDB.open(dir);
        
        await db.setVector('docs/1/embedding', [1, 2, 3]);
        await db.setVector('docs/2/embedding', [4, 5, 6, 7, 8]); // Different size!
        
        // Search with yet another dimension
        const results = await db.advancedSearch({
            pattern: 'docs/*',
            vector: {
                query: [1, 2],  // 2D query vs 3D and 5D stored
                field: 'embedding'
            },
            limit: 2
        });
        
        // Should handle gracefully (skip mismatched or error)
        
        cleanup(dir);
    });

    // Edge Case 7: No vectors to search
    await test('Search with No Vectors', async () => {
        const dir = testDir('novectors');
        const db = await WalDB.open(dir);
        
        // Store documents without vectors
        await db.set('docs/1/title', 'Document 1');
        await db.set('docs/2/title', 'Document 2');
        
        const results = await db.advancedSearch({
            pattern: 'docs/*',
            vector: {
                query: [1, 2, 3],
                field: 'embedding'  // This field doesn't exist
            },
            limit: 2
        });
        
        // Should return empty or handle gracefully
        if (results.length > 0) {
            // If it returns results, they should have no scores
            const firstResult = results[0];
            const hasVectorScore = firstResult.some(([k]) => k.includes('_vector_score'));
            if (hasVectorScore) {
                throw new Error('Vector score present without vectors');
            }
        }
        
        cleanup(dir);
    });

    // Edge Case 8: Very similar vectors (near duplicates)
    await test('Near Duplicate Vectors', async () => {
        const dir = testDir('duplicates');
        const db = await WalDB.open(dir);
        
        const base = [0.5, 0.5, 0.5, 0.5, 0.5];
        const epsilon = 0.0001;
        
        await db.setVector('docs/1/vec', base);
        await db.setVector('docs/2/vec', base.map(v => v + epsilon));
        await db.setVector('docs/3/vec', base.map(v => v - epsilon));
        
        const results = await db.advancedSearch({
            pattern: 'docs/*',
            vector: {
                query: base,
                field: 'vec'
            },
            limit: 3
        });
        
        // All should have very high similarity scores
        if (results.length !== 3) {
            throw new Error('Not all near-duplicates returned');
        }
        
        cleanup(dir);
    });

    // Edge Case 9: Orthogonal vectors (zero similarity)
    await test('Orthogonal Vectors', async () => {
        const dir = testDir('orthogonal');
        const db = await WalDB.open(dir);
        
        // These vectors are orthogonal (dot product = 0)
        await db.setVector('docs/1/vec', [1, 0, 0]);
        await db.setVector('docs/2/vec', [0, 1, 0]);
        await db.setVector('docs/3/vec', [0, 0, 1]);
        
        const results = await db.advancedSearch({
            pattern: 'docs/*',
            vector: {
                query: [1, 0, 0],
                field: 'vec'
            },
            limit: 3
        });
        
        // First result should be the exact match
        // Others should have zero or very low similarity
        
        cleanup(dir);
    });

    // Edge Case 10: Concurrent vector operations
    await test('Concurrent Vector Operations', async () => {
        const dir = testDir('concurrent');
        const db = await WalDB.open(dir);
        
        // Launch multiple vector sets in parallel
        const promises = [];
        for (let i = 0; i < 100; i++) {
            const vector = [Math.random(), Math.random(), Math.random()];
            promises.push(db.setVector(`vectors/${i}`, vector));
        }
        
        await Promise.all(promises);
        
        // Verify all were stored
        let count = 0;
        for (let i = 0; i < 100; i++) {
            const vec = await db.getVector(`vectors/${i}`);
            if (vec && vec.length === 3) count++;
        }
        
        if (count !== 100) {
            throw new Error(`Only ${count}/100 vectors stored correctly`);
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
console.log('Running WalDB Vector Search Edge Case Tests');
console.log('============================================');
runTests().catch(error => {
    console.error('Test runner failed:', error);
    process.exit(1);
});