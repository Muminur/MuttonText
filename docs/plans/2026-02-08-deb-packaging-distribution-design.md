# MuttonText .deb Packaging & Distribution Design

**Date:** 2026-02-08
**Status:** Approved

## Overview

Implement a dual-channel distribution system for MuttonText on Linux, providing both GitHub Releases for manual downloads and an APT repository for automated package management.

## Architecture

### Two-Tier Release System

#### 1. Nightly/Dev Builds (Automated)
- **Trigger:** Every push to main branch
- **Version Format:** `0.1.0-nightly-YYYYMMDD`
- **Distribution:**
  - GitHub Releases (marked as pre-release)
  - APT repository under `nightly` component
- **Automation:** Fully automated, no manual intervention

#### 2. Stable Releases (Manual)
- **Trigger:** Git tags matching `v*.*.*` (e.g., `v0.1.0`)
- **Version Format:** `0.1.0`, `0.2.0`, etc.
- **Distribution:**
  - GitHub Releases (official releases)
  - APT repository under `stable` component
- **Control:** Manual tag creation for controlled releases

### APT Repository Structure

Hosted on GitHub Pages at `https://<username>.github.io/muttontext/`

```
muttontext/
├── dists/
│   ├── stable/
│   │   └── main/
│   │       └── binary-amd64/
│   │           ├── Packages
│   │           └── Packages.gz
│   └── nightly/
│       └── main/
│           └── binary-amd64/
│               ├── Packages
│               └── Packages.gz
└── pool/
    ├── muttontext_0.1.0_amd64.deb
    ├── muttontext_0.1.0-nightly-20260208_amd64.deb
    └── ...
```

### User Configuration

**Stable Channel:**
```bash
echo "deb [trusted=yes] https://<username>.github.io/muttontext stable main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list
sudo apt update
sudo apt install muttontext
```

**Nightly Channel:**
```bash
echo "deb [trusted=yes] https://<username>.github.io/muttontext nightly main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list
sudo apt update
sudo apt install muttontext
```

## CI/CD Pipeline

### Workflow 1: `build-and-test.yml`

**Trigger:** Push to main branch

**Jobs:**
1. **Matrix Build & Test**
   - Ubuntu versions: 20.04, 22.04, 24.04
   - Steps per version:
     - Install dependencies
     - Run Rust unit tests: `cargo test --lib`
     - Run TypeScript type check: `npm run typecheck`
     - Build .deb package: `npm run tauri build`
     - Install .deb in clean Docker container
     - Run smoke tests:
       - Verify app launches
       - Verify window opens
       - Test basic text expansion
       - Verify config file creation

2. **Trigger Release**
   - On success: trigger `release-nightly.yml`
   - On failure: stop pipeline, notify

### Workflow 2: `release-nightly.yml`

**Trigger:** Successful completion of `build-and-test.yml`

**Steps:**
1. Generate nightly version: `0.1.0-nightly-YYYYMMDD`
2. Update `src-tauri/tauri.conf.json` version (temporary)
3. Build production .deb for all Ubuntu versions
4. Create GitHub pre-release:
   - Tag: `nightly-YYYYMMDD`
   - Attach all .deb files
   - Mark as pre-release
5. Update APT repository:
   - Checkout `gh-pages` branch
   - Copy .deb to `pool/`
   - Generate `dists/nightly/main/binary-amd64/Packages`
   - Compress to `Packages.gz`
   - Commit and push

### Workflow 3: `release-stable.yml`

**Trigger:** Git tag push matching `v*.*.*`

**Steps:**
1. Extract version from tag (e.g., `v0.1.1` → `0.1.1`)
2. Verify version matches `src-tauri/tauri.conf.json`
3. Build production .deb for all Ubuntu versions
4. Create GitHub Release:
   - Use tag as release name
   - Attach all .deb files
   - Mark as official release
   - Generate changelog from commits
5. Update APT repository:
   - Checkout `gh-pages` branch
   - Copy .deb to `pool/`
   - Generate `dists/stable/main/binary-amd64/Packages`
   - Compress to `Packages.gz`
   - Commit and push

## Testing Strategy (TDD)

### Test Pyramid

**Level 1: Unit Tests (Existing)**
- Rust backend: `cargo test --lib` (~580 tests)
- TypeScript frontend: type checking

**Level 2: Integration Tests (New)**
- Docker-based .deb installation tests
- Test on Ubuntu 20.04, 22.04, 24.04
- Verify:
  - Package installs without errors
  - Dependencies resolve correctly
  - App launches successfully
  - Window appears and is responsive
  - Config directory created (`~/.config/muttontext/`)

