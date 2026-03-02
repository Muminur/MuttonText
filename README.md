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

### One-Line Install (macOS)

```bash
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest \
  | grep "browser_download_url.*\.dmg" \
  | cut -d '"' -f 4 \
  | xargs -I {} sh -c 'curl -L -o /tmp/MuttonText.dmg "{}" && open /tmp/MuttonText.dmg'
```

This downloads the latest `.dmg`, mounts it, and opens it so you can drag MuttonText.app to your Applications folder. Grant Accessibility permissions when prompted.

### One-Line Install (Ubuntu/Debian)

```bash
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest | grep "browser_download_url.*\.deb" | cut -d '"' -f 4 | xargs -I {} sh -c 'wget -q --show-progress -O /tmp/MuttonText.deb "{}" && sudo dpkg -i /tmp/MuttonText.deb && sudo apt install -f -y && rm /tmp/MuttonText.deb'
```

If `sudo` can't prompt for a password (e.g. inside a non-interactive shell), replace `sudo` with `pkexec` for a graphical auth dialog.

This automatically downloads the latest `.deb` release, installs it, resolves any missing dependencies, and cleans up.

### Manual Installation

**Linux (Debian/Ubuntu) - `.deb` package:**
```bash
# Download latest release (check Releases page for current version)
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.0.1_amd64.deb

# Install
sudo dpkg -i MuttonText_0.0.1_amd64.deb

# Fix any missing dependencies
sudo apt install -f -y
```

**Linux (Fedora/RPM):**
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText-0.0.1-1.x86_64.rpm
sudo rpm -i MuttonText-0.0.1-1.x86_64.rpm
```

**macOS:**
```bash
# One-liner: download and open the latest DMG
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest \
  | grep "browser_download_url.*\.dmg" \
  | cut -d '"' -f 4 \
  | xargs -I {} sh -c 'curl -L -o /tmp/MuttonText.dmg "{}" && open /tmp/MuttonText.dmg'
# Then drag MuttonText.app to Applications and grant Accessibility permissions when prompted
```

> **Note:** Versions in the URLs above (e.g. `0.0.1`) may be outdated. The one-liner above always fetches the latest. You can also check the [Releases page](https://github.com/Muminur/MuttonText/releases/latest) for the current version.

### Updating MuttonText

MuttonText checks for updates automatically on startup (configurable in **Preferences → Updates**). When an update is detected you can download it directly from within the app.

You can also update manually:

**In-app update check (all platforms):**
1. Open MuttonText → Preferences (tray icon or menu)
2. Go to the **Updates** tab
3. Click **Check for Updates**
4. If a new version is available, click **Download Update**

**macOS — one-liner:**
```bash
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest \
  | grep "browser_download_url.*\.dmg" \
  | cut -d '"' -f 4 \
  | xargs -I {} sh -c 'curl -L -o /tmp/MuttonText.dmg "{}" && open /tmp/MuttonText.dmg'
# Drag the new MuttonText.app to Applications, replacing the old one
```

**Linux (Debian/Ubuntu) — one-liner:**
```bash
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest \
  | grep "browser_download_url.*\.deb" \
  | cut -d '"' -f 4 \
  | xargs -I {} sh -c 'wget -q --show-progress -O /tmp/MuttonText.deb "{}" && sudo dpkg -i /tmp/MuttonText.deb && sudo apt install -f -y && rm /tmp/MuttonText.deb'
```

**Linux (Fedora/RPM) — one-liner:**
```bash
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest \
  | grep "browser_download_url.*\.rpm" \
  | cut -d '"' -f 4 \
  | xargs -I {} sh -c 'wget -q --show-progress -O /tmp/MuttonText.rpm "{}" && sudo rpm -U /tmp/MuttonText.rpm && rm /tmp/MuttonText.rpm'
```

**Linux (AppImage):**
```bash
# Download new AppImage
curl -sL https://api.github.com/repos/Muminur/MuttonText/releases/latest \
  | grep "browser_download_url.*\.AppImage" \
  | cut -d '"' -f 4 \
  | xargs -I {} sh -c 'wget -q --show-progress -O ~/Applications/MuttonText.AppImage "{}" && chmod +x ~/Applications/MuttonText.AppImage'
```

**Windows:**
1. Download the latest `.exe` installer from the [Releases page](https://github.com/Muminur/MuttonText/releases/latest)
2. Run the installer — it will automatically replace the existing installation

### Uninstallation

**Linux (Debian/Ubuntu):**
```bash
sudo apt remove mutton-text
```

**Linux (Fedora/RPM):**
```bash
sudo rpm -e mutton-text
```

**macOS:**
```bash
# Drag MuttonText.app from Applications to Trash
# Optionally remove app data:
rm -rf ~/Library/Application\ Support/com.muttontext.app
rm -rf ~/Library/Preferences/com.muttontext.app.plist
rm -rf ~/Library/Caches/com.muttontext.app
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
  libxdo-devel libX11-devel libxcb-devel alsa-lib-devel
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

# One-command setup (installs Rust automatically if missing, checks all deps)
bash scripts/setup-dev.sh

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build

# Install the built package
# Linux:
sudo dpkg -i src-tauri/target/release/bundle/deb/MuttonText_0.0.1_amd64.deb
# macOS: open the generated .dmg from src-tauri/target/release/bundle/dmg/
open src-tauri/target/release/bundle/dmg/
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux (X11) | Full Support | Recommended desktop environment |
| Linux (Wayland) | Partial Support | Limited by Wayland security model |
| macOS 12+ | Full Support | Requires Accessibility permissions |
| Windows | Not Supported | Planned for future release |

## Integrations

Extend MuttonText with community-built integrations:

### Claude AI — Dynamic Text Expansion (Ubuntu/Linux)

Use Claude to expand selected text dynamically. Select rough notes → type `;;email` → Claude writes a polished email in-place.

```bash
bash integrations/claude-autokey/install.sh
```

**Triggers:** `;;email`, `;;tldr`, `;;reply`, `;;fix`

-> [Setup guide and tutorial](integrations/claude-autokey/TUTORIAL.md)

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
