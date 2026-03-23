# Business Requirements Document (BRD)
## Stark-Link

---

## 1. Executive Summary

**Project Name:** Stark-Link

**Project Type:** Cross-device control & communication platform

**Core Feature Summary:** A secure, low-latency application enabling seamless cross-device control, screen sharing, file transfers, clipboard synchronization, and remote management over local networks — installable on any device without requiring technical setup.

**Target Users:**
- Individuals who want unified control across all their devices (phone, laptop, desktop, tablet)
- Teams needing instant file sharing and collaboration within local networks
- Power users who want their devices to feel like one connected ecosystem

---

## 2. Problem Statement

### 2.1 Current Pain Points
- Controlling another device requires separate apps (TeamViewer, AnyDesk) that route through cloud servers — slow and privacy-invasive
- File transfer between own devices still involves cloud uploads, cables, or email-to-self
- Clipboard, notifications, and media remain siloed per device
- Existing solutions require accounts, subscriptions, or technical setup
- No single app provides control + transfer + sync in one unified experience

### 2.2 Market Opportunity
- Growing multi-device ownership (average user has 3.6 connected devices)
- Rising privacy concerns with cloud-routed remote control tools
- Demand for offline-first, zero-config, peer-to-peer solutions
- No dominant free solution for local-network cross-device control

---

## 3. Product Overview

### 3.1 Product Description
Stark-Link is a decentralized, peer-to-peer platform that turns all your devices into one connected ecosystem. Install it on any device, and instantly gain the ability to control, transfer files, sync clipboards, mirror screens, and manage devices — all over your local network with end-to-end encryption and zero cloud dependency.

### 3.2 Product Vision
One app. All your devices. Total control.

### 3.3 Product Scope

| Component | Description |
|-----------|-------------|
| stark-link-core | Rust core library — protocols, encryption, networking, device control |
| stark-link-desktop | Desktop app (Windows, macOS, Linux) built with Tauri + React |
| stark-link-mobile | Mobile app (Android, iOS) built with React Native |
| relay-server | WebSocket relay for signaling & remote connectivity (hosted on free tier) |
| stark-link-cli | Command-line tool for power users and scripting |

### 3.4 Distribution — Installable Packages (No Code Required)

| Platform | Format | Distribution |
|----------|--------|-------------|
| Windows | `.exe` / `.msi` installer | GitHub Releases, website |
| macOS | `.dmg` installer | GitHub Releases, website |
| Linux | `.AppImage`, `.deb`, `.rpm` | GitHub Releases, Snap Store |
| Android | `.apk` + Play Store | GitHub Releases, Google Play |
| iOS | App Store | Apple App Store |

Users download, install, and run. No source code, no terminal, no dependencies.

### 3.5 Free Tier Infrastructure

The relay/signaling server must run on **free cloud hosting** so users never need to self-host:

| Service | Purpose | Free Tier |
|---------|---------|-----------|
| Render / Railway | Relay WebSocket server | Free tier (750 hrs/month) |
| Cloudflare Workers | TURN/STUN signaling | 100K requests/day free |
| Supabase | Device registry & room codes | Free tier (500 MB) |
| GitHub Actions | CI/CD for building installers | 2000 mins/month free |
| GitHub Releases | Hosting installer downloads | Unlimited for public repos |

---

## 4. Business Objectives

### 4.1 Primary Objectives
1. **Seamless cross-device control** — control any connected device's screen, keyboard, and mouse from any other device
2. **Instant file transfers** — drag, drop, done — at full network speed
3. **Unified clipboard & notifications** — copy on phone, paste on laptop, see all notifications everywhere
4. **Zero-friction setup** — install the app, scan a QR code, you're connected
5. **No cost to users** — free to use with free-tier hosted infrastructure
6. **Works offline** — full functionality on local network without internet

### 4.2 Success Metrics