**Level 3: Smoke Tests (New)**
- Post-installation functional tests
- Test cases:
  - Create a simple text expansion rule
  - Trigger expansion in test window
  - Verify output matches expected
  - Test preferences save/load
  - Test app restart persists data

**Level 4: CI Tests (New)**
- Automated in GitHub Actions
- Run on every commit to main
- Block release on test failure
- Test matrix: Ubuntu 20.04, 22.04, 24.04

### TDD Implementation Order

1. **Write tests first** (before CI changes)
   - Create `src-tauri/tests/integration/deb_install.rs`
   - Create `tests/docker/` fixtures for each Ubuntu version
   - Mock installation and verification

2. **Implement CI workflows**
   - Start with basic build workflow
   - Add testing steps incrementally
   - Verify tests pass on existing code

3. **Add release automation**
   - Implement nightly builds
   - Test APT repository generation
   - Add stable release workflow

## Documentation

### README.md Updates

Add comprehensive "Installation on Linux" section:

#### Option 1: APT Repository (Recommended)

**Stable Channel** (recommended for most users):
```bash
# Add MuttonText repository
echo "deb [trusted=yes] https://<username>.github.io/muttontext stable main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Install
sudo apt update
sudo apt install muttontext
```

**Nightly Channel** (latest features, may be unstable):
```bash
# Add MuttonText nightly repository
echo "deb [trusted=yes] https://<username>.github.io/muttontext nightly main" | \
  sudo tee /etc/apt/sources.list.d/muttontext.list

# Install
sudo apt update
sudo apt install muttontext
```

#### Option 2: Manual .deb Installation

