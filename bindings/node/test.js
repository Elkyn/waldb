// Test suite for WalDB Node.js bindings
// Demonstrates usage and validates functionality

const WalDB = require('./index.js');
const fs = require('fs');
const path = require('path');

// Test utilities
function assert(condition, message) {
    if (!condition) {
        throw new Error(`Assertion failed: ${message}`);
    }
    console.log(`✓ ${message}`);
}

function cleanup(dbPath) {
    try {
        if (fs.existsSync(dbPath)) {
            fs.rmSync(dbPath, { recursive: true, force: true });
        }
    } catch (e) {
        // Ignore cleanup errors
    }
}

async function runTests() {
    const dbPath = './test_waldb_node';
    cleanup(dbPath);

    console.log('🧪 Starting WalDB Node.js tests...\n');

    try {
        // Test 1: Basic operations
        console.log('Test 1: Basic Operations');
        const db = WalDB.open(dbPath);
        
        db.set('users/alice/name', 'Alice Smith');
        db.set('users/alice/age', 30);
        db.set('users/bob/name', 'Bob Jones');
        
        assert(db.get('users/alice/name') === 'Alice Smith', 'Get string value');
        assert(db.get('users/alice/age') === 30, 'Get number value');
        assert(db.exists('users/alice'), 'Path exists');
        assert(!db.exists('users/charlie'), 'Path does not exist');
        
        // Test 2: Tree operations
        console.log('\nTest 2: Tree Operations');
        const users = db.get('users/');
        assert(typeof users === 'object', 'Get subtree returns object');
        assert(users.alice.name === 'Alice Smith', 'Subtree contains correct data');
        
        // Test 3: Firebase-style reference API
        console.log('\nTest 3: Reference API');
        const usersRef = db.ref('users');
        const aliceRef = usersRef.child('alice');
        
        aliceRef.set({ name: 'Alice Updated', age: 31 });
        const aliceData = aliceRef.get();
        assert(aliceData.name === 'Alice Updated', 'Reference set/get works');
        assert(aliceRef.exists(), 'Reference exists');
        
        // Test 4: Pattern matching
        console.log('\nTest 4: Pattern Matching');
        const names = db.getPattern('users/*/name');
        assert(Object.keys(names).length === 2, 'Pattern matching finds correct count');
        assert(names['users/alice/name'] === 'Alice Updated', 'Pattern matching gets correct value');
        
        // Test 5: Range queries
        console.log('\nTest 5: Range Queries');
        db.set('products/a001', 'Product A');
        db.set('products/a002', 'Product B');
        db.set('products/b001', 'Product C');
        
        const aProducts = db.getRange('products/a', 'products/b');
        assert(Object.keys(aProducts).length === 2, 'Range query finds correct count');
        
        // Test 6: List keys
        console.log('\nTest 6: List Keys');
        const productKeys = db.listKeys('products/');
        assert(productKeys.length === 3, 'List keys finds all products');
        assert(productKeys.includes('products/a001'), 'List keys includes expected key');
        
        // Test 7: Delete operations
        console.log('\nTest 7: Delete Operations');
        aliceRef.remove();
        assert(!aliceRef.exists(), 'Delete removes reference');
        
        db.delete('products/');
        const remainingProducts = db.listKeys('products/');
        assert(remainingProducts.length === 0, 'Delete removes subtree');
        
        // Test 8: JSON handling
        console.log('\nTest 8: JSON Handling');
        const complexData = {
            name: 'Test User',
            settings: {
                theme: 'dark',
                notifications: true
            },
            tags: ['developer', 'admin']
        };
        
        db.set('complex/user', complexData);
        const retrieved = db.get('complex/user');
        assert(retrieved.name === 'Test User', 'Complex JSON stored and retrieved');
        assert(retrieved.settings.theme === 'dark', 'Nested object preserved');
        assert(Array.isArray(retrieved.tags), 'Array preserved');
        
        // Test 9: Force overwrite
        console.log('\nTest 9: Force Overwrite');
        db.set('test/parent/child', 'child value');
        try {
            db.set('test/parent', 'parent value', false);
            assert(false, 'Should throw error when overwriting parent');
        } catch (e) {
            assert(true, 'Correctly prevents overwriting parent without force');
        }
        
        db.set('test/parent', 'parent value', true);
        assert(db.get('test/parent') === 'parent value', 'Force overwrite works');
        assert(!db.exists('test/parent/child'), 'Child removed after force overwrite');
        
        console.log('\n🎉 All tests passed!');
        
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        process.exit(1);
    } finally {
        cleanup(dbPath);
    }
}

// Performance benchmark
function benchmark() {
    console.log('\n📊 Running performance benchmark...');
    
    const dbPath = './benchmark_waldb_node';
    cleanup(dbPath);
    
    const db = WalDB.open(dbPath);
    const count = 10000;
    
    // Write benchmark
    console.log(`Writing ${count} records...`);
    const writeStart = Date.now();
    
    for (let i = 0; i < count; i++) {
        db.set(`benchmark/item_${i}`, `value_${i}`);
    }
    
    const writeTime = Date.now() - writeStart;
    const writesPerSec = Math.round(count / (writeTime / 1000));
    console.log(`Write performance: ${writesPerSec} writes/sec`);
    
    // Read benchmark
    console.log(`Reading ${count} records...`);
    const readStart = Date.now();
    
    for (let i = 0; i < count; i++) {
        db.get(`benchmark/item_${i}`);
    }
    
    const readTime = Date.now() - readStart;
    const readsPerSec = Math.round(count / (readTime / 1000));
    console.log(`Read performance: ${readsPerSec} reads/sec`);
    
    cleanup(dbPath);
}

if (require.main === module) {
    runTests().then(() => {
        benchmark();
    }).catch(console.error);
}