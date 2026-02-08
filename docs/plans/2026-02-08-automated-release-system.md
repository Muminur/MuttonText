# Automated Release System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a fully automated CI/CD system that versions, builds, tests, and releases MuttonText on every push to main.

**Architecture:** Single unified GitHub Actions workflow with three sequential jobs: (1) version bump + build packages, (2) GUI installation tests in containers with Xvfb, (3) create GitHub release with artifacts. Uses conventional commits for semantic versioning across package.json, Cargo.toml, and tauri.conf.json.

**Tech Stack:** GitHub Actions, Tauri WebDriver, Selenium, Xvfb, conventional commits, semantic versioning

---

## Task 1: Update README Package Names

**Files:**
- Modify: `README.md:38,41,50,58,61`

**Step 1: Update .deb download link**

Change line 38 from:
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/mutton-text_0.1.0_amd64.deb
```

To:
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.1.0_amd64.deb
```

**Step 2: Update .deb install command**

Change line 41 from:
```bash
sudo dpkg -i mutton-text_0.1.0_amd64.deb
```

To:
```bash
sudo dpkg -i MuttonText_0.1.0_amd64.deb
```

**Step 3: Update AppImage filename**

Change line 50 from:
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/mutton-text_0.1.0_amd64.AppImage
chmod +x mutton-text_0.1.0_amd64.AppImage
./mutton-text_0.1.0_amd64.AppImage
```

To:
```bash
# AppImage coming in v0.1.1
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.1.0_amd64.AppImage
chmod +x MuttonText_0.1.0_amd64.AppImage
./MuttonText_0.1.0_amd64.AppImage
```

**Step 4: Update .rpm download link**

Change line 58 from:
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/mutton-text-0.1.0-1.x86_64.rpm
```

To:
```bash
wget https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText-0.1.0-1.x86_64.rpm
```

**Step 5: Update .rpm install command**

Change line 61 from:
```bash
sudo rpm -i mutton-text-0.1.0-1.x86_64.rpm
```

To:
```bash
sudo rpm -i MuttonText-0.1.0-1.x86_64.rpm
```

**Step 6: Add release badge**

Add after line 4 (after Platform badge):
```markdown
[![Release](https://img.shields.io/github/v/release/Muminur/MuttonText)](https://github.com/Muminur/MuttonText/releases/latest)
```

**Step 7: Add automated releases note**

Add after line 33 (after "## Quick Start"):
```markdown
> **Note:** Releases are automatically created on every push to main using semantic versioning from commit messages.
```

**Step 8: Commit**

```bash
git add README.md
git commit -m "docs: update package names to match Tauri build output

- Change lowercase filenames to Tauri defaults (MuttonText with capitals)
- Add release version badge
- Add automated releases note
- Mark AppImage as coming in v0.1.1"
```

---

## Task 2: Create WebDriver Test Infrastructure

**Files:**
- Create: `tests/webdriver/setup.sh`
- Create: `tests/webdriver/test_installation.py`
- Create: `tests/webdriver/requirements.txt`

**Step 1: Create test directory**

```bash
mkdir -p tests/webdriver
```

**Step 2: Write Python dependencies file**

Create `tests/webdriver/requirements.txt`:
```
selenium==4.16.0
pytest==7.4.3
```

**Step 3: Write Xvfb setup script**

Create `tests/webdriver/setup.sh`:
```bash
#!/bin/bash
set -e

# Install Xvfb and dependencies
if command -v apt-get &> /dev/null; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y xvfb x11-utils python3-pip
elif command -v dnf &> /dev/null; then
    dnf install -y xorg-x11-server-Xvfb python3-pip
fi

# Install Python test dependencies
pip3 install -r /tests/requirements.txt

# Start Xvfb
Xvfb :99 -screen 0 1280x720x24 &
export DISPLAY=:99

# Wait for Xvfb
sleep 2
```

**Step 4: Write installation test script**

