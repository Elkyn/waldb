# Publishing WalDB Node.js Bindings

This guide explains how to publish the WalDB Node.js bindings.

## Publishing Process

### 1. Create a Release

```bash
# Update version in package.json
cd bindings/node
npm version patch  # or minor/major

# Commit and push
git add package.json package-lock.json
git commit -m "chore: bump version to x.x.x"
git push origin main

# Create and push tag
git tag v0.1.1
git push origin v0.1.1
```

### 2. GitHub Release

The `build-binaries.yml` workflow will:
- Build native binaries for all platforms
- Create a GitHub release with the binaries

### 3. Publish to npm

Either:
- **Automatic**: Create a GitHub release to trigger `npm-publish.yml`
- **Manual**: Run the workflow from GitHub Actions tab

## Prerequisites

1. **NPM Token**: Add `NPM_TOKEN` to GitHub repository secrets
2. **Permissions**: Ensure you have publish access to the `waldb` npm package

## Local Development

### Build and Test

```bash
cd bindings/node
npm install
npm run build-release
npm test
```

### Test Publishing (Dry Run)

```bash
npm pack  # Creates waldb-x.x.x.tgz
npm publish --dry-run
```

## Platform Support

The package requires compilation on install. Users need:
- Node.js >= 18
- Rust toolchain (for building from source)
- Build tools (gcc/clang on Linux/macOS, MSVC on Windows)

## Troubleshooting

### Build Failures

If the build fails on a user's machine:

1. **Missing Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Windows**: Install Visual Studio Build Tools

3. **Linux**: Install build-essential:
   ```bash
   sudo apt-get install build-essential  # Ubuntu/Debian
   sudo yum groupinstall "Development Tools"  # RHEL/CentOS
   ```

### Publishing Issues

1. **Authentication**:
   ```bash
   npm login
   npm whoami
   ```

2. **Version Conflicts**:
   ```bash
   npm view waldb versions  # Check existing versions
   ```

## Version Management

Follow semantic versioning:
- **Patch** (0.1.x): Bug fixes
- **Minor** (0.x.0): New features, backward compatible
- **Major** (x.0.0): Breaking changes