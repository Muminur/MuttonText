# .deb Packaging & Distribution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement dual-channel (.deb) distribution system with GitHub Releases and APT repository, automated CI/CD, comprehensive testing, and thorough documentation.

**Architecture:** Three GitHub Actions workflows (build-and-test, release-nightly, release-stable) with Docker-based integration tests, APT repository hosted on GitHub Pages with stable/nightly channels, full TDD approach with test-first implementation.

**Tech Stack:** GitHub Actions, Docker, dpkg-deb tools, Tauri bundler, GitHub Pages, bash scripting

---

## Task 1: Add xdotool Dependency to .deb Package

**Files:**
- Modify: `src-tauri/tauri.conf.json:38-42`

**Step 1: Write verification test**

Create: `tests/verify-deb-deps.sh`

```bash
#!/bin/bash
# Verify xdotool is declared as dependency in tauri.conf.json

set -e

if ! grep -q '"xdotool"' src-tauri/tauri.conf.json; then
    echo "ERROR: xdotool not found in deb dependencies"
    exit 1
fi

echo "✓ xdotool dependency verified"
```

**Step 2: Make test executable and run it**

Run:
```bash
chmod +x tests/verify-deb-deps.sh
./tests/verify-deb-deps.sh
```

Expected: FAIL with "ERROR: xdotool not found in deb dependencies"

**Step 3: Add xdotool to dependencies**

Modify `src-tauri/tauri.conf.json` - Update the deb depends array:

```json
"deb": {
  "depends": [
    "libwebkit2gtk-4.1-0",
    "libgtk-3-0",
    "libayatana-appindicator3-1",
    "xdotool"
  ]
}
```

**Step 4: Run test to verify it passes**

Run: `./tests/verify-deb-deps.sh`

Expected: PASS with "✓ xdotool dependency verified"

**Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json tests/verify-deb-deps.sh
git commit -m "feat: add xdotool as .deb dependency"
```

---

## Task 2: Create Docker Test Infrastructure

**Files:**
- Create: `tests/docker/ubuntu-20.04/Dockerfile`
- Create: `tests/docker/ubuntu-22.04/Dockerfile`
- Create: `tests/docker/ubuntu-24.04/Dockerfile`
- Create: `tests/docker/test-install.sh`

**Step 1: Create Ubuntu 20.04 test Dockerfile**

Create: `tests/docker/ubuntu-20.04/Dockerfile`

```dockerfile
FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

