#!/usr/bin/env node

/**
 * Simple test suite for WalDB Node.js bindings
 */

const native = require('./index.node');
const fs = require('fs');

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log(`✅ ${name}`);
        passed++;
    } catch (error) {
        console.log(`❌ ${name}: ${error.message}`);
        failed++;
    }
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message || 'Assertion failed');
    }
}

console.log('🧪 WalDB Node.js Bindings Test Suite');
console.log('====================================\n');

// Setup
const testPath = './test_waldb_simple';
if (fs.existsSync(testPath)) {
    fs.rmSync(testPath, { recursive: true, force: true });
}

// Tests
test('Can open database', () => {
    const result = native.open(testPath);
    assert(result === testPath, 'Should return path');
});

test('Can set and get value', () => {
    native.set(testPath, 'test/key', 'test value');
    const value = native.get(testPath, 'test/key');
    assert(value === 'test value', `Expected 'test value', got '${value}'`);
});

test('Can delete value', () => {
    native.set(testPath, 'delete/me', 'value');
    native.delete(testPath, 'delete/me');
    const value = native.get(testPath, 'delete/me');
    assert(value === null, 'Should be null after delete');
});

test('Can handle non-existent keys', () => {
    const value = native.get(testPath, 'does/not/exist');
    assert(value === null, 'Should return null for non-existent key');
});

test('Can get pattern matches', () => {
    native.set(testPath, 'users/alice/name', 'Alice');
    native.set(testPath, 'users/bob/name', 'Bob');
    const matches = native.getPattern(testPath, 'users/*/name');
    assert(matches['users/alice/name'] === 'Alice', 'Should find Alice');
    assert(matches['users/bob/name'] === 'Bob', 'Should find Bob');
});

test('Can get range', () => {
    native.set(testPath, 'range/a', '1');
    native.set(testPath, 'range/b', '2');
    native.set(testPath, 'range/c', '3');
    const range = native.getRange(testPath, 'range/a', 'range/c');
    assert(Object.keys(range).length === 2, 'Should have 2 items (a and b)');
});

test('Can flush', () => {
    native.flush(testPath);
    // If no error thrown, flush succeeded
});

test('Store caching works', () => {
    // Clear cache and set a value
    native.clearCache();
    native.set(testPath, 'cache/test', 'value1');
    
    // Get should work with cached store
    const value = native.get(testPath, 'cache/test');
    assert(value === 'value1', 'Should get value from cached store');
});

// Performance mini-benchmark
console.log('\n📊 Performance Check:');
const perfPath = './perf_test';
if (fs.existsSync(perfPath)) {
    fs.rmSync(perfPath, { recursive: true, force: true });
}

native.open(perfPath);
const writeStart = Date.now();
for (let i = 0; i < 1000; i++) {
    native.set(perfPath, `key_${i}`, `value_${i}`);
}
const writeTime = Date.now() - writeStart;

const readStart = Date.now();
for (let i = 0; i < 1000; i++) {
    native.get(perfPath, `key_${i}`);
}
const readTime = Date.now() - readStart;

console.log(`   Writes: ${Math.round(1000/(writeTime/1000))} ops/sec`);
console.log(`   Reads:  ${Math.round(1000/(readTime/1000))} ops/sec`);

// Cleanup
native.clearCache();
if (fs.existsSync(testPath)) {
    fs.rmSync(testPath, { recursive: true, force: true });
}
if (fs.existsSync(perfPath)) {
    fs.rmSync(perfPath, { recursive: true, force: true });
}

// Summary
console.log('\n' + '='.repeat(40));
console.log(`Results: ${passed} passed, ${failed} failed`);

if (failed > 0) {
    process.exit(1);
}