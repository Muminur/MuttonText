# MuttonText

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-lightgrey.svg)](#platform-support)
[![Release](https://img.shields.io/github/v/release/Muminur/MuttonText)](https://github.com/Muminur/MuttonText/releases/latest)

**Free, open-source, cross-platform text expansion for Linux and macOS**

MuttonText is a privacy-first text snippet expansion tool that automates repetitive typing through intelligent keyword-to-snippet substitution. Built with native performance and Beeftext compatibility, MuttonText brings powerful text expansion to Linux and macOS users.

## What is MuttonText?

MuttonText monitors your typing and instantly expands predefined keywords into full text snippets. Type `sig` and get your full email signature. Type `addr` and get your complete address. All processing happens locally on your machine - no cloud, no telemetry, ever.

## Key Features

- **Privacy First** - All data stays local. No telemetry. No cloud. Ever.
- **Native Performance** - Sub-50ms substitution latency through Rust backend
- **Beeftext Compatible** - Import/export Beeftext libraries seamlessly
- **Rich Variables** - Date/time, clipboard, input dialogs, nested combos
- **Combo Picker** - Quick search window for instant snippet access
- **Group Organization** - Organize snippets into hierarchical groups
- **Flexible Matching** - Strict word-boundary or loose substring matching
- **Application Exclusions** - Auto-pause in password managers and other apps
- **Automatic Backups** - Never lose your snippet library
- **System Tray Integration** - Quick pause/resume and status at a glance

## Screenshot

> Screenshot placeholder - coming soon

## Quick Start

> **Note:** Releases are automatically created on every push to main using semantic versioning from commit messages.

### Installation

**Linux (Debian/Ubuntu) - `.deb` package:**
```bash
# Download latest release
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.0.1_amd64.deb

# Install
sudo dpkg -i MuttonText_0.0.1_amd64.deb

# Fix any missing dependencies
sudo apt install -f
```

**Linux (AppImage):**
```bash
# AppImage coming in v0.1.1
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.0.1_amd64.AppImage
chmod +x MuttonText_0.0.1_amd64.AppImage
./MuttonText_0.0.1_amd64.AppImage
```

**Linux (Fedora/RPM):**
```bash
# Download latest release
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText-0.0.1-1.x86_64.rpm

# Install
sudo rpm -i MuttonText-0.0.1-1.x86_64.rpm
```

**macOS:**
```bash
# Download DMG from releases page
# Drag MuttonText.app to Applications
# Grant Accessibility and Input Monitoring permissions when prompted
```

### Uninstallation

**Linux (Debian/Ubuntu):**
```bash
sudo apt remove mutton-text
# Clean up unused dependencies
sudo apt autoremove
```

**Linux (Fedora/RPM):**
```bash
sudo rpm -e mutton-text
```

**Linux (AppImage):**
```bash
# Simply delete the AppImage file
rm MuttonText_0.0.1_amd64.AppImage
```

**macOS:**
```bash
# Drag MuttonText.app from Applications to Trash
```

### First Run

1. Launch MuttonText - it will appear in your system tray
2. Click the tray icon and choose "Open Main Window"
3. Create your first combo:
   - Keyword: `hello`
   - Snippet: `Hello, World!`
4. Type `hello` followed by a space in any application
5. Watch it expand to `Hello, World!`

## Build from Source

### Prerequisites

**All Platforms:**
- Rust 1.78+ (latest stable recommended) - [Install Rust](https://rustup.rs/)
- Node.js 18+ - [Install Node.js](https://nodejs.org/)
- Tauri CLI: `cargo install tauri-cli`

**Linux (Debian/Ubuntu):**
```bash
sudo apt install -y build-essential libssl-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libwebkit2gtk-4.1-dev \
  libxdo-dev libx11-dev libxcb1-dev libxcb-render0-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev
```

**Linux (Fedora):**
```bash
sudo dnf install -y @development-tools openssl-devel gtk3-devel \
  libappindicator-gtk3-devel librsvg2-devel webkit2gtk4.1-devel \
  libxdo-devel libX11-devel libxcb-devel
```

**macOS:**
```bash
xcode-select --install
```

### Build Steps

```bash
# Clone repository
git clone https://github.com/Muminur/MuttonText.git
cd MuttonText

# Install dependencies
npm install

# Verify setup
cd src-tauri && cargo check && cd ..
npm run typecheck

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build

# Install the built .deb package (Linux)
sudo dpkg -i src-tauri/target/release/bundle/deb/MuttonText_0.0.1_amd64.deb
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux (X11) | Full Support | Recommended desktop environment |
| Linux (Wayland) | Partial Support | Limited by Wayland security model |
| macOS 12+ | Full Support | Requires Accessibility permissions |
| Windows | Not Supported | Planned for future release |

## Documentation


- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines
- [CHANGELOG.md](CHANGELOG.md) - Version history

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:
- How to report bugs and request features
- Development setup and workflow
- Code style and testing requirements
- Pull request process

## License

MuttonText is licensed under the [MIT License](LICENSE).

## Acknowledgments

- [Beeftext](https://github.com/xmichelo/Beeftext) - Inspiration and feature reference
- [Tauri](https://tauri.app/) - Cross-platform framework
- All our [contributors](https://github.com/Muminur/MuttonText/graphs/contributors)