1. Download the latest .deb from [Releases](https://github.com/<username>/muttontext/releases)
2. Install:
   ```bash
   sudo dpkg -i muttontext_0.1.0_amd64.deb
   sudo apt-get install -f  # Install dependencies
   ```

#### Option 3: Build from Source

See [INSTALL.md](docs/INSTALL.md) for detailed build instructions.

### New Documentation: docs/INSTALL.md

Create comprehensive installation guide covering:

1. **System Requirements**
   - Supported Ubuntu versions (20.04+)
   - Required dependencies:
     - libwebkit2gtk-4.1-0
     - libgtk-3-0
     - libayatana-appindicator3-1
     - xdotool (for text expansion)
   - Recommended: 2GB RAM, 100MB disk space

2. **Installation Methods** (detailed steps with screenshots)
   - APT repository setup (stable vs nightly)
   - Manual .deb installation
   - Building from source

3. **Post-Installation Setup**
   - First launch configuration
   - Setting up autostart
   - Granting necessary permissions
   - Testing text expansion

4. **Troubleshooting**
   - App doesn't launch → check dependencies
   - Text expansion not working → verify xdotool installed
   - Self-expansion issues → check window exclusion
   - Performance issues → check system resources

5. **Updating**
   - Upgrading via APT: `sudo apt update && sudo apt upgrade`
   - Switching channels (stable ↔ nightly)
   - Manual update process

6. **Uninstallation**
   - Remove package: `sudo apt remove muttontext`
   - Remove config: `rm -rf ~/.config/muttontext/`
   - Remove APT source: `sudo rm /etc/apt/sources.list.d/muttontext.list`

7. **Verification**
   - Check version: `muttontext --version`
   - Verify installation: `dpkg -l | grep muttontext`
   - Test expansion: step-by-step guide

### Additional Documentation

- Add "Distribution" section to developer docs
- Document CI/CD release process for maintainers
- Add badges to README:
  - Latest release version
  - Build status
  - Test coverage (if available)

## Error Handling

### APT Repository Conflicts

**Issue:** User already has MuttonText repository configured
**Solution:**
- Check for existing entries before adding
- Provide clear error message with resolution steps
- Document channel switching process

**Issue:** User switches from stable to nightly or vice versa
**Solution:**
- Document proper channel switching:
  ```bash
  sudo rm /etc/apt/sources.list.d/muttontext.list
  # Add new channel configuration
  ```

### Build Failures

**Issue:** Tests fail in CI
**Solution:**
- CI stops before creating release
- GitHub Actions sends notification
- Clear error message in workflow logs
- Block merge if branch protection enabled

**Issue:** Build succeeds but package is broken
**Solution:**
- Smoke tests catch broken packages
- Installation tests verify basic functionality
- No release created on smoke test failure

### Dependency Issues

**Issue:** Missing runtime dependencies
**Solution:**
- .deb package declares all dependencies
- `apt` automatically installs dependencies
- Document manual dependency installation
- CI verifies dependencies on clean systems

**Issue:** xdotool not installed (required for expansion)
**Solution:**
- Add xdotool to .deb dependencies
- Update `tauri.conf.json` linux.deb.depends
- Verify in post-install smoke tests

### Version Conflicts

**Issue:** Nightly version blocks stable installation
**Solution:**
- APT priorities: stable > nightly
- Document version pinning if needed
- Clear upgrade path in docs

**Issue:** User has both stable and nightly configured
**Solution:**
- APT uses higher version by default
- Document best practice: use only one channel
- Provide channel switching instructions

## Implementation Checklist

### Phase 1: Testing Infrastructure (TDD)
- [ ] Create `tests/docker/` directory structure
- [ ] Write Dockerfile for Ubuntu 20.04 test environment
- [ ] Write Dockerfile for Ubuntu 22.04 test environment
- [ ] Write Dockerfile for Ubuntu 24.04 test environment
- [ ] Create `src-tauri/tests/integration/deb_install.rs`
- [ ] Write test: verify .deb installs without errors
- [ ] Write test: verify app launches
- [ ] Write test: verify config directory created
- [ ] Write test: verify basic text expansion works
- [ ] Run tests locally with Docker

### Phase 2: CI/CD Setup
- [ ] Create `.github/workflows/build-and-test.yml`
- [ ] Implement matrix build (Ubuntu 20.04, 22.04, 24.04)
- [ ] Add Rust test step: `cargo test --lib`
- [ ] Add TypeScript check step: `npm run typecheck`
- [ ] Add .deb build step: `npm run tauri build`
- [ ] Add Docker installation test step
- [ ] Add smoke test step
- [ ] Verify workflow runs successfully
- [ ] Create `.github/workflows/release-nightly.yml`
- [ ] Implement nightly version generation
- [ ] Add .deb build steps
- [ ] Add GitHub pre-release creation
- [ ] Create `.github/workflows/release-stable.yml`
- [ ] Implement stable release from tags
- [ ] Add GitHub release creation

### Phase 3: APT Repository
- [ ] Create `gh-pages` branch
- [ ] Initialize APT repository structure
- [ ] Write script: generate Packages index
- [ ] Write script: deploy to GitHub Pages
- [ ] Test stable channel configuration
- [ ] Test nightly channel configuration
- [ ] Update workflows to publish to APT repo
- [ ] Test end-to-end: commit → build → APT update

### Phase 4: Documentation
- [ ] Update README.md with Installation section
- [ ] Add APT repository instructions (stable)
- [ ] Add APT repository instructions (nightly)
- [ ] Add manual .deb installation instructions
- [ ] Add build-from-source reference
- [ ] Create `docs/INSTALL.md`
- [ ] Document system requirements
- [ ] Document all installation methods
- [ ] Add post-installation setup guide
- [ ] Add troubleshooting section
- [ ] Add update/upgrade instructions
- [ ] Add uninstallation instructions
- [ ] Add verification steps
- [ ] Add CI/CD badges to README
- [ ] Update any existing developer documentation

### Phase 5: Dependencies & Configuration
- [ ] Update `src-tauri/tauri.conf.json`
- [ ] Add xdotool to linux.deb.depends
- [ ] Verify all runtime dependencies listed
- [ ] Test .deb installs all dependencies correctly
- [ ] Update xdotool installation check in code

### Phase 6: Verification & Launch
- [ ] Test nightly build workflow end-to-end
- [ ] Test stable release workflow end-to-end
- [ ] Verify APT repository serves packages correctly
- [ ] Test installation from APT (stable channel)
- [ ] Test installation from APT (nightly channel)
- [ ] Test manual .deb installation
- [ ] Verify app works after installation
- [ ] Push all changes to GitHub
- [ ] Create initial stable release (v0.1.0)
- [ ] Announce availability to users

## Success Criteria

- [ ] Users can install MuttonText via APT repository
- [ ] Users can choose stable or nightly channel
- [ ] Every push to main creates nightly build automatically
- [ ] Manual tags create stable releases
- [ ] All builds tested on Ubuntu 20.04, 22.04, 24.04
- [ ] Documentation is comprehensive and clear
- [ ] Installation process is smooth and well-documented
- [ ] APT repository updates automatically
- [ ] GitHub has latest code and compiled releases

## Future Enhancements

- GPG signing for APT repository (enhanced security)
- Support for other Debian-based distros (Debian, Mint)
- AppImage and Flatpak distribution
- Auto-update mechanism within the app
- Release notes generation from commit messages
- Performance benchmarks in CI
- Code coverage reporting
