#!/usr/bin/env node

const { WalDB } = require('./index.js');
const path = require('path');
const os = require('os');

// Helper to create test directory
function testDir(name) {
    return path.join(os.tmpdir(), `waldb_test_vector_${name}_${process.pid}`);
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

    // Test 1: Basic vector storage and retrieval
    await test('Vector Set/Get', async () => {
        const dir = testDir('basic');
        const db = await WalDB.open(dir);
        
        const vector = [0.1, 0.2, 0.3, 0.4, 0.5];
        await db.setVector('embeddings/doc1', vector);
        
        const retrieved = await db.getVector('embeddings/doc1');
        if (!retrieved || retrieved.length !== vector.length) {
            throw new Error('Vector not retrieved correctly');
        }
        
        for (let i = 0; i < vector.length; i++) {
            if (Math.abs(retrieved[i] - vector[i]) > 0.0001) {
                throw new Error(`Vector mismatch at index ${i}`);
            }
        }
        
        cleanup(dir);
    });

    // Test 2: Vector search with cosine similarity
    await test('Vector Similarity Search', async () => {
        const dir = testDir('similarity');
        const db = await WalDB.open(dir);
        
        // Store some document embeddings
        await db.setVector('docs/1/embedding', [1.0, 0.0, 0.0]);
        await db.setVector('docs/2/embedding', [0.0, 1.0, 0.0]);
        await db.setVector('docs/3/embedding', [0.0, 0.0, 1.0]);
        await db.setVector('docs/4/embedding', [0.707, 0.707, 0.0]); // 45° between doc1 and doc2
        
        // Store document content
        await db.set('docs/1/title', 'Document One');
        await db.set('docs/2/title', 'Document Two');
        await db.set('docs/3/title', 'Document Three');
        await db.set('docs/4/title', 'Document Four');
        
        // Search for documents similar to [1, 0, 0] (should match doc1 best)
        const results = await db.advancedSearch({
            pattern: 'docs/*',
            vector: {
                query: [1.0, 0.0, 0.0],
                field: 'embedding',
                topK: 2
            },
            limit: 2
        });
        
        if (results.length !== 2) {
            throw new Error(`Expected 2 results, got ${results.length}`);
        }
        
        // Check that doc1 is the top result
        const topResult = results[0];
        const title = topResult.find(([k]) => k.endsWith('/title'));
        if (!title || title[1] !== 'Document One') {
            throw new Error('Expected Document One as top result');
        }
        
        cleanup(dir);
    });

    // Test 3: Text search
    await test('Text Search', async () => {
        const dir = testDir('text');
        const db = await WalDB.open(dir);
        
        // Store some documents
        await db.set('articles/1/title', 'Introduction to WalDB');
        await db.set('articles/1/content', 'WalDB is a high-performance database');
        await db.set('articles/2/title', 'Advanced WalDB Features');
        await db.set('articles/2/content', 'Learn about advanced features like vector search');
        await db.set('articles/3/title', 'Database Performance Tips');
        await db.set('articles/3/content', 'Tips for optimizing database performance');
        
        // Search for "WalDB"
        const results = await db.advancedSearch({
            pattern: 'articles/*',
            text: {
                query: 'WalDB',
                fields: ['title', 'content'],
                fuzzy: false,
                caseInsensitive: true
            }
        });
        
        // Should find articles 1 and 2
        if (results.length !== 2) {
            throw new Error(`Expected 2 results for 'WalDB', got ${results.length}`);
        }
        
        cleanup(dir);
    });

    // Test 4: Hybrid search (vector + text + filters)
    await test('Hybrid Search', async () => {
        const dir = testDir('hybrid');
        const db = await WalDB.open(dir);
        
        // Create product catalog with embeddings and metadata
        const products = [
            { id: '1', name: 'Red T-Shirt', category: 'clothing', price: 19.99, embedding: [0.8, 0.2, 0.1] },
            { id: '2', name: 'Blue Jeans', category: 'clothing', price: 49.99, embedding: [0.3, 0.7, 0.2] },
            { id: '3', name: 'Green Jacket', category: 'clothing', price: 89.99, embedding: [0.4, 0.5, 0.6] },
            { id: '4', name: 'Red Shoes', category: 'footwear', price: 79.99, embedding: [0.9, 0.1, 0.3] },
            { id: '5', name: 'Blue Hat', category: 'accessories', price: 24.99, embedding: [0.2, 0.8, 0.1] }
        ];
        
        for (const product of products) {
            await db.set(`products/${product.id}/name`, product.name);
            await db.set(`products/${product.id}/category`, product.category);
            await db.set(`products/${product.id}/price`, product.price.toString());
            await db.setVector(`products/${product.id}/embedding`, product.embedding);
        }
        
        // Search for red items in clothing category under $50
        const results = await db.advancedSearch({
            pattern: 'products/*',
            text: {
                query: 'Red',
                fields: ['name'],
                fuzzy: false,
                caseInsensitive: true
            },
            vector: {
                query: [0.85, 0.15, 0.15], // Similar to "red" embedding
                field: 'embedding',
                topK: 5
            },
            filters: [
                { field: 'category', op: '==', value: 'clothing' },
                { field: 'price', op: '<', value: '50' }
            ],
            scoring: {
                vector: 0.4,
                text: 0.4,
                filter: 0.2
            },
            limit: 2
        });
        
        // Should find Red T-Shirt as it matches all criteria
        if (results.length === 0) {
            throw new Error('No results found for hybrid search');
        }
        
        const topResult = results[0];
        const name = topResult.find(([k]) => k.endsWith('/name'));
        if (!name || name[1] !== 'Red T-Shirt') {
            throw new Error('Expected Red T-Shirt as top result');
        }
        
        cleanup(dir);
    });

    // Test 5: Advanced search with object reconstruction
    await test('Advanced Search Objects', async () => {
        const dir = testDir('objects');
        const db = await WalDB.open(dir);
        
        // Store data
        await db.set('users/alice/name', 'Alice');
        await db.set('users/alice/role', 'admin');
        await db.setVector('users/alice/preferences', [0.5, 0.5, 0.5]);
        
        await db.set('users/bob/name', 'Bob');
        await db.set('users/bob/role', 'user');
        await db.setVector('users/bob/preferences', [0.3, 0.7, 0.2]);
        
        // Search and get reconstructed objects
        const results = await db.advancedSearchObjects({
            pattern: 'users/*',
            vector: {
                query: [0.4, 0.6, 0.3],
                field: 'preferences',
                topK: 2
            }
        });
        
        if (results.length !== 2) {
            throw new Error(`Expected 2 results, got ${results.length}`);
        }
        
        // Check that we get proper objects
        const firstUser = results[0];
        if (!firstUser.name || !firstUser.role) {
            throw new Error('Object not properly reconstructed');
        }
        
        cleanup(dir);
    });

    // Test 6: Empty vector handling
    await test('Empty Vector Handling', async () => {
        const dir = testDir('empty');
        const db = await WalDB.open(dir);
        
        // Get non-existent vector
        const result = await db.getVector('non/existent');
        if (result !== null) {
            throw new Error('Expected null for non-existent vector');
        }
        
        cleanup(dir);
    });

    // Test 7: Invalid vector input
    await test('Invalid Vector Input', async () => {
        const dir = testDir('invalid');
        const db = await WalDB.open(dir);
        
        try {
            await db.setVector('test', 'not an array');
            throw new Error('Should have rejected non-array');
        } catch (e) {
            if (!e.message.includes('must be an array')) {
                throw e;
            }
        }
        
        try {
            await db.setVector('test', [1, 'two', 3]);
            throw new Error('Should have rejected non-numeric array');
        } catch (e) {
            if (!e.message.includes('must be an array of numbers')) {
                throw e;
            }
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
console.log('Running WalDB Vector/Text Search Tests');
console.log('=======================================');
runTests().catch(error => {
    console.error('Test runner failed:', error);
    process.exit(1);
});