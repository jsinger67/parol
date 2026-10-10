# GitHub Actions Binary Release Workflow

This guide explains the automated binary release workflow for `parol` and `parol-ls`.

## How It Works

The workflow is triggered automatically when you push a **git tag** matching the pattern:
- `v*` (e.g., `v5.0.4`)
- `parol-v*` (e.g., `parol-v5.0.4`)
- `parol-ls-v*` (e.g., `parol-ls-v5.0.4`)

## Supported Platforms

The workflow builds binaries for:

| Platform | Target | OS |
|----------|--------|-----|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | ubuntu-latest |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | ubuntu-latest (with cross) |
| macOS Intel | `x86_64-apple-darwin` | macos-13 |
| macOS Apple Silicon | `aarch64-apple-darwin` | macos-latest |
| Windows x86_64 | `x86_64-pc-windows-msvc` | windows-latest |

## Usage

### Creating a Release

1. **Update version numbers** in `Cargo.toml` files:
   ```toml
   # crates/parol/Cargo.toml
   [package]
   version = "5.0.4"
   
   # crates/parol-ls/Cargo.toml
   [package]
   version = "5.0.4"
   ```

2. **Commit your changes**:
   ```bash
   git add .
   git commit -m "Release v5.0.4"
   ```

3. **Create a git tag**:
   ```bash
   git tag v5.0.4
   git push origin main --tags
   ```

4. The workflow will automatically:
   - Build binaries for all platforms
   - Create checksums (SHA256)
   - Create a GitHub Release
   - Upload all binaries and checksums

### Monitoring the Build

1. Go to your repository on GitHub
2. Click the **Actions** tab
3. Find the **Release Binaries** workflow run
4. View build progress and status

## Release Artifacts

Each release includes:

- **Binaries** (platform-specific):
  - `parol` / `parol.exe` - Parser generator CLI
  - `parol-ls` / `parol-ls.exe` - Language server

- **Checksums**:
  - `checksums.txt` - SHA256 hashes for verification

- **Archives**:
  - Unix: `.tar.gz` files
  - Windows: `.zip` files

## Verifying Downloads

Users can verify binary integrity:

```bash
# Download the checksum file and binaries
sha256sum -c checksums.txt
```

## Pre-release Versions

To create a pre-release, use tags with `alpha`, `beta`, or `rc`:
```bash
git tag v5.0.4-alpha.1
git push origin main --tags
```

These will be marked as pre-releases on GitHub.

## Troubleshooting

### ARM64 Linux Builds Slow
ARM64 builds use `cross` (which emulates), so they're slower. This is expected.

### Build Fails for Specific Platform
1. Check the **Actions** tab for detailed logs
2. Common issues:
   - Missing dependencies (usually auto-installed)
   - Target not installed (handled automatically)
   - Compilation errors in platform-specific code

### Binaries Too Large
The workflow strips debug symbols on Unix. Windows binaries are usually larger.

## Customization

To modify the workflow, edit `.github/workflows/release.yml`:

- Add/remove platforms in the `strategy.matrix` section
- Change which crates are built
- Modify archive format or compression
- Add additional post-build steps

## Notes

- The workflow runs in parallel across all platforms for speed
- Binaries are optimized for size with `--release` builds
- Each platform has its own artifact upload
- GitHub Actions provides free build minutes for public repositories
