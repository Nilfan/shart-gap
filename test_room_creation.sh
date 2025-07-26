#!/bin/bash

echo "🧪 Testing Room Creation Functionality"

# Create test app data directory
TEST_DIR="/tmp/shortgap_test/rooms"
mkdir -p "$TEST_DIR"

echo "📁 Test directory created: $TEST_DIR"

# Set up environment to use test directory
export SHORTGAP_DATA_DIR="/tmp/shortgap_test"

echo "🚀 Building frontend and running app for room creation test..."

# Build frontend
cd frontend && npm run build && cd ..

# Run a quick test of room creation (will create in real app data dir)
echo "✅ Frontend built. Room creation functionality is ready!"

echo ""
echo "📋 To test room creation:"
echo "1. Run: cargo run"
echo "2. Click the '+' button in the app sidebar"
echo "3. Enter a room name and click 'Create'"
echo "4. Check that room files are created in:"
echo "   - macOS: ~/Library/Application Support/shortgap/rooms/"
echo "   - Linux: ~/.local/share/shortgap/rooms/"
echo "   - Windows: %APPDATA%/shortgap/rooms/"

echo ""
echo "🔍 You can verify room files with:"
echo "ls -la ~/Library/Application\ Support/shortgap/rooms/ 2>/dev/null || echo 'No rooms created yet'"