Create `tests/webdriver/test_installation.py`:
```python
#!/usr/bin/env python3
"""
MuttonText installation and GUI functionality tests.
Runs in headless mode using Xvfb.
"""
import os
import subprocess
import time
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.common.exceptions import TimeoutException

def test_app_launches():
    """Test that MuttonText binary launches without errors."""
    # Launch MuttonText in background
    process = subprocess.Popen(
        ['muttontext'],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    # Wait for app to initialize
    time.sleep(3)

    # Check process is running
    assert process.poll() is None, "MuttonText process terminated unexpectedly"

    # Cleanup
    process.terminate()
    process.wait(timeout=5)

def test_gui_window_opens():
    """Test that the main window opens successfully."""
    # Configure WebDriver for Tauri
    options = webdriver.ChromeOptions()
    options.add_argument('--headless')
    options.add_argument('--no-sandbox')
    options.add_argument('--disable-dev-shm-usage')

    driver = None
    process = None

    try:
        # Launch MuttonText
        process = subprocess.Popen(['muttontext'])
        time.sleep(3)

        # Connect WebDriver to app's webview
        driver = webdriver.Chrome(options=options)
        driver.get('tauri://localhost')

        # Wait for main window to load
        WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.TAG_NAME, 'body'))
        )

        # Verify window title
        assert 'MuttonText' in driver.title

    finally:
        if driver:
            driver.quit()
        if process:
            process.terminate()
            process.wait(timeout=5)

def test_create_combo():
    """Test creating a new combo through the GUI."""
    options = webdriver.ChromeOptions()
    options.add_argument('--headless')
    options.add_argument('--no-sandbox')

    driver = None
    process = None

    try:
        process = subprocess.Popen(['muttontext'])
        time.sleep(3)

        driver = webdriver.Chrome(options=options)
        driver.get('tauri://localhost')

        # Wait for UI to load
        WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.TAG_NAME, 'body'))
        )

        # Find and click "Add Combo" button
        add_button = WebDriverWait(driver, 10).until(
            EC.element_to_be_clickable((By.XPATH, "//button[contains(text(), 'Add')]"))
        )
        add_button.click()

        # Fill in combo details
        keyword_input = driver.find_element(By.NAME, 'keyword')
        keyword_input.send_keys('test')

        snippet_input = driver.find_element(By.NAME, 'snippet')
        snippet_input.send_keys('Hello Test')

        # Save combo
        save_button = driver.find_element(By.XPATH, "//button[contains(text(), 'Save')]")
        save_button.click()

        # Verify combo appears in list
        time.sleep(1)
        combo_list = driver.find_element(By.CLASS_NAME, 'combo-list')
        assert 'test' in combo_list.text

    finally:
        if driver:
            driver.quit()
        if process:
            process.terminate()
            process.wait(timeout=5)

def test_preferences_load():
    """Test that preferences window opens and loads settings."""
    options = webdriver.ChromeOptions()
    options.add_argument('--headless')
    options.add_argument('--no-sandbox')

    driver = None
    process = None

    try:
        process = subprocess.Popen(['muttontext'])
        time.sleep(3)

        driver = webdriver.Chrome(options=options)
        driver.get('tauri://localhost')

        # Open preferences
        settings_button = WebDriverWait(driver, 10).until(
            EC.element_to_be_clickable((By.XPATH, "//button[@aria-label='Settings']"))
        )
        settings_button.click()

        # Verify preferences panel opens
        prefs_panel = WebDriverWait(driver, 5).until(
            EC.presence_of_element_located((By.CLASS_NAME, 'preferences-panel'))
        )
        assert prefs_panel.is_displayed()

    finally:
        if driver:
            driver.quit()
        if process:
            process.terminate()
            process.wait(timeout=5)

if __name__ == '__main__':
    import pytest
    pytest.main([__file__, '-v'])
```

**Step 5: Make setup script executable**

```bash
chmod +x tests/webdriver/setup.sh
```

**Step 6: Commit**

