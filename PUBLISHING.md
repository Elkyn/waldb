# Publishing WalDB Node.js Bindings

This guide explains how to publish the WalDB Node.js bindings with multi-architecture support.

## Architecture Support

The package provides prebuilt binaries for:
- **Linux**: x64, ARM64
- **macOS**: x64, ARM64 (Apple Silicon)
- **Windows**: x64, ARM64

## Prerequisites

1. **NPM Account**: You need publish access to the `waldb` package on npm
2. **GitHub Repository**: Push access to create tags and releases
3. **Secrets Configuration**:
   - `NPM_TOKEN`: Your npm automation token (set in GitHub repository secrets)

## Publishing Process

### Automated Release (Recommended)

1. **Create a Git Tag**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **GitHub Actions Workflow**:
   - The `release.yml` workflow automatically triggers
   - Builds binaries for all supported platforms
   - Creates a GitHub release with prebuilt binaries
   - Publishes the package to npm

3. **Monitor Progress**:
   - Check GitHub Actions tab for build status
   - Verify release at: https://github.com/elkyn/waldb/releases
   - Confirm npm publication: https://www.npmjs.com/package/waldb

### Manual Release

1. **Update Version**:
   ```bash
   cd bindings/node
   npm version patch  # or minor/major
   ```

2. **Build Locally**:
   ```bash
   npm run build-release
   npm test
   ```

3. **Create Prebuilds** (optional for local testing):
   ```bash
   npm run prebuild
   ```

4. **Publish to npm**:
   ```bash
   npm publish --access public
   ```

5. **Create Git Tag**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

## Local Development

### Building for Your Platform

```bash
cd bindings/node
npm install
npm run build-release
npm test
```

### Cross-Compilation Setup

#### Linux ARM64 on x64
```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Build
cross build --release --target aarch64-unknown-linux-gnu
```

#### macOS Universal Binary
```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary (optional)
lipo -create \
  target/x86_64-apple-darwin/release/libwaldb_node.dylib \
  target/aarch64-apple-darwin/release/libwaldb_node.dylib \
  -output index.node
```

## Package Structure

```
waldb/
├── index.js           # JavaScript wrapper
├── index.d.ts         # TypeScript definitions
├── index.node         # Native binary (built or downloaded)
├── package.json       # Package metadata
├── .prebuildrc        # Prebuild configuration
└── prebuilds/         # Prebuilt binaries (not in npm package)
    └── waldb/
        └── v1.0.0/
            ├── node-napi-v6-darwin-arm64.tar.gz
            ├── node-napi-v6-darwin-x64.tar.gz
            ├── node-napi-v6-linux-arm64.tar.gz
            ├── node-napi-v6-linux-x64.tar.gz
            ├── node-napi-v6-win32-arm64.tar.gz
            └── node-napi-v6-win32-x64.tar.gz
```

## Installation Behavior

When users install the package:

1. **With Prebuilt Binary Available**:
   ```bash
   npm install waldb
   # Automatically downloads matching prebuild from GitHub releases
   ```

2. **Without Prebuilt Binary**:
   ```bash
   npm install waldb
   # Falls back to building from source
   # Requires Rust toolchain installed
   ```

## Troubleshooting

### Build Failures

1. **Missing Rust Toolchain**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Cross-compilation Issues**:
   - Ensure target is installed: `rustup target add aarch64-unknown-linux-gnu`
   - Use `cross` tool for Linux ARM builds

3. **Windows Build Issues**:
   - Install Visual Studio Build Tools
   - Ensure MSVC toolchain is available

### Publishing Issues

1. **npm Authentication**:
   ```bash
   npm login
   npm whoami  # Verify logged in
   ```

2. **Version Conflicts**:
   - Ensure version in package.json is not already published
   - Use `npm view waldb versions` to check existing versions

3. **GitHub Release Assets**:
   - Verify GITHUB_TOKEN has appropriate permissions
   - Check release assets are uploaded correctly

## Version Management

Follow semantic versioning:
- **Patch** (1.0.x): Bug fixes, performance improvements
- **Minor** (1.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes

## CI/CD Workflows

### `release.yml`
Main release workflow that:
- Builds binaries for all platforms
- Creates GitHub release
- Uploads prebuilt binaries
- Publishes to npm

### `prebuild.yml`
Dedicated prebuild workflow for:
- Building binaries without publishing
- Testing cross-compilation
- Manual prebuild generation

### `npm-publish.yml`
Standalone npm publishing for:
- Publishing without new binaries
- Re-publishing failed releases
- Testing npm publication

## Security Considerations

1. **Token Security**:
   - Never commit tokens to repository
   - Use GitHub Secrets for CI/CD
   - Rotate tokens regularly

2. **Binary Integrity**:
   - Binaries are built in CI environment
   - Downloaded over HTTPS
   - Consider signing binaries (future enhancement)

3. **Supply Chain**:
   - Pin dependency versions
   - Review dependency updates
   - Use lockfiles (package-lock.json)

## Testing Prebuilds

Test prebuild installation:
```bash
# Create test directory
mkdir test-install
cd test-install
npm init -y

# Install from npm (will download prebuild)
npm install waldb

# Test
node -e "const waldb = require('waldb'); console.log(waldb)"
```

## Monitoring

After release:
1. Check npm download stats: https://www.npmjs.com/package/waldb
2. Monitor GitHub issues for platform-specific problems
3. Verify all platform binaries are available in GitHub release

## Support Matrix

| Platform | Architecture | Node Version | Status |
|----------|-------------|--------------|--------|
| Linux    | x64         | >=18         | ✅     |
| Linux    | ARM64       | >=18         | ✅     |
| macOS    | x64         | >=18         | ✅     |
| macOS    | ARM64       | >=18         | ✅     |
| Windows  | x64         | >=18         | ✅     |
| Windows  | ARM64       | >=18         | ✅     |

## Future Enhancements

- [ ] Binary signing for enhanced security
- [ ] Alpine Linux support (musl libc)
- [ ] FreeBSD support
- [ ] Electron prebuilds
- [ ] WASM build for browser support