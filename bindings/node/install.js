#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const BINARY_NAME = 'index.node';
const PACKAGE_VERSION = require('./package.json').version;

// Platform mapping
function getPlatformTarget() {
    const platform = process.platform;
    const arch = process.arch;
    
    const mapping = {
        'darwin-x64': 'x86_64-apple-darwin',
        'darwin-arm64': 'aarch64-apple-darwin',
        'linux-x64': 'x86_64-unknown-linux-gnu',
        'linux-arm64': 'aarch64-unknown-linux-gnu',
        'win32-x64': 'x86_64-pc-windows-msvc',
        'win32-arm64': 'aarch64-pc-windows-msvc'
    };
    
    const key = `${platform}-${arch}`;
    return mapping[key];
}

async function downloadAndExtract() {
    const target = getPlatformTarget();
    
    if (!target) {
        console.log(`No prebuilt binary for ${process.platform}-${process.arch}`);
        return buildFromSource();
    }
    
    const downloadUrl = `https://github.com/Elkyn/waldb/releases/download/v${PACKAGE_VERSION}/waldb-${target}.tar.gz`;
    const tmpFile = path.join(__dirname, 'download.tar.gz');
    
    console.log(`Downloading prebuilt binary for ${target}...`);
    
    // Download file
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(tmpFile);
        
        https.get(downloadUrl, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                // Follow redirect
                https.get(response.headers.location, handleResponse);
            } else {
                handleResponse(response);
            }
            
            function handleResponse(res) {
                if (res.statusCode !== 200) {
                    console.log(`Download failed (${res.statusCode}), building from source...`);
                    fs.unlinkSync(tmpFile);
                    return buildFromSource().then(resolve).catch(reject);
                }
                
                res.pipe(file);
                
                file.on('finish', () => {
                    file.close(() => {
                        // Extract using tar command
                        try {
                            execSync(`tar -xzf download.tar.gz ${BINARY_NAME}`, { 
                                cwd: __dirname,
                                stdio: 'ignore'
                            });
                            fs.unlinkSync(tmpFile);
                            console.log('✅ Binary downloaded and extracted successfully');
                            resolve();
                        } catch (err) {
                            console.error('Extraction failed:', err.message);
                            fs.unlinkSync(tmpFile);
                            console.log('Building from source...');
                            buildFromSource().then(resolve).catch(reject);
                        }
                    });
                });
            }
        }).on('error', (err) => {
            console.error('Download failed:', err.message);
            if (fs.existsSync(tmpFile)) fs.unlinkSync(tmpFile);
            console.log('Building from source...');
            buildFromSource().then(resolve).catch(reject);
        });
    });
}

async function buildFromSource() {
    console.log('Building from source (requires Rust)...');
    
    // Check if Rust is installed
    try {
        execSync('cargo --version', { stdio: 'ignore' });
    } catch (e) {
        console.error('\n❌ Rust is not installed. Please install Rust:');
        console.error('   curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh\n');
        console.error('Or check https://www.rust-lang.org/tools/install for more options.\n');
        process.exit(1);
    }
    
    // Check if cargo-cp-artifact is installed
    try {
        execSync('cargo-cp-artifact --version', { stdio: 'ignore' });
    } catch (e) {
        console.log('Installing cargo-cp-artifact...');
        try {
            execSync('npm install cargo-cp-artifact', { stdio: 'inherit' });
        } catch (err) {
            console.error('Failed to install cargo-cp-artifact');
            process.exit(1);
        }
    }
    
    // Build the native module
    try {
        console.log('Building native module...');
        execSync('npm run build-release', { stdio: 'inherit' });
        console.log('✅ Built from source successfully');
    } catch (e) {
        console.error('❌ Build failed:', e.message);
        console.error('\nPlease ensure you have a C++ compiler installed:');
        console.error('- Windows: Visual Studio Build Tools');
        console.error('- macOS: Xcode Command Line Tools');
        console.error('- Linux: build-essential (apt) or gcc (yum)');
        process.exit(1);
    }
}

// Main
(async () => {
    // Check if binary already exists
    if (fs.existsSync(path.join(__dirname, BINARY_NAME))) {
        console.log('Binary already installed');
        process.exit(0);
    }
    
    try {
        await downloadAndExtract();
    } catch (err) {
        console.error('Installation failed:', err);
        process.exit(1);
    }
})();