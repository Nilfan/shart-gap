# ShortGap

## Overview

ShortGap is a local based copy of Dicsord. It has some killer features that Discord doesn't and it makes it more accessable in the countries where Discord usage is limited.

## Common architeture

ShortGap is written with Rust and web view ReactJS. It helps app stays fast and memory secure. It's a desktop app with a window where you can see all your rooms in the left part of window as a list and text a messages in the content area (middle of the window). 

List of rooms consist of rows with a name of room with two icons - clickable join a call and a circle (empty if chat is empty - not users connected - and green if some users are in the chat -> some user is a server). Under chat row connected users sublist is appeared.

All UI is implemeted with React using Rust web view. React send a messages to Rust to make an action and it occures changes.

In the end of list there's a row "Settings" with an icon "Settings" which opens window "Settings" where user can choose input and output for the sound, edit his hame and avatar. This window has a button Cancel which is just closes this window without changes and button Save which sends a message from web view part of app to Rust core to save changes - change sound i/o, change name and send changes to other users and so on. Add voice check in settings. It should consists of the bar where indicates a loudness of voice right now, a button "Record" which records a voice (it becomes Stop button on record and stops voice recording) and Play button which is playes recorded voice speech.

In the middle of page is a chat area where users can chat with each other, messages list goes from bottom to top from latest to earliest. Every row has a name of the user, time of message sending in local time of reader with date, hours and minutes.


### Features

- ShortGap is a local-based messenger so it allows user to create room and invite people to room via link.
- When new user joins to the chat server gets it address and share full users address list with a chat members. 
- It launches a server on the fastest user in the room according 5 minutes analyzation of user ping. If some user has lower ping then server role moves to a new one. Ping scores recalculates every 5 minutes and server share this score with a users.
- If server leaves a chat (disconnection) then second fastest user become a server according ping score shared by previous server
- ShortGap is an app for calling generally so it allows to join a call and talk with sound input and output.


# LLM notes
- Project scope: From scratch implementation
- Core priorities: party creation/management, user invites, server selection, sound I/O, web UI
- Architecture: Local server approach with ZooKeeper-like leader election
- UI Model: **CHANGED from multi-room Discord-like to simple party-focused interface**
- **Session-based Model**: **NEW - Single party sessions only, no persistence**
- Invite system: Encoded strings containing party info + user address
- Framework: Rust backend + React frontend via Tauri webview
- User Experience: Single party focus instead of multiple rooms
- Technical decisions made:
  - Multiple protocol support: TCP, WebSocket, WebRTC with hot-switching
  - Ping measurement: Average of TCP connect time, application-level ping, WebRTC RTT  
  - **Data persistence**: **CHANGED to session-based only - no file persistence**
  - Voice: WebRTC implementation with mute/unmute controls
  - Invite system: Party ID + multiple peer addresses with ordered fallback
  - Framework: Tauri for Rust-React integration
  - Name collision: Automatic suffix handling (*name*-*connection_order*)
  - Persistent storage: User name saved to localStorage (first-time only setup)
  - **Server Selection & Failover**: Robust handling of party maker disconnection
    - Online status tracking with `is_online` flag and `last_seen` timestamps
    - Ping freshness validation (5-minute threshold for recent pings)
    - Automatic server re-election when current server goes offline
    - Fallback to any online user if no ping data available
    - Periodic health checks and cleanup of stale users/connections
    - Smart election prioritizing users with best ping among online users only
- **Memory Management Changes**: 
  - Always add changes notes to claude.md LLM section with dates and time (Added on 2024-03-20 at 15:45 UTC)
  - **Session-Based Architecture Update** (Added on 2024-07-26 at 10:30 UTC)
    - Removed multi-room support in favor of single party sessions
    - Eliminated file persistence for party data
    - Simplified backend state management with current_party instead of rooms vector
    - Updated all Tauri commands to work with single party model
    - Modified frontend to use new party-focused API endpoints
    - Party data now exists only in memory during active session

## Change Log

### 2025-01-26 Windows Setup Improvements
- **Fixed Windows ICO icon issue**: Resolved "icon.ico not found" and "not in 3.00 format" errors by creating proper ICO file from PNG using online converter
- **Created Windows-specific build infrastructure**:
  - Created `windows/` folder for Windows-specific files
  - Moved PowerShell build script to `windows/build.ps1`
  - Created comprehensive `windows/README.md` with Windows setup guide
- **Enhanced PowerShell build script (`windows/build.ps1`)**:
  - Prerequisites checking (Rust, Node.js, Tauri CLI, ICO file)
  - Automatic dependency installation
  - Multiple build modes: Development, Production, Clean
  - Colored output with emojis and error handling
  - Help system with usage instructions
