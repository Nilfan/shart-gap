#!/bin/bash

# Simple run script for ShortGap

echo "🚀 Starting ShortGap..."

# Build the frontend first
echo "📦 Building React frontend..."
cd frontend
npm run build

if [ $? -ne 0 ]; then
    echo "❌ Frontend build failed!"
    exit 1
fi

cd ..

# Run the app directly with cargo
echo "🦀 Starting Rust app..."
cargo run

echo "✅ App started!"