| Metric | Target |
|--------|--------|
| File transfer completion rate | > 99% |
| Transfer speed vs network capacity | > 90% |
| Clipboard sync latency (local) | < 500ms |
| Device discovery time | < 5 seconds |
| Screen share latency (local) | < 50ms |
| Remote input latency | < 30ms |
| Time from install to first connection | < 2 minutes |
| App installer size | < 50 MB |

---

## 5. User Requirements

### 5.1 Core Features

#### 5.1.1 Device Discovery & Pairing
- **F1:** Automatic discovery of nearby devices using mDNS
- **F2:** QR code scanning for instant pairing
- **F3:** PIN-based secure pairing as fallback
- **F4:** Room code for connecting devices across networks (via relay)
- **F5:** Device list showing online/offline status, device type, battery level, and OS
- **F6:** Persistent trust — paired devices reconnect automatically

#### 5.1.2 Remote Device Control
- **F7:** View and control another device's screen in real-time
- **F8:** Remote mouse/trackpad input from controlling device
- **F9:** Remote keyboard input with full key mapping
- **F10:** Multi-monitor support — select which screen to control
- **F11:** Adjustable quality/resolution for screen sharing (auto-adapt to network speed)
- **F12:** One-click connect — tap a device to start controlling it
- **F13:** Permission prompt on target device before control is granted

#### 5.1.3 File Transfer
- **F14:** Send files of any type up to 10 GB
- **F15:** Drag-and-drop file sending from desktop
- **F16:** Share sheet integration (Android Share, iOS Share Extension, Windows Share Target)
- **F17:** Chunked transfer with real-time progress bar
- **F18:** Resume interrupted transfers
- **F19:** File integrity verification via SHA-256 checksum
- **F20:** Transfer queue with batch sending and priority
- **F21:** Folder transfer support

#### 5.1.4 Clipboard Synchronization
- **F22:** Real-time clipboard sync (text, images, URLs, file paths)
- **F23:** Selective sync — choose which devices receive clipboard
- **F24:** Clipboard history with latest 50 items
- **F25:** Conflict resolution — last-writer-wins with visual indicator

#### 5.1.5 Notification Mirroring
- **F26:** See phone notifications on desktop
- **F27:** Dismiss, reply, or act on notifications from any device
- **F28:** Notification filtering — choose which apps to mirror

#### 5.1.6 Media & System Control
- **F29:** Control media playback (play/pause/skip) on any connected device
- **F30:** Adjust volume on remote devices
- **F31:** View system info (battery, storage, CPU, RAM) of connected devices
- **F32:** Lock/unlock, sleep/wake remote devices (with permission)
- **F33:** Launch apps on remote devices

#### 5.1.7 Remote Commands
- **F34:** Execute shell commands on paired devices
- **F35:** Approval workflow — target device must approve sensitive commands
- **F36:** Command history and output streaming

### 5.2 Non-Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| NFR1 | Security | AES-256-GCM encryption, X25519 key exchange, all traffic encrypted |
| NFR2 | Latency | Local network operations < 100ms, screen share < 50ms |
| NFR3 | Reliability | 99.9% connection stability on local network |
| NFR4 | Usability | Install to first connection < 2 minutes |
| NFR5 | Compatibility | Windows 10+, macOS 12+, Linux (Ubuntu 20.04+), Android 8+, iOS 14+ |
| NFR6 | Size | Installer < 50 MB |
| NFR7 | Updates | Auto-update mechanism with silent background updates |
| NFR8 | Offline | Full local-network functionality without internet |
| NFR9 | Performance | < 5% CPU usage when idle, < 15% during screen share |

---

## 6. Functional Requirements

### 6.1 Device Discovery Module

**FR.D1:** The system shall broadcast device presence using mDNS on `_starklink._tcp.local.`

**FR.D2:** The system shall listen for incoming discovery broadcasts and maintain a live device list

**FR.D3:** The system shall update device status (online/offline/busy) in real-time

