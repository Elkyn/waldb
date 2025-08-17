#!/usr/bin/env node

// Type Preservation Tests for WalDB Node.js Bindings
const WalDB = require('./index.js');
const assert = require('assert');
const fs = require('fs');
const path = require('path');

// Test directory
const testDir = path.join(__dirname, 'test_waldb_types_' + process.pid);

// Clean up any existing test directory
if (fs.existsSync(testDir)) {
    fs.rmSync(testDir, { recursive: true });
}

// Initialize database
// const db = WalDB.open(testDir);

console.log('Testing WalDB Type Preservation...\n');

async function test(name, fn) {
    try {
        await fn();
        console.log(`✅ ${name}`);
    } catch (error) {
        console.error(`❌ ${name}`);
        console.error(`   Error: ${error.message}`);
        process.exit(1);
    }
}

async function runTests() {
  const db = await WalDB.open(testDir);
// Test primitive types
await test('String preservation', async () => {
    await db.set('string_test', 'hello world');
    const result = await db.getObject('string_test');
    assert.strictEqual(result, 'hello world');
    assert.strictEqual(typeof result, 'string');
});

await test('Number preservation', async () => {
    await db.set('integer_test', 42);
    const intResult = await db.getObject('integer_test');
    assert.strictEqual(intResult, 42);
    assert.strictEqual(typeof intResult, 'number');
    
    await db.set('float_test', 3.14159);
    const floatResult = await db.getObject('float_test');
    assert.strictEqual(floatResult, 3.14159);
    assert.strictEqual(typeof floatResult, 'number');
    
    await db.set('negative_test', -273.15);
    const negResult = await db.getObject('negative_test');
    assert.strictEqual(negResult, -273.15);
    assert.strictEqual(typeof negResult, 'number');
});

await test('Boolean preservation', async () => {
    await db.set('true_test', true);
    const trueResult = await db.getObject('true_test');
    assert.strictEqual(trueResult, true);
    assert.strictEqual(typeof trueResult, 'boolean');
    
    await db.set('false_test', false);
    const falseResult = await db.getObject('false_test');
    assert.strictEqual(falseResult, false);
    assert.strictEqual(typeof falseResult, 'boolean');
});

await test('Null preservation', async () => {
    await db.set('null_test', null);
    const result = await db.getObject('null_test');
    assert.strictEqual(result, null);
});

await test('Array preservation', async () => {
    const arr = [1, 'two', true, null, { nested: 'obj' }];
    await db.set('array_test', arr);
    const result = await db.getObject('array_test');
    assert.deepStrictEqual(result, arr);
    assert(Array.isArray(result));
});

// Test objects with mixed types
await test('Object with mixed types', async () => {
    const obj = {
        name: 'Alice',
        age: 30,
        active: true,
        balance: 1234.56,
        tags: ['developer', 'nodejs'],
        metadata: null,
        settings: {
            theme: 'dark',
            notifications: false,
            volume: 0.75
        }
    };
    
    await db.set('users/alice', obj);
    const result = await db.getObject('users/alice');
    
    assert.strictEqual(result.name, 'Alice');
    assert.strictEqual(typeof result.name, 'string');
    
    assert.strictEqual(result.age, 30);
    assert.strictEqual(typeof result.age, 'number');
    
    assert.strictEqual(result.active, true);
    assert.strictEqual(typeof result.active, 'boolean');
    
    assert.strictEqual(result.balance, 1234.56);
    assert.strictEqual(typeof result.balance, 'number');
    
    assert.deepStrictEqual(result.tags, ['developer', 'nodejs']);
    assert(Array.isArray(result.tags));
    
    assert.strictEqual(result.metadata, null);
    
    assert.strictEqual(result.settings.theme, 'dark');
    assert.strictEqual(result.settings.notifications, false);
    assert.strictEqual(result.settings.volume, 0.75);
});

// Test accessing nested properties directly
await test('Direct nested property access with types', async () => {
    const engine = {
        model: 'V8',
        cylinders: 8,
        turbo: false,
        displacement: 5.0,
        features: ['fuel-injection', 'variable-timing']
    };
    
    await db.set('System/Engines/abc123', engine);
    
    // Access nested properties directly
    assert.strictEqual(await db.getObject('System/Engines/abc123/model'), 'V8');
    assert.strictEqual(typeof await db.getObject('System/Engines/abc123/model'), 'string');
    
    assert.strictEqual(await db.getObject('System/Engines/abc123/cylinders'), 8);
    assert.strictEqual(typeof await db.getObject('System/Engines/abc123/cylinders'), 'number');
    
    assert.strictEqual(await db.getObject('System/Engines/abc123/turbo'), false);
    assert.strictEqual(typeof await db.getObject('System/Engines/abc123/turbo'), 'boolean');
    
    assert.strictEqual(await db.getObject('System/Engines/abc123/displacement'), 5.0);
    assert.strictEqual(typeof await db.getObject('System/Engines/abc123/displacement'), 'number');
    
    const features = await db.getObject('System/Engines/abc123/features');
    assert.deepStrictEqual(features, ['fuel-injection', 'variable-timing']);
    assert(Array.isArray(features));
});

// Test edge cases
await test('Empty string preservation', async () => {
    await db.set('empty_string', '');
    const result = await db.getObject('empty_string');
    assert.strictEqual(result, '');
    assert.strictEqual(typeof result, 'string');
});

await test('Zero preservation', async () => {
    await db.set('zero_value', 0);
    const result = await db.getObject('zero_value');
    assert.strictEqual(result, 0);
    assert.strictEqual(typeof result, 'number');
});

await test('String that looks like number', async () => {
    await db.set('string_number', '123');
    const result = await db.getObject('string_number');
    assert.strictEqual(result, '123');
    assert.strictEqual(typeof result, 'string', 'String "123" should remain a string');
});

await test('String that looks like boolean', async () => {
    await db.set('string_bool', 'true');
    const result = await db.getObject('string_bool');
    assert.strictEqual(result, 'true');
    assert.strictEqual(typeof result, 'string', 'String "true" should remain a string');
});

// Test with range queries
await test('Type preservation in range queries', async () => {
    await db.set('rangetest/item1', 100);
    await db.set('rangetest/item2', 'string value');
    await db.set('rangetest/item3', true);
    await db.set('rangetest/item4', null);
    await db.set('rangetest/item5', [1, 2, 3]);
    
    const results = await db.getRange('rangetest/item1', 'rangetest/item6');
    
    assert.strictEqual(results['rangetest/item1'], 100);
    assert.strictEqual(typeof results['rangetest/item1'], 'number');
    
    assert.strictEqual(results['rangetest/item2'], 'string value');
    assert.strictEqual(typeof results['rangetest/item2'], 'string');
    
    assert.strictEqual(results['rangetest/item3'], true);
    assert.strictEqual(typeof results['rangetest/item3'], 'boolean');
    
    assert.strictEqual(results['rangetest/item4'], null);
    
    assert.deepStrictEqual(results['rangetest/item5'], [1, 2, 3]);
    assert(Array.isArray(results['rangetest/item5']));
});

// Test with pattern matching
await test('Type preservation in pattern queries', async () => {
    await db.set('patterntest/test1', 42);
    await db.set('patterntest/test2', 'hello');
    await db.set('patterntest/test3', false);
    
    const results = await db.getPattern('patterntest/test*');
    
    assert.strictEqual(results['patterntest/test1'], 42);
    assert.strictEqual(typeof results['patterntest/test1'], 'number');
    
    assert.strictEqual(results['patterntest/test2'], 'hello');
    assert.strictEqual(typeof results['patterntest/test2'], 'string');
    
    assert.strictEqual(results['patterntest/test3'], false);
    assert.strictEqual(typeof results['patterntest/test3'], 'boolean');
});

// Test updating values with different types
await test('Type change on update', async () => {
    await db.set('mutable/value', 'initial string');
    assert.strictEqual(await db.getObject('mutable/value'), 'initial string');
    
    await db.set('mutable/value', 999);
    assert.strictEqual(await db.getObject('mutable/value'), 999);
    assert.strictEqual(typeof await db.getObject('mutable/value'), 'number');
    
    await db.set('mutable/value', true);
    assert.strictEqual(await db.getObject('mutable/value'), true);
    assert.strictEqual(typeof await db.getObject('mutable/value'), 'boolean');
});

// Test complex nested structure
await test('Complex nested structure with all types', async () => {
    const complex = {
        id: 12345,
        name: 'Complex Object',
        enabled: true,
        score: 98.76,
        tags: ['a', 'b', 'c'],
        metadata: null,
        nested: {
            level2: {
                level3: {
                    deep: 'value',
                    number: 42,
                    bool: false,
                    array: [1, 2, 3]
                }
            }
        },
        mixed_array: [
            'string',
            123,
            true,
            null,
            { obj: 'inside array' }
        ]
    };
    
    await db.set('complex/data', complex);
    const result = await db.getObject('complex/data');
    
    // Deep equality check
    assert.deepStrictEqual(result, complex);
    
    // Check specific nested values and their types
    assert.strictEqual(result.nested.level2.level3.deep, 'value');
    assert.strictEqual(typeof result.nested.level2.level3.deep, 'string');
    
    assert.strictEqual(result.nested.level2.level3.number, 42);
    assert.strictEqual(typeof result.nested.level2.level3.number, 'number');
    
    assert.strictEqual(result.nested.level2.level3.bool, false);
    assert.strictEqual(typeof result.nested.level2.level3.bool, 'boolean');
    
    // Check mixed array types
    assert.strictEqual(result.mixed_array[0], 'string');
    assert.strictEqual(typeof result.mixed_array[0], 'string');
    
    assert.strictEqual(result.mixed_array[1], 123);
    assert.strictEqual(typeof result.mixed_array[1], 'number');
    
    assert.strictEqual(result.mixed_array[2], true);
    assert.strictEqual(typeof result.mixed_array[2], 'boolean');
    
    assert.strictEqual(result.mixed_array[3], null);
    
    assert.deepStrictEqual(result.mixed_array[4], { obj: 'inside array' });
});

// Test special string cases
await test('Strings with colons (edge case for encoding)', async () => {
    await db.set('url_test', 'https://example.com:8080');
    const result = await db.getObject('url_test');
    assert.strictEqual(result, 'https://example.com:8080');
    assert.strictEqual(typeof result, 'string');
});

await test('Strings starting with type prefixes', async () => {
    await db.set('prefix_test1', 's:this looks like encoded');
    const result1 = await db.getObject('prefix_test1');
    assert.strictEqual(result1, 's:this looks like encoded');
    assert.strictEqual(typeof result1, 'string');
    
    await db.set('prefix_test2', 'n:123');
    const result2 = await db.getObject('prefix_test2');
    assert.strictEqual(result2, 'n:123');
    assert.strictEqual(typeof result2, 'string');
});

// Clean up
console.log('\n✅ All type preservation tests passed!');
fs.rmSync(testDir, { recursive: true });
process.exit(0);
}

runTests().catch(error => {
  console.error('Test runner failed:', error);
  process.exit(1);
});
