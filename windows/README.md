# ShortGap - Windows Setup Guide

This guide covers Windows-specific setup instructions for building and running ShortGap.

## Prerequisites

- **Rust** (latest stable) - [Install from rustup.rs](https://rustup.rs/)
- **Node.js** (v18 or higher) - [Download from nodejs.org](https://nodejs.org/)
- **Visual Studio Build Tools** - Required for native compilation

### Install Visual Studio Build Tools

1. Download from [Microsoft Visual Studio](https://visualstudio.microsoft.com/downloads/)
2. Install "Desktop development with C++" workload
3. Restart computer after installation

## Quick Setup (Recommended)

1. **Clone and install dependencies:**
   ```cmd
   git clone <your-repo-url>
   cd ShortGap
   cargo install tauri-cli
   cd frontend
   npm install
   cd ..
   ```

2. **Create ICO file (REQUIRED):**
   - Go to [https://convertio.co/png-ico/](https://convertio.co/png-ico/)
   - Upload `icons/icon.png`
   - Download and save as `icons/icon.ico`
   - ✅ ICO file is now working!

3. **Run using PowerShell script:**
   ```cmd
   .\windows\build.ps1
   ```

## PowerShell Build Script

The `windows/build.ps1` script handles the complete Windows build process.

### Usage

```powershell
# Development mode (default)
.\windows\build.ps1

# Production build
.\windows\build.ps1 -Release

# Clean build directories
.\windows\build.ps1 -Clean

# Show help
.\windows\build.ps1 -Help
```

### Script Features

- ✅ **Prerequisites checking**: Rust, Node.js, Tauri CLI, ICO file
- ✅ **Automatic installation**: Frontend dependencies, Tauri CLI if missing
- ✅ **Multiple build modes**: Development and production
- ✅ **Clean functionality**: Remove build directories
- ✅ **Colored output**: Success/error/info messages with emojis
- ✅ **Error handling**: Proper error checking and exit codes

## Manual Setup (Alternative)

If you prefer manual setup or need to troubleshoot:

### 1. Install Tauri CLI
```cmd
cargo install tauri-cli
```

### 2. Install Frontend Dependencies
```cmd
cd frontend
npm install
cd ..
```

### 3. Create ICO File
**REQUIRED**: Windows needs a proper ICO format file, not just a renamed PNG.

```cmd
# Use online converter (recommended)
# 1. Go to https://convertio.co/png-ico/
# 2. Upload icons/icon.png
# 3. Download and save as icons/icon.ico

# Alternative: Use ImageMagick (if installed)
magick icons/icon.png icons/icon.ico
```

### 4. Build and Run

**Development:**
```cmd
cd frontend && npm run build && cd .. && cargo tauri dev
```

**Production:**
```cmd
cd frontend && npm run build && cd .. && cargo tauri build
```

## Troubleshooting

### Common Windows Issues

#### 1. "icon.ico not found" or "not in 3.00 format" error
```cmd
# ✅ SOLVED: Create proper ICO file using online converter
# 1. Go to https://convertio.co/png-ico/
# 2. Upload icons/icon.png
# 3. Download and save as icons/icon.ico
# 4. Run: .\windows\build.ps1
```

#### 2. "tauri command not found"
```cmd
cargo install tauri-cli
# Restart terminal after installation
```

#### 3. Tauri CLI version incompatibility
```cmd
# If using Tauri v2 CLI with v1 project:
cargo install tauri-cli --version "^1.0"
```

#### 4. Visual Studio Build Tools required
- Download from [Microsoft Visual Studio](https://visualstudio.microsoft.com/downloads/)
- Install "Desktop development with C++" workload
- Restart computer after installation

#### 5. Frontend build fails
```cmd
cd frontend
npm install
npm run build
cd ..
```

#### 6. PowerShell execution policy error
```cmd
# If you get execution policy error:
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Or run with bypass:
powershell -ExecutionPolicy Bypass -File .\windows\build.ps1
```

### Build Directory Locations

After successful build:

**Development:**
- App runs directly in development mode

**Production:**
- Executable: `target/release/bundle/msi/` (MSI installer)
- Portable: `target/release/` (if configured)

## Environment Variables

You can set these environment variables for customization:

```cmd
# Skip prerequisite checks
set SHORTGAP_SKIP_CHECKS=1

# Custom build target
set CARGO_BUILD_TARGET=x86_64-pc-windows-msvc
```

## Performance Tips

1. **Use SSD**: Place project on SSD for faster builds
2. **Antivirus exclusion**: Add project folder to antivirus exclusions
3. **WSL alternative**: Consider using WSL2 for Linux-like experience
4. **Incremental builds**: Use `cargo check` for faster syntax checking

## Getting Help

If you encounter issues:

1. Check this troubleshooting section
2. Run `.\windows\build.ps1 -Help`
3. Check the main project [README.md](../README.md)
4. Submit an issue with:
   - Windows version
   - PowerShell version (`$PSVersionTable`)
   - Rust version (`cargo --version`)
   - Node.js version (`node --version`)
   - Error messages and logs

## Advanced Configuration

### Custom Tauri Configuration

Edit `tauri.conf.json` for Windows-specific settings:

```json
{
  "tauri": {
    "bundle": {
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    }
  }
}
```

### Building MSI Installer

The build script automatically creates an MSI installer for production builds:

```cmd
.\windows\build.ps1 -Release
```

Find the installer at: `target/release/bundle/msi/ShortGap_0.1.0_x64_en-US.msi`

## Contributing

When contributing on Windows:

1. Use the PowerShell build script for consistency
2. Test both development and production builds
3. Ensure ICO file is properly formatted
4. Run `cargo fmt` and `cargo clippy` before submitting

---

**Note**: This Windows setup guide is tested on Windows 10/11 with PowerShell 5.1+. For older Windows versions, manual setup may be required.