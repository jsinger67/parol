# Installing Pre-built Binaries

Pre-built binaries for `parol` and `parol-ls` are available on the [GitHub Releases](https://github.com/jsinger67/parol/releases) page.

## Quick Start

### Linux and macOS

```bash
# Download the latest release for your platform
# Replace <VERSION> with the release version (e.g., v5.0.4)
# Replace <TARGET> with your platform:
#   - x86_64-unknown-linux-gnu (Linux Intel)
#   - aarch64-unknown-linux-gnu (Linux ARM64)
#   - x86_64-apple-darwin (macOS Intel)
#   - aarch64-apple-darwin (macOS Apple Silicon)

wget https://github.com/jsinger67/parol/releases/download/<VERSION>/parol-<TARGET>.tar.gz
tar xzf parol-<TARGET>.tar.gz

# Move binaries to a directory in your PATH
sudo mv parol parol-ls /usr/local/bin/

# Verify installation
parol --version
parol-ls --version
```

### Windows

1. Go to [GitHub Releases](https://github.com/jsinger67/parol/releases)
2. Download the `.zip` file for your platform
3. Extract the files
4. Add the extraction directory to your `PATH` or move binaries to a directory already in `PATH`

Or use PowerShell:

```powershell
# Example: Download and extract parol for Windows
$version = "v5.0.4"  # Replace with desired version
$target = "x86_64-pc-windows-msvc"

Invoke-WebRequest -Uri "https://github.com/jsinger67/parol/releases/download/$version/parol-$target.zip" `
  -OutFile "parol.zip"
Expand-Archive -Path "parol.zip" -DestinationPath "."

# Verify installation
./parol.exe --version
./parol-ls.exe --version
```

## Verifying Checksums

Each release includes a `checksums.txt` file with SHA256 hashes:

```bash
# Download checksums.txt from the release
# Then verify:
sha256sum -c checksums.txt

# On macOS:
shasum -a 256 -c checksums.txt
```

## Platform Support

| Platform | Binary | Target |
|----------|--------|--------|
| Linux x86_64 | parol, parol-ls | x86_64-unknown-linux-gnu |
| Linux ARM64 | parol, parol-ls | aarch64-unknown-linux-gnu |
| macOS Intel | parol, parol-ls | x86_64-apple-darwin |
| macOS Apple Silicon | parol, parol-ls | aarch64-apple-darwin |
| Windows x86_64 | parol.exe, parol-ls.exe | x86_64-pc-windows-msvc |

## Alternative: Build from Source

If pre-built binaries aren't available for your platform, you can build from source:

```bash
cargo install --force parol
cargo install --force parol-ls
```

## Troubleshooting

### Binary not found after installation
- Ensure the binary directory is in your `PATH` environment variable
- Restart your terminal or reload your shell profile

### Permission denied (Linux/macOS)
```bash
chmod +x parol parol-ls
```

### "parol-ls is not a valid Win32 application"
- Ensure you downloaded the correct Windows binary
- Download `x86_64-pc-windows-msvc` for 64-bit Windows

### Version conflicts
If you have `parol` installed via `cargo install`, you may need to uninstall the older version:
```bash
cargo uninstall parol
cargo uninstall parol-ls
```
