#!/usr/bin/env bash
set -euo pipefail

# scripts/release.sh - Cut and push a new release for ramwise

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory has uncommitted changes. Please commit or stash them first." >&2
    exit 1
fi

CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "Warning: You are on branch '$CURRENT_BRANCH', not 'main'."
    read -rp "Do you want to continue? [y/N]: " CONFIRM
    if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

CURRENT_VER="$(grep '^version =' Cargo.toml | head -1 | cut -d'"' -f2)"
echo "Current version in Cargo.toml: $CURRENT_VER"

if [ $# -ge 1 ]; then
    NEW_VER="$1"
else
    read -rp "Enter new version (e.g. 0.1.0): " NEW_VER
fi

# Strip leading 'v' if provided
NEW_VER="${NEW_VER#v}"

if [ -z "$NEW_VER" ]; then
    echo "Error: Version cannot be empty." >&2
    exit 1
fi

TAG_NAME="v$NEW_VER"

if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo "Error: Git tag $TAG_NAME already exists." >&2
    exit 1
fi

echo "Updating Cargo.toml version to $NEW_VER..."
sed -i "s/^version = \".*\"/version = \"$NEW_VER\"/" Cargo.toml

echo "Updating Cargo.lock..."
cargo check --quiet

echo "Updating AUR PKGBUILD version..."
sed -i "s/^pkgver=.*/pkgver=$NEW_VER/" aur/ramwise/PKGBUILD
sed -i "s/^pkgrel=.*/pkgrel=1/" aur/ramwise/PKGBUILD
sed -i "s/^pkgver=.*/pkgver=$NEW_VER/" aur/ramwise-bin/PKGBUILD
sed -i "s/^pkgrel=.*/pkgrel=1/" aur/ramwise-bin/PKGBUILD

git add Cargo.toml Cargo.lock aur/ramwise/PKGBUILD aur/ramwise-bin/PKGBUILD
git commit -m "chore(release): $TAG_NAME"
git tag -a "$TAG_NAME" -m "Release $TAG_NAME"

echo ""
echo "Successfully created release commit and tag: $TAG_NAME"
echo ""
echo "To publish this release to GitHub and trigger automated binary builds:"
echo "    git push origin $CURRENT_BRANCH"
echo "    git push origin $TAG_NAME"
echo ""
read -rp "Push to origin now? [y/N]: " PUSH_CONFIRM
if [[ "$PUSH_CONFIRM" =~ ^[Yy]$ ]]; then
    git push origin "$CURRENT_BRANCH"
    git push origin "$TAG_NAME"
    echo "Pushed to GitHub! Check release build status at:"
    echo "https://github.com/Duckaet/ramwise/actions"
fi
