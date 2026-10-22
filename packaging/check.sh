#!/usr/bin/env bash
# Checks the packaging files against each other (docs/spec/release.md,
# "Checks"). Needs only bash, grep and sed.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
id=io.github.bhimio1.LaserCraft
version=$("$here/version.sh")
problems=0
fail() {
  echo "problem: $*" >&2
  problems=$((problems + 1))
}

newest=$(grep -o '<release version="[^"]*"' "$here/linux/$id.metainfo.xml" | head -1 | sed 's/.*"\(.*\)"/\1/')
[ "$newest" = "$version" ] ||
  fail "the newest release in $id.metainfo.xml is $newest, the workspace version is $version"

desktop="$here/linux/$id.desktop"
grep -q '^Exec=lasercraft' "$desktop" || fail "$id.desktop does not run lasercraft"
grep -q "^Icon=$id\$" "$desktop" || fail "$id.desktop does not name the icon $id"
grep -q '<id>'"$id"'</id>' "$here/linux/$id.metainfo.xml" || fail "the metadata's id is not $id"

manifest="$here/flatpak/$id.yml"
grep -q "^id: $id\$" "$manifest" || fail "the Flatpak manifest's id is not $id"
grep -q -- '- --device=all' "$manifest" || fail "the Flatpak manifest does not request serial access (--device=all)"

grep -q 'Name="lasercraft.exe"' "$here/windows/lasercraft.wxs" || fail "the installer does not install lasercraft.exe"
grep -q '<string>lasercraft</string>' "$here/macos/Info.plist" || fail "Info.plist does not run lasercraft"
grep -q "<string>$id</string>" "$here/macos/Info.plist" || fail "Info.plist's identifier is not $id"

if [ "$problems" -gt 0 ]; then
  exit 1
fi
echo "packaging files agree (version $version)"
