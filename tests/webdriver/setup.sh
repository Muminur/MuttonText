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
