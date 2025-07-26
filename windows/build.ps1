# ShortGap Build Script for Windows
# Usage: .\build.ps1 [-Release] [-Clean] [-Help]

param(
    [switch]$Release,
    [switch]$Clean,
    [switch]$Help
)

# Display help information
if ($Help) {
    Write-Host "ShortGap Build Script for Windows" -ForegroundColor Green
    Write-Host ""
    Write-Host "USAGE:"
    Write-Host "  .\build.ps1                 # Development build"
    Write-Host "  .\build.ps1 -Release        # Production build"
    Write-Host "  .\build.ps1 -Clean          # Clean build directories"
    Write-Host "  .\build.ps1 -Help           # Show this help"
    Write-Host ""
    Write-Host "REQUIREMENTS:"
    Write-Host "  - Rust (latest stable)"
    Write-Host "  - Node.js 18+"
    Write-Host "  - Tauri CLI: cargo install tauri-cli"
    Write-Host "  - Proper ICO file at icons/icon.ico"
    Write-Host ""
    exit 0
}

# Colors for output
$ErrorColor = "Red"
$SuccessColor = "Green" 
$InfoColor = "Cyan"
$WarningColor = "Yellow"

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor $InfoColor
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor $SuccessColor
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor $ErrorColor
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor $WarningColor
}

function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    $allGood = $true
    
    # Check Rust
    if (Test-Command "cargo") {
        $rustVersion = cargo --version
        Write-Success "Rust found: $rustVersion"
    } else {
        Write-Error "Rust not found. Install from: https://rustup.rs/"
        $allGood = $false
    }
    
    # Check Node.js
    if (Test-Command "npm") {
        $nodeVersion = node --version
        $npmVersion = npm --version
        Write-Success "Node.js found: $nodeVersion, npm: $npmVersion"
    } else {
        Write-Error "Node.js/npm not found. Install from: https://nodejs.org/"
        $allGood = $false
    }
    
    # Check Tauri CLI
    if (Test-Command "cargo-tauri") {
        $tauriVersion = cargo tauri --version
        Write-Success "Tauri CLI found: $tauriVersion"
    } else {
        Write-Warning "Tauri CLI not found. Installing..."
        try {
            cargo install tauri-cli
            Write-Success "Tauri CLI installed successfully"
        } catch {
            Write-Error "Failed to install Tauri CLI"
            $allGood = $false
        }
    }
    
    # Check ICO file
    if (Test-Path "icons/icon.ico") {
        Write-Success "ICO file found: icons/icon.ico"
    } else {
        Write-Error "ICO file missing. Create icons/icon.ico from icons/icon.png"
        Write-Info "Use https://convertio.co/png-ico/ to convert PNG to ICO"
        $allGood = $false
    }
    
    # Check frontend dependencies
    if (Test-Path "frontend/node_modules") {
        Write-Success "Frontend dependencies found"
    } else {
        Write-Warning "Frontend dependencies missing. Will install..."
    }
    
    return $allGood
}

function Install-FrontendDependencies {
    Write-Info "Installing frontend dependencies..."
    
    if (-not (Test-Path "frontend")) {
        Write-Error "Frontend directory not found"
        exit 1
    }
    
    Set-Location "frontend"
    try {
        npm install
        Write-Success "Frontend dependencies installed"
    } catch {
        Write-Error "Failed to install frontend dependencies"
        Set-Location ".."
        exit 1
    }
    Set-Location ".."
}

function Build-Frontend {
    Write-Info "Building React frontend..."
    
    Set-Location "frontend"
    try {
        npm run build
        Write-Success "Frontend built successfully"
    } catch {
        Write-Error "Frontend build failed"
        Set-Location ".."
        exit 1
    }
    Set-Location ".."
}

function Clean-BuildDirectories {
    Write-Info "Cleaning build directories..."
    
    $targets = @("target", "frontend/dist", "frontend/node_modules")
    foreach ($target in $targets) {
        if (Test-Path $target) {
            Write-Info "Removing $target..."
            Remove-Item $target -Recurse -Force
            Write-Success "Removed $target"
        }
    }
}

function Build-App {
    param([bool]$IsRelease)
    
    if ($IsRelease) {
        Write-Info "Building ShortGap for production..."
        try {
            cargo tauri build
            Write-Success "Production build completed!"
            Write-Info "Executable located in: target/release/bundle/"
        } catch {
            Write-Error "Production build failed"
            exit 1
        }
    } else {
        Write-Info "Starting ShortGap in development mode..."
        try {
            cargo tauri dev
        } catch {
            Write-Error "Development build failed"
            exit 1
        }
    }
}

# Main script execution
Write-Host ""
Write-Host "🦀 ShortGap Build Script" -ForegroundColor Green
Write-Host "========================" -ForegroundColor Green
Write-Host ""

# Handle clean flag
if ($Clean) {
    Clean-BuildDirectories
    Write-Success "Clean completed!"
    exit 0
}

# Check prerequisites
if (-not (Test-Prerequisites)) {
    Write-Error "Prerequisites check failed. Please fix the issues above."
    exit 1
}

Write-Host ""

# Install frontend dependencies if needed
if (-not (Test-Path "frontend/node_modules")) {
    Install-FrontendDependencies
}

# Build frontend
Build-Frontend

Write-Host ""

# Build the app
Build-App -IsRelease $Release

Write-Host ""
if ($Release) {
    Write-Success "🎉 ShortGap production build completed!"
    Write-Info "Check target/release/bundle/ for the executable"
} else {
    Write-Success "🎉 ShortGap development server started!"
}