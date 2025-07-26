#!/bin/bash

echo "🧪 Testing Room Members & Messaging Functionality"

echo ""
echo "📋 Features to Test:"
echo "1. ✅ Room Creation - Click '+' button, enter name, create room"
echo "2. ✅ Members Sublist - See user list below room with indented styling"
echo "3. ✅ Message Sending - Type message, press Send, see in chat"
echo "4. ✅ Message Persistence - Messages saved to room JSON file"
echo "5. ✅ Message Sync - Select room to sync and sort messages by time"
echo "6. ✅ Auto-scroll - Chat scrolls to newest messages"

echo ""
echo "🚀 To test:"
echo "cd frontend && npm run build && cd .. && cargo run"

echo ""
echo "📁 Room files with messages will be saved to:"
echo "~/Library/Application Support/shortgap/rooms/{room-id}.json"

echo ""
echo "🔍 Verify messages in room file:"
echo 'cat ~/Library/Application\ Support/shortgap/rooms/*.json | jq .messages'

echo ""
echo "✨ Expected UI improvements:"
echo "- Room members appear below room name with:"
echo "  - Left border and indented layout"
echo "  - Online status dots (green/gray)"
echo "  - Background highlighting on hover"
echo "- Messages display with:"
echo "  - User name and timestamp"
echo "  - Proper chronological order"
echo "  - Auto-scroll to bottom"