#!/bin/bash

# Test prebuild installation locally
set -e

echo "Testing WalDB prebuild installation..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create temporary test directory
TEST_DIR=$(mktemp -d)
echo -e "${YELLOW}Testing in: $TEST_DIR${NC}"

cd "$TEST_DIR"

# Initialize test package
npm init -y > /dev/null 2>&1

# Get current directory of waldb
WALDB_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Pack the current waldb package
echo -e "${YELLOW}Packing waldb from $WALDB_DIR...${NC}"
cd "$WALDB_DIR"
npm pack --silent

# Move tarball to test directory
TARBALL=$(ls -t waldb-*.tgz | head -1)
mv "$TARBALL" "$TEST_DIR/"

cd "$TEST_DIR"

# Install from local tarball
echo -e "${YELLOW}Installing waldb...${NC}"
npm install "./$TARBALL"

# Create test script
cat > test.js << 'EOF'
const waldb = require('waldb');

async function test() {
    console.log('Creating WalDB instance...');
    const db = new waldb.Store('/tmp/waldb_prebuild_test');
    
    console.log('Testing basic operations...');
    await db.set('test/key', 'value');
    const value = await db.get('test/key');
    
    if (value === 'value') {
        console.log('✅ Basic operations work!');
    } else {
        console.error('❌ Basic operations failed!');
        process.exit(1);
    }
    
    console.log('Testing vector operations...');
    await db.setVector('vec1', [1.0, 2.0, 3.0]);
    const vec = await db.getVector('vec1');
    
    if (vec && vec.length === 3) {
        console.log('✅ Vector operations work!');
    } else {
        console.error('❌ Vector operations failed!');
        process.exit(1);
    }
    
    console.log('Cleaning up...');
    await db.close();
    
    console.log('\n✅ All tests passed!');
}

test().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});
EOF

# Run test
echo -e "${YELLOW}Running tests...${NC}"
node test.js

# Check if binary was downloaded or built
if [ -f "node_modules/waldb/prebuilds" ]; then
    echo -e "${GREEN}✅ Prebuild was downloaded${NC}"
else
    echo -e "${YELLOW}⚠️  Built from source (no prebuild available)${NC}"
fi

# Show binary info
echo -e "\n${YELLOW}Binary information:${NC}"
file node_modules/waldb/index.node

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
cd /
rm -rf "$TEST_DIR"

echo -e "${GREEN}✅ Prebuild test completed successfully!${NC}"