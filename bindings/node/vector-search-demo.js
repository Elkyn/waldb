/**
 * WalDB Vector and Text Search Demo
 * Showcases the advanced search capabilities including:
 * - Vector similarity search with embeddings
 * - Full-text search with tokenization 
 * - Hybrid scoring combining multiple search types
 * - Efficient filtering and metadata support
 */

const WalDB = require('./index.js');
const fs = require('fs');

async function vectorSearchDemo() {
    console.log('🚀 WalDB Vector & Text Search Demo\n');
    
    // Create a sample e-commerce database
    const dbPath = '/tmp/waldb-ecommerce-demo';
    if (fs.existsSync(dbPath)) {
        fs.rmSync(dbPath, { recursive: true });
    }
    
    const db = await WalDB.open(dbPath);
    console.log('📊 Database opened at:', dbPath);
    
    // Sample product catalog with embeddings
    const products = [
        {
            id: 'electronics_001',
            name: 'Wireless Bluetooth Headphones',
            description: 'Premium noise-cancelling wireless headphones with 30-hour battery life',
            category: 'Electronics',
            brand: 'AudioTech',
            price: 299.99,
            rating: 4.5,
            inStock: true,
            tags: ['wireless', 'bluetooth', 'audio', 'premium'],
            // Embedding representing audio/wireless electronics
            embedding: [0.8, 0.2, 0.1, 0.9, 0.3, 0.7, 0.4, 0.6]
        },
        {
            id: 'electronics_002', 
            name: 'Smart Fitness Tracker',
            description: 'Advanced fitness tracker with heart rate monitoring and GPS',
            category: 'Electronics',
            brand: 'FitPro',
            price: 199.99,
            rating: 4.3,
            inStock: true,
            tags: ['fitness', 'tracker', 'health', 'smart'],
            // Embedding for fitness/health tech
            embedding: [0.3, 0.7, 0.8, 0.2, 0.9, 0.1, 0.6, 0.4]
        },
        {
            id: 'clothing_001',
            name: 'Premium Running Shoes',
            description: 'Lightweight running shoes with advanced cushioning technology',
            category: 'Clothing',
            brand: 'RunFast',
            price: 159.99,
            rating: 4.6,
            inStock: true,
            tags: ['running', 'shoes', 'athletic', 'lightweight'],
            // Embedding for athletic footwear
            embedding: [0.1, 0.9, 0.3, 0.5, 0.2, 0.8, 0.7, 0.4]
        },
        {
            id: 'electronics_003',
            name: 'Wireless Bluetooth Speaker',
            description: 'Portable waterproof speaker with powerful bass and wireless connectivity',
            category: 'Electronics', 
            brand: 'SoundWave',
            price: 89.99,
            rating: 4.4,
            inStock: false,
            tags: ['wireless', 'bluetooth', 'speaker', 'portable'],
            // Embedding similar to headphones (audio/wireless)
            embedding: [0.7, 0.3, 0.2, 0.8, 0.4, 0.6, 0.5, 0.7]
        },
        {
            id: 'home_001',
            name: 'Smart Coffee Maker',
            description: 'Programmable coffee maker with smartphone app control and timer',
            category: 'Home',
            brand: 'BrewMaster',
            price: 249.99,
            rating: 4.2,
            inStock: true,
            tags: ['coffee', 'smart', 'appliance', 'programmable'],
            // Embedding for smart home appliances
            embedding: [0.4, 0.1, 0.6, 0.3, 0.7, 0.5, 0.9, 0.2]
        }
    ];
    
    // Store products with their embeddings
    console.log('\n📦 Populating product catalog...');
    for (const product of products) {
        const { embedding, ...productData } = product;
        await db.set(`products/${product.id}`, productData);
        await db.setVector(`products/${product.id}/embedding`, embedding);
    }
    console.log(`✅ Stored ${products.length} products with embeddings`);
    
    // Demo 1: Vector Similarity Search
    console.log('\n🔍 Demo 1: Vector Similarity Search');
    console.log('Query: Find products similar to wireless audio devices');
    
    const audioQueryVector = [0.8, 0.25, 0.15, 0.85, 0.35, 0.65, 0.45, 0.65]; // Similar to headphones
    
    const vectorResults = await db.advancedSearchObjects({
        pattern: 'products/*',
        vector: {
            query: audioQueryVector,
            field: 'embedding',
            threshold: 0.5  // Only results with >50% similarity
        },
        limit: 3
    });
    
    console.log('Results ranked by vector similarity:');
    vectorResults.forEach((product, i) => {
        const score = product._searchMeta?.vectorScore || 0;
        console.log(`  ${i + 1}. ${product.name} (${(score * 100).toFixed(1)}% similar)`);
        console.log(`     Category: ${product.category}, Price: $${product.price}`);
    });
    
    // Demo 2: Text Search
    console.log('\n📝 Demo 2: Full-Text Search');
    console.log('Query: Search for "wireless bluetooth" in product names and descriptions');
    
    const textResults = await db.advancedSearchObjects({
        pattern: 'products/*',
        text: {
            query: 'wireless bluetooth',
            fields: ['name', 'description'],
            caseSensitive: false
        },
        limit: 5
    });
    
    console.log('Text search results:');
    textResults.forEach((product, i) => {
        const score = product._searchMeta?.textScore || 0;
        console.log(`  ${i + 1}. ${product.name} (text score: ${score.toFixed(2)})`);
        console.log(`     "${product.description.substring(0, 60)}..."`);
    });
    
    // Demo 3: Hybrid Search with Filters
    console.log('\n🎯 Demo 3: Hybrid Search (Vector + Text + Filters)');
    console.log('Query: Electronics under $200 with "wireless" + similar to audio devices');
    
    const hybridResults = await db.advancedSearchObjects({
        pattern: 'products/*',
        filters: [
            { field: 'category', op: '==', value: 'Electronics' },
            { field: 'price', op: '<', value: '200' },
            { field: 'inStock', op: '==', value: 'true' }
        ],
        vector: {
            query: audioQueryVector,
            field: 'embedding',
            threshold: 0.3
        },
        text: {
            query: 'wireless bluetooth',
            fields: ['name', 'description', 'tags'],
            caseSensitive: false
        },
        scoring: {
            vector: 1.5,  // Prioritize vector similarity
            text: 1.0,    // Standard text relevance  
            filter: 0.5   // Lower priority for filters
        },
        limit: 3
    });
    
    console.log('Hybrid search results:');
    hybridResults.forEach((product, i) => {
        const meta = product._searchMeta || {};
        const vScore = meta.vectorScore || 0;
        const tScore = meta.textScore || 0;
        const totalScore = meta.totalScore || 0;
        
        console.log(`  ${i + 1}. ${product.name}`);
        console.log(`     Price: $${product.price}, Brand: ${product.brand}`);
        console.log(`     Vector: ${(vScore * 100).toFixed(1)}%, Text: ${tScore.toFixed(1)}, Combined: ${totalScore.toFixed(2)}`);
        console.log(`     Tags: [${product.tags.join(', ')}]`);
    });
    
    // Demo 4: Advanced Filtering and Analytics
    console.log('\n📊 Demo 4: Advanced Analytics Queries');
    
    // Find high-rated products in each category
    const categories = ['Electronics', 'Clothing', 'Home'];
    
    for (const category of categories) {
        const categoryResults = await db.searchObjects({
            pattern: 'products/*',
            filters: [
                { field: 'category', op: '==', value: category },
                { field: 'rating', op: '>=', value: '4.0' }
            ],
            limit: 10
        });
        
        if (categoryResults.length > 0) {
            const avgPrice = categoryResults.reduce((sum, p) => sum + p.price, 0) / categoryResults.length;
            const avgRating = categoryResults.reduce((sum, p) => sum + p.rating, 0) / categoryResults.length;
            
            console.log(`${category}: ${categoryResults.length} high-rated products`);
            console.log(`  Average price: $${avgPrice.toFixed(2)}, Average rating: ${avgRating.toFixed(1)}`);
        }
    }
    
    // Demo 5: Real-time Recommendations
    console.log('\n🤝 Demo 5: Product Recommendations');
    console.log('Scenario: User is viewing Wireless Bluetooth Headphones');
    
    const currentProduct = products[0]; // Headphones
    const recommendationResults = await db.advancedSearchObjects({
        pattern: 'products/*',
        filters: [
            { field: 'id', op: '!=', value: currentProduct.id }  // Exclude current product
        ],
        vector: {
            query: currentProduct.embedding,
            field: 'embedding',
            threshold: 0.4
        },
        text: {
            query: currentProduct.tags.join(' '),
            fields: ['tags', 'description'],
            caseSensitive: false
        },
        scoring: {
            vector: 2.0,  // Strongly prefer similar products
            text: 0.5     // Light text matching boost
        },
        limit: 3
    });
    
    console.log(`Recommendations for "${currentProduct.name}":`);
    recommendationResults.forEach((product, i) => {
        const similarity = product._searchMeta?.vectorScore || 0;
        console.log(`  ${i + 1}. ${product.name} (${(similarity * 100).toFixed(1)}% similar)`);
        console.log(`     $${product.price} - ${product.description.substring(0, 50)}...`);
    });
    
    // Performance Summary
    console.log('\n⚡ Performance Summary');
    console.log('All searches completed in real-time with:');
    console.log('• Vector similarity calculations using cosine distance');
    console.log('• Full-text search with tokenization and fuzzy matching');
    console.log('• Multi-field filtering with numeric and string comparisons');
    console.log('• Hybrid scoring combining multiple relevance signals');
    console.log('• Results grouped and ranked by combined relevance scores');
    
    // Cleanup
    await db.flush();
    fs.rmSync(dbPath, { recursive: true });
    console.log('\n✅ Demo completed successfully!');
    console.log('\n🚀 WalDB now supports state-of-the-art vector and text search capabilities!');
}

// Run the demo
vectorSearchDemo().catch(console.error);