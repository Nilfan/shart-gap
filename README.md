# ShortGap

A Discord-like local messenger application built with Rust and React, featuring peer-to-peer communication with dynamic server selection.

## Features

- **Local P2P Messaging**: Create and join rooms without central servers
- **Multiple Protocol Support**: TCP, WebSocket, and WebRTC with hot-switching
- **Dynamic Server Selection**: Automatic role assignment based on ping measurements
- **Voice Communication**: WebRTC-based audio calls (architecture ready)
- **Invite System**: Shareable room codes for easy joining
- **Cross-Platform**: Desktop application built with Tauri

## Quick Start

### Prerequisites
- Rust (latest stable version)
- Node.js 18+
- npm or yarn

### Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd ShortGap
```

2. Install frontend dependencies:
```bash
cd frontend
npm install
cd ..
```

3. Run the development version with embedded frontend:
```bash
./dev.sh
# OR manually:
cd frontend && npm run build && cd .. && cargo run
```

### Building for Production

```bash
./build.sh
# OR manually:
cd frontend && npm run build && cd .. && cargo build --release
```

## Architecture

ShortGap uses a hybrid architecture combining:
- **Rust Backend**: High-performance networking and data management
- **React Frontend**: Modern web UI with CSS modules
- **Tauri Bridge**: Seamless communication between frontend and backend

### Key Components

- **Networking Layer**: Multi-protocol support with automatic switching
- **Room Management**: JSON-based persistence with real-time synchronization
- **Ping System**: Continuous measurement for optimal server selection
- **UI Components**: Modular React components for rooms, chat, and settings

## Usage

1. **Create a Room**: Click the "+" button in the sidebar
2. **Join a Room**: Click the "⚡" button and paste an invite code
3. **Chat**: Select a room and start messaging
4. **Voice Calls**: Click the microphone icon to join/leave calls
5. **Settings**: Configure your profile and audio devices

## Technical Details

### Protocol Switching
ShortGap supports three communication protocols:
- **TCP**: Reliable, traditional socket communication
- **WebSocket**: Web-compatible with lower overhead
- **WebRTC**: Peer-to-peer with NAT traversal for voice

### Server Selection
The application automatically selects the best-performing user as the server based on:
- TCP connection time
- Application-level ping
- WebRTC round-trip time

### Data Storage
Rooms are stored locally as JSON files in the system's app data directory:
- **Windows**: `%APPDATA%/shortgap/rooms/`
- **macOS**: `~/Library/Application Support/shortgap/rooms/`
- **Linux**: `~/.local/share/shortgap/rooms/`

## Development

### Project Structure
```
src/               # Rust backend
├── main.rs        # Application entry point
├── commands.rs    # Tauri command handlers
├── networking.rs  # Multi-protocol networking
├── room.rs        # Room management
├── user.rs        # User profiles
├── invite.rs      # Invite code system
├── ping.rs        # Ping measurement
└── protocol.rs    # Protocol switching

frontend/src/      # React frontend
├── components/    # React components
├── styles/        # CSS modules
└── App.tsx        # Main application
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## Security Notes

- Invite codes are base64-encoded but not encrypted
- Consider implementing authentication for production use
- Network traffic is not encrypted by default

## License

[Your License Here]

## Future Enhancements

- End-to-end encryption
- File sharing capabilities
- Mobile app versions
- Advanced voice features (noise cancellation, etc.)
- Plugin system for extensions# shart-gap
