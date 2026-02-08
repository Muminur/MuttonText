#!/bin/bash
# Verify xdotool is declared as dependency in tauri.conf.json

set -e

if ! grep -q '"xdotool"' src-tauri/tauri.conf.json; then
    echo "ERROR: xdotool not found in deb dependencies"
    exit 1
fi

echo "✓ xdotool dependency verified"