- **Reorganized documentation structure**:
  - Removed Windows-specific details from root README.md
  - Added clear links to Windows setup guide throughout root README
  - Streamlined root README for cross-platform clarity
  - Updated build commands to reference `.\windows\build.ps1`
- **Working Windows development environment**: Successfully resolved all Windows compilation issues and created automated build process
- **Fixed PowerShell script path handling**: Updated `windows/build.ps1` to automatically navigate to project root directory when called from anywhere, ensuring correct relative paths for all operations

## Implementation Status

### ✅ Completed Features
- **Core Architecture**: Tauri-based desktop app with Rust backend and React frontend
- **Session-Based Party Management**: **UPDATED** - Single party focus with session-only persistence
  - Only one active party per application instance
  - No file persistence - party exists only during session
  - Party automatically cleared when leaving or closing app
- **User Management**: 
  - Persistent user names (localStorage, first-time setup only)
  - User profiles with name, avatar, audio device settings
  - Name collision handling with automatic suffix (*name*-*connection_order*)
  - Online/offline status tracking with automatic cleanup
- **Multi-Protocol Support**: TCP, WebSocket, WebRTC protocols with hot-switching capability
- **Dynamic Server Selection**: 
  - Ping-based automatic server role assignment
  - **Robust failover when party maker goes offline**
  - Health checks and automatic re-election of offline servers
  - Smart selection prioritizing online users with fresh ping data
- **Invite System**: Base64-encoded invite codes with party ID and peer addresses
- **Voice Call System**: 
  - **Join/Leave Call**: Functional call state management with user tracking
  - **Call Server Assignment**: Dynamic call server role assignment
  - **Call Status Tracking**: Room-level call activity monitoring
  - **System Messages**: Automatic notifications for call join/leave events
  - **UI Integration**: Call buttons in chat header with proper state management
- **Party-Focused UI**: 
  - **Name Input Screen**: First-time user name setup with persistent storage
  - **Welcome Screen**: Two large buttons - "Start a party" and "Join a party"
  - **Party Members Sidebar**: Left sidebar showing all party members with "(you)" indicator
  - **ChatArea**: Full-height chat interface with message display and input
  - **Settings Modal**: Audio device selection with voice check functionality
  - **Top Header**: User name display with party controls (copy invite, leave party)
  - **Control Buttons**: Mute/unmute and Settings buttons at bottom of party members list
  - **Call Controls**: Join/Leave call buttons in chat area header
- **UI Features**:
  - Material Icons integration for mute/unmute controls
  - Copy invite link button (replaces raw invite code display)
  - Minimum window size: 1020x600 pixels
  - Responsive layout with proper flex containers
  - Prettier code formatting configuration
- **Messaging System**:
  - Real-time message display with timestamps
  - System message support for call events
  - Message persistence during session only
  - Auto-scroll to latest messages
- **Peer Discovery**: Fallback connection system with ordered peer lists

### 🚧 Pending Implementation
- **WebRTC Voice Communication**: Actual audio streaming implementation (infrastructure ready)
- **Real Network Protocol Implementation**: TCP/WebSocket/WebRTC connection handling
- **Real-time Message Synchronization**: Network-based message broadcasting between peers
- **Voice Quality Controls**: Echo cancellation, noise suppression, volume controls

### 🏗️ Project Structure
```
/ShortGap
├── src/                    # Rust backend
│   ├── main.rs            # App entry point and UPDATED state management (single party)
│   ├── commands.rs        # UPDATED Tauri command handlers for party-focused API
│   ├── networking.rs      # Multi-protocol networking manager
│   ├── room.rs           # Room data structures (reused for party)
│   ├── user.rs           # User management
│   ├── invite.rs         # Invite code generation/parsing
│   ├── ping.rs           # Ping measurement and server selection
│   └── protocol.rs       # Protocol switching coordination
├── frontend/              # React frontend
│   ├── src/
│   │   ├── components/    # React components
│   │   │   ├── PartyMembers/ # Party members sidebar with controls
│   │   │   ├── ChatArea/  # Main chat interface
│   │   │   ├── Settings/  # User settings modal
│   │   │   └── VoiceCheck/ # Voice testing component
│   │   ├── styles/        # CSS modules
│   │   └── App.tsx        # UPDATED Main app component (party-focused)
│   └── package.json       # Frontend dependencies
├── Cargo.toml            # Rust dependencies
└── tauri.conf.json       # Tauri configuration

## Development Commands

### Prerequisites
- Rust (latest stable)
- Node.js (v18+)
- npm or yarn

### Setup
```bash
# Install Tauri CLI (one-time setup)
cargo install tauri-cli

