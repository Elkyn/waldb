const WalDB = require('./index.js');
const crypto = require('crypto');

async function benchmarkFiles() {
    const db = await WalDB.open('/tmp/bench-files');
    
    console.log('📁 FILE OPERATIONS BENCHMARK\n');
    
    // Generate test files of different sizes
    const sizes = [
        { name: '1KB', size: 1024 },
        { name: '10KB', size: 10 * 1024 },
        { name: '100KB', size: 100 * 1024 },
        { name: '1MB', size: 1024 * 1024 }
    ];
    
    for (const { name, size } of sizes) {
        const data = crypto.randomBytes(size);
        
        // Write benchmark
        const writeStart = Date.now();
        await db.setFile(`files/test-${name}`, data);
        const writeTime = Date.now() - writeStart;
        
        // Read benchmark
        const readStart = Date.now();
        const retrieved = await db.getFile(`files/test-${name}`);
        const readTime = Date.now() - readStart;
        
        console.log(`${name}:`);
        console.log(`  Write: ${writeTime}ms`);
        console.log(`  Read: ${readTime}ms`);
    }
    
    // Test deduplication
    console.log('\n🔄 DEDUPLICATION TEST');
    const testData = crypto.randomBytes(100 * 1024);
    
    const start = Date.now();
    for (let i = 0; i < 10; i++) {
        await db.setFile(`dup/file${i}`, testData);
    }
    const dupTime = Date.now() - start;
    console.log(`10 identical 100KB files: ${dupTime}ms (${dupTime/10}ms per file)`);
}

async function benchmarkSearch() {
    const db = await WalDB.open('/tmp/bench-search');
    
    console.log('\n🔍 SEARCH OPERATIONS BENCHMARK\n');
    
    // Create test dataset
    console.log('Creating test dataset...');
    const roles = ['admin', 'user', 'moderator'];
    const departments = ['engineering', 'sales', 'marketing', 'support'];
    
    for (let i = 0; i < 1000; i++) {
        await db.set(`employees/${i}`, {
            name: `Employee ${i}`,
            age: 20 + Math.floor(Math.random() * 40),
            salary: 30000 + Math.floor(Math.random() * 70000),
            role: roles[Math.floor(Math.random() * roles.length)],
            department: departments[Math.floor(Math.random() * departments.length)],
            active: Math.random() > 0.1
        });
    }
    
    // Benchmark different search scenarios
    const searches = [
        {
            name: 'Simple equality',
            filters: [{ field: 'role', op: '==', value: 'admin' }]
        },
        {
            name: 'Numeric comparison',
            filters: [{ field: 'age', op: '>', value: '40' }]
        },
        {
            name: 'Multiple filters',
            filters: [
                { field: 'department', op: '==', value: 'engineering' },
                { field: 'salary', op: '>', value: '50000' }
            ]
        },
        {
            name: 'Complex query',
            filters: [
                { field: 'active', op: '==', value: 'true' },
                { field: 'age', op: '<', value: '35' },
                { field: 'salary', op: '>=', value: '60000' }
            ]
        }
    ];
    
    for (const { name, filters } of searches) {
        const start = Date.now();
        const results = await db.search({
            pattern: 'employees/*',
            filters,
            limit: 100
        });
        const time = Date.now() - start;
        
        console.log(`${name}:`);
        console.log(`  Time: ${time}ms`);
        console.log(`  Results: ${results.length} matches`);
    }
    
    // Benchmark with different result sizes
    console.log('\n📊 RESULT SIZE IMPACT');
    for (const limit of [10, 50, 100, 500]) {
        const start = Date.now();
        const results = await db.search({
            pattern: 'employees/*',
            filters: [{ field: 'active', op: '==', value: 'true' }],
            limit
        });
        const time = Date.now() - start;
        console.log(`Limit ${limit}: ${time}ms (${results.length} results)`);
    }
}

async function main() {
    console.log('🚀 WalDB NEW FEATURES BENCHMARK\n');
    console.log('================================\n');
    
    await benchmarkFiles();
    await benchmarkSearch();
    
    console.log('\n✅ Benchmark complete!');
}

main().catch(console.error);