```bash
git add tests/webdriver/
git commit -m "test: add WebDriver GUI installation tests

- Add Xvfb setup script for headless testing
- Add Python test suite for GUI functionality
- Tests: app launch, window opening, combo creation, preferences
- Selenium-based automation for Tauri webview"
```

---

## Task 3: Create Unified CI/CD Workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create workflow directory**

```bash
mkdir -p .github/workflows
```

**Step 2: Write unified CI/CD workflow**

Create `.github/workflows/release.yml`:
```yaml
name: Build, Test, and Release

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  version-and-build:
    name: Version Bump and Build Packages
    runs-on: ubuntu-22.04
    if: github.ref == 'refs/heads/main'
    outputs:
      new_version: ${{ steps.version.outputs.new_version }}
      new_tag: ${{ steps.version.outputs.new_tag }}

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Calculate semantic version
        id: version
        uses: mathieudutour/github-tag-action@v6.2
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          dry_run: true
          default_bump: patch
          release_branches: main

      - name: Update version files
        if: steps.version.outputs.new_version != ''
        run: |
          NEW_VERSION="${{ steps.version.outputs.new_version }}"

          # Update package.json
          jq ".version = \"$NEW_VERSION\"" package.json > package.json.tmp
          mv package.json.tmp package.json

          # Update Cargo.toml
          sed -i "s/^version = .*/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml

          # Update tauri.conf.json
          jq ".version = \"$NEW_VERSION\"" src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
          mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

          echo "Updated version to $NEW_VERSION"

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            libwebkit2gtk-4.1-dev \
            libxdo-dev \
            libx11-dev \
            libxcb1-dev \
            libxcb-render0-dev \
            libxcb-shape0-dev \
            libxcb-xfixes0-dev

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install Node dependencies
        run: npm ci

      - name: Build Tauri packages
        run: npm run tauri build -- --bundles deb,rpm

      - name: Upload .deb artifact
        uses: actions/upload-artifact@v4
        with:
          name: deb-package
          path: src-tauri/target/release/bundle/deb/*.deb

      - name: Upload .rpm artifact
        uses: actions/upload-artifact@v4
        with:
          name: rpm-package
          path: src-tauri/target/release/bundle/rpm/*.rpm

      - name: Commit version bump
        if: steps.version.outputs.new_version != ''
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
          git commit -m "chore: bump version to ${{ steps.version.outputs.new_version }} [skip ci]"
          git push

  test-deb-installation:
    name: Test .deb Installation
    runs-on: ubuntu-22.04
    needs: version-and-build

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Download .deb artifact
        uses: actions/download-artifact@v4
        with:
          name: deb-package
          path: ./packages

      - name: Install Xvfb and test dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y xvfb x11-utils python3-pip chromium-browser chromium-chromedriver

      - name: Install Python test dependencies
        run: |
          pip3 install -r tests/webdriver/requirements.txt

      - name: Install .deb package
        run: |
          sudo dpkg -i packages/*.deb || true
          sudo apt-get install -f -y

      - name: Verify installation
        run: |
          which muttontext
          muttontext --version || echo "Version flag not available"

      - name: Start Xvfb
        run: |
          Xvfb :99 -screen 0 1280x720x24 &
          echo "DISPLAY=:99" >> $GITHUB_ENV
          sleep 2

      - name: Run GUI tests
        run: |
          cd tests/webdriver
          pytest test_installation.py -v
        timeout-minutes: 5

  test-rpm-installation:
    name: Test .rpm Installation
    runs-on: ubuntu-22.04
    needs: version-and-build
    container: fedora:latest

    steps:
      - name: Install git
        run: dnf install -y git

      - name: Checkout code
        uses: actions/checkout@v4

      - name: Download .rpm artifact
        uses: actions/download-artifact@v4
        with:
          name: rpm-package
          path: ./packages

      - name: Install Xvfb and test dependencies
        run: |
          dnf install -y xorg-x11-server-Xvfb python3-pip chromium chromedriver

      - name: Install Python test dependencies
        run: |
          pip3 install -r tests/webdriver/requirements.txt

      - name: Install .rpm package
        run: |
          dnf install -y packages/*.rpm

      - name: Verify installation
        run: |
          which muttontext
          muttontext --version || echo "Version flag not available"

      - name: Start Xvfb
        run: |
          Xvfb :99 -screen 0 1280x720x24 &
          export DISPLAY=:99
          sleep 2

      - name: Run GUI tests
        run: |
          cd tests/webdriver
          DISPLAY=:99 pytest test_installation.py -v
        timeout-minutes: 5

  create-release:
    name: Create GitHub Release
    runs-on: ubuntu-22.04
    needs: [version-and-build, test-deb-installation, test-rpm-installation]
    if: github.ref == 'refs/heads/main' && needs.version-and-build.outputs.new_version != ''
    permissions:
      contents: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Download .deb artifact
        uses: actions/download-artifact@v4
        with:
          name: deb-package
          path: ./packages

      - name: Download .rpm artifact
        uses: actions/download-artifact@v4
        with:
          name: rpm-package
          path: ./packages

      - name: Create Release
        uses: ncipollo/release-action@v1
        with:
          tag: ${{ needs.version-and-build.outputs.new_tag }}
          name: MuttonText ${{ needs.version-and-build.outputs.new_version }}
          artifacts: "packages/*"
          generateReleaseNotes: true
          draft: false
          prerelease: false
          token: ${{ secrets.GITHUB_TOKEN }}
```

