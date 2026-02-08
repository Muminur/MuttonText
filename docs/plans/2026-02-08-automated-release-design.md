# Automated Public Release System Design

**Date:** 2026-02-08
**Status:** Approved
**Goal:** Transform MuttonText into a publicly available project with fully automated releases

---

## Overview & Architecture

**Goal:** Transform MuttonText into a publicly available project with fully automated releases on every main branch push, using semantic versioning and installation verification tests.

**Core Components:**

1. **Unified CI/CD Pipeline** - Single GitHub Actions workflow that handles build → test → release in one flow
2. **Semantic Version Management** - Automatic version bumping based on conventional commit messages
3. **Installation Verification** - Automated tests that verify .deb and .rpm packages install and run correctly
4. **Release Artifact Publishing** - Automatic GitHub release creation with properly named packages

**Workflow Trigger:** Every push to the `main` branch initiates the complete pipeline. If any stage fails, the release is not created.

**Key Principles:**
- **Fail-fast:** Tests must pass before release creation
- **Traceability:** Every release maps to a semantic version and commit
- **Consistency:** Package naming matches Tauri defaults, README updated to reflect reality
- **Accessibility:** Public repository with easy download links

**Success Criteria:**
- Repository is public and accessible
- Push to main → automatic version bump → packages built → tests pass → release published
- Installation commands in README work for end users
- No manual intervention needed for standard releases

---

## Version Management

**Conventional Commits Standard:**

The system will parse commit messages to determine version bumps:
- `feat: add combo picker hotkey` → Minor version bump (0.1.0 → 0.2.0)
- `fix: browser backspace timing` → Patch version bump (0.1.0 → 0.1.1)
- `feat!: redesign preferences API` or `BREAKING CHANGE:` → Major version bump (0.1.0 → 1.0.0)
- `chore:`, `docs:`, `style:` → No version bump, no release

**Implementation Approach:**

We'll use a GitHub Action that:
1. Reads the current version from `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`
2. Analyzes commits since the last release using conventional commit parsing
3. Calculates the new version (major.minor.patch)
4. Updates version files automatically
5. Creates a git tag (e.g., `v0.1.1`)
6. Commits the version bump back to main

**Version Synchronization:**

Three files must stay in sync:
- `src-tauri/tauri.conf.json` → `version` field
- `src-tauri/Cargo.toml` → `version` field
- `package.json` → `version` field

The automation handles all three, preventing drift.

**Initial State:** Since we're starting at `v0.1.0` and creating the first automated release, the system will begin tracking from this baseline.

---

## CI/CD Pipeline Structure

**Single Unified Workflow** (`.github/workflows/ci-cd.yml`):

### Job 1: Version & Build
- Triggers on push to `main` branch
- Analyzes commits for version bump calculation
- Updates version in all config files
- Runs `npm install` and `npm run tauri build`
- Builds .deb and .rpm packages (skips AppImage)
- Uploads build artifacts for downstream jobs

### Job 2: Installation Testing
- Depends on Job 1 completing successfully
- Runs in parallel for .deb and .rpm
- Uses Ubuntu container for .deb testing
- Uses Fedora container for .rpm testing
- Tests: Install package → Launch app → Verify process starts → Clean shutdown
- Validates dependencies are correctly declared
- Fails pipeline if installation or launch fails

### Job 3: Create Release
- Depends on Jobs 1 & 2 both passing
- Only runs on main branch (double-check protection)
- Creates GitHub release using the calculated version tag
- Uploads `.deb` and `.rpm` as release assets
- Auto-generates release notes from commits
- Marks release as "latest"

**Caching Strategy:** Reuse existing Rust and npm cache configuration from current CI to speed up builds.

**Runtime:** Estimated 5-8 minutes per full pipeline run.

---

## Installation Testing

**Test Strategy:** Full GUI interaction testing using headless display servers and WebDriver.

### Debian/Ubuntu Test (.deb)
```yaml
- Container: ubuntu:22.04
- Setup: Install Xvfb (virtual framebuffer for headless GUI)
- Install: dpkg -i MuttonText_X.Y.Z_amd64.deb
- Dependency check: apt-get install -f
- Launch: Start app with Xvfb display
- GUI Tests:
  ✓ App window opens successfully
  ✓ Create a test combo (keyword: "test", snippet: "Hello Test")
  ✓ Verify combo appears in the list
  ✓ Test expansion in a text field
  ✓ Open preferences, verify settings load
  ✓ System tray icon is accessible
  ✓ Clean shutdown via UI
```

