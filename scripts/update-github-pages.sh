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

# Verify we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "ERROR: Not in a git repository"
    exit 1
fi

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo "ERROR: Working tree has uncommitted changes. Commit or stash them first."
    exit 1
fi

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