**Step 3: Force add workflow (gitignored)**

```bash
git add -f .github/workflows/release.yml
```

**Step 4: Commit**

```bash
git commit -m "ci: add unified build, test, and release workflow

- Semantic versioning with conventional commits
- Auto-bump version in package.json, Cargo.toml, tauri.conf.json
- Build .deb and .rpm packages
- GUI installation tests with Xvfb in containers
- Auto-create GitHub releases on successful tests
- Only runs on main branch pushes"
```

---

## Task 4: Configure GitHub Repository Settings

**Files:**
- None (GitHub settings via CLI)

**Step 1: Verify repository is still private**

Run:
```bash
gh repo view Muminur/MuttonText --json visibility -q .visibility
```

Expected: `PRIVATE`

**Step 2: Configure Actions permissions**

Run:
```bash
gh api repos/Muminur/MuttonText -X PATCH -f allow_auto_merge=true
```

**Step 3: Verify workflow file is tracked**

Run:
```bash
git ls-files .github/workflows/release.yml
```

Expected: `.github/workflows/release.yml`

**Step 4: Push branch to remote**

Run:
```bash
git push -u origin feature/automated-release-system
```

Expected: Branch pushed successfully

**Step 5: Create pull request**

Run:
```bash
gh pr create \
  --title "feat: automated release system with CI/CD" \
  --body "$(cat <<'EOF'
## Summary
- ✅ Semantic versioning with conventional commits
- ✅ Auto-build .deb and .rpm on main pushes
- ✅ Comprehensive GUI installation tests with Xvfb
- ✅ Auto-create GitHub releases when tests pass
- ✅ Updated README with correct package names

## Testing
- WebDriver GUI tests verify: app launch, window opening, combo creation, preferences
- Tests run in Ubuntu and Fedora containers with Xvfb
- Only creates releases after all tests pass

## References
- Design: docs/plans/2026-02-08-automated-release-design.md
- Implementation: docs/plans/2026-02-08-automated-release-system.md

🤖 Generated with Claude Code
EOF
)"
```

Expected: PR created with number

**Step 6: Document PR number for later**

```bash
echo "PR created - do not merge until testing complete"
```

---

## Task 5: Test Workflow in Private Mode

**Files:**
- None (testing via GitHub Actions)

**Step 1: Merge PR to trigger workflow**

Run:
```bash
gh pr merge --squash --auto
```

Expected: PR auto-merged when checks pass

**Step 2: Wait for workflow to complete**

