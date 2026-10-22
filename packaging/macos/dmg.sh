#!/usr/bin/env bash
# Builds LaserCraft-<version>-universal.dmg (docs/spec/release.md).
#
# Usage: packaging/macos/dmg.sh <arm64 binary> <x86_64 binary> <out dir>
# Needs (macOS): lipo, rsvg-convert (librsvg), iconutil, hdiutil.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
arm=$1
intel=$2
out=$3
version=$("$here/../version.sh")

work=$(mktemp -d)
app="$work/dmg/LaserCraft.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

# One binary for Apple silicon and Intel.
lipo -create "$arm" "$intel" -output "$app/Contents/MacOS/lasercraft"
chmod 755 "$app/Contents/MacOS/lasercraft"
sed "s/@VERSION@/$version/g" "$here/Info.plist" > "$app/Contents/Info.plist"
cp "$root/LICENSE" "$app/Contents/Resources/LICENSE"

# The icon: every size iconutil wants, from the SVG.
iconset="$work/LaserCraft.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  rsvg-convert -w "$size" -h "$size" "$root/packaging/icons/lasercraft.svg" -o "$iconset/icon_${size}x${size}.png"
  double=$((size * 2))
  rsvg-convert -w "$double" -h "$double" "$root/packaging/icons/lasercraft.svg" -o "$iconset/icon_${size}x${size}@2x.png"
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/LaserCraft.icns"

# Drag to Applications.
ln -s /Applications "$work/dmg/Applications"
mkdir -p "$out"
hdiutil create -volname "LaserCraft $version" -srcfolder "$work/dmg" -ov -format UDZO \
  "$out/LaserCraft-$version-universal.dmg"
echo "built $out/LaserCraft-$version-universal.dmg"
