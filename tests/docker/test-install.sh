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