**FR.D4:** The system shall support manual connection via room code or QR code for cross-network pairing

**FR.D5:** The system shall display device metadata: name, OS, type (desktop/mobile/tablet), battery level

### 6.2 Connection Management Module

**FR.C1:** The system shall establish WebSocket connections over TLS on port 42424

**FR.C2:** The system shall upgrade to WebRTC data channels for high-bandwidth operations (screen share, large file transfer)

**FR.C3:** The system shall implement connection states: DISCONNECTED, CONNECTING, HANDSHAKE, PAIRED, CONTROLLING

**FR.C4:** The system shall maintain heartbeat with 30-second intervals

**FR.C5:** The system shall automatically reconnect within 60 seconds of connection loss

**FR.C6:** The system shall support multiple simultaneous device connections

### 6.3 Encryption Module

**FR.E1:** The system shall generate X25519 key pairs on first launch and store securely

**FR.E2:** The system shall derive session keys using ECDH key exchange

**FR.E3:** The system shall encrypt all messages and streams using AES-256-GCM

**FR.E4:** The system shall use unique 12-byte random nonces for each encryption operation

**FR.E5:** The system shall display device fingerprints during pairing for TOFU verification

### 6.4 Remote Control Module

**FR.RC1:** The system shall capture and stream the screen of the controlled device using hardware-accelerated encoding (H.264/VP9)

**FR.RC2:** The system shall transmit mouse events (move, click, scroll, drag) from controller to target

**FR.RC3:** The system shall transmit keyboard events with correct key mapping across OS platforms

**FR.RC4:** The system shall require explicit permission grant on the target device before control begins

**FR.RC5:** The system shall allow the target user to revoke control at any time

**FR.RC6:** The system shall adapt stream quality based on available bandwidth (auto bitrate)

**FR.RC7:** The system shall support controlling mobile devices from desktop (touch simulation)

### 6.5 File Transfer Module

**FR.F1:** The system shall support transfers up to 10 GB per file

**FR.F2:** The system shall split files into chunks of up to 5 MB

**FR.F3:** The system shall use WebRTC data channels for transfer when available, WebSocket as fallback

**FR.F4:** The system shall verify chunk integrity with SHA-256

**FR.F5:** The system shall support up to 5 concurrent transfers

**FR.F6:** The system shall implement transfer pause/resume capability

**FR.F7:** The system shall deduplicate transfers by content hash

**FR.F8:** The system shall compress chunks with lz4 before encryption

### 6.6 Clipboard Sync Module

**FR.CL1:** The system shall capture system clipboard changes via OS-native APIs

**FR.CL2:** The system shall sync text, images, URLs, and file paths

**FR.CL3:** The system shall allow selective broadcast or targeted sync

**FR.CL4:** The system shall maintain a searchable clipboard history (50 items)

**FR.CL5:** The system shall use delta encoding for large text clipboard changes

### 6.7 Notification Module

**FR.N1:** The system shall capture device notifications via OS accessibility/notification APIs

**FR.N2:** The system shall forward notifications to all paired devices with app icon, title, and body

**FR.N3:** The system shall support notification actions (dismiss, reply) from remote devices

**FR.N4:** The system shall allow per-app notification filtering

### 6.8 Relay Server Module

**FR.R1:** The relay server shall accept WebSocket connections and manage room-based routing

**FR.R2:** The relay server shall create/join rooms with 6-character codes

**FR.R3:** The relay server shall forward encrypted messages between peers (zero-knowledge)

**FR.R4:** The relay server shall provide WebRTC signaling (STUN/TURN coordination)

**FR.R5:** The relay server shall run on free-tier cloud platforms (Render/Railway)

**FR.R6:** The relay server shall auto-sleep when no connections and wake on request

### 6.9 Auto-Update Module

**FR.U1:** The system shall check GitHub Releases for new versions on startup

