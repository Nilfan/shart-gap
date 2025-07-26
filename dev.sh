#!/bin/bash

# Development script for ShortGap with embedded frontend

echo "🚀 Starting ShortGap development with embedded frontend..."

# Build the frontend first
echo "📦 Building React frontend..."
cd frontend
npm run build

if [ $? -ne 0 ]; then
    echo "❌ Frontend build failed!"
    exit 1
fi

cd ..

# Start Tauri in dev mode with the built frontend
echo "🦀 Starting Rust app with embedded webview..."
cargo tauri dev

echo "✅ Development server started with embedded frontend!"