#!/usr/bin/env bash
set -euo pipefail

# scripts/publish-aur.sh - Publish or update package on AUR (Arch User Repository)

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKGNAME="${1:-ramwise}"

if [ "$PKGNAME" != "ramwise" ] && [ "$PKGNAME" != "ramwise-bin" ]; then
    echo "Usage: $0 [ramwise|ramwise-bin]"
    exit 1
fi

PKGBUILD_SOURCE="$REPO_DIR/aur/$PKGNAME/PKGBUILD"
if [ ! -f "$PKGBUILD_SOURCE" ]; then
    echo "Error: PKGBUILD not found at $PKGBUILD_SOURCE" >&2
    exit 1
fi

command -v makepkg >/dev/null 2>&1 || { echo "Error: makepkg command not found." >&2; exit 1; }
command -v git >/dev/null 2>&1 || { echo "Error: git command not found." >&2; exit 1; }

echo "=== AUR Publisher for $PKGNAME ==="

# Check SSH connection
echo "Checking SSH connection to aur.archlinux.org..."
if ! ssh -o ConnectTimeout=5 -T aur@aur.archlinux.org 2>&1 | grep -E -q "Interactive shell is disabled|authenticated"; then
    echo "Warning: Could not authenticate with aur@aur.archlinux.org via SSH."
    echo "Make sure your SSH key is added to your AUR account at https://aur.archlinux.org/account"
    echo "and configured in ~/.ssh/config:"
    echo ""
    echo "Host aur.archlinux.org"
    echo "    IdentityFile ~/.ssh/aur"
    echo "    User aur"
    echo ""
    read -rp "Continue anyway? [y/N]: " PROCEED
    if [[ ! "$PROCEED" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Prepare work directory on real disk (avoid tmpfs /tmp)
WORK_DIR="$REPO_DIR/dist/aur-work/$PKGNAME"
mkdir -p "$REPO_DIR/dist/aur-work"
rm -rf "$WORK_DIR"

echo "Cloning AUR repository for $PKGNAME..."
if ! git -c init.defaultBranch=master clone "ssh://aur@aur.archlinux.org/${PKGNAME}.git" "$WORK_DIR" 2>/dev/null; then
    echo "Package does not exist on AUR yet or repository is empty. Initializing new local repository..."
    mkdir -p "$WORK_DIR"
    cd "$WORK_DIR"
    git -c init.defaultBranch=master init
    git remote add origin "ssh://aur@aur.archlinux.org/${PKGNAME}.git"
else
    cd "$WORK_DIR"
fi

# Copy PKGBUILD
cp "$PKGBUILD_SOURCE" "$WORK_DIR/PKGBUILD"

# Check version
PKGVER="$(grep '^pkgver=' PKGBUILD | cut -d'=' -f2)"
echo "Packaging $PKGNAME version $PKGVER..."

# Compute and update checksums
echo "Fetching source and calculating sha256 checksums..."
if command -v updpkgsums >/dev/null 2>&1; then
    updpkgsums
else
    # Fallback if pacman-contrib's updpkgsums is not installed
    echo "updpkgsums not found, downloading source to compute checksum manually..."
    SOURCE_URL="$(grep '^source=' PKGBUILD | sed -E 's/source=\(("[^"]*"::)?"?([^")]+)"?\)/\2/' | sed "s/\$pkgname/$PKGNAME/g" | sed "s/\$pkgver/$PKGVER/g")
    TARBALL="source-${PKGVER}.tar.gz"
    curl -fsSL "$SOURCE_URL" -o "$TARBALL"
    SUM="$(sha256sum "$TARBALL" | awk '{print $1}')"
    rm -f "$TARBALL"
    sed -i "s/sha256sums=('.*')/sha256sums=('$SUM')/" PKGBUILD
fi

# Generate .SRCINFO
echo "Generating .SRCINFO..."
makepkg --printsrcinfo > .SRCINFO

# Sync updated PKGBUILD and .SRCINFO back to repo aur/ folder
cp "$WORK_DIR/PKGBUILD" "$PKGBUILD_SOURCE"
cp "$WORK_DIR/.SRCINFO" "$REPO_DIR/aur/$PKGNAME/.SRCINFO"

echo ""
echo "=== PKGBUILD and .SRCINFO ready ==="
git status -s

echo ""
read -rp "Test building package locally with makepkg -s? [y/N]: " TEST_BUILD
if [[ "$TEST_BUILD" =~ ^[Yy]$ ]]; then
    makepkg -s --noconfirm
    echo "Build succeeded!"
fi

echo ""
read -rp "Commit and push to AUR (ssh://aur@aur.archlinux.org/${PKGNAME}.git)? [y/N]: " PUSH_AUR
if [[ "$PUSH_AUR" =~ ^[Yy]$ ]]; then
    git add PKGBUILD .SRCINFO
    git commit -m "Update to $PKGVER" || echo "Nothing new to commit."
    git push -u origin master
    echo ""
    echo "Successfully published to AUR!"
    echo "Check your package at: https://aur.archlinux.org/packages/$PKGNAME"
fi