**FR.U2:** The system shall download and apply updates silently in the background

**FR.U3:** The system shall prompt the user to restart after update is downloaded

**FR.U4:** The system shall support rollback to previous version if update fails

---

## 7. Technical Architecture

### 7.1 System Architecture

```
 ââââââââââââââââââââââ   âââââââââââââââââââââââ   ââââââââââââââââââââââ
 â  Windows Desktop   â   â  macOS Desktop      â   â  Linux Desktop     â
 â  (.exe installer)  â   â  (.dmg installer)   â   â  (.AppImage)       â
 ââââââââââââ¬ââââââââââ   âââââââââââ¬ââââââââââââ   ââââââââââââ¬ââââââââââ
             â                       â                          â
             ââââââââââââââââââââââââââ¼ââââââââââââââââââââââââââ
                                     â
                          ââââââââââââ´âââââââââââ
                          â  stark-link-core    â
                          â  (Rust Library)     â
                          â                     â
                          â  â Discovery â      â
                          â  â Control   â      â
                          â  â Transfer  â      â
                          â  â Crypto    â      â
                          â  â Clipboard â      â
                          â  â Notify    â      â
                          ââââââââââââ¬âââââââââââ
                                     â
                   âââââââââââââââââââ¼âââââââââââââââââââ
                   â                                     â
          ââââââââââ´âââââââââââ              ââââââââââââ´âââââââââââ
          â  Local Network    â              â  Relay Server       â
          â  mDNS + WebRTC    â              â  (Free Tier Cloud)  â
          â  (P2P Direct)     â              â  WebSocket + STUN   â
          âââââââââââââââââââââ              âââââââââââââââââââââââ
                   â                                     â
             ââââââ´ââââââââ                        ââââââ´âââââââââ
             â  Android   â                        â  iOS         â
             â  (.apk)    â                        â  (App Store) â
             ââââââââââââââ                        ââââââââââââââââ
```

### 7.2 Technology Stack

| Layer | Technology |
|-------|------------|
| Core Logic | Rust (stark-link-core) — compiled to native + WASM |
| Desktop UI | React + TypeScript + Tauri 2.0 |
| Mobile UI | React Native + Rust FFI bridge |
| Screen Capture | platform-native (DXGI on Windows, CGDisplay on macOS, PipeWire on Linux) |
| Video Encoding | H.264 (hardware) / VP9 (software fallback) |
| Encryption | X25519, AES-256-GCM (via ring crate) |
| Discovery | mDNS (mdns-sd crate) |
| P2P Transport | WebRTC data channels (webrtc-rs) |
| Signaling | WebSocket over TLS |
| Compression | lz4 for data, zstd for files |
| Storage | SQLite (via rusqlite) |
| Relay Server | Node.js + ws (or Rust with axum) |
| CI/CD | GitHub Actions — builds installers for all platforms |
| Auto-Update | Tauri updater (desktop), in-app update (mobile) |

### 7.3 Protocol Stack

| Layer | Protocol |
|-------|----------|
| Application | Custom binary protocol (MessagePack) + JSON signaling |
| Streaming | WebRTC (screen share, large transfers) |
| Signaling | WebSocket (WSS) |
| Transport | TCP (signaling), UDP (WebRTC/media) |
| Discovery | mDNS (UDP multicast) |

### 7.4 Build & Distribution Pipeline

```
GitHub Push â GitHub Actions CI/CD
    â
    âââ Build Windows â .exe/.msi â GitHub Releases
    âââ Build macOS   â .dmg      â GitHub Releases
    âââ Build Linux   â .AppImage â GitHub Releases
    âââ Build Android â .apk      â GitHub Releases + Play Store
    âââ Build iOS     â .ipa      â App Store (TestFlight)
    â
    âââ Deploy Relay Server â Render/Railway (auto-deploy from main)
```

---

## 8. User Stories

