#!/usr/bin/env bash
# Drops the build host's libwayland from an AppImage and repacks it in place. linuxdeploy bundles it with GTK, but it
# clashes with the host Mesa's libEGL and WebKit aborts at start on newer distros. The Tauri bundler gives no way to
# exclude a library, so it is removed after the fact. Any updater signature must be made again afterwards.
set -euo pipefail

appimage="$(realpath "$1")"
tool_version="1.9.1"
tool_sha256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
cd "$work"

curl -fsSL -o appimagetool "https://github.com/AppImage/appimagetool/releases/download/$tool_version/appimagetool-x86_64.AppImage"
echo "$tool_sha256  appimagetool" | sha256sum -c -
chmod +x appimagetool

# Reusing the AppImage's own runtime keeps appimagetool from downloading an unpinned one.
head -c "$("$appimage" --appimage-offset)" "$appimage" > runtime
"$appimage" --appimage-extract > /dev/null
find squashfs-root -name 'libwayland-*.so*' -print -delete

APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 ./appimagetool --no-appstream --runtime-file "$work/runtime" \
  "$work/squashfs-root" "$work/repacked.AppImage"
mv "$work/repacked.AppImage" "$appimage"