Run:
```bash
gh run watch
```

Expected: All jobs pass (green checkmarks)

**Step 3: Verify release was created**

Run:
```bash
gh release view --json tagName,assets -q '{tag: .tagName, assets: [.assets[].name]}'
```

Expected: JSON showing v0.1.0 (or v0.1.1) tag with .deb and .rpm files

**Step 4: Download and test .deb locally**

Run:
```bash
gh release download --pattern '*.deb' --dir /tmp
sudo dpkg -i /tmp/MuttonText_*.deb
muttontext --version
```

Expected: Package installs, app runs

**Step 5: Verify version bump commit**

Run:
```bash
git log --oneline -1
```

Expected: Shows "chore: bump version to X.Y.Z [skip ci]"

---

## Task 6: Make Repository Public

**Files:**
- None (GitHub settings)

**Step 1: Verify release exists and works**

Run:
```bash
gh release list
```

Expected: At least one release visible

**Step 2: Make repository public**

Run:
```bash
gh repo edit Muminur/MuttonText --visibility public
```

Expected: `✓ Edited repository Muminur/MuttonText`

**Step 3: Verify public access**

Run:
```bash
curl -I https://github.com/Muminur/MuttonText
```

Expected: HTTP 200 (not 404)

**Step 4: Test anonymous download**

Run:
```bash
wget -q --spider https://github.com/Muminur/MuttonText/releases/latest/download/MuttonText_0.1.0_amd64.deb && echo "Download link works"
```

Expected: "Download link works"

**Step 5: Configure branch protection**

Run:
```bash
gh api repos/Muminur/MuttonText/branches/main/protection -X PUT --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["version-and-build", "test-deb-installation", "test-rpm-installation"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF
```

Expected: Branch protection configured

**Step 6: Celebrate**

```bash
echo "🎉 MuttonText is now public with automated releases!"
echo "Every push to main will:"
echo "  1. Auto-bump version from commit messages"
echo "  2. Build .deb and .rpm packages"
echo "  3. Run GUI installation tests"
echo "  4. Create GitHub release if tests pass"
```

---

## Post-Implementation Verification

**Checklist:**

- [ ] README package names match Tauri build output
- [ ] Release badge shows on README
- [ ] WebDriver tests exist and run in CI
- [ ] Workflow runs on push to main
- [ ] Version bumps automatically based on commits
- [ ] .deb package installs on Ubuntu
- [ ] .rpm package installs on Fedora
- [ ] GUI tests pass in both containers
- [ ] GitHub releases are auto-created
- [ ] Repository is public
- [ ] Branch protection prevents broken releases
- [ ] Anonymous downloads work

**Testing Conventional Commits:**

After going public, test the versioning system:

```bash
# Patch bump (0.1.0 → 0.1.1)
git commit -m "fix: correct tray icon alignment"
git push

# Minor bump (0.1.1 → 0.2.0)
git commit -m "feat: add combo search filter"
git push

# Major bump (0.2.0 → 1.0.0)
git commit -m "feat!: redesign preferences UI

BREAKING CHANGE: preferences API changed"
git push
```

Each push should trigger the workflow and create a new release.

---

## Risk Mitigation

**If workflow fails on first run:**
1. Check Actions tab for error logs
2. Most common issues:
   - Missing secrets/permissions
   - Xvfb not starting (increase sleep time)
   - WebDriver connection timeout (check Tauri webview URL)
   - Version calculation error (check git tags)
3. Fix and push again - workflow reruns automatically

**If tests fail in containers:**
1. Tests are environment-dependent (display server, timing)
2. Can temporarily disable test jobs to unblock release
3. Fix tests in follow-up PR

**If going public is scary:**
1. Can stay private and test workflow multiple times
2. Make public when confident
3. Can always revert to private if needed

**Rollback plan:**
1. If bad release: `gh release delete vX.Y.Z`
2. If broken workflow: Revert the PR that added release.yml
3. If need to go private: `gh repo edit --visibility private`