# Install basic dependencies
RUN apt-get update && apt-get install -y \
    wget \
    ca-certificates \
    xdotool \
    && rm -rf /var/lib/apt/lists/*

# Copy .deb package (will be mounted at runtime)
WORKDIR /test

CMD ["/bin/bash"]
```

**Step 2: Create Ubuntu 22.04 test Dockerfile**

Create: `tests/docker/ubuntu-22.04/Dockerfile`

```dockerfile
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    wget \
    ca-certificates \
    xdotool \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /test

CMD ["/bin/bash"]
```

**Step 3: Create Ubuntu 24.04 test Dockerfile**

Create: `tests/docker/ubuntu-24.04/Dockerfile`

```dockerfile
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    wget \
    ca-certificates \
    xdotool \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /test

CMD ["/bin/bash"]
```

**Step 4: Create installation test script**

Create: `tests/docker/test-install.sh`

```bash
#!/bin/bash
# Test .deb installation in Docker container

set -e

UBUNTU_VERSION=${1:-20.04}
DEB_PATH=${2:-src-tauri/target/release/bundle/deb/muttontext_0.1.0_amd64.deb}

echo "Testing installation on Ubuntu ${UBUNTU_VERSION}"

# Build Docker image
docker build -t muttontext-test:ubuntu-${UBUNTU_VERSION} \
    tests/docker/ubuntu-${UBUNTU_VERSION}/

# Run installation test
docker run --rm \
    -v "$(pwd)/${DEB_PATH}:/test/muttontext.deb:ro" \
    muttontext-test:ubuntu-${UBUNTU_VERSION} \
    bash -c "
        set -e
        echo '=== Installing .deb package ==='
        dpkg -i /test/muttontext.deb || apt-get install -f -y

        echo '=== Verifying installation ==='
        dpkg -l | grep muttontext

        echo '=== Checking dependencies ==='
        dpkg -s xdotool
        dpkg -s libwebkit2gtk-4.1-0

        echo '✓ Installation test passed for Ubuntu ${UBUNTU_VERSION}'
    "
```

**Step 5: Make test script executable**

Run:
```bash
chmod +x tests/docker/test-install.sh
```

**Step 6: Test the script (will fail if no .deb exists)**

Run: `./tests/docker/test-install.sh 20.04`

Expected: Either passes if .deb exists, or fails gracefully with file not found

**Step 7: Commit**

```bash
git add tests/docker/
git commit -m "test: add Docker-based .deb installation tests"
```

---

## Task 3: Create APT Repository Scripts

**Files:**
- Create: `scripts/generate-apt-repo.sh`
- Create: `scripts/update-github-pages.sh`

**Step 1: Create Packages index generation script**

Create: `scripts/generate-apt-repo.sh`

```bash
#!/bin/bash
# Generate APT repository Packages index

set -e

CHANNEL=${1:-stable}  # stable or nightly
DEB_FILE=${2}
REPO_DIR=${3:-apt-repo}

if [ -z "$DEB_FILE" ]; then
    echo "Usage: $0 <channel> <deb-file> [repo-dir]"
    exit 1
fi

if [ ! -f "$DEB_FILE" ]; then
    echo "ERROR: .deb file not found: $DEB_FILE"
    exit 1
fi

echo "Generating APT repository for channel: $CHANNEL"

# Create directory structure
mkdir -p "$REPO_DIR/pool"
mkdir -p "$REPO_DIR/dists/$CHANNEL/main/binary-amd64"

# Copy .deb to pool
DEB_FILENAME=$(basename "$DEB_FILE")
cp "$DEB_FILE" "$REPO_DIR/pool/$DEB_FILENAME"

# Generate Packages file
cd "$REPO_DIR"
dpkg-scanpackages --multiversion pool > "dists/$CHANNEL/main/binary-amd64/Packages"
gzip -c "dists/$CHANNEL/main/binary-amd64/Packages" > "dists/$CHANNEL/main/binary-amd64/Packages.gz"

echo "✓ APT repository generated at $REPO_DIR"
echo "  Channel: $CHANNEL"
echo "  Package: $DEB_FILENAME"
```

**Step 2: Create GitHub Pages deployment script**

Create: `scripts/update-github-pages.sh`

```bash
#!/bin/bash
# Deploy APT repository to GitHub Pages

set -e

REPO_DIR=${1:-apt-repo}
COMMIT_MSG=${2:-"chore: update APT repository"}

if [ ! -d "$REPO_DIR" ]; then
    echo "ERROR: Repository directory not found: $REPO_DIR"
    exit 1
fi

echo "Deploying to GitHub Pages..."

# Save current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Check if gh-pages exists
if git show-ref --verify --quiet refs/heads/gh-pages; then
    echo "Checking out existing gh-pages branch"
    git checkout gh-pages
else
    echo "Creating new gh-pages branch"
    git checkout --orphan gh-pages
    git rm -rf . || true
fi

# Copy APT repository contents
cp -r "$REPO_DIR"/* .

# Commit and push
git add -A
git commit -m "$COMMIT_MSG" || echo "No changes to commit"
git push origin gh-pages

# Return to original branch
git checkout "$CURRENT_BRANCH"

echo "✓ Deployed to GitHub Pages"
```

**Step 3: Make scripts executable**

Run:
```bash
chmod +x scripts/generate-apt-repo.sh
chmod +x scripts/update-github-pages.sh
```

**Step 4: Test APT generation script (requires .deb)**

Run:
```bash
# Only test if .deb exists
if [ -f "src-tauri/target/release/bundle/deb/muttontext_0.1.0_amd64.deb" ]; then
    ./scripts/generate-apt-repo.sh stable \
        src-tauri/target/release/bundle/deb/muttontext_0.1.0_amd64.deb \
        /tmp/test-apt-repo
    ls -la /tmp/test-apt-repo/dists/stable/main/binary-amd64/
fi
```

Expected: Creates Packages and Packages.gz files

**Step 5: Commit**

```bash
git add scripts/
git commit -m "feat: add APT repository generation scripts"
```

---

## Task 4: Create Build and Test Workflow

**Files:**
- Create: `.github/workflows/build-and-test.yml`

**Step 1: Create workflow file**

Create: `.github/workflows/build-and-test.yml`

```yaml
name: Build and Test

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test on Ubuntu ${{ matrix.ubuntu }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        ubuntu: ['20.04', '22.04', '24.04']
      fail-fast: false

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            xdotool \
            dpkg-dev

      - name: Install frontend dependencies
        run: npm ci

      - name: Run TypeScript type check
        run: npm run typecheck

      - name: Run Rust tests
        run: cd src-tauri && cargo test --lib

      - name: Build .deb package
        run: npm run tauri build -- --bundles deb

      - name: Verify .deb created
        run: |
          DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -n 1)
          if [ -z "$DEB_FILE" ]; then
            echo "ERROR: No .deb file found"
            exit 1
          fi
          echo "Found: $DEB_FILE"
          ls -lh "$DEB_FILE"

      - name: Test .deb installation (Docker)
        run: |
          DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -n 1)
          ./tests/docker/test-install.sh ${{ matrix.ubuntu }} "$DEB_FILE"

      - name: Upload .deb artifact
        uses: actions/upload-artifact@v4
        with:
          name: muttontext-ubuntu-${{ matrix.ubuntu }}-deb
          path: src-tauri/target/release/bundle/deb/*.deb
          retention-days: 7

  trigger-release:
    name: Trigger Nightly Release
    needs: test
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - name: Trigger release workflow
        run: echo "Tests passed - nightly release will be triggered"
```

**Step 2: Create .github directory**

Run:
```bash
mkdir -p .github/workflows
```

**Step 3: Commit workflow**

```bash
git add .github/workflows/build-and-test.yml
git commit -m "ci: add build and test workflow"
```

**Step 4: Push and verify workflow runs**

Run:
```bash
git push origin main
```

Expected: GitHub Actions workflow triggers and runs

**Step 5: Monitor workflow**

Check: https://github.com/Muminur/MuttonText/actions

Expected: Workflow runs on Ubuntu 20.04, 22.04, 24.04 matrix

---

## Task 5: Create Nightly Release Workflow

**Files:**
- Create: `.github/workflows/release-nightly.yml`

**Step 1: Create nightly release workflow**

Create: `.github/workflows/release-nightly.yml`

```yaml
name: Nightly Release

on:
  workflow_run:
    workflows: ["Build and Test"]
    types:
      - completed
    branches: [ main ]

permissions:
  contents: write

jobs:
  release:
    name: Create Nightly Release
    if: ${{ github.event.workflow_run.conclusion == 'success' }}
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Generate nightly version
        id: version
        run: |
          BASE_VERSION=$(jq -r '.version' src-tauri/tauri.conf.json)
          NIGHTLY_VERSION="${BASE_VERSION}-nightly-$(date +%Y%m%d)"
          echo "version=${NIGHTLY_VERSION}" >> $GITHUB_OUTPUT
          echo "tag=nightly-$(date +%Y%m%d)" >> $GITHUB_OUTPUT
          echo "Nightly version: ${NIGHTLY_VERSION}"

      - name: Update version in tauri.conf.json
        run: |
          jq ".version = \"${{ steps.version.outputs.version }}\"" \
            src-tauri/tauri.conf.json > tauri.conf.json.tmp
          mv tauri.conf.json.tmp src-tauri/tauri.conf.json

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            dpkg-dev

      - name: Install frontend dependencies
        run: npm ci

      - name: Build .deb packages
        run: npm run tauri build -- --bundles deb

      - name: Find .deb file
        id: deb
        run: |
          DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -n 1)
          echo "path=${DEB_FILE}" >> $GITHUB_OUTPUT
          echo "name=$(basename ${DEB_FILE})" >> $GITHUB_OUTPUT

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ steps.version.outputs.tag }}
          name: Nightly Build ${{ steps.version.outputs.version }}
          body: |
            ## Nightly Build - $(date +%Y-%m-%d)

            **⚠️ This is a pre-release nightly build**

            Automated build from the latest main branch commit.

            ### Installation

            **Ubuntu/Debian:**
            ```bash
            wget https://github.com/Muminur/MuttonText/releases/download/${{ steps.version.outputs.tag }}/${{ steps.deb.outputs.name }}
            sudo dpkg -i ${{ steps.deb.outputs.name }}
            sudo apt-get install -f
            ```

            **APT Repository (Nightly Channel):**
            ```bash
            echo "deb [trusted=yes] https://muminur.github.io/MuttonText nightly main" | sudo tee /etc/apt/sources.list.d/muttontext.list
            sudo apt update
            sudo apt install muttontext
            ```

            ### Commit
            ${{ github.sha }}
          files: ${{ steps.deb.outputs.path }}
          prerelease: true
          draft: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Setup APT repository
        run: |
          ./scripts/generate-apt-repo.sh nightly \
            "${{ steps.deb.outputs.path }}" \
            apt-repo

      - name: Configure Git
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"

      - name: Deploy to GitHub Pages
        run: |
          # Check if gh-pages exists
          if git ls-remote --exit-code --heads origin gh-pages; then
            git fetch origin gh-pages
            git checkout gh-pages
            git pull origin gh-pages
          else
            git checkout --orphan gh-pages
            git rm -rf . || true
          fi

          # Copy APT repo structure
          mkdir -p dists/nightly pool
          cp -r apt-repo/dists/nightly/* dists/nightly/
          cp apt-repo/pool/* pool/

          # Commit and push
          git add -A
          git commit -m "chore: update nightly APT repository - ${{ steps.version.outputs.version }}" || echo "No changes"
          git push origin gh-pages
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Step 2: Commit workflow**

```bash
git add .github/workflows/release-nightly.yml
git commit -m "ci: add nightly release workflow"
```

---

## Task 6: Create Stable Release Workflow

**Files:**
- Create: `.github/workflows/release-stable.yml`

**Step 1: Create stable release workflow**

Create: `.github/workflows/release-stable.yml`

```yaml
name: Stable Release

on:
  push:
    tags:
      - 'v*.*.*'

permissions:
  contents: write

jobs:
  release:
    name: Create Stable Release
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Extract version from tag
        id: version
        run: |
          TAG=${GITHUB_REF#refs/tags/v}
          echo "version=${TAG}" >> $GITHUB_OUTPUT
          echo "tag=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT
          echo "Stable version: ${TAG}"

      - name: Verify version matches tauri.conf.json
        run: |
          CONF_VERSION=$(jq -r '.version' src-tauri/tauri.conf.json)
          if [ "$CONF_VERSION" != "${{ steps.version.outputs.version }}" ]; then
            echo "ERROR: Tag version (${{ steps.version.outputs.version }}) does not match tauri.conf.json ($CONF_VERSION)"
            exit 1
          fi
          echo "✓ Version verified: $CONF_VERSION"

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            dpkg-dev

      - name: Install frontend dependencies
        run: npm ci

      - name: Run tests
        run: |
          npm run typecheck
          cd src-tauri && cargo test --lib

      - name: Build .deb packages
        run: npm run tauri build -- --bundles deb

      - name: Find .deb file
        id: deb
        run: |
          DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -n 1)
          echo "path=${DEB_FILE}" >> $GITHUB_OUTPUT
          echo "name=$(basename ${DEB_FILE})" >> $GITHUB_OUTPUT

      - name: Generate changelog
        id: changelog
        run: |
          # Get commits since last tag
          LAST_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
          if [ -n "$LAST_TAG" ]; then
            CHANGELOG=$(git log ${LAST_TAG}..HEAD --pretty=format:"- %s" --no-merges)
          else
            CHANGELOG=$(git log --pretty=format:"- %s" --no-merges -10)
          fi
          echo "changelog<<EOF" >> $GITHUB_OUTPUT
          echo "$CHANGELOG" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ steps.version.outputs.tag }}
          name: MuttonText ${{ steps.version.outputs.version }}
          body: |
            ## MuttonText ${{ steps.version.outputs.version }}

            ### Installation

            **Ubuntu/Debian:**
            ```bash
            wget https://github.com/Muminur/MuttonText/releases/download/${{ steps.version.outputs.tag }}/${{ steps.deb.outputs.name }}
            sudo dpkg -i ${{ steps.deb.outputs.name }}
            sudo apt-get install -f
            ```

            **APT Repository (Stable Channel):**
            ```bash
            echo "deb [trusted=yes] https://muminur.github.io/MuttonText stable main" | sudo tee /etc/apt/sources.list.d/muttontext.list
            sudo apt update
            sudo apt install muttontext
            ```

            ### Changes
            ${{ steps.changelog.outputs.changelog }}

            ### Full Changelog
            https://github.com/Muminur/MuttonText/compare/${{ github.event.before }}...${{ steps.version.outputs.tag }}
          files: ${{ steps.deb.outputs.path }}
          prerelease: false
          draft: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Setup APT repository
        run: |
          ./scripts/generate-apt-repo.sh stable \
            "${{ steps.deb.outputs.path }}" \
            apt-repo

      - name: Configure Git
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"

      - name: Deploy to GitHub Pages
        run: |
          # Fetch gh-pages
          if git ls-remote --exit-code --heads origin gh-pages; then
            git fetch origin gh-pages
            git checkout gh-pages
            git pull origin gh-pages
          else
            git checkout --orphan gh-pages
            git rm -rf . || true
          fi

          # Copy APT repo structure
          mkdir -p dists/stable pool
          cp -r apt-repo/dists/stable/* dists/stable/
          cp apt-repo/pool/* pool/

          # Commit and push
          git add -A
          git commit -m "chore: update stable APT repository - v${{ steps.version.outputs.version }}" || echo "No changes"
          git push origin gh-pages
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Step 2: Commit workflow**

```bash
git add .github/workflows/release-stable.yml
git commit -m "ci: add stable release workflow"
```

---

## Task 7: Create Installation Documentation

**Files:**
- Create: `docs/INSTALL.md`

**Step 1: Create comprehensive installation guide**

Create: `docs/INSTALL.md`

```markdown
# MuttonText Installation Guide

Complete installation instructions for Linux users.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation Methods](#installation-methods)
  - [Option 1: APT Repository (Recommended)](#option-1-apt-repository-recommended)
  - [Option 2: Manual .deb Installation](#option-2-manual-deb-installation)
  - [Option 3: Build from Source](#option-3-build-from-source)
- [Post-Installation Setup](#post-installation-setup)
- [Updating](#updating)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)
- [Verification](#verification)

---

## System Requirements

### Supported Distributions
- Ubuntu 20.04 LTS or later
- Debian 11 (Bullseye) or later
- Other Debian-based distributions (Linux Mint, Pop!_OS, etc.)

### Required Dependencies
The following dependencies are automatically installed with the .deb package:
- `libwebkit2gtk-4.1-0` - WebKit rendering engine
- `libgtk-3-0` - GTK+ 3 graphical toolkit
- `libayatana-appindicator3-1` - System tray support
- `xdotool` - Text expansion functionality

### Recommended System Specs
- **RAM:** 2GB minimum, 4GB recommended
- **Disk Space:** 100MB for application
- **Display:** 1024x768 minimum resolution

---

## Installation Methods

### Option 1: APT Repository (Recommended)

The APT repository provides automatic updates and easy installation.

#### Stable Channel (Recommended for Most Users)

The stable channel provides tested, production-ready releases:

```bash
# Add MuttonText stable repository
echo "deb [trusted=yes] https://muminur.github.io/MuttonText stable main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Update package list
sudo apt update

# Install MuttonText
sudo apt install muttontext

# Verify installation
muttontext --version
```

#### Nightly Channel (Latest Features)

The nightly channel provides cutting-edge features but may be less stable:

```bash
# Add MuttonText nightly repository
echo "deb [trusted=yes] https://muminur.github.io/MuttonText nightly main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Update package list
sudo apt update

# Install MuttonText
sudo apt install muttontext

# Verify installation
muttontext --version
```

**Note:** Using `[trusted=yes]` skips GPG verification. This is acceptable for trusted sources like GitHub Pages. Future versions may add GPG signing.

---

### Option 2: Manual .deb Installation

Download and install the .deb package directly.

#### Step 1: Download Latest Release

Visit the [Releases page](https://github.com/Muminur/MuttonText/releases) and download the latest `.deb` file.

Or use wget:

```bash
# Download latest stable release
wget https://github.com/Muminur/MuttonText/releases/latest/download/muttontext_0.1.0_amd64.deb
```

#### Step 2: Install Package

```bash
# Install the package
sudo dpkg -i muttontext_0.1.0_amd64.deb

# Install any missing dependencies
sudo apt-get install -f
```

#### Step 3: Verify Installation

```bash
# Check if installed
dpkg -l | grep muttontext

# Check version
muttontext --version
```

---

### Option 3: Build from Source

For developers or users who want to build from source.

#### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (v18 or later)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Tauri dependencies
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  xdotool
```

#### Build Steps

```bash
# Clone repository
git clone https://github.com/Muminur/MuttonText.git
cd MuttonText

# Install frontend dependencies
npm install

# Build release version
npm run tauri build

# Install the built .deb
sudo dpkg -i src-tauri/target/release/bundle/deb/muttontext_0.1.0_amd64.deb
```

---

## Post-Installation Setup

### First Launch

1. **Launch MuttonText:**
   ```bash
   muttontext
   ```
   Or search for "MuttonText" in your application menu.

2. **Initial Configuration:**
   - MuttonText will create a config directory at `~/.config/muttontext/`
   - Default settings are applied automatically

### Enable Autostart (Optional)

To start MuttonText automatically on login:

```bash
# Create autostart directory if it doesn't exist
mkdir -p ~/.config/autostart

# Create desktop entry
cat > ~/.config/autostart/muttontext.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=MuttonText
Exec=muttontext
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
EOF
```

### Test Text Expansion

1. Open any text editor
2. Create a simple expansion rule in MuttonText (e.g., `test` → `Hello from MuttonText`)
3. Type the trigger keyword in the text editor
4. Verify the expansion occurs

---

## Updating

### APT Repository Method

If you installed via APT repository, updates are automatic:

```bash
# Update package list
sudo apt update

# Upgrade MuttonText
sudo apt upgrade muttontext

# Or upgrade all packages
sudo apt upgrade
```

### Manual Method

1. Download the latest .deb from [Releases](https://github.com/Muminur/MuttonText/releases)
2. Install over existing version:
   ```bash
   sudo dpkg -i muttontext_NEW_VERSION_amd64.deb
   ```

### Switching Channels (Stable ↔ Nightly)

To switch between stable and nightly channels:

```bash
# Remove existing repository configuration
sudo rm /etc/apt/sources.list.d/muttontext.list

# Add desired channel (replace 'stable' with 'nightly' or vice versa)
echo "deb [trusted=yes] https://muminur.github.io/MuttonText stable main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Update and reinstall
sudo apt update
sudo apt install --reinstall muttontext
```

---

## Troubleshooting

### App Doesn't Launch

**Check Dependencies:**
```bash
# Verify all dependencies are installed
dpkg -s libwebkit2gtk-4.1-0
dpkg -s libgtk-3-0
dpkg -s libayatana-appindicator3-1
dpkg -s xdotool

# Reinstall if missing
sudo apt-get install -f
```

**Check Error Logs:**
```bash
# Launch from terminal to see errors
muttontext
```

### Text Expansion Not Working

**Verify xdotool is installed:**
```bash
which xdotool
# Should output: /usr/bin/xdotool

# If not installed
sudo apt-get install xdotool
```

**Check Window Exclusions:**
- MuttonText may be paused
- Check system tray icon for pause status
- Some applications (password managers) are auto-excluded

### Self-Expansion in MuttonText Window

MuttonText automatically excludes its own window from expansion. If you experience self-expansion:

1. Check if window detection is working
2. Verify the app is running with correct permissions
3. Report issue on [GitHub Issues](https://github.com/Muminur/MuttonText/issues)

### Performance Issues

**Check System Resources:**
```bash
# Monitor CPU/memory usage
top | grep muttontext
```

**Reduce Combo Count:**
- Large combo libraries may impact performance
- Consider organizing combos into groups and disabling unused groups

### Browser Expansion Issues

If expansions are being cut off in browsers:
- This is a known issue with browser DOM update timing
- MuttonText includes built-in delays to handle this
- If issues persist, try using the Clipboard paste method in preferences

---

## Uninstallation

### Remove Package

```bash
# Uninstall MuttonText
sudo apt remove muttontext

# Remove configuration files (optional)
rm -rf ~/.config/muttontext/

# Remove APT repository (optional)
sudo rm /etc/apt/sources.list.d/muttontext.list
```

### Remove Autostart Entry

```bash
rm ~/.config/autostart/muttontext.desktop
```

---

## Verification

### Check Installation Status

```bash
# Verify package is installed
dpkg -l | grep muttontext

# Should output something like:
# ii  muttontext  0.1.0  amd64  Free text expansion tool
```

### Check Version

```bash
muttontext --version

# Should output:
# MuttonText 0.1.0
```

### Verify Dependencies

```bash
# Check all dependencies
apt-cache depends muttontext

# Verify each is installed
dpkg -s libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1 xdotool
```

### Test Expansion

1. Launch MuttonText
2. Create test combo: `testmutton` → `MuttonText is working!`
3. Open any text editor
4. Type `testmutton` followed by space
5. Verify expansion occurs

---

## Getting Help

- **Documentation:** [GitHub README](https://github.com/Muminur/MuttonText)
- **Issues:** [GitHub Issues](https://github.com/Muminur/MuttonText/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Muminur/MuttonText/discussions)

---

**Last Updated:** 2026-02-08
```

**Step 2: Commit documentation**

```bash
git add docs/INSTALL.md
git commit -m "docs: add comprehensive installation guide"
```

---

## Task 8: Update README with Installation Section

**Files:**
- Modify: `README.md`

**Step 1: Read current README**

Run: `head -60 README.md`

Review current installation section.

**Step 2: Update README installation section**

Modify `README.md` - Replace the "Installation" section (around lines 31-50) with:

```markdown
## Installation

### Ubuntu/Debian Users

#### Option 1: APT Repository (Recommended)

**Stable Channel:**
```bash
# Add MuttonText stable repository
echo "deb [trusted=yes] https://muminur.github.io/MuttonText stable main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Install
sudo apt update
sudo apt install muttontext
```

**Nightly Channel** (bleeding edge):
```bash
# Add MuttonText nightly repository
echo "deb [trusted=yes] https://muminur.github.io/MuttonText nightly main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Install
sudo apt update
sudo apt install muttontext
```

#### Option 2: Manual .deb Installation

Download the latest `.deb` from [Releases](https://github.com/Muminur/MuttonText/releases):

```bash
# Download latest release
wget https://github.com/Muminur/MuttonText/releases/latest/download/muttontext_0.1.0_amd64.deb

# Install
sudo dpkg -i muttontext_0.1.0_amd64.deb
sudo apt-get install -f  # Install dependencies
```

#### Option 3: Build from Source

See [INSTALL.md](docs/INSTALL.md) for detailed build instructions.

### Complete Installation Guide

For comprehensive installation instructions, troubleshooting, and more, see [INSTALL.md](docs/INSTALL.md).
```

**Step 3: Add CI badges at top of README**

Add after the existing badges (around line 4):

```markdown
[![Build Status](https://github.com/Muminur/MuttonText/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/Muminur/MuttonText/actions/workflows/build-and-test.yml)
[![Latest Release](https://img.shields.io/github/v/release/Muminur/MuttonText)](https://github.com/Muminur/MuttonText/releases/latest)
```

**Step 4: Commit README updates**

```bash
git add README.md
git commit -m "docs: update README with APT repository installation"
```

---

## Task 9: Push All Changes and Trigger First Build

**Step 1: Review all changes**

Run:
```bash
git log --oneline -10
git status
```

**Step 2: Push to GitHub**

Run:
```bash
git push origin main
```

Expected: Triggers `build-and-test.yml` workflow

**Step 3: Monitor workflow execution**

Visit: https://github.com/Muminur/MuttonText/actions

Expected:
- Workflow runs successfully on Ubuntu 20.04, 22.04, 24.04
- Tests pass
- .deb packages are created

**Step 4: Wait for nightly release**

After build-and-test succeeds, `release-nightly.yml` triggers automatically.

Expected:
- Nightly release created with tag `nightly-YYYYMMDD`
- .deb attached to release
- APT repository updated on gh-pages

**Step 5: Verify GitHub Pages**

Enable GitHub Pages if not already:
1. Go to Settings → Pages
2. Source: Deploy from branch `gh-pages`
3. Save

Visit: https://muminur.github.io/MuttonText/

Expected: See APT repository structure

---

## Task 10: Test Installation from APT Repository

**Step 1: Add nightly repository**

Run on a test machine or VM:
```bash
echo "deb [trusted=yes] https://muminur.github.io/MuttonText nightly main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list
```

**Step 2: Update package list**

Run:
```bash
sudo apt update
```

Expected: No errors, MuttonText repository recognized

**Step 3: Install MuttonText**

Run:
```bash
sudo apt install muttontext
```

Expected:
- Package installs without errors
- All dependencies installed
- xdotool included

**Step 4: Verify installation**

Run:
```bash
dpkg -l | grep muttontext
which muttontext
muttontext --version
```

Expected: All commands succeed

**Step 5: Launch and test**

Run:
```bash
muttontext &
```

Expected:
- App launches
- Window appears
- Can create expansion rules
- Text expansion works

---

## Task 11: Create First Stable Release

**Step 1: Ensure main branch is clean**

Run:
```bash
git status
git log --oneline -5
```

**Step 2: Create stable release tag**

Run:
```bash
git tag v0.1.0
git push origin v0.1.0
```

Expected: Triggers `release-stable.yml` workflow

**Step 3: Monitor stable release workflow**

Visit: https://github.com/Muminur/MuttonText/actions

Expected:
- Workflow runs successfully
- Tests pass before release
- Stable GitHub Release created
- APT stable channel updated

**Step 4: Verify stable release**

Visit: https://github.com/Muminur/MuttonText/releases

Expected:
- Release v0.1.0 exists
- .deb file attached
- Release notes include installation instructions

**Step 5: Test stable channel installation**

Run on test machine:
```bash
sudo rm /etc/apt/sources.list.d/muttontext.list  # Remove nightly
echo "deb [trusted=yes] https://muminur.github.io/MuttonText stable main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list
sudo apt update
sudo apt install muttontext
```

Expected: Installs stable v0.1.0

---

## Task 12: Final Verification and Documentation

**Step 1: Test all installation methods**

Verify on clean Ubuntu VM:
1. APT stable channel ✓
2. APT nightly channel ✓
3. Manual .deb download and install ✓

**Step 2: Verify automatic updates work**

On VM with APT repository configured:
```bash
# Push a new commit to main (triggers nightly)
# Wait for nightly release
sudo apt update
sudo apt upgrade
```

Expected: Upgrade available and installs successfully

**Step 3: Create verification checklist document**

Create: `docs/DISTRIBUTION_CHECKLIST.md`

```markdown
# MuttonText Distribution Verification Checklist

Use this checklist to verify the distribution system is working correctly.

## CI/CD Verification

- [ ] Push to main triggers `build-and-test.yml`
- [ ] Tests run on Ubuntu 20.04, 22.04, 24.04
- [ ] .deb packages build successfully
- [ ] Docker installation tests pass
- [ ] Successful build triggers `release-nightly.yml`
- [ ] Nightly release created with correct version format
- [ ] GitHub Pages updated with APT repo structure

## APT Repository Verification

### Nightly Channel
- [ ] Repository accessible at https://muminur.github.io/MuttonText
- [ ] `dists/nightly/main/binary-amd64/Packages` exists
- [ ] Packages file contains correct package metadata
- [ ] `apt update` recognizes repository
- [ ] `apt install muttontext` installs from nightly channel

### Stable Channel
- [ ] Tag push triggers `release-stable.yml`
- [ ] Stable release created on GitHub
- [ ] `dists/stable/main/binary-amd64/Packages` exists
- [ ] `apt install muttontext` installs stable version

## Installation Testing

### APT Installation (Stable)
- [ ] Add repository command works
- [ ] `apt update` succeeds
- [ ] `apt install muttontext` succeeds
- [ ] All dependencies installed (xdotool, libwebkit2gtk, etc.)
- [ ] App launches successfully
- [ ] Text expansion works

### APT Installation (Nightly)
- [ ] Add nightly repository command works
- [ ] Nightly version installs
- [ ] Can switch between channels

### Manual .deb Installation
- [ ] .deb downloads from GitHub Releases
- [ ] `dpkg -i` installs package
- [ ] `apt-get install -f` resolves dependencies
- [ ] App works identically to APT installation

## Documentation Verification

- [ ] README.md has updated installation section
- [ ] README.md has CI/CD badges
- [ ] INSTALL.md exists and is comprehensive
- [ ] INSTALL.md covers all installation methods
- [ ] INSTALL.md includes troubleshooting section
- [ ] Links to releases work
- [ ] Installation commands are copy-pasteable

## Update Testing

- [ ] `apt update && apt upgrade` detects new versions
- [ ] Upgrade process preserves config files
- [ ] Upgrade process completes without errors
- [ ] App runs correctly after upgrade

## Uninstallation Testing

- [ ] `apt remove muttontext` removes package
- [ ] Config files remain (or removed based on preference)
- [ ] No broken dependencies left behind
- [ ] Can reinstall after uninstall

## Success Criteria

All checklist items must pass for a successful distribution setup.
```

**Step 4: Commit verification checklist**

```bash
git add docs/DISTRIBUTION_CHECKLIST.md
git commit -m "docs: add distribution verification checklist"
```

**Step 5: Final commit and push**

```bash
git push origin main
```

**Step 6: Update memory file**

Document what was accomplished in your auto memory at `/home/muminur/.claude/projects/-home-muminur-MuttonText/memory/MEMORY.md`

Add to session history:
```
## Session History (2026-02-08)
11. Implemented dual-channel .deb distribution system:
    - Added xdotool to .deb dependencies
    - Created Docker-based installation tests (Ubuntu 20.04, 22.04, 24.04)
    - Implemented three GitHub Actions workflows (build-and-test, release-nightly, release-stable)
    - Created APT repository generation scripts
    - Set up GitHub Pages for APT hosting (stable/nightly channels)
    - Wrote comprehensive INSTALL.md guide
    - Updated README with APT installation instructions
    - Tested full workflow: push → build → release → APT update
```

---

## Success Criteria

✅ **All criteria must be met:**

1. xdotool is declared as .deb dependency
2. Docker tests pass on Ubuntu 20.04, 22.04, 24.04
3. GitHub Actions workflows run successfully
4. Nightly builds trigger automatically on main push
5. Stable releases trigger on version tags
6. APT repository hosted on GitHub Pages
7. Both stable and nightly channels functional
8. Documentation is comprehensive and accurate
9. Installation via APT works end-to-end
10. Manual .deb installation works
11. Build artifacts uploaded to GitHub Releases
12. All tests pass in CI before release

---

## Notes

- **TDD Approach:** Tests created before implementation where possible
- **Incremental Commits:** Each task commits working changes
- **Documentation First:** Installation docs created alongside technical implementation
- **Real Testing:** Docker-based tests verify actual installation
- **GitHub Username:** Muminur (hardcoded in scripts, update if needed)
- **Version:** Current version is 0.1.0 (update tauri.conf.json for new releases)

## Dependencies on Skills

- **@tauri-testing** - For understanding Tauri testing best practices
- **@tauri-linux-packaging** - For .deb packaging specifics
- **@tauri-pipeline-github** - For GitHub Actions CI/CD setup

## Future Enhancements

After implementation, consider:
- GPG signing for APT repository
- Multi-architecture support (arm64)
- AppImage and Flatpak bundles
- Snap package
- AUR package for Arch Linux
- Auto-update functionality in app
- Code coverage reporting in CI