### 8.1 Setup & Discovery
- **US1:** As a user, I download the .exe, install it, and see nearby devices within seconds — no account needed
- **US2:** As a user, I scan a QR code on my phone to pair it with my laptop instantly
- **US3:** As a user, I see all my paired devices with their status, battery, and OS info

### 8.2 Remote Control
- **US4:** As a user, I tap my phone in the device list and see its screen on my laptop — I can control it with my mouse and keyboard
- **US5:** As a user, I control my home desktop from my laptop in another room
- **US6:** As a user, I get a permission prompt before anyone can control my device
- **US7:** As a user, I press Escape (or a hotkey) to instantly revoke remote control

### 8.3 File Transfer
- **US8:** As a user, I drag a file onto a device in the app and it transfers instantly
- **US9:** As a user, I use my phone's Share button to send a photo to my laptop via Stark-Link
- **US10:** As a user, I see a progress bar and can pause/resume large transfers
- **US11:** As a user, I send an entire folder to another device

### 8.4 Clipboard & Notifications
- **US12:** As a user, I copy text on my phone and paste it on my laptop seamlessly
- **US13:** As a user, I see my phone's notifications on my desktop and can dismiss or reply
- **US14:** As a user, I choose which apps' notifications to mirror

### 8.5 Media & System Control
- **US15:** As a user, I pause music playing on my desktop from my phone
- **US16:** As a user, I check my laptop's battery and storage from my phone
- **US17:** As a user, I lock my desktop remotely from my phone when I walk away

### 8.6 Security
- **US18:** As a user, I verify a PIN/fingerprint during pairing so I know I'm connecting to the right device
- **US19:** As a user, all my data is encrypted and never passes through any cloud server in readable form

---

## 9. Acceptance Criteria

### 9.1 Installation & Setup
- [ ] User downloads .exe, installs, and launches in under 2 minutes
- [ ] No account creation, no sign-up, no internet required for local use
- [ ] App size is under 50 MB

### 9.2 Device Discovery
- [ ] Two devices on the same network discover each other within 5 seconds
- [ ] QR code pairing completes in under 30 seconds
- [ ] Device list updates within 2 seconds when devices connect/disconnect

### 9.3 Remote Control
- [ ] Screen sharing starts within 3 seconds of permission grant
- [ ] Input latency is under 30ms on local gigabit network
- [ ] Screen share renders at minimum 30 FPS at 1080p on local network
- [ ] Target device shows clear "being controlled" indicator
- [ ] Control can be revoked instantly by target user

### 9.4 File Transfer
- [ ] 1 GB file transfers in under 2 minutes on gigabit network
- [ ] Transfer resumes correctly after interruption
- [ ] SHA-256 checksum verification passes on all transfers
- [ ] Drag-and-drop works on all desktop platforms

### 9.5 Clipboard Sync
- [ ] Text copied on one device appears on another within 500ms
- [ ] Image clipboard sync completes within 2 seconds

### 9.6 Notifications
- [ ] Phone notification appears on desktop within 1 second
- [ ] Reply from desktop is delivered to phone app correctly

### 9.7 Security
- [ ] All traffic is encrypted with AES-256-GCM
- [ ] Relay server cannot read any message content (zero-knowledge)
- [ ] Session keys are derived using X25519 ECDH

---

## 10. Constraints and Assumptions

### 10.1 Technical Constraints
- Remote control requires OS-level permissions (Accessibility on macOS, Admin on Windows)
- iOS screen capture is limited by Apple's APIs (screen broadcast extension required)
- Relay server free tier has cold-start latency (~2-5 seconds)
- WebRTC NAT traversal may fail on strict enterprise networks

### 10.2 Assumptions
- Users have local network with sufficient bandwidth (>10 Mbps for screen share)
- Users can grant OS-level permissions when prompted
- Free-tier cloud services remain available with current limits
- Users have basic ability to install an application

