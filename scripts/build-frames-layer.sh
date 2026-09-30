#!/usr/bin/env bash
# Builds the Vulkan frame layer against glibc 2.31 (Debian 11, the base of Steam's sniper runtime) and stages it
# where tauri.linux.conf.json picks it up. Needs cargo-zigbuild and zig on PATH.
set -euo pipefail

floor="2.31"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stage="$root/target/bundle-extra"
lib="libdp_frames_layer.so"

cd "$root"
cargo zigbuild -p dp-frames-layer --release --target "x86_64-unknown-linux-gnu.$floor"

built="$root/target/x86_64-unknown-linux-gnu/release/$lib"
mkdir -p "$stage"
cp "$built" "$stage/$lib"

worst="$(objdump -T "$stage/$lib" | grep -o 'GLIBC_[0-9.]*' | sed 's/GLIBC_//' | sort -uV | tail -n1 || true)"
echo "highest glibc symbol version needed: ${worst:-none} (floor $floor)"
if [ -n "$worst" ] && [ "$(printf '%s\n%s\n' "$worst" "$floor" | sort -V | tail -n1)" != "$floor" ]; then
  echo "$lib needs glibc $worst, newer than $floor" >&2
  exit 1
fi
