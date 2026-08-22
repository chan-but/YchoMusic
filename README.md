# YchoMusic

A lightweight, beautiful local music player built with Rust + Tauri + Svelte 5.

Dark liquid-capsule UI, immersive lyrics, mini player, playlist shelves, and a full Rust audio engine supporting FLAC/MP3/OGG/WAV/AAC/ALAC.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

## Screenshots

> Coming soon

---

## Table of Contents

- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Build](#build)
- [Settings](#settings)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Roadmap](#roadmap)
- [License](#license)

---

## Features

### Playback

- **Multi-format support**: MP3, FLAC, OGG, WAV, AAC, ALAC via Symphonia + Rodio
- **Play modes**: Shuffle, Repeat One, Repeat List
- **Volume control** with real-time numeric display
- **Position memory**: resumes playback position after restart
- **Gapless playback** between tracks in the queue

### UI / UX

- **Dark liquid-capsule design**: all features presented as floating capsule widgets, no traditional sidebar
- **Home page**: Stats, Favorites, Albums/Artists combined capsule, Playlist shelf with auto-scroll
- **Immersive player**: full-screen lyrics view with adjustable font size & line height
- **Mini player**: compact mode with single-line lyrics, always-on-top option
- **Custom window chrome**: frameless window with custom titlebar controls
- **SVG icons**: clean, scalable iconography throughout

### Playlist Management

- **Unlimited custom playlists** with drag-and-drop reordering
- **Playlist shelf**: visual shelf on home page with seamless circular auto-scroll (3 modes: LTR, RTL, bounce)
- **Source tracking**: playlists created from albums/artists auto-sync when library is rescanned
- **Right-click context menus** on songs, albums, and artists (add to shelf, play, edit tags, locate file)

### Library

- **Folder-based scanning** with recursive directory traversal
- **Multi-artist splitting**: tracks with `&`, `/`, `;` in artist field are categorized under each artist independently
- **ID3 tag editing** via built-in Music Tags editor
- **Favorites/heart system** with animation
- **Search overlay** with fuzzy matching

### Lyrics

- **Inline lyrics** with current-line highlighting
- **Desktop lyrics** overlay (configurable 1-10 lines)
- **Click-to-seek**: click any lyric line to jump to that timestamp
- **Font customization**: adjustable font size (12-32px) and line height (1.5-4.0)

### Settings

- **Appearance**: theme selection (dark, glassmorphism, cyberpunk, etc.)
- **General**: font family (detects local system fonts), font size (10-24px), close behavior (ask / tray / exit)
- **Playback**: auto-play, default volume with manual input, play mode
- **Lyrics**: mode, line count, desktop lyrics, font size, line height
- **System**: cache management with size limit, global hotkeys (6 configurable shortcuts)
- **Playlist**: click-cover-plays toggle, auto-scroll modes & speed, loop marker toggle
- **Save & Restart**: save button with restart prompt for settings that require it

---

## Tech Stack

### Frontend

| Technology | Purpose |
|---|---|
| [Svelte 5](https://svelte.dev) | UI framework with runes (`$state`, `$derived`, `$effect`) |
| [Vite 6](https://vitejs.dev) | Build tool & dev server |
| [TypeScript 5](https://www.typescriptlang.org) | Type safety |
| [Tailwind CSS 4](https://tailwindcss.com) | Utility-first styling |
| [Tauri API 2](https://tauri.app) | IPC bridge to Rust backend |

### Backend (Rust)

| Crate | Purpose |
|---|---|
| [Tauri 2](https://tauri.app) | Desktop app framework, window management, IPC |
| [Symphonia 0.6](https://github.com/pdeljanov/Symphonia) | Audio decoding (MP3, FLAC, OGG, AAC, ALAC, WAV) |
| [Rodio 0.18](https://github.com/RustAudio/rodio) | Audio playback engine |
| [Lofty 0.18](https://github.com/Serial-ATA/lofty-rs) | ID3/metadata tag reading & writing |
| [Rusqlite 0.31](https://github.com/rusqlite/rusqlite) | SQLite database for library & stats |
| [Tauri Plugin Dialog](https://tauri.app) | File/folder picker dialogs |
| [Tauri Plugin Global Shortcut](https://tauri.app) | System-wide hotkey registration |

---

## Project Structure

```
YchoMusic/
├── src/                          # Frontend (Svelte + TS)
│   ├── components/               # Reusable UI components
│   │   ├── Capsule.svelte        # Generic capsule widget wrapper
│   │   ├── FullscreenPlayer.svelte  # Fullscreen immersive player
│   │   ├── Lyrics.svelte         # Lyrics renderer with highlighting
│   │   ├── MiniPlayerView.svelte # Mini mode compact UI
│   │   ├── SearchOverlay.svelte  # Global search modal
│   │   ├── Spectrum.svelte       # Audio spectrum visualizer
│   │   └── WindowControls.svelte # Custom titlebar buttons
│   ├── lib/                      # Utilities
│   │   ├── quality.ts            # Audio quality detection
│   │   ├── tauri.ts              # Tauri IPC wrappers
│   │   └── window.ts            # Window mode management
│   ├── routes/                   # Page-level components
│   │   ├── Home.svelte           # Home page with capsules
│   │   ├── Songs.svelte          # Song list with sort/filter
│   │   ├── Albums.svelte         # Album grid + detail
│   │   ├── Artists.svelte        # Artist grid + detail
│   │   ├── Player.svelte         # Main player view
│   │   ├── PlaylistEditor.svelte # Playlist shelf editor
│   │   ├── PlaylistList.svelte   # Playlist track listing
│   │   ├── Settings.svelte       # Settings page
│   │   ├── Stats.svelte          # Listening statistics
│   │   └── MusicTags.svelte      # Tag editor
│   ├── stores/                   # Svelte stores
│   │   ├── library.ts            # Track/album/artist data
│   │   ├── player.ts             # Playback state & controls
│   │   ├── playlists.ts          # Custom playlist CRUD + shelf sync
│   │   ├── settings.ts           # Settings store + runtime apply
│   │   ├── router.ts             # In-memory route state
│   │   ├── ui.ts                 # UI state (overlays, menus)
│   │   └── windowMode.ts         # Normal/mini/immersive mode
│   ├── styles/
│   │   └── global.css            # Global styles & CSS variables
│   ├── types/
│   │   └── index.ts              # TypeScript interfaces
│   ├── App.svelte                # Root component
│   └── main.ts                   # Entry point
│
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── audio/                # Audio engine
│   │   │   ├── decoder.rs        # Symphonia decoder integration
│   │   │   ├── player.rs         # Rodio playback control
│   │   │   └── mod.rs
│   │   ├── commands/             # Tauri IPC command handlers
│   │   │   ├── audio.rs          # Play/pause/seek/volume
│   │   │   ├── browse.rs         # Album/artist browsing + multi-artist split
│   │   │   ├── favorites.rs      # Heart/favorite management
│   │   │   ├── folders.rs        # Folder scanning & management
│   │   │   ├── library.rs        # Track scanning, ID3 parsing, metadata extraction
│   │   │   ├── lyrics.rs         # LRC parsing & lyrics retrieval
│   │   │   ├── playlist.rs       # Playlist persistence
│   │   │   ├── settings.rs       # Settings get/set with persistence
│   │   │   ├── stats.rs          # Listening statistics tracking
│   │   │   └── mod.rs
│   │   ├── db/                   # SQLite database
│   │   │   ├── init.rs           # Connection & migration
│   │   │   ├── schema.rs         # Table definitions
│   │   │   └── mod.rs
│   │   ├── models/               # Data models
│   │   │   ├── track.rs          # Track struct
│   │   │   ├── player.rs         # Player state enum
│   │   │   ├── playlist.rs       # Playlist struct
│   │   │   └── mod.rs
│   │   └── main.rs               # Tauri app entry, window event handling
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri window & bundle config
│
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── .gitignore
└── README.md
```

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Tauri 2 CLI prerequisites](https://tauri.app/start/prerequisites/)

### Install & Run

```bash
# Install frontend dependencies
npm install

# Start development (Tauri dev server + Vite HMR)
npm run tauri dev
```

---

## Build

```bash
# Build production bundle (Vite build + Rust release compile)
npm run tauri build
```

Output binaries are placed in `src-tauri/target/release/bundle/`.

---

## Settings

| Category | Setting | Description |
|---|---|---|
| **General** | Font Family | Detects and lists local system fonts |
| | Font Size | 10-24px, applied globally via `rem` |
| | Close Behavior | Ask / Minimize to tray / Exit |
| **Appearance** | Theme | Dark, glassmorphism, cyberpunk |
| | Spectrum | Toggle audio spectrum visualizer |
| **Playback** | Auto-play | Resume on app start |
| | Default Volume | 0-100 with manual input |
| | Play Mode | Shuffle / Repeat One / Repeat List |
| **Lyrics** | Mode | Inline / File-based |
| | Font Size | 12-32px (adjustable in player via "字" button) |
| | Line Height | 1.5-4.0 |
| | Desktop Lyrics | Toggleable overlay (1-10 lines) |
| **Playlist** | Click Cover Plays | Toggle: click cover to play vs. open list |
| | Auto-scroll | 3 modes: LTR / RTL / Bounce |
| | Scroll Speed | 1-10 |
| | Loop Marker | Show arrow indicator on circular scroll |
| **System** | Cache | Enable/disable with size limit input |
| | Global Hotkeys | 6 configurable shortcuts (see below) |

---

## Keyboard Shortcuts

All shortcuts are customizable in Settings > System > Global Hotkeys.

| Action | Default |
|---|---|
| Previous track | `MediaTrackPrevious` |
| Next track | `MediaTrackNext` |
| Play / Pause | `MediaPlayPause` |
| Mode switch | `Ctrl+M` |
| Mini mode | `Ctrl+Shift+M` |
| Fullscreen mode | `Ctrl+Shift+F` |

---

## Roadmap

- [ ] Time-grid statistics (calendar heatmap of listening history)
- [ ] Audio output device selection
- [ ] Playlist export (M3U)
- [ ] Audio gain / normalization
- [ ] 3D coverflow effects on album scroller
- [ ] GPU acceleration toggle
- [ ] Auto-update checker

---

## License

MIT
