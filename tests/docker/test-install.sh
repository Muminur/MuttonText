#!/bin/bash
# Test .deb installation in Docker container

set -e
set -u
set -o pipefail

UBUNTU_VERSION=${1:-20.04}

# Extract version from tauri.conf.json
VERSION=$(jq -r '.version' src-tauri/tauri.conf.json)

# Detect architecture (fallback to amd64 if dpkg not available)
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")

# Build dynamic .deb path
DEB_PATH=${2:-src-tauri/target/release/bundle/deb/muttontext_${VERSION}_${ARCH}.deb}

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
        set -u
        set -o pipefail
        echo '=== Installing .deb package ==='
        dpkg -i /test/muttontext.deb || apt-get install -f -y

        echo '=== Verifying installation ==='
        dpkg -l | grep muttontext

        echo '=== Checking dependencies ==='
        dpkg -s xdotool
        dpkg -s libwebkit2gtk-4.1-0

        echo \"✓ Installation test passed for Ubuntu ${UBUNTU_VERSION}\"
    "