### 10.3 Platform-Specific Limitations

| Platform | Limitation | Workaround |
|----------|-----------|------------|
| iOS | No background screen capture | Use ReplayKit broadcast extension |
| iOS | No direct .apk-style install | App Store distribution only |
| macOS | Accessibility permission required | Guided setup wizard |
| Linux | Wayland screen capture varies | Support PipeWire + X11 fallback |
| Android | Battery optimization kills background | Foreground service with notification |

---

## 11. Phased Delivery

### Phase 1 — Foundation (MVP)
- [ ] Rust core library (discovery, encryption, connection management)
- [ ] Device discovery via mDNS
- [ ] QR code and PIN pairing
- [ ] File transfer (chunked, resumable, encrypted)
- [ ] Clipboard sync (text)
- [ ] Windows desktop app (.exe installer)
- [ ] Android app (.apk)
- [ ] Relay server on free tier
- [ ] GitHub Actions CI/CD for builds

### Phase 2 — Control
- [ ] Remote screen viewing and control
- [ ] Remote keyboard and mouse input
- [ ] Notification mirroring
- [ ] macOS and Linux desktop apps
- [ ] iOS app
- [ ] Auto-update system
- [ ] Image clipboard sync

### Phase 3 — Power Features
- [ ] Media playback control
- [ ] System info monitoring
- [ ] Remote lock/sleep/wake
- [ ] Remote app launching
- [ ] Shell command execution
- [ ] CLI tool
- [ ] Folder transfer
- [ ] Share sheet integration (all platforms)

### Phase 4 — Ecosystem
- [ ] Plugin/extension system
- [ ] Multi-device groups with group encryption
- [ ] Screen recording and sharing
- [ ] Local REST API for third-party integrations
- [ ] Enterprise features (centralized device management)

---

## 12. Appendix

### A. Protocol Message Types
**Signaling:** Hello, PairRequest, PairAccept, PairReject, DeviceInfo
**Clipboard:** ClipboardSync, ClipboardHistory
**File Transfer:** FileTransferStart, FileTransferChunk, FileTransferComplete, FileTransferCancel, FileTransferPause, FileTransferResume
**Remote Control:** ScreenShareStart, ScreenShareStop, MouseEvent, KeyboardEvent, ControlRequest, ControlRevoke
**System:** MediaControl, SystemInfo, RemoteLock, AppLaunch
**Commands:** CommandExecute, CommandResponse
**Notifications:** NotificationSync, NotificationAction
**Connection:** Ping, Pong, Error

### B. Error Codes
| Code | Description |
|------|-------------|
| INVALID_MESSAGE | Malformed or unrecognized message |
| AUTH_FAILED | Pairing or authentication failure |
| TRANSFER_FAILED | File transfer error |
| CONTROL_DENIED | Remote control permission denied |
| DEVICE_OFFLINE | Target device not reachable |
| TIMEOUT | Operation timed out |
| PERMISSION_DENIED | OS-level permission not granted |
| RELAY_UNAVAILABLE | Relay server not reachable |

### C. Port Configuration
| Port | Protocol | Purpose |
|------|----------|---------|
| 42424 | TCP (WebSocket/TLS) | Signaling & control messages |
| 42425 | UDP (WebRTC) | Screen share & data transfer |
| 5353 | UDP (mDNS) | Device discovery |

### D. Free Tier Limits & Fallbacks
| Service | Limit | Fallback |
|---------|-------|----------|
| Render | 750 hrs/month, sleeps after 15 min idle | Auto-wake on connection |
| Cloudflare Workers | 100K req/day | Rate limit non-essential calls |
| GitHub Releases | Unlimited (public repo) | — |
| GitHub Actions | 2000 min/month | Cache builds, only build on release tags |

---

**Document Version:** 2.0
**Date:** March 23, 2026
**Status:** Draft — Updated with cross-device control, installable distribution, and free-tier hosting