# Install frontend dependencies
cd frontend
npm install

# Build frontend and run with embedded webview
cd ..
./dev.sh
# OR manually:
# cd frontend && npm run build && cd .. && cargo tauri dev
```

### Build
```bash
cargo tauri build
```

### Test Rust Code
```bash
cargo test
cargo check
```

## Key Implementation Details

### Architecture Decisions
1. **Tauri Framework**: Chosen for native performance with web UI flexibility
2. **Multi-Protocol Design**: Supports TCP, WebSocket, and WebRTC with runtime switching
3. **Peer-to-Peer Model**: Decentralized with dynamic server role assignment
4. **Session-Based Storage**: **UPDATED** - Simple memory-based party storage (no file persistence)
5. **CSS Modules**: Component-scoped styling for maintainable UI

### Security Considerations
- Invite codes are base64-encoded but not encrypted (consider adding encryption)
- Network protocols should implement authentication in production

### Performance Features
- Ping-based server selection for optimal performance
- Hot protocol switching without connection loss
- Efficient React components with CSS modules
- Tokio async runtime for concurrent operations

## Party Creation & Messaging Flow

### User Onboarding
#### First Time Launch
1. User sees name input screen
2. User enters name and clicks "Continue"
3. Name saved to localStorage for future sessions
4. User proceeds to main party interface

#### Subsequent Launches
1. App automatically loads saved name from localStorage
2. User goes directly to main party interface

### Party Creation
#### User Interface
1. User clicks "Start a party" button on welcome screen
2. Party created instantly with auto-generated invite code
3. Party members sidebar appears with creator listed
4. Chat area becomes available immediately

#### Backend Process
1. **User Initialization**: Uses saved name from localStorage
2. **Party Creation**: `create_party()` command called via Tauri bridge
3. **Party Object**: New `Room` struct created with:
   - Unique UUID
   - Default name "Party"
   - Creator as first user (and initial server)
   - Empty messages list
   - TCP protocol (default)
4. **Invite Generation**: Auto-generates invite code
5. **Session Storage**: **UPDATED** - Party stored only in memory (no file persistence)
6. **State Update**: Party set as current_party in app state
7. **UI Update**: Full party interface rendered

### Party Members Display
- **Left Sidebar**: Dedicated party members list when in party
- **Visual Design**: Clean list with avatars and status indicators
- **Current User**: Marked with "(you)" indicator
- **Online Status**: Green/gray dots indicate user online status
- **Member Count**: Badge showing total party members
- **Control Buttons**: Mute/unmute and Settings at bottom of sidebar

### Messaging System
#### Sending Messages
1. User types message in chat input
2. `send_message()` command sends to backend (no roomId parameter needed)
3. Message added to current party's message list
4. UI refreshes to show new message immediately

#### Message Synchronization
1. **Party Context**: Messages automatically sync within current party
2. **Message Ordering**: Backend sorts all messages by timestamp
3. **Session Persistence**: **UPDATED** - Messages persist only during active session
4. **UI Update**: Chat area displays message history
5. **Real-time Updates**: Party members see messages immediately

#### Message Structure
- **ID**: Unique message identifier
- **User Info**: Sender name and ID
- **Content**: Message text
- **Timestamp**: Local time with date/time display
- **Auto-scroll**: Chat scrolls to newest messages

### Current UI Layout
- **Top Header**: User name, Copy invite link, Leave party buttons
- **Left Sidebar**: Party members list with mute/settings controls at bottom
- **Chat Area Header**: Room name, member count, protocol info, and Join/Leave call button
- **Main Chat**: Full-height message interface with real-time updates
- **Message Input**: Send messages with auto-scroll to latest
- **Welcome Screen**: Two large buttons when not in party
- **Name Input**: First-time setup screen with persistent storage

### Recent Updates (July 2025)
- ✅ **Call System Integration**: Complete join/leave call functionality with UI
- ✅ **Message System**: Full message persistence and display with system notifications
- ✅ **Code Quality**: Added Prettier formatting configuration
- ✅ **UI Polish**: Enhanced chat area with proper call controls and status display
- ✅ **State Management**: Improved party state synchronization and error handling
- ✅ **Session-Based Architecture**: **NEW** - Removed multi-room logic for single party sessions
  - Simplified backend from rooms vector to single current_party
  - Updated all API endpoints to work without roomId parameters
  - Removed file persistence in favor of session-only storage
  - Enhanced user experience with cleaner party-focused workflow