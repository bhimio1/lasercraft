#!/usr/bin/env bash
# Builds LaserCraft-<version>-x86_64.AppImage (docs/spec/release.md).
#
# Usage: packaging/linux/appimage.sh <lasercraft binary> <out dir>
# Needs: rsvg-convert (librsvg) and appimagetool on PATH, or APPIMAGETOOL
# naming it.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
binary=$1
out=$2
id=io.github.bhimio1.LaserCraft
version=$("$here/../version.sh")
tool=${APPIMAGETOOL:-appimagetool}

appdir=$(mktemp -d)/LaserCraft.AppDir
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" "$appdir/usr/share/metainfo" \
  "$appdir/usr/share/icons/hicolor/scalable/apps" "$appdir/usr/share/icons/hicolor/256x256/apps" \
  "$appdir/usr/share/doc/lasercraft"
install -m755 "$binary" "$appdir/usr/bin/lasercraft"
install -m644 "$here/$id.desktop" "$appdir/usr/share/applications/$id.desktop"
install -m644 "$here/$id.metainfo.xml" "$appdir/usr/share/metainfo/$id.appdata.xml"
install -m644 "$root/packaging/icons/lasercraft.svg" "$appdir/usr/share/icons/hicolor/scalable/apps/$id.svg"
rsvg-convert -w 256 -h 256 "$root/packaging/icons/lasercraft.svg" \
  -o "$appdir/usr/share/icons/hicolor/256x256/apps/$id.png"
install -m644 "$root/LICENSE" "$appdir/usr/share/doc/lasercraft/LICENSE"

# The AppDir's top level: desktop entry, icon, and the program to run.
cp "$appdir/usr/share/applications/$id.desktop" "$appdir/"
cp "$appdir/usr/share/icons/hicolor/256x256/apps/$id.png" "$appdir/$id.png"
ln -s "$id.png" "$appdir/.DirIcon"
ln -s usr/bin/lasercraft "$appdir/AppRun"

mkdir -p "$out"
ARCH=x86_64 "$tool" "$appdir" "$out/LaserCraft-$version-x86_64.AppImage"
echo "built $out/LaserCraft-$version-x86_64.AppImage"
