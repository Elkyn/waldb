#!/usr/bin/env node

// Fundamental behavior tests - make sure basics work correctly
const WalDB = require('./index.js');
const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { cleanupTestDir } = require('./test-utils.js');

// Test directory
const testDir = path.join(__dirname, 'test_fundamentals_' + process.pid);

// Clean up any existing test directory
if (fs.existsSync(testDir)) {
    fs.rmSync(testDir, { recursive: true, force: true });
}

console.log('Testing WalDB Fundamental Behaviors...\n');

async function test(name, fn) {
    try {
        await fn();
        console.log(`✅ ${name}`);
    } catch (error) {
        console.error(`❌ ${name}`);
        console.error(`   Error: ${error.message}`);
        console.error(`   Stack: ${error.stack}`);
        process.exit(1);
    }
}

async function runTests() {
    // Test 1: Basic CRUD
    await test('Basic CRUD operations', async () => {
        const db = await WalDB.open(testDir + '/crud');
        
        // Create
        await db.set('key1', 'value1');
        assert.strictEqual(await db.getObject('key1'), 'value1');
        
        // Update
        await db.set('key1', 'updated');
        assert.strictEqual(await db.getObject('key1'), 'updated');
        
        // Delete
        await db.delete('key1');
        assert.strictEqual(await db.getObject('key1'), null);
        
        // Read non-existent
        assert.strictEqual(await db.getObject('never_existed'), null);
    });

    // Test 2: Persistence
    await test('Data persists across reopens', async () => {
        const db1 = await WalDB.open(testDir + '/persist');
        await db1.set('persistent', 'data');
        await db1.flush();
        
        // Reopen
        const db2 = await WalDB.open(testDir + '/persist');
        assert.strictEqual(await db2.getObject('persistent'), 'data');
    });

    // Test 3: Tree structure rules
    await test('Tree structure parent/child rules', async () => {
        const db = await WalDB.open(testDir + '/tree');
        
        // Can't write child under scalar
        await db.set('scalar', 'value');
        await assert.rejects(async () => {
            await db.set('scalar/child', 'should fail');
        }, /Cannot write under scalar parent/);
        
        // Can write siblings
        await db.set('parent/child1', 'value1');
        await db.set('parent/child2', 'value2');
        assert.strictEqual(await db.getObject('parent/child1'), 'value1');
        assert.strictEqual(await db.getObject('parent/child2'), 'value2');
        
        // Can replace subtree with force
        await db.set('parent', 'new_value', true);
        assert.strictEqual(await db.getObject('parent'), 'new_value');
        assert.strictEqual(await db.getObject('parent/child1'), null);
    });

    // Test 4: Empty string handling
    await test('Empty string vs null/undefined', async () => {
        const db = await WalDB.open(testDir + '/empty');
        
        // Empty string is a valid value
        await db.set('empty', '');
        assert.strictEqual(await db.getObject('empty'), '');
        assert.strictEqual(typeof await db.getObject('empty'), 'string');
        
        // Null is different from empty string
        await db.set('null_val', null);
        assert.strictEqual(await db.getObject('null_val'), null);
        
        // Non-existent is null
        assert.strictEqual(await db.getObject('nonexistent'), null);
        
        // After delete, becomes null
        await db.set('to_delete', 'value');
        await db.delete('to_delete');
        assert.strictEqual(await db.getObject('to_delete'), null);
    });

    // Test 5: Path edge cases
    await test('Path edge cases', async () => {
        const db = await WalDB.open(testDir + '/paths');
        
        // Root level keys
        await db.set('root', 'value');
        assert.strictEqual(await db.getObject('root'), 'value');
        
        // Deep nesting
        const deepPath = 'a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p';
        await db.set(deepPath, 'deep');
        assert.strictEqual(await db.getObject(deepPath), 'deep');
        
        // Paths with special characters
        await db.set('path with spaces/key', 'value');
        assert.strictEqual(await db.getObject('path with spaces/key'), 'value');
        
        await db.set('path-with-dash', 'value');
        assert.strictEqual(await db.getObject('path-with-dash'), 'value');
        
        await db.set('path_with_underscore', 'value');
        assert.strictEqual(await db.getObject('path_with_underscore'), 'value');
        
        await db.set('path.with.dots', 'value');
        assert.strictEqual(await db.getObject('path.with.dots'), 'value');
        
        // Unicode in paths
        await db.set('用户/名字', '张三');
        assert.strictEqual(await db.getObject('用户/名字'), '张三');
        
        await db.set('emoji/😀', 'happy');
        assert.strictEqual(await db.getObject('emoji/😀'), 'happy');
    });

    // Test 6: Value edge cases
    await test('Value edge cases', async () => {
        const db = await WalDB.open(testDir + '/values');
        
        // Very long string
        const longString = 'x'.repeat(100000);
        await db.set('long', longString);
        assert.strictEqual(await db.getObject('long'), longString);
        
        // Binary-like string
        const binaryString = '\x00\x01\x02\xFF';
        await db.set('binary', binaryString);
        assert.strictEqual(await db.getObject('binary'), binaryString);
        
        // JSON string
        const jsonString = '{"nested": {"key": "value"}}';
        await db.set('json', jsonString);
        assert.strictEqual(await db.getObject('json'), jsonString);
        
        // Number-like strings
        await db.set('number_string', '123.456');
        assert.strictEqual(await db.getObject('number_string'), '123.456');
        assert.strictEqual(typeof await db.getObject('number_string'), 'string');
        
        // Boolean-like strings
        await db.set('bool_string', 'false');
        assert.strictEqual(await db.getObject('bool_string'), 'false');
        assert.strictEqual(typeof await db.getObject('bool_string'), 'string');
    });

    // Test 7: Overwrite behavior
    await test('Overwrite and update behavior', async () => {
        const db = await WalDB.open(testDir + '/overwrite');
        
        // Simple overwrite
        await db.set('key', 'v1');
        await db.set('key', 'v2');
        await db.set('key', 'v3');
        assert.strictEqual(await db.getObject('key'), 'v3');
        
        // Type changes on overwrite
        await db.set('mutable', 'string');
        assert.strictEqual(await db.getObject('mutable'), 'string');
        
        await db.set('mutable', 123);
        assert.strictEqual(await db.getObject('mutable'), 123);
        
        await db.set('mutable', true);
        assert.strictEqual(await db.getObject('mutable'), true);
        
        await db.set('mutable', null);
        assert.strictEqual(await db.getObject('mutable'), null);
        
        await db.set('mutable', ['array']);
        assert.deepStrictEqual(await db.getObject('mutable'), ['array']);
        
        await db.set('mutable', {obj: 'value'});
        assert.deepStrictEqual(await db.getObject('mutable'), {obj: 'value'});
    });

    // Test 8: Delete behavior
    await test('Delete behavior and cascading', async () => {
        const db = await WalDB.open(testDir + '/delete');
        
        // Setup tree
        await db.set('tree/branch1/leaf1', 'v1');
        await db.set('tree/branch1/leaf2', 'v2');
        await db.set('tree/branch2/leaf3', 'v3');
        
        // Delete leaf - only leaf affected
        await db.delete('tree/branch1/leaf1');
        assert.strictEqual(await db.getObject('tree/branch1/leaf1'), null);
        assert.strictEqual(await db.getObject('tree/branch1/leaf2'), 'v2');
        
        // Delete branch - all children deleted (cascade)
        await db.delete('tree/branch1');
        assert.strictEqual(await db.getObject('tree/branch1/leaf2'), null);
        assert.strictEqual(await db.getObject('tree/branch2/leaf3'), 'v3'); // Other branch unaffected
        
        // Delete root - everything gone
        await db.delete('tree');
        assert.strictEqual(await db.getObject('tree/branch2/leaf3'), null);
    });

    // Test 9: Concurrent operations (single process)
    await test('Rapid consecutive operations', async () => {
        const db = await WalDB.open(testDir + '/rapid');
        
        // Rapid writes
        for (let i = 0; i < 1000; i++) {
            await db.set(`rapid/key${i}`, `value${i}`);
        }
        
        // Verify all
        for (let i = 0; i < 1000; i++) {
            assert.strictEqual(await db.getObject(`rapid/key${i}`), `value${i}`);
        }
        
        // Rapid updates
        for (let i = 0; i < 1000; i++) {
            await db.set(`rapid/key${i}`, `updated${i}`);
        }
        
        // Verify updates
        for (let i = 0; i < 1000; i++) {
            assert.strictEqual(await db.getObject(`rapid/key${i}`), `updated${i}`);
        }
        
        // Rapid deletes
        for (let i = 0; i < 500; i++) {
            await db.delete(`rapid/key${i}`);
        }
        
        // Verify deletes
        for (let i = 0; i < 500; i++) {
            assert.strictEqual(await db.getObject(`rapid/key${i}`), null);
        }
        for (let i = 500; i < 1000; i++) {
            assert.strictEqual(await db.getObject(`rapid/key${i}`), `updated${i}`);
        }
    });

    // Test 10: Object/array at root
    await test('Objects and arrays at root level', async () => {
        const db = await WalDB.open(testDir + '/root_objects');
        
        // Object at root
        const rootObj = {
            name: 'Root Object',
            value: 42,
            nested: {
                deep: true
            }
        };
        await db.set('', rootObj);  // Empty string as root
        
        // Should be able to access nested
        assert.strictEqual(await db.getObject('name'), 'Root Object');
        assert.strictEqual(await db.getObject('value'), 42);
        assert.strictEqual(await db.getObject('nested/deep'), true);
    });

    // Test 11: Range boundary conditions
    await test('Range query boundaries', async () => {
        const db = await WalDB.open(testDir + '/ranges');
        
        await db.set('a', '1');
        await db.set('b', '2');
        await db.set('c', '3');
        await db.set('d', '4');
        
        // Inclusive start, exclusive end
        let range = await db.getRange('b', 'd');
        assert.strictEqual(range['b'], '2');
        assert.strictEqual(range['c'], '3');
        assert.strictEqual(range['d'], undefined); // Exclusive
        
        // Empty range
        range = await db.getRange('x', 'y');
        assert.deepStrictEqual(Object.keys(range), []);
        
        // Reverse range (should be empty)
        range = await db.getRange('d', 'a');
        assert.deepStrictEqual(Object.keys(range), []);
    });

    // Test 12: Pattern matching edge cases
    await test('Pattern matching edge cases', async () => {
        const db = await WalDB.open(testDir + '/patterns');
        
        await db.set('test1', 'v1');
        await db.set('test2', 'v2');
        await db.set('test', 'v3');
        await db.set('testing', 'v4');
        
        // Star matches zero or more
        let matches = await db.getPattern('test*');
        assert.strictEqual(Object.keys(matches).length, 4);
        
        // Question mark matches exactly one
        matches = await db.getPattern('test?');
        assert.strictEqual(matches['test1'], 'v1');
        assert.strictEqual(matches['test2'], 'v2');
        assert.strictEqual(matches['test'], undefined); // Too short
        assert.strictEqual(matches['testing'], undefined); // Too long
    });

    // Test 13: References (Firebase-style API)
    await test('Reference API', async () => {
        const db = await WalDB.open(testDir + '/refs');
        
        const ref = db.ref('users/alice');
        await ref.set({name: 'Alice', age: 30});
        
        const childRef = ref.child('profile');
        await childRef.set({bio: 'Developer'});
        
        assert.deepStrictEqual(await ref.get(), {
            name: 'Alice',
            age: 30,
            profile: {
                bio: 'Developer'
            }
        });
        
        assert.strictEqual((await childRef.get()).bio, 'Developer');
        
        await childRef.remove();
        assert.strictEqual((await ref.get()).profile, undefined);
    });

    // Test 14: Write-after-delete
    await test('Write after delete', async () => {
        const db = await WalDB.open(testDir + '/write_after_delete');
        
        await db.set('key', 'value1');
        await db.delete('key');
        await db.set('key', 'value2'); // Should work
        assert.strictEqual(await db.getObject('key'), 'value2');
        
        // Complex case: delete parent, recreate structure
        await db.set('parent/child/grandchild', 'v1');
        await db.delete('parent');
        await db.set('parent/child/grandchild', 'v2');
        assert.strictEqual(await db.getObject('parent/child/grandchild'), 'v2');
    });

    // Test 15: Special key names
    await test('Special key names', async () => {
        const db = await WalDB.open(testDir + '/special_keys');
        
        // Keys that might be problematic
        await db.set('__proto__', 'value');
        assert.strictEqual(await db.getObject('__proto__'), 'value');
        
        await db.set('constructor', 'value');
        assert.strictEqual(await db.getObject('constructor'), 'value');
        
        await db.set('toString', 'value');
        assert.strictEqual(await db.getObject('toString'), 'value');
        
        await db.set('', 'empty key');  // Empty string as key
        assert.strictEqual(await db.getObject(''), 'empty key');
        
        // Very long key
        const longKey = 'k'.repeat(10000);
        await db.set(longKey, 'value');
        assert.strictEqual(await db.getObject(longKey), 'value');
    });

    // Clean up
    console.log('\n✅ All fundamental behavior tests passed!');
    await cleanupTestDir(testDir, db);
    process.exit(0);
}

// Run all tests
runTests().catch(error => {
    console.error('Test runner failed:', error);
    process.exit(1);
});