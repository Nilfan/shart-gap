#!/bin/bash

# Build script for ShortGap with embedded frontend

echo "🏗️  Building ShortGap with embedded frontend..."

# Build the frontend first
echo "📦 Building React frontend..."
cd frontend
npm run build

if [ $? -ne 0 ]; then
    echo "❌ Frontend build failed!"
    exit 1
fi

cd ..

# Now build the Tauri app with embedded frontend
echo "🦀 Building Rust app with embedded webview..."
cargo tauri build

if [ $? -eq 0 ]; then
    echo "✅ Build complete! Frontend is now embedded in the Rust app."
    echo "📁 You can find the executable in src-tauri/target/release/"
else
    echo "❌ Rust build failed!"
    exit 1
fi