### Fedora/RHEL Test (.rpm)
```yaml
- Container: fedora:latest with Xvfb
- Same GUI test suite as Debian
```

### Technical Implementation
- Use **Tauri WebDriver** with Selenium for browser automation
- Xvfb provides virtual display (`:99`) for headless execution
- WebDriver connects to MuttonText's webview for UI automation
- Test scripts verify core user workflows work end-to-end

### Test Coverage
1. Installation & launch
2. Core functionality (create/edit/delete combos)
3. Text expansion mechanics
4. Preferences loading
5. System tray integration

---

## README Updates

**Package Naming Corrections:**

Update all download links in README.md to match Tauri's actual build outputs:

**Before (incorrect):**
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/mutton-text_0.1.0_amd64.deb
sudo dpkg -i mutton-text_0.1.0_amd64.deb
```

**After (correct):**
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.1.0_amd64.deb
sudo dpkg -i MuttonText_0.1.0_amd64.deb
```

**Changes Required:**
- Line 38: Update .deb filename (lowercase → MuttonText with capitals)
- Line 41: Update dpkg command with correct filename
- Line 58: Update .rpm filename (mutton-text → MuttonText)
- Line 61: Update rpm command with correct filename
- Line 50: Update AppImage filename (keep for documentation, note "coming soon")
- Line 75: Update uninstall command (package name stays `mutton-text` - that's the Debian package name, different from the filename)

**Version Placeholders:**

Change hardcoded versions to dynamic or keep version numbers and update them via automation.

**Additional Improvements:**
- Add badge showing latest release version
- Add "automated releases" note in Quick Start section

---

## Repository Settings & Going Public

**Making Repository Public:**

```bash
gh repo edit Muminur/MuttonText --visibility public
```

This single command transitions the repository from private to public, making:
- Source code visible to everyone
- Releases downloadable without authentication
- README installation instructions functional
- Contribution and community engagement possible

### Required GitHub Settings

**1. Branch Protection (main):**
- Require status checks to pass (CI/CD must succeed)
- Require conventional commit format (optional but recommended)
- Prevent force pushes
- Prevent deletions

**2. Actions Permissions:**
- Allow GitHub Actions to create releases
- Enable workflow write permissions
- Grant `contents: write` and `packages: write` permissions

**3. Release Settings:**
- Enable automatic release notes generation
- Set release discussion category (optional)
- Configure asset retention policy

### Timing Consideration

The repository should go public **after** the first automated release is successfully created and tested. This ensures:
- The public sees a working installation experience immediately
- CI/CD is battle-tested in private mode first
- No embarrassing failed releases visible

**Recommended Sequence:**
1. Implement all changes in private mode
2. Test automated release creation (push to main)
3. Verify packages install correctly
4. Then run `gh repo edit --visibility public`

---

## Implementation Summary

### What We're Building

A fully automated, public release system for MuttonText that:
- ✅ Auto-bumps versions using conventional commits
- ✅ Builds .deb and .rpm packages on every main push
- ✅ Runs comprehensive GUI installation tests in containers
- ✅ Creates GitHub releases automatically when tests pass
- ✅ Uses correct Tauri-generated package names
- ✅ Makes repository publicly accessible

### Key Files to Modify
1. `.github/workflows/ci-cd.yml` - Unified pipeline (extend existing or replace)
2. `README.md` - Update package filenames and installation commands
3. GitHub repository settings - Make public, configure Actions permissions

### Dependencies Needed
- GitHub Actions for version bumping (e.g., `mathieudutour/github-tag-action`)
- WebDriver dependencies for GUI testing (Selenium, Xvfb)
- Tauri WebDriver bindings for test automation

### Risk Mitigation
- Test in private mode first before going public
- Branch protection prevents broken releases
- Installation tests catch packaging issues before users see them

### Timeline Estimate
- Implementation: 2-3 hours
- Testing & validation: 1 hour
- First automated release: ~10 minutes
- Go public: 1 command

---

## Package Scope

**v0.1.0 Release Includes:**
- ✅ `.deb` package (Debian/Ubuntu)
- ✅ `.rpm` package (Fedora/RHEL)
- ❌ AppImage (deferred to v0.1.1 - requires linuxdeploy setup)

This covers the majority of Linux desktop users while allowing for a faster initial release.
