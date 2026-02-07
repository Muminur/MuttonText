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
