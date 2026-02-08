#!/usr/bin/env python3
"""
MuttonText installation and basic functionality tests.
Runs in headless mode using Xvfb - simplified smoke tests.
"""
import os
import subprocess
import time
import signal
import shutil

def test_binary_exists():
    """Test that muttontext binary is installed."""
    binary_path = shutil.which('muttontext')
    assert binary_path is not None, "muttontext binary not found in PATH"
    print(f"✓ Binary found at: {binary_path}")

def test_app_launches():
    """Test that MuttonText binary launches without immediate crashes."""
    process = None
    try:
        # Launch MuttonText in background
        process = subprocess.Popen(
            ['muttontext'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env={**os.environ, 'DISPLAY': os.environ.get('DISPLAY', ':99')}
        )

        # Wait for app to initialize
        time.sleep(3)

        # Check process is still running
        poll_result = process.poll()
        assert poll_result is None, f"MuttonText process terminated unexpectedly with code {poll_result}"
        print("✓ App launched successfully and is running")

    finally:
        # Cleanup
        if process and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()

def test_version_command():
    """Test that --version flag works (if implemented)."""
    try:
        result = subprocess.run(
            ['muttontext', '--version'],
            capture_output=True,
            timeout=5
        )
        # Don't fail if version flag doesn't exist yet
        if result.returncode == 0:
            print(f"✓ Version output: {result.stdout.decode().strip()}")
        else:
            print("⚠ Version flag not implemented yet")
    except (subprocess.TimeoutExpired, FileNotFoundError):
        print("⚠ Version command timed out or not available")

def test_help_command():
    """Test that --help flag works (if implemented)."""
    try:
        result = subprocess.run(
            ['muttontext', '--help'],
            capture_output=True,
            timeout=5
        )
        # Don't fail if help flag doesn't exist yet
        if result.returncode == 0:
            print("✓ Help command works")
        else:
            print("⚠ Help flag not implemented yet")
    except (subprocess.TimeoutExpired, FileNotFoundError):
        print("⚠ Help command timed out or not available")

if __name__ == '__main__':
    import pytest
    pytest.main([__file__, '-v', '-s'])
