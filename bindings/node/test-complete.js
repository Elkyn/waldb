#!/usr/bin/env node

// Complete integration test - Firebase compatibility with type preservation
const WalDB = require('./index.js');
const assert = require('assert');
const fs = require('fs');
const path = require('path');

// Test directory
const testDir = path.join(__dirname, 'test_waldb_complete_' + process.pid);

// Clean up any existing test directory
if (fs.existsSync(testDir)) {
    fs.rmSync(testDir, { recursive: true });
}

console.log('Testing Complete WalDB Firebase Compatibility...\n');

async function runTests() {
    // Initialize database
    const db = await WalDB.open(testDir);

    // Test 1: Store complex nested object with mixed types
    console.log('1. Testing object storage with type preservation...');
    const engine = {
        id: 'abc123',
        model: 'V8 TwinTurbo',
        cylinders: 8,
        displacement: 5.0,
        turbo: true,
        features: ['fuel-injection', 'variable-timing', 'direct-injection'],
        specs: {
            horsepower: 650,
            torque: 580,
            redline: 7200,
            emissions: {
                standard: 'Euro 6',
                co2: 245.5,
                compliant: true
            }
        },
        serviceHistory: null,
        warranty: {
            years: 3,
            miles: 60000,
            comprehensive: true
        }
    };

    await db.set('System/Engines/abc123', engine);

    // Test 2: Verify complete object retrieval with types
    console.log('2. Verifying complete object retrieval...');
    const retrieved = await db.getObject('System/Engines/abc123');
    assert.deepStrictEqual(retrieved, engine);
    assert.strictEqual(typeof retrieved.cylinders, 'number');
    assert.strictEqual(typeof retrieved.turbo, 'boolean');
    assert.strictEqual(typeof retrieved.model, 'string');
    assert(Array.isArray(retrieved.features));
    assert.strictEqual(retrieved.serviceHistory, null);
    console.log('   ✓ Complete object retrieved with correct types');

    // Test 3: Direct nested property access
    console.log('3. Testing direct nested property access...');
    assert.strictEqual(await db.getObject('System/Engines/abc123/model'), 'V8 TwinTurbo');
    assert.strictEqual(await db.getObject('System/Engines/abc123/cylinders'), 8);
    assert.strictEqual(await db.getObject('System/Engines/abc123/turbo'), true);
    assert.strictEqual(await db.getObject('System/Engines/abc123/displacement'), 5.0);
    assert.strictEqual(await db.getObject('System/Engines/abc123/specs/horsepower'), 650);
    assert.strictEqual(await db.getObject('System/Engines/abc123/specs/emissions/co2'), 245.5);
    assert.strictEqual(await db.getObject('System/Engines/abc123/specs/emissions/compliant'), true);
    assert.strictEqual(await db.getObject('System/Engines/abc123/serviceHistory'), null);
    console.log('   ✓ All nested properties accessible with correct types');

    // Test 4: Update nested property
    console.log('4. Testing nested property updates...');
    await db.set('System/Engines/abc123/specs/horsepower', 675);
    assert.strictEqual(await db.getObject('System/Engines/abc123/specs/horsepower'), 675);
    
    // Verify parent object still intact
    const afterUpdate = await db.getObject('System/Engines/abc123');
    assert.strictEqual(afterUpdate.specs.horsepower, 675);
    assert.strictEqual(afterUpdate.specs.torque, 580); // Other properties unchanged
    console.log('   ✓ Nested properties updated correctly');

    // Test 5: Store multiple users (like Firebase RTDB)
    console.log('5. Testing multiple user storage...');
    const users = {
        alice: {
            name: 'Alice Johnson',
            age: 28,
            score: 95.5,
            active: true,
            roles: ['admin', 'developer'],
            metadata: null
        },
        bob: {
            name: 'Bob Smith',
            age: 34,
            score: 87.3,
            active: false,
            roles: ['user'],
            metadata: {
                lastLogin: '2024-01-15',
                attempts: 3
            }
        },
        charlie: {
            name: 'Charlie Brown',
            age: 45,
            score: 92.1,
            active: true,
            roles: ['user', 'moderator'],
            metadata: {
                lastLogin: '2024-01-20',
                attempts: 1
            }
        }
    };

    // Store entire users object
    await db.set('users', users);
    
    // Verify retrieval
    const allUsers = await db.getObject('users');
    assert.deepStrictEqual(allUsers, users);
    console.log('   ✓ Multiple users stored and retrieved correctly');

    // Test 6: Access individual users
    console.log('6. Testing individual user access...');
    const alice = await db.getObject('users/alice');
    assert.strictEqual(alice.name, 'Alice Johnson');
    assert.strictEqual(alice.age, 28);
    assert.strictEqual(alice.score, 95.5);
    assert.strictEqual(alice.active, true);
    assert.deepStrictEqual(alice.roles, ['admin', 'developer']);
    assert.strictEqual(alice.metadata, null);
    
    const bob = await db.getObject('users/bob');
    assert.strictEqual(bob.active, false);
    assert.strictEqual(bob.metadata.attempts, 3);
    console.log('   ✓ Individual users accessible with correct types');

    // Test 7: Pattern matching on users
    console.log('7. Testing pattern matching...');
    const matches = await db.getPattern('users/*/name');
    assert.strictEqual(Object.keys(matches).length, 3);
    assert.strictEqual(matches['users/alice/name'], 'Alice Johnson');
    assert.strictEqual(matches['users/bob/name'], 'Bob Smith');
    assert.strictEqual(matches['users/charlie/name'], 'Charlie Brown');
    console.log('   ✓ Pattern matching works correctly');

    // Test 8: Range queries
    console.log('8. Testing range queries...');
    const range = await db.getRange('users/alice', 'users/charlie');
    assert.strictEqual(range['users/alice/name'], 'Alice Johnson');
    assert.strictEqual(range['users/bob/name'], 'Bob Smith');
    assert.strictEqual(range['users/charlie/name'], undefined); // Exclusive end
    console.log('   ✓ Range queries work correctly');

    // Test 9: Firebase-style Reference API
    console.log('9. Testing Firebase-style Reference API...');
    const aliceRef = db.ref('users/alice');
    const aliceData = await aliceRef.get();
    assert.deepStrictEqual(aliceData, users.alice);
    
    // Update via reference
    await aliceRef.child('score').set(98.0);
    assert.strictEqual(await db.getObject('users/alice/score'), 98.0);
    
    // Add new property via reference
    await aliceRef.child('verified').set(true);
    assert.strictEqual(await db.getObject('users/alice/verified'), true);
    console.log('   ✓ Reference API works correctly');

    // Test 10: Delete operations
    console.log('10. Testing delete operations...');
    await db.delete('users/bob/metadata');
    assert.strictEqual(await db.getObject('users/bob/metadata'), null);
    assert.strictEqual(await db.getObject('users/bob/name'), 'Bob Smith'); // Other properties intact
    
    // Delete entire user
    await db.delete('users/charlie');
    assert.strictEqual(await db.getObject('users/charlie'), null);
    assert.strictEqual(await db.getObject('users/alice/name'), 'Alice Johnson'); // Others unaffected
    console.log('   ✓ Delete operations work correctly');

    // Test 11: Replace subtree
    console.log('11. Testing subtree replacement...');
    const newEngine = {
        id: 'def456',
        model: 'V6 Hybrid',
        cylinders: 6,
        displacement: 3.5,
        turbo: false,
        electric: true,
        power: 400
    };
    
    await db.set('System/Engines/abc123', newEngine, true); // Force replace
    const replaced = await db.getObject('System/Engines/abc123');
    assert.deepStrictEqual(replaced, newEngine);
    assert.strictEqual(await db.getObject('System/Engines/abc123/warranty'), null); // Old properties gone
    console.log('   ✓ Subtree replacement works correctly');

    // Test 12: Persistence
    console.log('12. Testing persistence...');
    await db.flush();
    
    // Open new instance (no cache, so this is a true reopen)
    const db2 = await WalDB.open(testDir);
    // Get individual properties since the object was flattened
    assert.strictEqual(await db2.getObject('System/Engines/abc123/model'), 'V6 Hybrid');
    assert.strictEqual(await db2.getObject('System/Engines/abc123/cylinders'), 6);
    assert.strictEqual(await db2.getObject('System/Engines/abc123/electric'), true);
    
    const persistedAlice = await db2.getObject('users/alice');
    assert.strictEqual(persistedAlice.score, 98.0);
    assert.strictEqual(persistedAlice.verified, true);
    console.log('   ✓ Data persists correctly across reopens');

    // Test 13: Empty values and null handling
    console.log('13. Testing empty values and null handling...');
    await db.set('test/empty_string', '');
    await db.set('test/null_value', null);
    await db.set('test/zero', 0);
    await db.set('test/false', false);
    
    assert.strictEqual(await db.getObject('test/empty_string'), '');
    assert.strictEqual(await db.getObject('test/null_value'), null);
    assert.strictEqual(await db.getObject('test/zero'), 0);
    assert.strictEqual(await db.getObject('test/false'), false);
    assert.strictEqual(await db.getObject('test/nonexistent'), null);
    console.log('   ✓ Empty values and null handled correctly');

    // Test 14: Large dataset
    console.log('14. Testing large dataset...');
    const largeObj = {};
    for (let i = 0; i < 1000; i++) {
        largeObj[`key${i}`] = {
            id: i,
            value: `value${i}`,
            active: i % 2 === 0,
            score: i * 1.5,
            tags: [`tag${i}`, `tag${i+1}`]
        };
    }
    
    await db.set('large_dataset', largeObj);
    const retrievedLarge = await db.getObject('large_dataset');
    assert.deepStrictEqual(retrievedLarge, largeObj);
    
    // Test specific nested access
    assert.strictEqual(await db.getObject('large_dataset/key500/id'), 500);
    assert.strictEqual(await db.getObject('large_dataset/key500/value'), 'value500');
    assert.strictEqual(await db.getObject('large_dataset/key500/active'), true);
    assert.strictEqual(await db.getObject('large_dataset/key500/score'), 750);
    console.log('   ✓ Large dataset handled correctly');

    // Test 15: Complex updates
    console.log('15. Testing complex update scenarios...');
    
    // Add new branch to existing structure
    await db.set('System/Engines/xyz789', {
        model: 'Electric Motor',
        power: 300,
        torque: 400,
        battery: {
            capacity: 75,
            type: 'Lithium-ion'
        }
    });
    
    // Verify both engines exist
    assert.strictEqual(await db.getObject('System/Engines/abc123/model'), 'V6 Hybrid');
    assert.strictEqual(await db.getObject('System/Engines/xyz789/model'), 'Electric Motor');
    assert.strictEqual(await db.getObject('System/Engines/xyz789/battery/capacity'), 75);
    
    console.log('   ✓ Complex updates work correctly');

    // Clean up
    console.log('\n✅ All Firebase compatibility tests passed!');
    fs.rmSync(testDir, { recursive: true });
    process.exit(0);
}

// Run tests
runTests().catch(error => {
    console.error('Test failed:', error);
    process.exit(1);
});